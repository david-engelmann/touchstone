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
use crate::pyrepr::repr_text;
use crate::registry::{
    add_result, close_experiment, list_view, load_registry, open_experiment, parse_arg_json,
    show_view, RegError, RegPath,
};
use crate::util::{fsdecode_units, fsencode_bytes, stderr_render};

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
                scalars.push(
                    0x10000 + (((unit as u32) - 0xD800) << 10) + (units[index + 1] as u32 - 0xDC00),
                );
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
    let first = scalars[1];
    let digit = |c: u32| ('0' as u32..='9' as u32).contains(&c);
    let negative_number =
        digit(first) || (first == '.' as u32 && scalars.get(2).is_some_and(|c| digit(*c)));
    !negative_number
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
    prog: "verdictloop open",
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
    prog: "verdictloop result",
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
    prog: "verdictloop close",
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
    prog: "verdictloop list",
    help: LIST_HELP,
    positionals: &["registry"],
    options: &[HELP_OPT, OUTPUT_OPT],
    required: &[("registry", true)],
    has_short_o: true,
};

const SHOW_SPEC: SubSpec = SubSpec {
    usage: SHOW_USAGE,
    prog: "verdictloop show",
    help: SHOW_HELP,
    positionals: &["registry", "experiment"],
    options: &[HELP_OPT, OUTPUT_OPT],
    required: &[("registry", true), ("experiment", true)],
    has_short_o: true,
};

const CHECK_SPEC: SubSpec = SubSpec {
    usage: CHECK_USAGE,
    prog: "verdictloop check",
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
/// between its digits. Anything else is not a float.
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
            "verdictloop",
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
                            "verdictloop",
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
            for command in COMMANDS {
                if token.text.units() == Text::from_str(command).units() {
                    let command: &'static str = command;
                    return parse_sub(command, &tokens[index + 1..], &extras);
                }
            }
            return parse_error(
                TOP_USAGE,
                "verdictloop",
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
        "verdictloop",
        "the following arguments are required: command",
    )
}

/// Path.resolve() for the write guard: canonicalize
/// what exists, then append the rest with lexical
/// normalization.
fn resolve_path(path: &Path) -> PathBuf {
    if let Ok(canonical) = std::fs::canonicalize(path) {
        return canonical;
    }
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("/"))
            .join(path)
    };
    let mut ancestor: &Path = &absolute;
    let mut rest: Vec<std::path::Component> = Vec::new();
    while !ancestor.exists() {
        match ancestor.components().next_back() {
            Some(component) => rest.push(component),
            None => break,
        }
        match ancestor.parent() {
            Some(parent) => ancestor = parent,
            None => break,
        }
    }
    let mut base = std::fs::canonicalize(ancestor).unwrap_or_else(|_| ancestor.to_path_buf());
    for component in rest.iter().rev() {
        match component {
            std::path::Component::Normal(name) => base.push(name),
            std::path::Component::ParentDir => {
                base.pop();
            }
            _ => {}
        }
    }
    base
}

/// The OS path for a text argument, in the operating
/// system's own encoding.
pub fn os_path(text: &Text) -> PathBuf {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        PathBuf::from(std::ffi::OsString::from_vec(fsencode_bytes(text.units())))
    }
    #[cfg(not(unix))]
    {
        PathBuf::from(String::from_utf16_lossy(text.units()))
    }
}

fn write_atomically(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    // mkstemp's shape: an unpredictable name in the
    // target directory, created exclusively at mode
    // 0600, so a file already at any guessed name is
    // never touched.
    let mut attempt = 0;
    let temp = loop {
        let suffix = random_suffix(attempt);
        let candidate = parent.join(format!("{name}.{suffix}"));
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
    let mut bytes = [0u8; 9];
    let filled = std::fs::File::open("/dev/urandom")
        .and_then(|mut file| std::io::Read::read_exact(&mut file, &mut bytes))
        .is_ok();
    if !filled {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        bytes = (nanos ^ ((std::process::id() as u128) << 64) ^ (attempt as u128)).to_le_bytes()
            [..9]
            .try_into()
            .expect("nine bytes");
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
        "close" => report((|| {
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
            close_experiment(&registry, &object(pairs))?;
            Ok(0)
        })()),
        "list" => {
            if let Some(out) = &parsed.output {
                if resolve_path(&os_path(out)) == resolve_path(&registry.os) {
                    eprintln!("error, a view never overwrites the registry");
                    return 1;
                }
            }
            report((|| {
                let document = list_view(&registry)?;
                Ok(emit_view(&document, parsed.output.as_ref()))
            })())
        }
        "show" => {
            if let Some(out) = &parsed.output {
                if resolve_path(&os_path(out)) == resolve_path(&registry.os) {
                    eprintln!("error, a view never overwrites the registry");
                    return 1;
                }
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

/// The CLI over explicit argument tokens, the shape
/// the ported suite drives.
pub fn run_with_args(tokens: Vec<String>) -> i32 {
    let tokens: Vec<Token> = tokens
        .iter()
        .map(|token| Token::from_units(fsdecode_units(token.as_bytes())))
        .collect();
    run_tokens(tokens)
}

fn run_tokens(tokens: Vec<Token>) -> i32 {
    match parse_top(&tokens) {
        ParseOutcome::Run(command, parsed) => execute(command, &parsed),
        ParseOutcome::Exit(code) => code,
    }
}
