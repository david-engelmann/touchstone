//! VL-5 to VL-12. The registry: an append-only event
//! file, folded into state on every read. The port
//! reproduces the Python's defect catalogue, refusal
//! phrases, and suggestion arithmetic exactly; where
//! the registry's own reading differs from the
//! flighttape crate's reader (duplicate keys refused,
//! the NaN and Infinity constants refused, a per-line
//! parse depth cap), the difference is closed here
//! in the loader (VL-15).

use std::collections::{HashMap, HashSet};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

use flighttape::identity::tape_digest;
use flighttape::value::{object, set_key, Text, Value};

use crate::emit::{dumps_line, Mode};
use crate::judge::{self, Consult, Transport};
use crate::pyrepr::repr_value;
use crate::util::{
    cmp_f64_int, decode_pair, int_to_f64, join_msgs, json_text_depths, normalize_path_display,
    overflow_threshold, rational_to_f64, scan_line, Big, Msg,
};

/// The registry line's parse depth caps, probed
/// through the shipped CLI against the Python: a
/// line nesting past 9994 refuses outright, and a
/// line whose deepest object sits past 9992 refuses
/// (the Python's reader calls its duplicate key
/// hook as each object closes, and the hook spends
/// stack a deep array never does; mixed lines
/// follow the same two measures, object position
/// counting, array position counting only toward
/// the total).
pub const LINE_DEPTH_CAP: usize = 9994;
pub const LINE_OBJECT_DEPTH_CAP: usize = 9992;

/// The CLI's JSON argument parse depth cap, probed
/// the same way: text nesting 9994 parses, 9995
/// refuses.
pub const ARG_DEPTH_CAP: usize = 9994;

const VERDICTS: [&str; 3] = ["keep", "kill", "rerun"];
const EVENT_TYPES: [&str; 3] = ["opened", "result", "closed"];
const DIRECTIONS: [&str; 2] = ["lower", "higher"];

/// A named refusal, the family form (VL-12): the CLI
/// prints it after "error, " and exits 1.
#[derive(Debug, PartialEq)]
pub struct RegistryError(pub Msg);

/// A registry failure: either a named refusal, or a
/// place where the Python dies with an uncaught
/// exception (an OverflowError in the suggestion
/// rule over integers too large to convert, an
/// encode failure on append). The port answers the
/// crash class with a silent exit 1; the class is
/// named in the round dossier, as a crash is not
/// behavior to reproduce.
#[derive(Debug, PartialEq)]
pub enum RegError {
    Named(RegistryError),
    Crash,
}

fn named(text: &str) -> RegError {
    RegError::Named(RegistryError(Msg::str(text)))
}

/// An operating system error's CPython rendering with
/// no file name, for the CLI's stream failures.
pub fn os_exc_text(err: &std::io::Error) -> String {
    os_exc(err, None).render_stderr()
}

fn named_msg(msg: Msg) -> RegError {
    RegError::Named(RegistryError(msg))
}

/// The registry file: its OS path and the display
/// form its messages name (the typed path as pathlib
/// prints it).
#[derive(Clone)]
pub struct RegPath {
    pub os: PathBuf,
    pub display: Msg,
}

impl RegPath {
    pub fn from_typed(text: &Text) -> RegPath {
        RegPath {
            os: crate::cli::os_path(text),
            display: Msg(normalize_path_display(text.units())),
        }
    }

    pub fn new(text: &str) -> RegPath {
        RegPath::from_typed(&Text::from_str(text))
    }
}

/// An operating system error in CPython's rendering:
/// "[Errno N] message", with the file name appended
/// where the Python's exception carries one. The
/// wording of the message itself is the platform's
/// strerror text for the errnos the differential
/// exercises; other errnos fall back to the standard
/// library's rendering, a named deviation.
fn os_exc(err: &std::io::Error, path: Option<&Msg>) -> Msg {
    let mut out = Msg::new();
    match err.raw_os_error() {
        Some(errno) => {
            let text = match errno {
                1 => "Operation not permitted",
                2 => "No such file or directory",
                13 => "Permission denied",
                20 => "Not a directory",
                21 => "Is a directory",
                24 => "Too many open files",
                28 => "No space left on device",
                30 => "Read-only file system",
                32 => "Broken pipe",
                36 => "File name too long",
                40 => "Too many levels of symbolic links",
                _ => "",
            };
            if text.is_empty() {
                out.push_str(&err.to_string());
            } else {
                out.push_str(&format!("[Errno {errno}] {text}"));
            }
        }
        None => out.push_str(&err.to_string()),
    }
    if let Some(path) = path {
        out.push_str(": '");
        out.push_msg(path);
        out.push_str("'");
    }
    out
}

fn is_number(value: &Value) -> bool {
    match value {
        Value::Int(_) => true,
        Value::Float(number) => number.is_finite(),
        _ => false,
    }
}

fn is_negative_number(value: &Value) -> bool {
    match value {
        Value::Int(digits) => digits.starts_with('-'),
        Value::Float(number) => *number < 0.0,
        _ => false,
    }
}

fn is_zero_number(value: &Value) -> bool {
    match value {
        Value::Int(digits) => digits == "0",
        Value::Float(number) => *number == 0.0,
        _ => false,
    }
}

fn is_positive_number(value: &Value) -> bool {
    match value {
        Value::Int(digits) => !digits.starts_with('-') && digits != "0",
        Value::Float(number) => *number > 0.0,
        _ => false,
    }
}

fn as_str_text(value: &Value) -> Option<&Text> {
    match value {
        Value::Str(text) => Some(text),
        _ => None,
    }
}

fn is_nonempty_str(value: &Value) -> bool {
    matches!(value, Value::Str(text) if !text.units().is_empty())
}

fn text_eq(text: &Text, plain: &str) -> bool {
    text.units() == Text::from_str(plain).units()
}

fn str_in(value: &Value, options: &[&str]) -> bool {
    match as_str_text(value) {
        Some(text) => options.iter().any(|option| text_eq(text, option)),
        None => false,
    }
}

fn obj_get<'a>(value: &'a Value, key: &Text) -> Option<&'a Value> {
    match value {
        Value::Object(pairs) => pairs
            .iter()
            .find(|(name, _)| name.units() == key.units())
            .map(|(_, item)| item),
        _ => None,
    }
}

