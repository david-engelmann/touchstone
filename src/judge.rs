//! VL-20 to VL-26. The decision card, the judge's
//! three calls, and the two transports that answer
//! them. The shape is secondtake's judge (ST-13 to
//! ST-19) re-implemented in this crate, as VL-21
//! provides: the same transport shape and
//! configuration surface, verdictloop's own card,
//! labels, and score questions.

use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use flighttape::identity::tape_digest;
use flighttape::json::{parse, ParseError};
use flighttape::value::{object, Text, Value};

use crate::emit::py_float;
use crate::util::json_text_depths;

pub const MODEL: &str = "jev-1.13.0";
pub const LABELS: [&str; 4] = ["keep", "kill", "rerun", "abstain"];
pub const CONFIDENCE_FLOOR: f64 = 0.6;

const FRAMING_EVIDENCE: &str = "\n\nRate the strength of the evidence \
behind the recorded verdict, on the scale given.";
const FRAMING_RISK: &str = "\n\nRate the risk that the measures that \
regressed make the recorded verdict wrong, on the scale given.";

#[derive(Debug, Clone, PartialEq)]
pub struct JudgeError(pub String);

impl std::fmt::Display for JudgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for JudgeError {}

fn err(message: impl Into<String>) -> JudgeError {
    JudgeError(message.into())
}

fn text(value: &str) -> Value {
    Value::Str(Text::from_str(value))
}

/// The numeric value of an int or float as an f64,
/// for the judge's range checks: a huge integer
/// becomes an infinity, which fails every bound.
/// (The registry module's own converter answers the
/// same question with the crash class; the judge's
/// answers are model output, not registry state, so
/// the range check is the whole story here.)
fn number_as_f64(value: &Value) -> Option<f64> {
    match value {
        Value::Int(digits) => digits.parse::<f64>().ok(),
        Value::Float(number) => Some(*number),
        _ => None,
    }
}

