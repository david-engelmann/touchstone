//! VL-12. A thin command line: open, result, close,
//! list, show, check. Failures are named errors with
//! exit 1. The parse reproduces the Python's argparse
//! behavior for this grammar byte for byte: its help
//! texts, its error shapes, and the order they fire
//! in. Help fires at the end of the token that carries
//! it; a scan error (a missing option value, an
//! ignored explicit argument, an ambiguous long
//! option, an invalid float) fires when its token is
//! processed and outranks a pending help; extras are
//! only reported after the required check. Argument
//! tokens are carried as text in the operating
//! system's own decoding (fsdecode), so argument
//! bytes survive as the Python's do.

use std::io::Write;
use std::path::{Path, PathBuf};

use flighttape::value::{object, Text, Value};

use crate::emit::{dumps_view, Mode};
use crate::helptext::*;
use crate::judge::{FixtureTransport, JudgeError, LiveTransport, Transport};
use crate::pyrepr::repr_text;
use crate::registry::{
    add_result, close_experiment, list_view, load_registry, open_experiment, parse_arg_json,
    show_view, RegError, RegPath,
};
use crate::util::{fsdecode_units, fsencode_bytes, normalize_path_display, stderr_render};

/// One argument token: its text, and its decoded
/// scalars for the parser's character work.
struct Token {
    text: Text,
    scalars: Vec<u32>,
}

impl Token {
    fn from_units(units: Vec<u16>) -> Token {
        let mut scalars = Vec::new();
        let mut index = 0;
        while index < units.len() {
            let unit = units[index];
            if (0xD800..=0xDBFF).contains(&unit)
                && index + 1 < units.len()
                && (0xDC00..=0xDFFF).contains(&units[index + 1])
            {
                scalars.push(crate::util::decode_pair(unit, units[index + 1]));
                index += 2;
            } else {
                scalars.push(unit as u32);
                index += 1;
            }
        }
        Token {
            text: Text::from_units(units),
            scalars,
        }
    }

    fn rendered(&self) -> String {
        stderr_render(self.text.units())
    }
}

/// Whether a token is shaped like an option: it starts
/// with a dash, is not a bare dash, and does not look
/// like a negative number (no option in this grammar
/// does, so argparse lets those through as values).
fn option_shaped(token: &Token) -> bool {
    let scalars = &token.scalars;
    if scalars.len() < 2 || scalars[0] != '-' as u32 {
        return false;
    }
    !is_negative_number(&scalars[1..])
}

/// argparse's negative number matcher, the full token
/// pattern ^-\d+$|^-\d*\.\d+$ with \d a Unicode
/// decimal digit: a token matching it whole is a
/// value, never an option.
fn is_negative_number(rest: &[u32]) -> bool {
    let digit = |c: &u32| crate::util::decimal_digit(*c).is_some();
    if !rest.is_empty() && rest.iter().all(digit) {
        return true;
    }
    match rest.iter().position(|c| *c == '.' as u32) {
        Some(dot) => {
            let (before, after) = (&rest[..dot], &rest[dot + 1..]);
            !after.is_empty() && before.iter().all(digit) && after.iter().all(digit)
        }
        None => false,
    }
}

fn scalars_of(text: &str) -> Vec<u32> {
    text.chars().map(|ch| ch as u32).collect()
}

/// Split a long-option token at its first '=': the
/// name's scalars, and the glued value's text when an
/// '=' is present.
fn split_long(token: &Token) -> (&[u32], Option<Text>) {
    match token.scalars.iter().position(|c| *c == '=' as u32) {
        Some(at) => {
            let units = token.text.units();
            let mut unit_at = 0usize;
            let mut scalar_at = 0usize;
            while scalar_at < at {
                if (0xD800..=0xDBFF).contains(&units[unit_at])
                    && unit_at + 1 < units.len()
                    && (0xDC00..=0xDFFF).contains(&units[unit_at + 1])
                {
                    unit_at += 2;
                } else {
                    unit_at += 1;
                }
                scalar_at += 1;
            }
            (
                &token.scalars[..at],
                Some(Text::from_units(units[unit_at + 1..].to_vec())),
            )
        }
        None => (&token.scalars[..], None),
    }
}

/// The text of a token from a scalar index on, for
/// the value glued behind an '=' in a short cluster.
fn text_from_scalar(token: &Token, scalar_index: usize) -> Text {
    let units = token.text.units();
    let mut unit_at = 0usize;
    let mut scalar_at = 0usize;
    while scalar_at < scalar_index && unit_at < units.len() {
        if (0xD800..=0xDBFF).contains(&units[unit_at])
            && unit_at + 1 < units.len()
            && (0xDC00..=0xDFFF).contains(&units[unit_at + 1])
        {
            unit_at += 2;
        } else {
            unit_at += 1;
        }
        scalar_at += 1;
    }
    Text::from_units(units[unit_at.min(units.len())..].to_vec())
}