/// The timestamp gate (VL-5a, VL-5c): a date and a
/// time in the pinned prefix form, then the crate's
/// fromisoformat grammar over the whole string.
fn is_timestamp(value: &Value) -> bool {
    let text = match as_str_text(value) {
        Some(text) => text,
        None => return false,
    };
    let units = text.units();
    if units.len() < 16 {
        return false;
    }
    let digit = |unit: u16| (0x30..=0x39).contains(&unit);
    let gate = digit(units[0])
        && digit(units[1])
        && digit(units[2])
        && digit(units[3])
        && units[4] == '-' as u16
        && digit(units[5])
        && digit(units[6])
        && units[7] == '-' as u16
        && digit(units[8])
        && digit(units[9])
        && (units[10] == 'T' as u16 || units[10] == ' ' as u16)
        && digit(units[11])
        && digit(units[12])
        && units[13] == ':' as u16
        && digit(units[14])
        && digit(units[15]);
    if !gate {
        return false;
    }
    match text.as_str() {
        Some(plain) => flighttape::timeparse::parse_timestamp(&plain).is_some(),
        None => false,
    }
}

fn defect(position: usize, text: &str) -> Msg {
    Msg::str(&format!("line {position}: {text}"))
}

fn defect_msg(position: usize, msg: Msg) -> Msg {
    let mut out = Msg::str(&format!("line {position}: "));
    out.push_msg(&msg);
    out
}

/// VL-5 to VL-11a: one event's own defects, in the
/// Python's check order and phrases.
pub fn event_defects(event: &Value, position: usize) -> Vec<Msg> {
    if !matches!(event, Value::Object(_)) {
        return vec![defect(position, "the event is not an object")];
    }
    let mut defects: Vec<Msg> = Vec::new();
    let kind = event.get("type");
    let kind_name: Option<&str> = EVENT_TYPES
        .iter()
        .find(|name| str_in(kind_or_missing(kind), &[name]))
        .copied();
    if kind_name.is_none() {
        let mut msg = Msg::str("unknown event type ");
        msg.push_str(&repr_value(kind_or_missing(kind)));
        defects.push(defect_msg(position, msg));
    }
    if !is_nonempty_str(event.get("experiment").unwrap_or(&Value::Null)) {
        defects.push(defect(position, "experiment is not a non-empty string"));
    }
    if !is_timestamp(event.get("at").unwrap_or(&Value::Null)) {
        defects.push(defect(position, "at is not an ISO timestamp"));
    }
    if !defects.is_empty() || kind_name.is_none() {
        return defects;
    }
    match kind_name {
        Some("opened") => opened_defects(event, position, &mut defects),
        Some("result") => {
            let measures = event.get("measures").unwrap_or(&Value::Null);
            measures_defects(measures, position, "measures", &mut defects);
        }
        Some("closed") => closed_defects(event, position, &mut defects),
        _ => unreachable!("a known kind"),
    }
    defects
}

fn kind_or_missing(kind: Option<&Value>) -> &Value {
    kind.unwrap_or(&Value::Null)
}

fn measures_defects(measures: &Value, position: usize, field: &str, defects: &mut Vec<Msg>) {
    match measures {
        Value::Object(pairs) if !pairs.is_empty() => {
            if pairs.iter().any(|(_, value)| !is_number(value)) {
                defects.push(defect(
                    position,
                    &format!("{field} holds a value that is not a finite number"),
                ));
            }
        }
        _ => {
            defects.push(defect(
                position,
                &format!("{field} is not a non-empty object"),
            ));
        }
    }
}

fn not_in_baseline_msg(name: &Value) -> Msg {
    let mut msg = Msg::str("judged measure ");
    msg.push_str(&repr_value(name));
    msg.push_str(" is not in the baseline measures");
    msg
}

fn opened_defects(event: &Value, position: usize, defects: &mut Vec<Msg>) {
    for field in ["source", "proposal", "baseline"] {
        if !is_nonempty_str(event.get(field).unwrap_or(&Value::Null)) {
            defects.push(defect(
                position,
                &format!("{field} is not a non-empty string"),
            ));
        }
    }
    let measures = event.get("baseline_measures").unwrap_or(&Value::Null);
    measures_defects(measures, position, "baseline_measures", defects);
    let judged = event.get("judged").unwrap_or(&Value::Null);
    match judged {
        Value::Array(entries) if !entries.is_empty() => {
            let mut names: Vec<&Text> = Vec::new();
            for entry in entries {
                if let Value::Object(_) = entry {
                    if let Some(name) = entry.get("name").and_then(as_str_text) {
                        names.push(name);
                    }
                }
            }
            if names.iter().any(|name| {
                names
                    .iter()
                    .filter(|other| other.units() == name.units())
                    .count()
                    > 1
            }) {
                defects.push(defect(position, "judged names are not unique"));
            }
            for entry in entries {
                let entry_ok = matches!(entry, Value::Object(_))
                    && entry.get("name").and_then(as_str_text).is_some()
                    && str_in(entry.get("direction").unwrap_or(&Value::Null), &DIRECTIONS);
                if !entry_ok {
                    defects.push(defect(
                        position,
                        "a judged entry needs a name and a direction of lower or higher",
                    ));
                } else if let Some(margin) = entry.get("margin_pct") {
                    if !is_number(margin) || is_negative_number(margin) {
                        defects.push(defect(
                            position,
                            "a judged margin_pct is not a non-negative number",
                        ));
                    } else if matches!(measures, Value::Object(_))
                        && obj_get(
                            measures,
                            entry.get("name").and_then(as_str_text).expect("checked"),
                        )
                        .is_none()
                    {
                        defects.push(defect_msg(
                            position,
                            not_in_baseline_msg(entry.get("name").unwrap_or(&Value::Null)),
                        ));
                    }
                } else if matches!(measures, Value::Object(_))
                    && obj_get(
                        measures,
                        entry.get("name").and_then(as_str_text).expect("checked"),
                    )
                    .is_none()
                {
                    defects.push(defect_msg(
                        position,
                        not_in_baseline_msg(entry.get("name").unwrap_or(&Value::Null)),
                    ));
                }
            }
        }
        _ => defects.push(defect(position, "judged is not a non-empty list")),
    }
    if let Some(margin) = event.get("margin_pct") {
        if !is_number(margin) || is_negative_number(margin) {
            defects.push(defect(position, "margin_pct is not a non-negative number"));
        }
    }
}