fn int_is_negative(digits: &str) -> bool {
    digits.starts_with('-') && digits != "-0"
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

/// A text's plain spelling for the card: pairs
/// decode, and a lone surrogate is spelled as its
/// JSON escape, so the card string is always
/// well-formed and rebuilds identically.
fn plain(text: &Text) -> String {
    let units = text.units();
    let mut out = String::new();
    let mut index = 0;
    while index < units.len() {
        let unit = units[index];
        if (0xD800..=0xDBFF).contains(&unit)
            && index + 1 < units.len()
            && (0xDC00..=0xDFFF).contains(&units[index + 1])
        {
            let scalar = crate::util::decode_pair(unit, units[index + 1]);
            out.push(char::from_u32(scalar).expect("a valid pair"));
            index += 2;
        } else if (0xD800..=0xDFFF).contains(&unit) {
            out.push_str(&format!("\\u{unit:04x}"));
            index += 1;
        } else {
            out.push(char::from_u32(unit as u32).expect("a BMP scalar"));
            index += 1;
        }
    }
    out
}

/// How a value prints inside a sentence: the
/// Python's str() of the stored number.
pub fn value_text(value: &Value) -> String {
    match value {
        Value::Int(digits) => digits.clone(),
        Value::Float(number) => py_float(*number),
        _ => unreachable!("sentences carry numbers only"),
    }
}

/// The decision card (VL-22): a deterministic
/// rendering of the computed state the close rests
/// on, so the check fold rebuilds it byte for byte
/// (VL-26). Its outcome words are the suggestion
/// rule's own two (regressed, within margin); the
/// card adds no arithmetic of its own.
pub fn build_card(
    opened: &Value,
    result: &Value,
    verdict: &str,
    suggestion: &str,
    regressed: &[Text],
    unmeasured: &[Text],
) -> String {
    let experiment = opened.get("experiment").expect("an opening names itself");
    let proposal = opened
        .get("proposal")
        .expect("an opening names its proposal");
    let (experiment, proposal) = match (experiment, proposal) {
        (Value::Str(experiment), Value::Str(proposal)) => (plain(experiment), plain(proposal)),
        _ => unreachable!("an opening's names are text"),
    };
    let baseline = opened.get("baseline_measures").unwrap_or(&Value::Null);
    let measures = result.get("measures").unwrap_or(&Value::Null);
    let default_margin = opened.get("margin_pct").cloned();
    let mut lines: Vec<String> = Vec::new();
    lines.push(format!("Experiment {experiment}: {proposal}."));
    if let Some(Value::Array(entries)) = opened.get("judged") {
        for entry in entries {
            let name = match entry.get("name") {
                Some(Value::Str(name)) => name.clone(),
                _ => continue,
            };
            if unmeasured.iter().any(|item| item.units() == name.units()) {
                lines.push(format!(
                    "{}: unmeasured, the result records no value for it.",
                    plain(&name)
                ));
                continue;
            }
            let direction = match entry.get("direction") {
                Some(Value::Str(direction)) => plain(direction),
                _ => "lower".to_string(),
            };
            let margin = entry
                .get("margin_pct")
                .cloned()
                .or_else(|| default_margin.clone())
                .unwrap_or(Value::Int("10".to_string()));
            let base = obj_get(baseline, &name).expect("a judged measure rests on the baseline");
            let current = obj_get(measures, &name).expect("a measured entry has a current value");
            let outcome = if regressed.iter().any(|item| item.units() == name.units()) {
                "regressed"
            } else {
                "within margin"
            };
            lines.push(format!(
                "{}: baseline {}, current {}, direction {}, margin {} percent: {outcome}.",
                plain(&name),
                value_text(base),
                value_text(current),
                direction,
                value_text(&margin),
            ));
        }
    }
    lines.push(format!(
        "Regressed judged measures: {}.",
        if regressed.is_empty() {
            "none".to_string()
        } else {
            regressed.iter().map(plain).collect::<Vec<_>>().join(", ")
        }
    ));
    lines.push(format!(
        "The author recorded verdict {verdict}; the deterministic rule suggests {suggestion}."
    ));
    lines.join("\n")
}

fn answer_decision(response: &Value) -> Result<Value, JudgeError> {
    let model_ok = matches!(response.get("model"), Some(Value::Str(m)) if m.as_str().as_deref() == Some(MODEL));
    if !model_ok {
        return Err(err("the judge's answer names an unpinned model"));
    }
    let answers = match response.get("answers") {
        Some(answers) if answers.as_object().is_some() => answers,
        _ => return Err(err("the judge's answer carries no answers")),
    };
    match answers.get("decision") {
        Some(decision) if decision.as_object().is_some() => Ok(decision.clone()),
        _ => Err(err("the judge's answer carries no decision")),
    }
}

fn bounded(value: Option<&Value>, low: f64, high: f64) -> bool {
    match value.and_then(number_as_f64) {
        Some(number) => number.is_finite() && number >= low && number <= high,
        None => false,
    }
}

/// Whether a validated confidence sits below the
/// floor: ints compare exactly, floats as floats.
pub fn confidence_below_floor(confidence: &Value) -> bool {
    match confidence {
        Value::Int(digits) => int_is_negative(digits) || digits == "0",
        Value::Float(number) => *number < CONFIDENCE_FLOOR,
        _ => false,
    }
}

/// A judge transport: answers a card with the
/// recorded response shapes, or raises JudgeError.
pub trait Transport {
    fn ask(&self, card: &str, kind: &str) -> Result<Value, JudgeError>;
}

/// The three calls and their parsed answers. Any
/// malformed answer is unavailable, never repaired
/// in flight.
pub fn judge_card(card: &str, transport: &dyn Transport) -> Result<Value, JudgeError> {
    let decision = answer_decision(&transport.ask(card, "choice")?)?;
    let choice = decision.get("choice");
    let confidence = decision.get("confidence");
    let probabilities = decision.get("probabilities");
    let choice_label = match choice {
        Some(Value::Str(label)) => label.as_str(),
        _ => None,
    };
    let probabilities_ok = match probabilities {
        Some(probabilities) if probabilities.as_object().is_some() => {
            let entries = probabilities.as_object().expect("checked object");
            let keys_match = entries.len() == LABELS.len()
                && LABELS.iter().all(|label| {
                    entries
                        .iter()
                        .any(|(key, _)| key.as_str().as_deref() == Some(label))
                });
            keys_match
                && LABELS
                    .iter()
                    .all(|label| bounded(probabilities.get(label), 0.0, 1.0))
                && {
                    let sum: f64 = entries
                        .iter()
                        .filter_map(|(_, value)| number_as_f64(value))
                        .sum();
                    (sum - 1.0).abs() <= 0.02
                }
        }
        _ => false,
    };
    let choice_ok = matches!(decision.get("type"), Some(Value::Str(t)) if t.as_str().as_deref() == Some("choice"))
        && matches!(choice_label.as_deref(), Some(label) if LABELS.contains(&label))
        && bounded(confidence, 0.0, 1.0)
        && probabilities_ok;
    if !choice_ok {
        return Err(err("the judge's choice answer is malformed"));
    }
    let mut scores: Vec<(String, Value)> = Vec::new();
    for (kind, key) in [
        ("evidence", "evidence_strength"),
        ("risk", "regression_risk"),
    ] {
        let decision = answer_decision(&transport.ask(card, kind)?)?;
        let score_ok = matches!(decision.get("type"), Some(Value::Str(t)) if t.as_str().as_deref() == Some("score"))
            && bounded(decision.get("score"), 0.0, 10.0);
        if !score_ok {
            return Err(err(format!("the judge's {kind} answer is malformed")));
        }
        scores.push((
            key.to_string(),
            decision
                .get("score")
                .expect("a bounded score exists")
                .clone(),
        ));
    }
    Ok(object(vec![
        ("model", text(MODEL)),
        ("choice", choice.expect("a valid choice exists").clone()),
        (
            "confidence",
            confidence.expect("a bounded confidence exists").clone(),
        ),
        (
            "probabilities",
            probabilities.expect("valid probabilities exist").clone(),
        ),
        ("evidence_strength", scores[0].1.clone()),
        ("regression_risk", scores[1].1.clone()),
    ]))
}

/// The outcome of consulting the judge at close
/// time (VL-20, VL-21, VL-24).
pub enum Consult {
    /// The judgment section the close records.
    Record(Value),
    /// The close does not land; the refusal names
    /// the abstention.
    Refuse(String),
}

/// The band logic of VL-20 to VL-24, given the
/// computed suggestion: agreement never consults,
/// a contested close with no judge closes as
/// before, and a consulted judge's abstention (the
/// choice itself, or a confidence under the floor)
/// refuses the close. The only Err is a card that
/// cannot be digested, which no recorded state
/// produces.
pub fn consult(
    opened: &Value,
    result: &Value,
    verdict: &str,
    suggestion: &str,
    regressed: &[Text],
    unmeasured: &[Text],
    judge: Option<&dyn Transport>,
) -> Result<Consult, JudgeError> {
    if verdict == suggestion {
        return Ok(Consult::Record(object(vec![
            ("status", text("not_consulted")),
            ("reason", text("the verdict and the suggestion agree")),
        ])));
    }
    let judge = match judge {
        Some(judge) => judge,
        None => {
            return Ok(Consult::Record(object(vec![
                ("status", text("not_consulted")),
                ("reason", text("no judge is configured")),
            ])));
        }
    };
    let card = build_card(opened, result, verdict, suggestion, regressed, unmeasured);
    let card_digest = tape_digest(&Value::Str(Text::from_str(&card)))
        .map_err(|_| err("the card cannot be digested"))?;
    let mut judgment: Vec<(Text, Value)> = vec![
        (Text::from_str("card_sha256"), text(&card_digest)),
        (Text::from_str("card"), text(&card)),
    ];
    let answers = match judge_card(&card, judge) {
        Ok(answers) => answers,
        Err(error) => {
            judgment.push((Text::from_str("status"), text("unavailable")));
            judgment.push((Text::from_str("reason"), text(&error.0)));
            return Ok(Consult::Record(Value::Object(judgment)));
        }
    };
    if let Value::Object(fields) = &answers {
        for (key, value) in fields {
            judgment.push((key.clone(), value.clone()));
        }
    }
    let choice = answers
        .get("choice")
        .and_then(|choice| match choice {
            Value::Str(label) => label.as_str(),
            _ => None,
        })
        .unwrap_or_default();
    if choice == "abstain" {
        return Ok(Consult::Refuse(
            "the judge abstained; the close does not land".to_string(),
        ));
    }
    let confidence = answers.get("confidence").cloned().unwrap_or(Value::Null);
    if confidence_below_floor(&confidence) {
        return Ok(Consult::Refuse(format!(
            "the judge's confidence {} is below 0.6; the close does not land",
            value_text(&confidence)
        )));
    }
    judgment.push((Text::from_str("status"), text("judged")));
    Ok(Consult::Record(Value::Object(judgment)))
}

/// Replays recorded responses, keyed by the card's
/// digest.
#[derive(Debug)]
pub struct FixtureTransport {
    by_digest: HashMap<String, Value>,
}

impl FixtureTransport {
    pub fn load(path: &Path, display: &str) -> Result<FixtureTransport, JudgeError> {
        let raw = match std::fs::read(path) {
            Ok(raw) => raw,
            Err(_) => {
                return Err(err(format!("no judgments file at {display}")));
            }
        };
        let data = match String::from_utf8(raw).map_err(|_| ()).and_then(|text| {
            // The Python's reader refuses nesting
            // past its recursion boundary (probed
            // through this surface in secondtake: a
            // text depth of 9991 reads, 9992 refuses);
            // the crate's iterative reader accepts
            // it, so the cap is applied to the text.
            if json_text_depths(&text).0 > 9991 {
                return Err(());
            }
            parse(&text).map_err(|_| ())
        }) {
            Ok(data) => data,
            Err(_) => {
                return Err(err(format!(
                    "the judgments file at {display} is not valid JSON"
                )));
            }
        };
        let cards = match data.get("cards") {
            Some(cards) if cards.as_array().is_some() => {
                cards.as_array().expect("checked array").clone()
            }
            _ => {
                return Err(err("the judgments file carries no cards list"));
            }
        };
        let mut by_digest: HashMap<String, Value> = HashMap::new();
        for entry in &cards {
            let (digest, responses) = match (entry.get("card_sha256"), entry.get("responses")) {
                (Some(Value::Str(digest)), Some(responses)) if responses.as_object().is_some() => {
                    (digest, responses.clone())
                }
                _ => {
                    return Err(err(
                        "a judgments entry carries no card digest or no responses",
                    ));
                }
            };
            let digest = match digest.as_str() {
                Some(digest) => digest,
                None => {
                    return Err(err(
                        "a judgments entry carries no card digest or no responses",
                    ));
                }
            };
            if by_digest.contains_key(&digest) {
                return Err(err("the judgments file records a card digest twice"));
            }
            if let Some(card) = entry.get("card") {
                let digests = match card {
                    Value::Str(card_text) => tape_digest(&Value::Str(card_text.clone())).ok(),
                    _ => None,
                };
                if digests.as_deref() != Some(digest.as_str()) {
                    return Err(err(
                        "a recorded card does not digest to the digest it is filed under",
                    ));
                }
            }
            by_digest.insert(digest.to_string(), responses);
        }
        Ok(FixtureTransport { by_digest })
    }
}

impl Transport for FixtureTransport {
    fn ask(&self, card: &str, kind: &str) -> Result<Value, JudgeError> {
        let digest = tape_digest(&Value::Str(Text::from_str(card)))
            .map_err(|_| err("the card cannot be digested"))?;
        let responses = match self.by_digest.get(&digest) {
            Some(responses) => responses,
            None => {
                return Err(err("the card is not recorded in the judgments file"));
            }
        };
        match responses.get(kind) {
            Some(response) if response.as_object().is_some() => Ok(response.clone()),
            _ => Err(err(format!("the recorded {kind} answer is missing"))),
        }
    }
}

/// Calls Jev through the connector's call script.
/// The script path is an OS string, passed to the
/// process uninterpreted, as the Python's own
/// argument passing does.
#[derive(Debug)]
pub struct LiveTransport {
    script: std::ffi::OsString,
}

impl LiveTransport {
    pub fn new(script: std::ffi::OsString) -> LiveTransport {
        LiveTransport { script }
    }
}

impl Transport for LiveTransport {
    fn ask(&self, card: &str, kind: &str) -> Result<Value, JudgeError> {
        let framing = match kind {
            "evidence" => FRAMING_EVIDENCE,
            "risk" => FRAMING_RISK,
            _ => "",
        };
        let state = format!("{card}{framing}");
        let mut command = Command::new("python3");
        command
            .arg(&self.script)
            .arg("--state")
            .arg(&state)
            .arg("--labels")
            .arg(LABELS.join(","));
        if kind != "choice" {
            command.arg("--question-type").arg("score");
        }
        command.stdout(Stdio::piped()).stderr(Stdio::piped());
        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(_) => return Err(err("the judge call could not be made")),
        };
        // Both streams drain on their own threads from
        // the start, as the Python's subprocess.run
        // drains them: a helper that writes more than
        // a pipe buffer before exiting never blocks.
        let mut stdout_pipe = child.stdout.take().expect("stdout is piped");
        let out_thread = std::thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = stdout_pipe.read_to_end(&mut buf);
            buf
        });
        let mut stderr_pipe = child.stderr.take().expect("stderr is piped");
        let err_thread = std::thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = stderr_pipe.read_to_end(&mut buf);
            buf
        });
        let deadline = Instant::now() + Duration::from_secs(180);
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) => {
                    if Instant::now() >= deadline {
                        let _ = child.kill();
                        let _ = child.wait();
                        return Err(err("the judge call could not be made"));
                    }
                    std::thread::sleep(Duration::from_millis(5));
                }
                Err(_) => return Err(err("the judge call could not be made")),
            }
        };
        let out = out_thread.join().unwrap_or_default();
        let err_out = err_thread.join().unwrap_or_default();
        // The Python decodes both streams inside
        // subprocess.run, before it ever looks at the
        // return code: undecodable output in either
        // stream is "could not be read", outranking a
        // failed call.
        let text_out = match String::from_utf8(out) {
            Ok(text) => text,
            Err(_) => return Err(err("the judge's answer could not be read")),
        };
        if String::from_utf8(err_out).is_err() {
            return Err(err("the judge's answer could not be read"));
        }
        if !status.success() {
            return Err(err("the judge call failed"));
        }
        match parse(&text_out) {
            Ok(value) => Ok(value),
            Err(ParseError::TooLarge) | Err(ParseError::Invalid) => {
                Err(err("the judge's answer is not JSON"))
            }
        }
    }
}