/// The outcome of matching a long-option name against
/// a level's option set.
enum LongMatch {
    Exact(&'static str),
    Unique(&'static str),
    Ambiguous(Vec<&'static str>),
    None,
}

fn match_long(name: &[u32], options: &[&'static str]) -> LongMatch {
    let mut candidates: Vec<&'static str> = Vec::new();
    for option in options {
        let full = scalars_of(option);
        if full == name {
            return LongMatch::Exact(option);
        }
        if full.len() > name.len() && full[..name.len()] == *name {
            candidates.push(option);
        }
    }
    match candidates.len() {
        0 => LongMatch::None,
        1 => LongMatch::Unique(candidates[0]),
        _ => LongMatch::Ambiguous(candidates),
    }
}

enum ParseOutcome {
    Run(&'static str, Box<SubParsed>),
    Exit(i32),
}

fn parse_error(usage: &str, prog: &str, message: &str) -> ParseOutcome {
    eprintln!("{usage}\n{prog}: error: {message}");
    ParseOutcome::Exit(2)
}

#[derive(Clone)]
enum ParsedValue {
    Text(Text),
    Float(f64),
}

struct SubParsed {
    positionals: Vec<Text>,
    values: Vec<(&'static str, ParsedValue)>,
    output: Option<Text>,
}

impl SubParsed {
    fn get_text(&self, canonical: &str) -> Option<Text> {
        self.values.iter().rev().find_map(|(name, value)| {
            if *name == canonical {
                match value {
                    ParsedValue::Text(text) => Some(text.clone()),
                    _ => None,
                }
            } else {
                None
            }
        })
    }

    fn get_float(&self, canonical: &str) -> Option<f64> {
        self.values.iter().rev().find_map(|(name, value)| {
            if *name == canonical {
                match value {
                    ParsedValue::Float(number) => Some(*number),
                    _ => None,
                }
            } else {
                None
            }
        })
    }
}

#[derive(Clone, Copy, PartialEq)]
enum OptKind {
    Help,
    Value,
    Float,
    Output,
}

struct OptSpec {
    canonical: &'static str,
    label: &'static str,
    kind: OptKind,
}

struct SubSpec {
    usage: &'static str,
    prog: &'static str,
    help: &'static str,
    positionals: &'static [&'static str],
    options: &'static [OptSpec],
    /// The required arguments in definition order:
    /// ("registry", true) names a positional by its
    /// index label, ("--source", false) an option.
    required: &'static [(&'static str, bool)],
    has_short_o: bool,
}

const HELP_OPT: OptSpec = OptSpec {
    canonical: "--help",
    label: "-h/--help",
    kind: OptKind::Help,
};

const OPEN_SPEC: SubSpec = SubSpec {
    usage: OPEN_USAGE,
    prog: "touchstone open",
    help: OPEN_HELP,
    positionals: &["registry", "experiment"],
    options: &[
        HELP_OPT,
        OptSpec {
            canonical: "--source",
            label: "--source",
            kind: OptKind::Value,
        },
        OptSpec {
            canonical: "--proposal",
            label: "--proposal",
            kind: OptKind::Value,
        },
        OptSpec {
            canonical: "--baseline",
            label: "--baseline",
            kind: OptKind::Value,
        },
        OptSpec {
            canonical: "--measures",
            label: "--measures",
            kind: OptKind::Value,
        },
        OptSpec {
            canonical: "--judged",
            label: "--judged",
            kind: OptKind::Value,
        },
        OptSpec {
            canonical: "--margin",
            label: "--margin",
            kind: OptKind::Float,
        },
        OptSpec {
            canonical: "--at",
            label: "--at",
            kind: OptKind::Value,
        },
    ],
    required: &[
        ("registry", true),
        ("experiment", true),
        ("--source", false),
        ("--proposal", false),
        ("--baseline", false),
        ("--measures", false),
        ("--judged", false),
        ("--at", false),
    ],
    has_short_o: false,
};

const RESULT_SPEC: SubSpec = SubSpec {
    usage: RESULT_USAGE,
    prog: "touchstone result",
    help: RESULT_HELP,
    positionals: &["registry", "experiment"],
    options: &[
        HELP_OPT,
        OptSpec {
            canonical: "--measures",
            label: "--measures",
            kind: OptKind::Value,
        },
        OptSpec {
            canonical: "--at",
            label: "--at",
            kind: OptKind::Value,
        },
    ],
    required: &[
        ("registry", true),
        ("experiment", true),
        ("--measures", false),
        ("--at", false),
    ],
    has_short_o: false,
};

const CLOSE_SPEC: SubSpec = SubSpec {
    usage: CLOSE_USAGE,
    prog: "touchstone close",
    help: CLOSE_HELP,
    positionals: &["registry", "experiment"],
    options: &[
        HELP_OPT,
        OptSpec {
            canonical: "--verdict",
            label: "--verdict",
            kind: OptKind::Value,
        },
        OptSpec {
            canonical: "--rationale",
            label: "--rationale",
            kind: OptKind::Value,
        },
        OptSpec {
            canonical: "--at",
            label: "--at",
            kind: OptKind::Value,
        },
        OptSpec {
            canonical: "--successor",
            label: "--successor",
            kind: OptKind::Value,
        },
        OptSpec {
            canonical: "--judge",
            label: "--judge",
            kind: OptKind::Value,
        },
    ],
    required: &[
        ("registry", true),
        ("experiment", true),
        ("--verdict", false),
        ("--rationale", false),
        ("--at", false),
    ],
    has_short_o: false,
};

const OUTPUT_OPT: OptSpec = OptSpec {
    canonical: "--output",
    label: "-o/--output",
    kind: OptKind::Output,
};

const LIST_SPEC: SubSpec = SubSpec {
    usage: LIST_USAGE,
    prog: "touchstone list",
    help: LIST_HELP,
    positionals: &["registry"],
    options: &[HELP_OPT, OUTPUT_OPT],
    required: &[("registry", true)],
    has_short_o: true,
};

const SHOW_SPEC: SubSpec = SubSpec {
    usage: SHOW_USAGE,
    prog: "touchstone show",
    help: SHOW_HELP,
    positionals: &["registry", "experiment"],
    options: &[HELP_OPT, OUTPUT_OPT],
    required: &[("registry", true), ("experiment", true)],
    has_short_o: true,
};

const CHECK_SPEC: SubSpec = SubSpec {
    usage: CHECK_USAGE,
    prog: "touchstone check",
    help: CHECK_HELP,
    positionals: &["registry"],
    options: &[HELP_OPT],
    required: &[("registry", true)],
    has_short_o: false,
};

fn spec_for(command: &str) -> &'static SubSpec {
    match command {
        "open" => &OPEN_SPEC,
        "result" => &RESULT_SPEC,
        "close" => &CLOSE_SPEC,
        "list" => &LIST_SPEC,
        "show" => &SHOW_SPEC,
        _ => &CHECK_SPEC,
    }
}

/// CPython's float() over an argument string, for
/// --margin: surrounding whitespace is stripped, the
/// words inf, infinity, and nan read in any case, and
/// a decimal numeral may carry single underscores
/// between its digits, which may be any Unicode
/// decimal digits. Anything else is not a float.
fn py_float_parse(text: &Text) -> Option<f64> {
    let plain = text.as_str()?;
    let trimmed = plain.trim();
    if trimmed.is_empty() {
        return None;
    }
    let (sign, body) = match trimmed.strip_prefix('-') {
        Some(rest) => (-1.0, rest),
        None => match trimmed.strip_prefix('+') {
            Some(rest) => (1.0, rest),
            None => (1.0, trimmed),
        },
    };
    // CPython reads each Unicode decimal digit at its
    // ASCII value; the numeral grammar below runs on
    // the normalized spelling.
    let normalized: String = body
        .chars()
        .map(|ch| match crate::util::decimal_digit(ch as u32) {
            Some(value) => char::from_digit(value, 10).expect("a decimal digit"),
            None => ch,
        })
        .collect();
    let body: &str = &normalized;
    let lowered = body.to_ascii_lowercase();
    if lowered == "inf" || lowered == "infinity" {
        return Some(sign * f64::INFINITY);
    }
    if lowered == "nan" {
        return Some(f64::NAN);
    }
    // A digit run: digit (underscore digit)*.
    fn digit_run(chars: &[char], at: &mut usize) -> bool {
        let start = *at;
        if *at >= chars.len() || !chars[*at].is_ascii_digit() {
            return false;
        }
        *at += 1;
        loop {
            if *at < chars.len() && chars[*at].is_ascii_digit() {
                *at += 1;
            } else if *at + 1 < chars.len() && chars[*at] == '_' && chars[*at + 1].is_ascii_digit()
            {
                *at += 2;
            } else {
                break;
            }
        }
        *at > start
    }
    let chars: Vec<char> = body.chars().collect();
    let mut at = 0usize;
    let int_ok = digit_run(&chars, &mut at);
    let mut frac_ok = false;
    if at < chars.len() && chars[at] == '.' {
        let mut probe = at + 1;
        frac_ok = digit_run(&chars, &mut probe);
        if int_ok || frac_ok {
            at = probe;
        }
    }
    if !int_ok && !frac_ok {
        return None;
    }
    if at < chars.len() && (chars[at] == 'e' || chars[at] == 'E') {
        let mut probe = at + 1;
        if probe < chars.len() && (chars[probe] == '+' || chars[probe] == '-') {
            probe += 1;
        }
        if !digit_run(&chars, &mut probe) {
            return None;
        }
        at = probe;
    }
    if at != chars.len() {
        return None;
    }
    let cleaned: String = body.chars().filter(|ch| *ch != '_').collect();
    cleaned.parse::<f64>().ok().map(|value| sign * value)
}

/// Take the value for an option: a glued form wins,
/// else the next token, which an option-shaped token
/// can never be. On success `index` still points at
/// the option token; the caller advances past both.
fn take_value(
    spec: &SubSpec,
    tokens: &[Token],
    index: &mut usize,
    glued: Option<Text>,
    opt: &OptSpec,
) -> Result<ParsedValue, ParseOutcome> {
    let value = if let Some(value) = glued {
        value
    } else {
        match tokens.get(*index + 1) {
            Some(next) if !option_shaped(next) => {
                *index += 1;
                next.text.clone()
            }
            _ => {
                return Err(parse_error(
                    spec.usage,
                    spec.prog,
                    &format!("argument {}: expected one argument", opt.label),
                ))
            }
        }
    };
    match opt.kind {
        OptKind::Float => match py_float_parse(&value) {
            Some(number) => Ok(ParsedValue::Float(number)),
            None => Err(parse_error(
                spec.usage,
                spec.prog,
                &format!(
                    "argument {}: invalid float value: {}",
                    opt.label,
                    repr_text(&value)
                ),
            )),
        },
        _ => Ok(ParsedValue::Text(value)),
    }
}

/// Parse one subcommand's tokens.
fn parse_sub(command: &'static str, tokens: &[Token], top_extras: &[Text]) -> ParseOutcome {
    let spec = spec_for(command);
    let candidates: Vec<&'static str> = spec.options.iter().map(|opt| opt.canonical).collect();
    let mut positionals: Vec<Text> = Vec::new();
    let mut extras: Vec<Text> = top_extras.to_vec();
    let mut values: Vec<(&'static str, ParsedValue)> = Vec::new();
    let mut separated = false;
    let mut index = 0usize;

    fn push_positional(
        token: &Text,
        spec: &SubSpec,
        positionals: &mut Vec<Text>,
        extras: &mut Vec<Text>,
    ) {
        if positionals.len() < spec.positionals.len() {
            positionals.push(token.clone());
        } else {
            extras.push(token.clone());
        }
    }

    while index < tokens.len() {
        let token = &tokens[index];
        if separated {
            push_positional(&token.text, spec, &mut positionals, &mut extras);
            index += 1;
            continue;
        }
        if token.scalars == scalars_of("--") {
            // The first "--" always ends option parsing,
            // but when every positional is already
            // filled argparse keeps the "--" itself
            // among the unrecognized arguments.
            separated = true;
            if positionals.len() == spec.positionals.len() {
                extras.push(token.text.clone());
            }
            index += 1;
            continue;
        }
        if token.scalars.len() >= 2
            && token.scalars[0] == '-' as u32
            && token.scalars[1] == '-' as u32
        {
            let (name, glued) = split_long(token);
            let chosen = match match_long(name, &candidates) {
                LongMatch::Exact(option) | LongMatch::Unique(option) => option,
                LongMatch::Ambiguous(matches) => {
                    return parse_error(
                        spec.usage,
                        spec.prog,
                        &format!(
                            "ambiguous option: {} could match {}",
                            token.rendered(),
                            matches.join(", ")
                        ),
                    );
                }
                LongMatch::None => {
                    extras.push(token.text.clone());
                    index += 1;
                    continue;
                }
            };
            let opt = spec
                .options
                .iter()
                .find(|opt| opt.canonical == chosen)
                .expect("a spec option");
            if opt.kind == OptKind::Help {
                if let Some(value) = glued {
                    return parse_error(
                        spec.usage,
                        spec.prog,
                        &format!(
                            "argument -h/--help: ignored explicit argument {}",
                            repr_text(&value)
                        ),
                    );
                }
                print!("{}", spec.help);
                return ParseOutcome::Exit(0);
            }
            match take_value(spec, tokens, &mut index, glued, opt) {
                Ok(value) => values.push((opt.canonical, value)),
                Err(outcome) => return outcome,
            }
            index += 1;
            continue;
        }
        if option_shaped(token) {
            // A short cluster: h arms help, and o (where
            // the command has it) takes the rest of the
            // cluster or the next token as its value. A
            // first letter the command does not know
            // makes the whole token an extra; an unknown
            // later letter just ends the cluster. Help
            // fires when the token ends, unless a scan
            // error in it fired first.
            let cluster = &token.scalars[1..];
            let known_first =
                cluster[0] == 'h' as u32 || (spec.has_short_o && cluster[0] == 'o' as u32);
            if !known_first {
                extras.push(token.text.clone());
                index += 1;
                continue;
            }
            let mut help_pending = false;
            let mut at = 0usize;
            let mut failed: Option<ParseOutcome> = None;
            while at < cluster.len() {
                match cluster[at] {
                    c if c == 'h' as u32 => {
                        if cluster.get(at + 1) == Some(&('=' as u32)) {
                            // A flag that takes no value,
                            // given one: argparse refuses
                            // the explicit argument.
                            let value = text_from_scalar(token, at + 3);
                            return parse_error(
                                spec.usage,
                                spec.prog,
                                &format!(
                                    "argument -h/--help: ignored explicit argument {}",
                                    repr_text(&value)
                                ),
                            );
                        }
                        help_pending = true;
                        at += 1;
                    }
                    c if c == 'o' as u32 && spec.has_short_o => {
                        let rest = &cluster[at + 1..];
                        let glued = if rest.is_empty() {
                            None
                        } else {
                            let units = token.text.units();
                            let mut start = at + 2;
                            if rest[0] == '=' as u32 {
                                start += 1;
                            }
                            Some(Text::from_units(units[start.min(units.len())..].to_vec()))
                        };
                        let opt = spec
                            .options
                            .iter()
                            .find(|opt| opt.kind == OptKind::Output)
                            .expect("an output option");
                        match take_value(spec, tokens, &mut index, glued, opt) {
                            Ok(value) => values.push((opt.canonical, value)),
                            Err(outcome) => failed = Some(outcome),
                        }
                        break;
                    }
                    _ => break,
                }
            }
            if let Some(outcome) = failed {
                return outcome;
            }
            if help_pending {
                print!("{}", spec.help);
                return ParseOutcome::Exit(0);
            }
            index += 1;
            continue;
        }
        push_positional(&token.text, spec, &mut positionals, &mut extras);
        index += 1;
    }

    let mut missing: Vec<&str> = Vec::new();
    for (label, is_positional) in spec.required {
        if *is_positional {
            let at = spec
                .positionals
                .iter()
                .position(|name| name == label)
                .expect("a spec positional");
            if positionals.len() <= at {
                missing.push(label);
            }
        } else if !values.iter().any(|(name, _)| name == label) {
            missing.push(label);
        }
    }
    if !missing.is_empty() {
        return parse_error(
            spec.usage,
            spec.prog,
            &format!(
                "the following arguments are required: {}",
                missing.join(", ")
            ),
        );
    }
    if !extras.is_empty() {
        let rendered: Vec<String> = extras.iter().map(|t| stderr_render(t.units())).collect();
        return parse_error(
            TOP_USAGE,
            "touchstone",
            &format!("unrecognized arguments: {}", rendered.join(" ")),
        );
    }
    let output = values.iter().rev().find_map(|(name, value)| {
        if *name == "--output" {
            match value {
                ParsedValue::Text(text) => Some(text.clone()),
                _ => None,
            }
        } else {
            None
        }
    });
    ParseOutcome::Run(
        command,
        Box::new(SubParsed {
            positionals,
            values,
            output,
        }),
    )
}

const COMMANDS: [&str; 6] = ["open", "result", "close", "list", "show", "check"];

/// Parse the top level: help, the command word, and
/// the held extras an unknown token becomes.
fn parse_top(tokens: &[Token]) -> ParseOutcome {
    let mut extras: Vec<Text> = Vec::new();
    let mut index = 0usize;
    while index < tokens.len() {
        let token = &tokens[index];
        if token.scalars.len() >= 2
            && token.scalars[0] == '-' as u32
            && token.scalars[1] == '-' as u32
            && token.scalars != scalars_of("--")
        {
            let (name, glued) = split_long(token);
            match match_long(name, &["--help"]) {
                LongMatch::Exact("--help") | LongMatch::Unique("--help") => {
                    if let Some(value) = glued {
                        return parse_error(
                            TOP_USAGE,
                            "touchstone",
                            &format!(
                                "argument -h/--help: ignored explicit argument {}",
                                repr_text(&value)
                            ),
                        );
                    }
                    print!("{TOP_HELP}");
                    return ParseOutcome::Exit(0);
                }
                _ => {
                    extras.push(token.text.clone());
                    index += 1;
                    continue;
                }
            }
        }
        if token.scalars == scalars_of("--") || !option_shaped(token) {
            if token.scalars == scalars_of("--") && index == tokens.len() - 1 {
                // A separator with nothing after it:
                // argparse has consumed it, and the
                // command is simply missing.
                break;
            }
            for command in COMMANDS {
                if token.text.units() == Text::from_str(command).units() {
                    let command: &'static str = command;
                    return parse_sub(command, &tokens[index + 1..], &extras);
                }
            }
            return parse_error(
                TOP_USAGE,
                "touchstone",
                &format!(
                    "argument command: invalid choice: {} (choose from 'open', 'result', 'close', 'list', 'show', 'check')",
                    repr_text(&token.text)
                ),
            );
        }
        if token.scalars[1] == 'h' as u32 {
            print!("{TOP_HELP}");
            return ParseOutcome::Exit(0);
        }
        extras.push(token.text.clone());
        index += 1;
    }
    parse_error(
        TOP_USAGE,
        "touchstone",
        "the following arguments are required: command",
    )
}

/// The recursion budget of the interpreter's
/// resolution walk, measured at this command
/// (VL-36): a resolution nesting 991 splices
/// completes, and the 992nd nesting raises the
/// interpreter's RecursionError out of resolve().
/// The count is of nesting depth, not of splices
/// in total: the top call of a resolution stands
/// at depth 0 and each splice's call stands one
/// deeper. The number belongs to this CLI's
/// stack as measured at it, not to the line.
const MAX_WALK_DEPTH: usize = 991;

/// A path's bytes in the operating system's
/// encoding.
fn os_bytes(value: &std::ffi::OsStr) -> Vec<u8> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt as _;
        value.as_bytes().to_vec()
    }
    #[cfg(not(unix))]
    {
        value.to_string_lossy().into_owned().into_bytes()
    }
}

/// A transient path over raw bytes, for the
/// walk's stats and link reads.
fn path_from_bytes(bytes: &[u8]) -> PathBuf {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt as _;
        PathBuf::from(std::ffi::OsString::from_vec(bytes.to_vec()))
    }
    #[cfg(not(unix))]
    {
        PathBuf::from(String::from_utf8_lossy(bytes).into_owned())
    }
}