fn closed_defects(event: &Value, position: usize, defects: &mut Vec<Msg>) {
    if !str_in(event.get("verdict").unwrap_or(&Value::Null), &VERDICTS) {
        defects.push(defect(position, "verdict is not one of keep, kill, rerun"));
    }
    if !is_nonempty_str(event.get("rationale").unwrap_or(&Value::Null)) {
        defects.push(defect(position, "rationale is not a non-empty string"));
    }
    let rests_ok = match event.get("rests_on") {
        Some(Value::Object(_)) => {
            let rests = event.get("rests_on").expect("checked");
            matches!(rests.get("baseline"), Some(Value::Int(_)))
                && matches!(rests.get("result"), Some(Value::Int(_)))
        }
        _ => false,
    };
    if !rests_ok {
        defects.push(defect(
            position,
            "rests_on does not name the baseline and result positions",
        ));
    }
    let suggested = event.get("suggested").unwrap_or(&Value::Null);
    if !matches!(suggested, Value::Str(_)) || !str_in(suggested, &VERDICTS) {
        defects.push(defect(position, "the recorded suggestion is not a verdict"));
    }
    for field in ["regressed", "unmeasured"] {
        let ok = match event.get(field) {
            Some(Value::Array(items)) => items.iter().all(|item| matches!(item, Value::Str(_))),
            _ => false,
        };
        if !ok {
            defects.push(defect(
                position,
                &format!("{field} is not a list of measure names"),
            ));
        }
    }
    if event.get("successor").is_some()
        && !str_in(event.get("verdict").unwrap_or(&Value::Null), &["rerun"])
    {
        defects.push(defect(
            position,
            "a successor is recorded only on a rerun verdict",
        ));
    }
    judgment_defects(event, position, defects);
}

/// A recorded judgment number as an f64 for the
/// range checks below: a huge integer becomes an
/// infinity and fails every bound, where the
/// registry's own converter would meet the crash
/// class. A forged line must name a defect, never
/// crash the read.
fn judgment_number(value: &Value) -> Option<f64> {
    match value {
        Value::Int(digits) => digits.parse::<f64>().ok(),
        Value::Float(number) => Some(*number),
        _ => None,
    }
}

fn judgment_bounded(value: Option<&Value>, low: f64, high: f64) -> bool {
    match value.and_then(judgment_number) {
        Some(number) => number.is_finite() && number >= low && number <= high,
        None => false,
    }
}

fn is_digest_text(value: Option<&Value>) -> bool {
    match value {
        Some(Value::Str(text)) => {
            let units = text.units();
            units.len() == 64
                && units
                    .iter()
                    .all(|unit| (0x30..=0x39).contains(unit) || (0x61..=0x66).contains(unit))
        }
        _ => false,
    }
}

fn judgment_probabilities_ok(value: Option<&Value>) -> bool {
    let entries = match value {
        Some(Value::Object(pairs)) => pairs,
        _ => return false,
    };
    if entries.len() != judge::LABELS.len() {
        return false;
    }
    let mut sum = 0.0;
    for label in judge::LABELS {
        let found = entries
            .iter()
            .find(|(name, _)| name.units() == Text::from_str(label).units());
        match found {
            Some((_, item)) if judgment_bounded(Some(item), 0.0, 1.0) => {
                sum += judgment_number(item).expect("bounded");
            }
            _ => return false,
        }
    }
    (sum - 1.0).abs() <= 0.02
}

const ANSWER_FIELDS: [&str; 6] = [
    "model",
    "choice",
    "confidence",
    "probabilities",
    "evidence_strength",
    "regression_risk",
];

/// The fields the writer emits for a judged close,
/// in emission order (VL-24a): the card and its
/// digest, the answer in judge_card's order, and
/// the status last, as consult builds the section.
const JUDGED_FIELDS: [&str; 9] = [
    "card_sha256",
    "card",
    "model",
    "choice",
    "confidence",
    "probabilities",
    "evidence_strength",
    "regression_risk",
    "status",
];

/// The fields the writer emits for an unavailable
/// close, in emission order (VL-24a).
const UNAVAILABLE_FIELDS: [&str; 4] = ["card_sha256", "card", "status", "reason"];

/// Whether a judgment section's fields are exactly
/// the writer's, names and order (VL-24a): the
/// closed shape the fold's exact-object comparison
/// already imposes on a not_consulted section.
fn judgment_fields_are_emitted(judgment: &Value, expected: &[&str]) -> bool {
    match judgment {
        Value::Object(pairs) => {
            pairs.len() == expected.len()
                && pairs
                    .iter()
                    .zip(expected.iter())
                    .all(|((name, _), want)| name.as_str().as_deref() == Some(*want))
        }
        _ => false,
    }
}