/// os.path.join on byte strings: an absolute
/// second part stands alone, an empty or
/// separator-ending first part takes the second
/// as it is, anything else joins on a separator.
fn join_bytes(first: &[u8], second: &[u8]) -> Vec<u8> {
    if second.first() == Some(&0x2F) {
        return second.to_vec();
    }
    let mut out = first.to_vec();
    if !out.is_empty() && !out.ends_with(b"/") {
        out.push(0x2F);
    }
    out.extend_from_slice(second);
    out
}

/// os.path.split on byte strings: the head is
/// everything up to the last separator, trailing
/// separators stripped unless the head is nothing
/// but separators, and the tail is what follows.
fn split_bytes(path: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let index = path
        .iter()
        .rposition(|byte| *byte == 0x2F)
        .map_or(0, |i| i + 1);
    let (mut head, tail) = (path[..index].to_vec(), path[index..].to_vec());
    if !head.is_empty() && !head.iter().all(|byte| *byte == 0x2F) {
        while head.ends_with(b"/") {
            head.pop();
        }
    }
    (head, tail)
}

/// The interpreter's realpath walk itself
/// (posixpath._joinrealpath with strict=False):
/// `path` is the resolved prefix, `rest` the
/// spelling still to walk, and `seen` the links
/// already met, an open entry (None) marking a
/// resolution still in progress and a finished
/// one carrying its result. A component that is
/// a symlink hands the walk its target spelling,
/// resolved against the prefix the link stands
/// in, and the walk continues against the splice,
/// however long the chain runs and wherever in
/// the spelling the link stands; a dangling link
/// therefore lands the resolution on its target
/// spelling rather than on its own name. A stat
/// failure on a component (a missing name, an
/// unsearchable prefix) reads the component as
/// no link and the walk carries it lexically,
/// the interpreter's non-strict rule. `depth` is
/// the nesting of this call, the top call of a
/// resolution standing at 0; a call entered past
/// MAX_WALK_DEPTH aborts the walk, read here as
/// None, the way the interpreter's RecursionError
/// aborts the resolution. A link met again while
/// its own resolution is still open is a loop:
/// the walk leaves that component and the rest
/// of the spelling standing in the result
/// unresolved, and resolve_path's closing stat
/// judges the finished path. A link whose target
/// bytes cannot be read aborts the walk the way
/// the interpreter's uncaught raise does,
/// likewise read as None.
fn joinrealpath(
    mut path: Vec<u8>,
    rest: Vec<u8>,
    seen: &mut std::collections::HashMap<Vec<u8>, Option<Vec<u8>>>,
    depth: usize,
) -> Option<(Vec<u8>, bool)> {
    if depth > MAX_WALK_DEPTH {
        return None;
    }
    let mut rest: &[u8] = &rest;
    if rest.first() == Some(&0x2F) {
        rest = &rest[1..];
        path = vec![0x2F];
    }
    while !rest.is_empty() {
        let (name, tail) = match rest.iter().position(|byte| *byte == 0x2F) {
            Some(index) => (&rest[..index], &rest[index + 1..]),
            None => (rest, &rest[rest.len()..]),
        };
        rest = tail;
        if name.is_empty() || name == b"." {
            continue;
        }
        if name == b".." {
            if !path.is_empty() {
                let (head, popped) = split_bytes(&path);
                path = head;
                if popped == b".." {
                    path = join_bytes(&join_bytes(&path, b".."), b"..");
                }
            } else {
                path = b"..".to_vec();
            }
            continue;
        }
        let newpath = join_bytes(&path, name);
        let is_link = match std::fs::symlink_metadata(path_from_bytes(&newpath)) {
            Ok(meta) => meta.file_type().is_symlink(),
            Err(_) => false,
        };
        if !is_link {
            path = newpath;
            continue;
        }
        if let Some(cached) = seen.get(&newpath) {
            match cached {
                Some(resolved) => {
                    path = resolved.clone();
                    continue;
                }
                None => return Some((join_bytes(&newpath, rest), false)),
            }
        }
        seen.insert(newpath.clone(), None);
        let target = match std::fs::read_link(path_from_bytes(&newpath)) {
            Ok(target) => os_bytes(target.as_os_str()),
            Err(_) => return None,
        };
        let (resolved, ok) = joinrealpath(path, target, seen, depth + 1)?;
        if !ok {
            return Some((join_bytes(&resolved, rest), false));
        }
        seen.insert(newpath, Some(resolved.clone()));
        path = resolved;
    }
    Some((path, true))
}