/// The judgment section's own shape (VL-26): a
/// close recorded before Round 3 carries none and
/// is sound without it; a section that is present
/// must be internally consistent, and a judged or
/// unavailable section's fields must be exactly
/// the writer's (VL-24a). The card's re-derivation
/// against the rested measures is the fold's work,
/// beside VL-9b.
fn judgment_defects(event: &Value, position: usize, defects: &mut Vec<Msg>) {
    let judgment = match event.get("judgment") {
        Some(judgment) => judgment,
        None => return,
    };
    if !matches!(judgment, Value::Object(_)) {
        defects.push(defect(position, "the judgment section is not an object"));
        return;
    }
    let status = judgment.get("status").unwrap_or(&Value::Null);
    let status_name = ["judged", "not_consulted", "unavailable"]
        .iter()
        .copied()
        .find(|name| str_in(status, &[name]));
    let status_name = match status_name {
        Some(name) => name,
        None => {
            defects.push(defect(
                position,
                "the judgment status is not one of judged, not_consulted, unavailable",
            ));
            return;
        }
    };
    match status_name {
        "not_consulted" => {
            if !is_nonempty_str(judgment.get("reason").unwrap_or(&Value::Null)) {
                defects.push(defect(
                    position,
                    "a judgment that was not consulted names no reason",
                ));
            }
            if judgment.get("card").is_some()
                || judgment.get("card_sha256").is_some()
                || ANSWER_FIELDS
                    .iter()
                    .any(|field| judgment.get(field).is_some())
            {
                defects.push(defect(
                    position,
                    "a judgment that was not consulted carries a card or an answer",
                ));
            }
        }
        "unavailable" => {
            if !judgment_fields_are_emitted(judgment, &UNAVAILABLE_FIELDS) {
                defects.push(defect(
                    position,
                    "the unavailable section's fields are not the recorded fields",
                ));
            }
            if !is_digest_text(judgment.get("card_sha256")) {
                defects.push(defect(
                    position,
                    "an unavailable judgment does not name its card digest",
                ));
            }
            if !matches!(judgment.get("card"), Some(Value::Str(_))) {
                defects.push(defect(
                    position,
                    "an unavailable judgment does not carry its card",
                ));
            }
            if !is_nonempty_str(judgment.get("reason").unwrap_or(&Value::Null)) {
                defects.push(defect(position, "an unavailable judgment names no reason"));
            }
            if ANSWER_FIELDS
                .iter()
                .any(|field| judgment.get(field).is_some())
            {
                defects.push(defect(
                    position,
                    "an unavailable judgment carries an answer",
                ));
            }
        }
        _ => {
            if !judgment_fields_are_emitted(judgment, &JUDGED_FIELDS) {
                defects.push(defect(
                    position,
                    "the judged section's fields are not the recorded fields",
                ));
            }
            match judgment.get("model") {
                Some(Value::Str(model))
                    if model.units() == Text::from_str(judge::MODEL).units() => {}
                Some(Value::Str(_)) => {
                    defects.push(defect(position, "a judged close names an unpinned model"))
                }
                _ => defects.push(defect(position, "a judged close does not name its model")),
            }
            if !is_digest_text(judgment.get("card_sha256")) {
                defects.push(defect(
                    position,
                    "a judged close does not name its card digest",
                ));
            }
            if !matches!(judgment.get("card"), Some(Value::Str(_))) {
                defects.push(defect(position, "a judged close does not carry its card"));
            }
            if !str_in(judgment.get("choice").unwrap_or(&Value::Null), &VERDICTS) {
                defects.push(defect(
                    position,
                    "a judged close names a choice that is not a verdict",
                ));
            }
            let confidence = judgment.get("confidence");
            if !judgment_bounded(confidence, 0.0, 1.0) {
                defects.push(defect(
                    position,
                    "the judgment confidence is not a number within 0 and 1",
                ));
            } else if judge::confidence_below_floor(confidence.expect("a bounded confidence")) {
                defects.push(defect(
                    position,
                    "the judgment confidence is below the floor of 0.6",
                ));
            }
            if !judgment_bounded(judgment.get("evidence_strength"), 0.0, 10.0)
                || !judgment_bounded(judgment.get("regression_risk"), 0.0, 10.0)
            {
                defects.push(defect(
                    position,
                    "the judgment scores are not numbers within 0 and 10",
                ));
            }
            if !judgment_probabilities_ok(judgment.get("probabilities")) {
                defects.push(defect(
                    position,
                    "the judgment probabilities do not cover the four choices",
                ));
            }
        }
    }
}

/// Parse one CLI JSON argument (a measures object, a
/// judged list): the crate reader's acceptance, with
/// the Python reader's depth refusal (VL-15's second
/// surface).
pub fn parse_arg_json(text: &Text, what: &str) -> Result<Value, RegError> {
    // The Python's reader works on the decoded
    // argument string, lone surrogates included; the
    // crate reader takes a Rust string, which cannot
    // hold one. A lone surrogate can only ride inside
    // a JSON string, where its escape form parses to
    // the same text, so the argument is re-spelled
    // with escapes before it is read. The depth
    // refusal is decided from the text before the
    // parse, as in the registry loader.
    let units = text.units();
    let mut plain = String::new();
    let mut index = 0;
    while index < units.len() {
        let unit = units[index];
        if (0xD800..=0xDBFF).contains(&unit)
            && index + 1 < units.len()
            && (0xDC00..=0xDFFF).contains(&units[index + 1])
        {
            let scalar = decode_pair(unit, units[index + 1]);
            plain.push(char::from_u32(scalar).expect("a valid pair"));
            index += 2;
        } else if (0xD800..=0xDFFF).contains(&unit) {
            plain.push_str(&format!("\\u{unit:04x}"));
            index += 1;
        } else {
            plain.push(char::from_u32(unit as u32).expect("a BMP scalar"));
            index += 1;
        }
    }
    if json_text_depths(&plain).0 > ARG_DEPTH_CAP {
        return Err(named(&format!("{what} is not valid JSON")));
    }
    match flighttape::json::parse(&plain) {
        Ok(value) => Ok(value),
        Err(_) => Err(named(&format!("{what} is not valid JSON"))),
    }
}

/// The Python's blank test for a registry line:
/// str.strip() empties it. Its whitespace set is
/// Rust's White_Space set plus the four separators
/// U+001C to U+001F.
fn is_blank_line(line: &str) -> bool {
    line.chars()
        .all(|ch| ch.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&ch))
}

/// What a load returns: events carrying their 1-based
/// positions, and defects naming every bad line and
/// every sequence violation (VL-5, VL-11, VL-11a).
pub type LoadOutcome = Result<(Vec<(usize, Value)>, Vec<Msg>), RegError>;

pub fn load_registry(path: &RegPath) -> LoadOutcome {
    let raw = match std::fs::read(&path.os) {
        Ok(raw) => raw,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            let mut msg = Msg::str("no such registry, ");
            msg.push_msg(&path.display);
            return Ok((Vec::new(), vec![msg]));
        }
        Err(err) => {
            let mut msg = Msg::str("cannot read the registry at ");
            msg.push_msg(&path.display);
            msg.push_str(": ");
            msg.push_msg(&os_exc(&err, Some(&path.display)));
            return Ok((Vec::new(), vec![msg]));
        }
    };
    let body = if raw.starts_with(&[0xEF, 0xBB, 0xBF]) {
        &raw[3..]
    } else {
        &raw[..]
    };
    let text = match String::from_utf8(body.to_vec()) {
        Ok(text) => text,
        Err(_) => {
            return Ok((
                Vec::new(),
                vec![Msg::str("the registry is not valid UTF-8")],
            ))
        }
    };
    let mut lines: Vec<&str> = text.split('\n').collect();
    if lines.last() == Some(&"") {
        lines.pop();
    }
    let mut events: Vec<(usize, Value)> = Vec::new();
    let mut defects: Vec<Msg> = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let position = index + 1;
        if is_blank_line(line) {
            defects.push(defect(position, "blank line"));
            continue;
        }
        // The depth refusal is decided from the text
        // before the parse: past the caps the value
        // itself would nest too deep to scan or drop
        // safely, and the Python names the same defect
        // from its reader's recursion limit.
        let (depth, object_depth) = json_text_depths(line);
        if depth > LINE_DEPTH_CAP || object_depth > LINE_OBJECT_DEPTH_CAP {
            defects.push(defect(position, "not valid JSON"));
            continue;
        }
        let value = match flighttape::json::parse(line) {
            Ok(value) => value,
            Err(_) => {
                defects.push(defect(position, "not valid JSON"));
                continue;
            }
        };
        let scan = scan_line(line);
        if scan.duplicate_key || scan.constant {
            defects.push(defect(position, "not valid JSON"));
            continue;
        }
        let found = event_defects(&value, position);
        if !found.is_empty() {
            defects.extend(found);
            continue;
        }
        events.push((position, value));
    }
    defects.extend(fold_defects(&events)?);
    Ok((events, defects))
}

fn experiment_of(event: &Value) -> Text {
    match event.get("experiment") {
        Some(Value::Str(text)) => text.clone(),
        _ => Text::from_str(""),
    }
}

fn int_position(value: &Value) -> Option<usize> {
    match value {
        Value::Int(digits) => digits.parse::<usize>().ok(),
        _ => None,
    }
}