/// Path.resolve() for the write guard: pathlib's
/// resolve, in byte form: absolute against the
/// working directory, then the interpreter's own
/// walk (joinrealpath), finished lexically. It
/// fails (None) where the interpreter's resolve
/// raises: the working directory read behind a
/// relative spelling failing (VL-35; a root
/// spelling is never substituted for it), the
/// walk nested past its budget, a link whose
/// target bytes cannot be read, or the closing
/// stat of the finished path reporting the
/// kernel's loop errno (VL-36). The guard reads
/// the failure as the interpreter's crash, this
/// crate's crash class (VL-13).
fn resolve_path(path: &Path) -> Option<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().ok()?.join(path)
    };
    let spelling = os_bytes(absolute.as_os_str());
    let mut seen = std::collections::HashMap::new();
    let (resolved, _ok) = joinrealpath(Vec::new(), spelling, &mut seen, 0)?;
    // The lexical finish over the walk's result:
    // a dot component drops and a dot-dot pops
    // the component before it, whatever kind of
    // component that is, so the tail a standing
    // loop remainder left behind (a cancelled
    // loop's spelling) finishes the way the
    // interpreter's resolve returns it, and an
    // uncancelled remainder reaches the closing
    // stat still naming its loop.
    let mut parts: Vec<&[u8]> = Vec::new();
    for part in resolved.split(|&byte| byte == 0x2F) {
        if part.is_empty() || part == b"." {
            continue;
        }
        if part == b".." {
            parts.pop();
            continue;
        }
        parts.push(part);
    }
    let mut finished: Vec<u8> = vec![0x2F];
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            finished.push(0x2F);
        }
        finished.extend_from_slice(part);
    }
    // The closing stat of resolve(): only the
    // kernel's loop report fails the resolution;
    // any other stat failure (a missing tail, an
    // unsearchable prefix) is swallowed there,
    // as the interpreter swallows it.
    if let Err(error) = std::fs::metadata(path_from_bytes(&finished)) {
        if error.raw_os_error() == Some(40) {
            return None;
        }
    }
    Some(path_from_bytes(&finished))
}