fn fold_defects(events: &[(usize, Value)]) -> Result<Vec<Msg>, RegError> {
    let mut defects: Vec<Msg> = Vec::new();
    let mut opened: HashMap<Vec<u16>, usize> = HashMap::new();
    let mut closed: HashSet<Vec<u16>> = HashSet::new();
    let mut positions: HashMap<usize, (Vec<u16>, &'static str)> = HashMap::new();
    let mut by_position: HashMap<usize, &Value> = HashMap::new();
    for (position, event) in events {
        let kind = kind_name(event);
        positions.insert(*position, (experiment_of(event).units().to_vec(), kind));
        by_position.insert(*position, event);
    }
    for (position, event) in events {
        let experiment = experiment_of(event);
        let key = experiment.units().to_vec();
        let kind = kind_name(event);
        if kind == "opened" {
            match opened.entry(key.clone()) {
                std::collections::hash_map::Entry::Occupied(_) => {
                    let mut msg = Msg::str("duplicate opening of ");
                    msg.push_text(&experiment);
                    defects.push(defect_msg(*position, msg));
                }
                std::collections::hash_map::Entry::Vacant(slot) => {
                    slot.insert(*position);
                }
            }
        } else if !opened.contains_key(&key) {
            let mut msg = Msg::str(&format!("{kind} event for unopened experiment "));
            msg.push_text(&experiment);
            defects.push(defect_msg(*position, msg));
        } else if kind == "result" && closed.contains(&key) {
            let mut msg = Msg::str("result after the close of ");
            msg.push_text(&experiment);
            defects.push(defect_msg(*position, msg));
        } else if kind == "closed" {
            if closed.contains(&key) {
                let mut msg = Msg::str("second close of ");
                msg.push_text(&experiment);
                defects.push(defect_msg(*position, msg));
            }
            closed.insert(key.clone());
            let rests = event.get("rests_on").expect("a checked close");
            let baseline_ok = match rests.get("baseline") {
                Some(Value::Int(digits)) => {
                    opened.get(&key).map(|pos| pos.to_string()) == Some(digits.clone())
                }
                _ => false,
            };
            if !baseline_ok {
                let mut msg = Msg::str("the close rests on a baseline that is not the opening of ");
                msg.push_text(&experiment);
                defects.push(defect_msg(*position, msg));
            }
            let result_at = rests.get("result").and_then(int_position);
            let result_ok = match result_at {
                Some(at) => {
                    positions.get(&at).map(|(exp, kind)| (exp.clone(), *kind))
                        == Some((key.clone(), "result"))
                }
                None => false,
            };
            if !result_ok {
                let mut msg = Msg::str("the close rests on a result that is not a result of ");
                msg.push_text(&experiment);
                defects.push(defect_msg(*position, msg));
            } else {
                let at = result_at.expect("checked");
                let later = events
                    .iter()
                    .filter(|(pos, other)| {
                        *pos > at
                            && *pos < *position
                            && experiment_of(other).units() == experiment.units()
                            && kind_name(other) == "result"
                    })
                    .map(|(pos, _)| *pos)
                    .next_back();
                if let Some(later) = later {
                    defects.push(defect(
                        *position,
                        &format!(
                            "the close rests on a result that is not the current one; a later result stands at line {later}"
                        ),
                    ));
                } else {
                    let opened_event = by_position[&opened[&key]];
                    let result_event = by_position[&at];
                    let found = suggested_verdict(opened_event, result_event)?;
                    let recorded = (
                        event.get("suggested").and_then(as_str_text).cloned(),
                        text_list(event.get("regressed")),
                        text_list(event.get("unmeasured")),
                    );
                    let found_text = (
                        Some(Text::from_str(found.0)),
                        found.1.clone(),
                        found.2.clone(),
                    );
                    if recorded != found_text {
                        defects.push(defect(
                            *position,
                            "the recorded suggestion does not match the measures it rests on",
                        ));
                    }
                    if let Some(judgment) = event.get("judgment") {
                        judgment_fold_defects(
                            judgment,
                            event,
                            opened_event,
                            result_event,
                            &found,
                            *position,
                            &mut defects,
                        )?;
                    }
                }
            }
        }
    }
    Ok(defects)
}

/// The judgment's consistency against the rested
/// measures (VL-26), beside the VL-9b suggestion
/// re-derivation: an uncontested close records
/// exactly the not_consulted section the band
/// prescribes, and a consulted close's card is
/// rebuilt from the opening and the rested result
/// and must digest to the recorded digest. The
/// section's own shape was settled at the event
/// level, so only the statuses arrive here.
fn judgment_fold_defects(
    judgment: &Value,
    event: &Value,
    opened_event: &Value,
    result_event: &Value,
    found: &(&'static str, Vec<Text>, Vec<Text>),
    position: usize,
    defects: &mut Vec<Msg>,
) -> Result<(), RegError> {
    let verdict = match VERDICTS
        .iter()
        .copied()
        .find(|name| str_in(event.get("verdict").unwrap_or(&Value::Null), &[name]))
    {
        Some(verdict) => verdict,
        None => return Ok(()),
    };
    let status = judgment.get("status").unwrap_or(&Value::Null);
    if str_in(status, &["not_consulted"]) {
        let reason = if verdict == found.0 {
            "the verdict and the suggestion agree"
        } else {
            "no judge is configured"
        };
        let expected = object(vec![
            ("status", Value::Str(Text::from_str("not_consulted"))),
            ("reason", Value::Str(Text::from_str(reason))),
        ]);
        if judgment != &expected {
            defects.push(defect(
                position,
                "the recorded judgment does not match the close it rests on",
            ));
        }
        return Ok(());
    }
    if !str_in(status, &["judged", "unavailable"]) {
        return Ok(());
    }
    if verdict == found.0 {
        defects.push(defect(
            position,
            "a judgment was recorded where the verdict and the suggestion agree",
        ));
        return Ok(());
    }
    let card = judge::build_card(
        opened_event,
        result_event,
        verdict,
        found.0,
        &found.1,
        &found.2,
    );
    let digest = match tape_digest(&Value::Str(Text::from_str(&card))) {
        Ok(digest) => digest,
        Err(_) => return Err(RegError::Crash),
    };
    let card_ok = judgment.get("card") == Some(&Value::Str(Text::from_str(&card)))
        && judgment.get("card_sha256") == Some(&Value::Str(Text::from_str(&digest)));
    if !card_ok {
        defects.push(defect(
            position,
            "the recorded judgment does not match the card it rests on",
        ));
    }
    Ok(())
}

fn kind_name(event: &Value) -> &'static str {
    match event.get("type") {
        Some(Value::Str(text)) => {
            if text_eq(text, "opened") {
                "opened"
            } else if text_eq(text, "result") {
                "result"
            } else if text_eq(text, "closed") {
                "closed"
            } else {
                ""
            }
        }
        _ => "",
    }
}

fn text_list(value: Option<&Value>) -> Vec<Text> {
    match value {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|item| match item {
                Value::Str(text) => Some(text.clone()),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// One experiment's folded state (VL-10).
pub struct State {
    pub experiment: Text,
    pub opened: Value,
    pub opened_at_position: usize,
    pub results: Vec<(usize, Value)>,
    pub closed: Option<Value>,
    pub closed_at_position: Option<usize>,
}

impl State {
    pub fn state_name(&self) -> &'static str {
        if self.closed.is_some() {
            "closed"
        } else if !self.results.is_empty() {
            "resulted"
        } else {
            "open"
        }
    }
}

/// Experiment states in opening order (VL-10).
pub fn fold(events: &[(usize, Value)]) -> Vec<State> {
    let mut order: Vec<Text> = Vec::new();
    let mut states: HashMap<Vec<u16>, State> = HashMap::new();
    for (position, event) in events {
        let experiment = experiment_of(event);
        let key = experiment.units().to_vec();
        match kind_name(event) {
            "opened" => {
                if !states.contains_key(&key) {
                    order.push(experiment.clone());
                }
                states.insert(
                    key,
                    State {
                        experiment,
                        opened: event.clone(),
                        opened_at_position: *position,
                        results: Vec::new(),
                        closed: None,
                        closed_at_position: None,
                    },
                );
            }
            "result" => {
                if let Some(state) = states.get_mut(&key) {
                    state.results.push((*position, event.clone()));
                }
            }
            "closed" => {
                if let Some(state) = states.get_mut(&key) {
                    state.closed = Some(event.clone());
                    state.closed_at_position = Some(*position);
                }
            }
            _ => {}
        }
    }
    order
        .iter()
        .filter_map(|text| states.remove(text.units()))
        .collect()
}

/// Whether (value against base, by direction) regresses
/// past the margin: the VL-9a rule in the Python's own
/// arithmetic. Integers compare exactly until the one
/// true division, which is correctly rounded; a float
/// anywhere takes the float pipeline; an integer too
/// large to convert where the Python converts is the
/// crash class.
fn regressed(
    direction: &str,
    base: &Value,
    value: &Value,
    margin: &Value,
) -> Result<bool, RegError> {
    if is_zero_number(base) {
        return Ok(direction == "lower" && is_positive_number(value));
    }
    match (base, value) {
        (Value::Int(base_digits), Value::Int(value_digits)) => {
            let base_big = Big::from_int_str(base_digits);
            let value_big = Big::from_int_str(value_digits);
            let diff = if direction == "lower" {
                value_big.sub(&base_big)
            } else {
                base_big.sub(&value_big)
            };
            // The one division must not overflow: the
            // Python raises where the correctly rounded
            // quotient leaves the double range, at the
            // rounding midpoint above the largest
            // double.
            let threshold = overflow_threshold();
            if diff.abs().compare(&threshold.mul(&base_big.abs())) != std::cmp::Ordering::Less {
                return Err(RegError::Crash);
            }
            let quotient = rational_to_f64(&diff, &base_big.abs());
            let scaled = quotient * 100.0;
            Ok(gt_margin(scaled, margin))
        }
        _ => {
            let base_f = number_as_f64(base)?;
            let value_f = number_as_f64(value)?;
            let diff = if direction == "lower" {
                value_f - base_f
            } else {
                base_f - value_f
            };
            let scaled = diff / base_f.abs() * 100.0;
            Ok(gt_margin(scaled, margin))
        }
    }
}

fn number_as_f64(value: &Value) -> Result<f64, RegError> {
    match value {
        Value::Int(digits) => int_to_f64(digits).ok_or(RegError::Crash),
        Value::Float(number) => Ok(*number),
        _ => Err(RegError::Crash),
    }
}

fn gt_margin(scaled: f64, margin: &Value) -> bool {
    match margin {
        Value::Int(digits) => {
            if scaled.is_infinite() {
                return scaled > 0.0;
            }
            cmp_f64_int(scaled, digits) == std::cmp::Ordering::Greater
        }
        Value::Float(number) => scaled > *number,
        _ => false,
    }
}

/// The VL-9 rule: (suggestion, regressed, unmeasured).
pub fn suggested_verdict(
    opened: &Value,
    result: &Value,
) -> Result<(&'static str, Vec<Text>, Vec<Text>), RegError> {
    let baseline = opened.get("baseline_measures").unwrap_or(&Value::Null);
    let measures = result.get("measures").unwrap_or(&Value::Null);
    let default_margin = opened.get("margin_pct").cloned();
    let mut regressed_names: Vec<Text> = Vec::new();
    let mut unmeasured: Vec<Text> = Vec::new();
    if let Some(Value::Array(entries)) = opened.get("judged") {
        for entry in entries {
            let name = match entry.get("name").and_then(as_str_text) {
                Some(name) => name.clone(),
                None => continue,
            };
            let value = match obj_get(measures, &name) {
                Some(value) => value,
                None => {
                    unmeasured.push(name);
                    continue;
                }
            };
            let margin = entry
                .get("margin_pct")
                .cloned()
                .or_else(|| default_margin.clone())
                .unwrap_or(Value::Int("10".to_string()));
            let base = obj_get(baseline, &name).ok_or(RegError::Crash)?;
            let direction = match entry.get("direction").and_then(as_str_text) {
                Some(text) if text_eq(text, "higher") => "higher",
                _ => "lower",
            };
            if regressed(direction, base, value, &margin)? {
                regressed_names.push(name);
            }
        }
    }
    let suggestion = if regressed_names.is_empty() {
        "keep"
    } else if regressed_names.len() >= 2 {
        "kill"
    } else {
        "rerun"
    };
    Ok((suggestion, regressed_names, unmeasured))
}

fn require_sound(path: &RegPath) -> Result<Vec<(usize, Value)>, RegError> {
    let (events, defects) = load_registry(path)?;
    if !defects.is_empty() {
        let mut msg = Msg::str("the registry has defects: ");
        msg.push_msg(&join_msgs(&defects, "; "));
        return Err(named_msg(msg));
    }
    Ok(events)
}

fn next_position(path: &RegPath) -> usize {
    let raw = match std::fs::read(&path.os) {
        Ok(raw) => raw,
        Err(_) => return 1,
    };
    let body = if raw.starts_with(&[0xEF, 0xBB, 0xBF]) {
        &raw[3..]
    } else {
        &raw[..]
    };
    let text = match String::from_utf8(body.to_vec()) {
        Ok(text) => text,
        Err(_) => return 1,
    };
    let mut lines: Vec<&str> = text.split('\n').collect();
    if lines.last() == Some(&"") {
        lines.pop();
    }
    lines.len() + 1
}

fn append(path: &RegPath, event: &Value) -> Result<(), RegError> {
    let outcome = (|| -> Result<Result<(), RegError>, std::io::Error> {
        let mut prefix: &[u8] = b"";
        let mut tail = [0u8; 1];
        if let Ok(metadata) = std::fs::metadata(&path.os) {
            if metadata.len() > 0 {
                let mut handle = std::fs::File::open(&path.os)?;
                handle.seek(SeekFrom::End(-1))?;
                handle.read_exact(&mut tail)?;
                if tail != *b"\n" {
                    prefix = b"\n";
                }
            }
        }
        let mut handle = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&path.os)?;
        // The Python renders its line before opening the
        // file but writes through the text layer, so a
        // line holding a lone surrogate fails at the
        // write, after the open has created the file,
        // and nothing lands. The port renders at the
        // same point in the sequence: after the open,
        // before any byte is written.
        let line = match dumps_line(event, Mode::Strict) {
            Ok(line) => line,
            Err(_) => return Ok(Err(RegError::Crash)),
        };
        let mut bytes = Vec::with_capacity(prefix.len() + line.len() + 1);
        bytes.extend_from_slice(prefix);
        bytes.extend_from_slice(&line);
        bytes.push(b'\n');
        handle.write_all(&bytes)?;
        handle.flush()?;
        Ok(Ok(()))
    })();
    match outcome {
        Ok(inner) => inner,
        Err(err) => {
            let mut msg = Msg::str("cannot append to the registry at ");
            msg.push_msg(&path.display);
            msg.push_str(": ");
            msg.push_msg(&os_exc(&err, Some(&path.display)));
            Err(named_msg(msg))
        }
    }
}

fn refusal_defects(defects: Vec<Msg>) -> RegError {
    named_msg(join_msgs(&defects, "; "))
}

pub fn open_experiment(path: &RegPath, event: &Value) -> Result<(), RegError> {
    let events = if path.os.exists() {
        require_sound(path)?
    } else {
        Vec::new()
    };
    let defects = event_defects(event, next_position(path));
    if !defects.is_empty() {
        return Err(refusal_defects(defects));
    }
    let states = fold(&events);
    let experiment = experiment_of(event);
    if states
        .iter()
        .any(|state| state.experiment.units() == experiment.units())
    {
        let mut msg = Msg::str("experiment ");
        msg.push_text(&experiment);
        msg.push_str(" is already open");
        return Err(named_msg(msg));
    }
    append(path, event)
}

pub fn add_result(path: &RegPath, event: &Value) -> Result<(), RegError> {
    let events = require_sound(path)?;
    let defects = event_defects(event, next_position(path));
    if !defects.is_empty() {
        return Err(refusal_defects(defects));
    }
    let states = fold(&events);
    let experiment = experiment_of(event);
    let state = match states
        .iter()
        .find(|state| state.experiment.units() == experiment.units())
    {
        Some(state) => state,
        None => {
            let mut msg = Msg::str("experiment ");
            msg.push_text(&experiment);
            msg.push_str(" is not open");
            return Err(named_msg(msg));
        }
    };
    if state.closed.is_some() {
        let mut msg = Msg::str("experiment ");
        msg.push_text(&experiment);
        msg.push_str(" is already closed");
        return Err(named_msg(msg));
    }
    append(path, event)
}

pub fn close_experiment(
    path: &RegPath,
    event: &Value,
    judge: Option<&dyn Transport>,
) -> Result<Value, RegError> {
    let events = require_sound(path)?;
    let states = fold(&events);
    let experiment = experiment_of(event);
    let state = match states
        .iter()
        .find(|state| state.experiment.units() == experiment.units())
    {
        Some(state) => state,
        None => {
            let mut msg = Msg::str("experiment ");
            msg.push_text(&experiment);
            msg.push_str(" is not open");
            return Err(named_msg(msg));
        }
    };
    if state.closed.is_some() {
        let mut msg = Msg::str("experiment ");
        msg.push_text(&experiment);
        msg.push_str(" is already closed");
        return Err(named_msg(msg));
    }
    if state.results.is_empty() {
        let mut msg = Msg::str("experiment ");
        msg.push_text(&experiment);
        msg.push_str(" has no result to rest a verdict on");
        return Err(named_msg(msg));
    }
    let (result_position, result_event) = state.results.last().expect("checked");
    let (suggestion, regressed_names, unmeasured) = suggested_verdict(&state.opened, result_event)?;
    let mut full = event.clone();
    let rests = object(vec![
        ("baseline", Value::Int(state.opened_at_position.to_string())),
        ("result", Value::Int(result_position.to_string())),
    ]);
    set_key(&mut full, "rests_on", rests);
    set_key(
        &mut full,
        "suggested",
        Value::Str(Text::from_str(suggestion)),
    );
    set_key(
        &mut full,
        "regressed",
        Value::Array(
            regressed_names
                .iter()
                .map(|name| Value::Str(name.clone()))
                .collect(),
        ),
    );
    set_key(
        &mut full,
        "unmeasured",
        Value::Array(
            unmeasured
                .iter()
                .map(|name| Value::Str(name.clone()))
                .collect(),
        ),
    );
    let defects = event_defects(&full, next_position(path));
    if !defects.is_empty() {
        return Err(refusal_defects(defects));
    }
    // The judgment (VL-20 to VL-24): every
    // deterministic refusal above precedes any call,
    // as it precedes the close itself. An abstention
    // refuses the close and nothing is appended; the
    // experiment stays open.
    let verdict = VERDICTS
        .iter()
        .copied()
        .find(|name| str_in(event.get("verdict").unwrap_or(&Value::Null), &[name]))
        .expect("a checked verdict");
    let judgment = match judge::consult(
        &state.opened,
        result_event,
        verdict,
        suggestion,
        &regressed_names,
        &unmeasured,
        judge,
    ) {
        Ok(Consult::Record(judgment)) => judgment,
        Ok(Consult::Refuse(message)) => return Err(named(&message)),
        Err(_) => return Err(RegError::Crash),
    };
    set_key(&mut full, "judgment", judgment);
    let defects = event_defects(&full, next_position(path));
    if !defects.is_empty() {
        // The recorded judgment is built to the
        // catalogue's own shapes; a defect here is
        // the port's failure, not the caller's.
        return Err(RegError::Crash);
    }
    append(path, &full)?;
    Ok(full)
}

pub fn list_view(path: &RegPath) -> Result<Value, RegError> {
    let events = require_sound(path)?;
    let states = fold(&events);
    let mut entries = Vec::new();
    for state in &states {
        let opened = &state.opened;
        let mut entry = object(vec![
            ("experiment", Value::Str(state.experiment.clone())),
            (
                "source",
                opened.get("source").cloned().unwrap_or(Value::Null),
            ),
            (
                "proposal",
                opened.get("proposal").cloned().unwrap_or(Value::Null),
            ),
            ("state", Value::Str(Text::from_str(state.state_name()))),
            ("results", Value::Int(state.results.len().to_string())),
        ]);
        if let Some(closed) = &state.closed {
            let verdict = closed.get("verdict").cloned().unwrap_or(Value::Null);
            let suggested = closed.get("suggested").cloned().unwrap_or(Value::Null);
            let agrees = match (&verdict, &suggested) {
                (Value::Str(a), Value::Str(b)) => a.units() == b.units(),
                _ => false,
            };
            set_key(&mut entry, "verdict", verdict);
            set_key(&mut entry, "suggested", suggested);
            set_key(&mut entry, "agrees", Value::Bool(agrees));
            if let Some(judgment) = closed.get("judgment") {
                set_key(&mut entry, "judgment", judgment.clone());
            }
        }
        entries.push(entry);
    }
    Ok(object(vec![("experiments", Value::Array(entries))]))
}

/// Assign a key on an object value the way Python's
/// dict assignment does: an existing key keeps its
/// position with the new value, a new key appends.
fn set_key_text(obj: &mut Value, key: &Text, value: Value) {
    if let Value::Object(pairs) = obj {
        for (name, existing) in pairs.iter_mut() {
            if name.units() == key.units() {
                *existing = value;
                return;
            }
        }
        pairs.push((key.clone(), value));
    }
}

fn with_position(position: usize, event: &Value) -> Value {
    let mut out = object(vec![("position", Value::Int(position.to_string()))]);
    if let Value::Object(pairs) = event {
        for (key, value) in pairs {
            set_key_text(&mut out, key, value.clone());
        }
    }
    out
}

pub fn show_view(path: &RegPath, experiment: &Text) -> Result<Value, RegError> {
    let events = require_sound(path)?;
    let states = fold(&events);
    let state = match states
        .iter()
        .find(|state| state.experiment.units() == experiment.units())
    {
        Some(state) => state,
        None => {
            let mut msg = Msg::str("no experiment ");
            msg.push_text(experiment);
            return Err(named_msg(msg));
        }
    };
    let closed = match (&state.closed, state.closed_at_position) {
        (Some(event), Some(position)) => with_position(position, event),
        _ => Value::Null,
    };
    Ok(object(vec![
        ("experiment", Value::Str(state.experiment.clone())),
        ("state", Value::Str(Text::from_str(state.state_name()))),
        (
            "opened",
            with_position(state.opened_at_position, &state.opened),
        ),
        (
            "results",
            Value::Array(
                state
                    .results
                    .iter()
                    .map(|(position, event)| with_position(*position, event))
                    .collect(),
            ),
        ),
        ("closed", closed),
    ]))
}