/// The OS path for a text argument: pathlib's
/// lexical normalization first (the same form the
/// path's display form takes, so a trailing slash
/// or dot component names the same file), in the
/// operating system's own encoding.
pub fn os_path(text: &Text) -> PathBuf {
    let units = normalize_path_display(text.units());
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        PathBuf::from(std::ffi::OsString::from_vec(fsencode_bytes(&units)))
    }
    #[cfg(not(unix))]
    {
        PathBuf::from(String::from_utf16_lossy(&units))
    }
}

/// A typed path's display form, as the Python
/// prints it in its messages.
fn display_path(text: &Text) -> String {
    stderr_render(&normalize_path_display(text.units()))
}

/// posixpath.normpath on byte strings: empty and
/// dot components drop, a dotdot pops the
/// component before it (and stands only at the
/// head of a relative spelling), and exactly two
/// leading separators keep their pair, the
/// anchor form.
fn normpath_bytes(path: &[u8]) -> Vec<u8> {
    let slashes: &[u8] = if path.starts_with(b"//") && !path.starts_with(b"///") {
        b"//"
    } else if path.starts_with(b"/") {
        b"/"
    } else {
        b""
    };
    let mut parts: Vec<&[u8]> = Vec::new();
    for part in path.split(|&byte| byte == 0x2F) {
        if part.is_empty() || part == b"." {
            continue;
        }
        if part == b".." {
            match parts.last() {
                Some(last) if *last != b".." => {
                    parts.pop();
                }
                _ => {
                    if slashes.is_empty() {
                        parts.push(part);
                    }
                }
            }
            continue;
        }
        parts.push(part);
    }
    if parts.is_empty() {
        if slashes.is_empty() {
            return b".".to_vec();
        }
        return slashes.to_vec();
    }
    let mut out = slashes.to_vec();
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            out.push(0x2F);
        }
        out.extend_from_slice(part);
    }
    out
}

/// The directory tempfile.mkstemp creates the
/// view's temporary file in (VL-37): the
/// constructed path's parent absolutized
/// lexically, the working directory joined when
/// the parent is relative and the result
/// normalized without resolving any link,
/// exactly os.path.abspath's shape in the
/// Python.
fn temp_parent(path: &Path) -> std::io::Result<PathBuf> {
    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    let parent_bytes = os_bytes(parent.as_os_str());
    let absolute = if parent_bytes.first() == Some(&0x2F) {
        parent_bytes
    } else {
        let cwd = std::env::current_dir()?;
        join_bytes(&os_bytes(cwd.as_os_str()), &parent_bytes)
    };
    Ok(path_from_bytes(&normpath_bytes(&absolute)))
}

fn write_atomically(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let parent = temp_parent(path)?;
    // The candidate is assembled from raw bytes end
    // to end (VL-43): the temporary parent's bytes,
    // the output name's bytes as spelled, the dot,
    // and the suffix. A lossy reading of the name
    // would run the candidate longer than the name
    // it stands for and move the refusal boundary.
    let parent_bytes = os_bytes(parent.as_os_str());
    let name_bytes = path.file_name().map(os_bytes).unwrap_or_default();
    // mkstemp's shape: an unpredictable name in the
    // target directory, created exclusively at mode
    // 0600, so a file already at any guessed name is
    // never touched.
    let mut attempt = 0;
    let temp = loop {
        let suffix = random_suffix(attempt);
        let mut candidate_bytes = join_bytes(&parent_bytes, &name_bytes);
        candidate_bytes.push(b'.');
        candidate_bytes.extend_from_slice(suffix.as_bytes());
        let candidate = path_from_bytes(&candidate_bytes);
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt as _;
            options.mode(0o600);
        }
        match options.open(&candidate) {
            Ok(mut handle) => match handle.write_all(bytes) {
                Ok(()) => break candidate,
                Err(error) => {
                    drop(handle);
                    let _ = std::fs::remove_file(&candidate);
                    return Err(error);
                }
            },
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists && attempt < 16 => {
                attempt += 1;
            }
            Err(error) => return Err(error),
        }
    };
    match std::fs::rename(&temp, path) {
        Ok(()) => Ok(()),
        Err(error) => {
            let _ = std::fs::remove_file(&temp);
            Err(error)
        }
    }
}

fn random_suffix(attempt: u32) -> String {
    // mkstemp's suffix is eight characters (VL-42):
    // four drawn bytes, each rendered as two
    // hexadecimal characters.
    let mut bytes = [0u8; 4];
    let filled = std::fs::File::open("/dev/urandom")
        .and_then(|mut file| std::io::Read::read_exact(&mut file, &mut bytes))
        .is_ok();
    if !filled {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let mix = (nanos ^ ((std::process::id() as u128) << 64) ^ (attempt as u128)).to_le_bytes();
        for lane in 0..4 {
            bytes[lane] = mix[lane] ^ mix[lane + 4] ^ mix[lane + 8] ^ mix[lane + 12];
        }
    }
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Emit a view document (list, show): to stdout, or
/// atomically to the output path (VL-12).
fn emit_view(document: &Value, out: Option<&Text>) -> i32 {
    match out {
        Some(out) => {
            let path = os_path(out);
            let written = match dumps_view(document, Mode::Strict) {
                Ok(bytes) => write_atomically(&path, &bytes),
                Err(_) => Err(std::io::Error::other("unwritable text")),
            };
            match written {
                Ok(()) => 0,
                Err(_) => {
                    eprintln!(
                        "error, cannot write the view, {}",
                        stderr_render(out.units())
                    );
                    1
                }
            }
        }
        None => match dumps_view(document, Mode::Stdout) {
            Ok(bytes) => {
                let stdout = std::io::stdout();
                let mut handle = stdout.lock();
                match handle.write_all(&bytes).and_then(|_| handle.flush()) {
                    Ok(()) => 0,
                    Err(_) => {
                        eprintln!("error, cannot write to stdout");
                        1
                    }
                }
            }
            Err(_) => {
                eprintln!("error, cannot write to stdout");
                1
            }
        },
    }
}

/// Report a registry failure the way the Python's
/// main does: a named refusal after "error, ", and
/// the crash class as a bare exit 1.
fn report(result: Result<i32, RegError>) -> i32 {
    match result {
        Ok(code) => code,
        Err(RegError::Named(error)) => {
            eprintln!("error, {}", error.0.render_stderr());
            1
        }
        Err(RegError::Crash) => 1,
    }
}

fn text_arg(parsed: &SubParsed, canonical: &str) -> Text {
    parsed.get_text(canonical).expect("a required argument")
}

/// The write guard's outcome (VL-12): a view whose
/// output path resolves to the registry itself is
/// refused before the view runs, and a resolution
/// that fails outright takes the crash class,
/// exactly as the Python's uncaught raise from
/// resolve() behaves (VL-35, VL-36).
enum Guard {
    Pass,
    Refuse,
    Crash,
}

fn view_guard(registry: &RegPath, parsed: &SubParsed) -> Guard {
    match &parsed.output {
        Some(out) => match (resolve_path(&os_path(out)), resolve_path(&registry.os)) {
            (Some(out), Some(registry)) if out == registry => {
                eprintln!("error, a view never overwrites the registry");
                Guard::Refuse
            }
            (Some(_), Some(_)) => Guard::Pass,
            _ => Guard::Crash,
        },
        None => Guard::Pass,
    }
}

fn execute(command: &str, parsed: &SubParsed) -> i32 {
    let registry = RegPath::from_typed(&parsed.positionals[0]);
    match command {
        "open" => report((|| {
            let measures =
                parse_arg_json(&text_arg(parsed, "--measures"), "the baseline measures")?;
            let judged = parse_arg_json(&text_arg(parsed, "--judged"), "the judged list")?;
            let mut pairs = vec![
                ("type", Value::Str(Text::from_str("opened"))),
                ("experiment", Value::Str(parsed.positionals[1].clone())),
                ("at", Value::Str(text_arg(parsed, "--at"))),
                ("source", Value::Str(text_arg(parsed, "--source"))),
                ("proposal", Value::Str(text_arg(parsed, "--proposal"))),
                ("baseline", Value::Str(text_arg(parsed, "--baseline"))),
                ("baseline_measures", measures),
                ("judged", judged),
            ];
            if let Some(margin) = parsed.get_float("--margin") {
                pairs.push(("margin_pct", Value::Float(margin)));
            }
            open_experiment(&registry, &object(pairs))?;
            Ok(0)
        })()),
        "result" => report((|| {
            let measures = parse_arg_json(&text_arg(parsed, "--measures"), "the measures")?;
            let event = object(vec![
                ("type", Value::Str(Text::from_str("result"))),
                ("experiment", Value::Str(parsed.positionals[1].clone())),
                ("at", Value::Str(text_arg(parsed, "--at"))),
                ("measures", measures),
            ]);
            add_result(&registry, &event)?;
            Ok(0)
        })()),
        "close" => {
            // The judge's configuration surface is
            // secondtake's (VL-21): the word live names
            // the connector call script in JEV_CALL,
            // and any other value names a judgments
            // file the fixture transport replays. The
            // transport is built before the close runs,
            // as secondtake builds it before the
            // comparison; no model is called until the
            // close's own refusals have passed.
            let judge: Option<Box<dyn Transport>> = match parsed.get_text("--judge") {
                Some(word) if word.units() == Text::from_str("live").units() => {
                    match std::env::var_os("JEV_CALL") {
                        Some(script) if !script.is_empty() => {
                            Some(Box::new(LiveTransport::new(script)))
                        }
                        _ => {
                            eprintln!(
                                "error, live judging needs the JEV_CALL environment variable naming the connector call script"
                            );
                            return 1;
                        }
                    }
                }
                Some(path) => match FixtureTransport::load(&os_path(&path), &display_path(&path)) {
                    Ok(transport) => Some(Box::new(transport)),
                    Err(JudgeError(message)) => {
                        eprintln!("error, {message}");
                        return 1;
                    }
                },
                None => None,
            };
            report((|| {
                let mut pairs = vec![
                    ("type", Value::Str(Text::from_str("closed"))),
                    ("experiment", Value::Str(parsed.positionals[1].clone())),
                    ("at", Value::Str(text_arg(parsed, "--at"))),
                    ("verdict", Value::Str(text_arg(parsed, "--verdict"))),
                    ("rationale", Value::Str(text_arg(parsed, "--rationale"))),
                ];
                if let Some(successor) = parsed.get_text("--successor") {
                    if !successor.units().is_empty() {
                        pairs.push(("successor", Value::Str(successor)));
                    }
                }
                close_experiment(&registry, &object(pairs), judge.as_deref())?;
                Ok(0)
            })())
        }
        "list" => {
            if !matches!(view_guard(&registry, parsed), Guard::Pass) {
                return 1;
            }
            report((|| {
                let document = list_view(&registry)?;
                Ok(emit_view(&document, parsed.output.as_ref()))
            })())
        }
        "show" => {
            if !matches!(view_guard(&registry, parsed), Guard::Pass) {
                return 1;
            }
            report((|| {
                let document = show_view(&registry, &parsed.positionals[1])?;
                Ok(emit_view(&document, parsed.output.as_ref()))
            })())
        }
        _ => {
            // check: defects print to stdout, one per
            // line; a defect the stream cannot write is
            // the crash class, and a stream failure is
            // the Python's caught OSError.
            let (_events, defects) = match load_registry(&registry) {
                Ok(pair) => pair,
                Err(RegError::Crash) => return 1,
                Err(RegError::Named(error)) => {
                    eprintln!("error, {}", error.0.render_stderr());
                    return 1;
                }
            };
            let stdout = std::io::stdout();
            let mut handle = stdout.lock();
            for defect in &defects {
                let mut bytes = match defect.render_stdout() {
                    Some(bytes) => bytes,
                    None => return 1,
                };
                bytes.push(b'\n');
                if let Err(err) = handle.write_all(&bytes) {
                    eprintln!("error, {}", crate::registry::os_exc_text(&err));
                    return 1;
                }
            }
            if defects.is_empty() {
                if let Err(err) = handle
                    .write_all(b"the registry is sound\n")
                    .and_then(|_| handle.flush())
                {
                    eprintln!("error, {}", crate::registry::os_exc_text(&err));
                    return 1;
                }
                0
            } else {
                1
            }
        }
    }
}

pub fn run_cli() -> i32 {
    let tokens: Vec<Token> = std::env::args_os()
        .skip(1)
        .map(|arg| {
            #[cfg(unix)]
            let bytes = arg.as_encoded_bytes().to_vec();
            #[cfg(not(unix))]
            let bytes = arg.to_string_lossy().into_owned().into_bytes();
            Token::from_units(fsdecode_units(&bytes))
        })
        .collect();
    run_tokens(tokens)
}

fn run_tokens(tokens: Vec<Token>) -> i32 {
    match parse_top(&tokens) {
        ParseOutcome::Run(command, parsed) => execute(command, &parsed),
        ParseOutcome::Exit(code) => code,
    }
}
