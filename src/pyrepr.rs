//! CPython's repr for the values the defect phrases
//! name (an unknown event type, a judged measure).
//! The flighttape crate's repr derives printability
//! from its own table, which disagrees with CPython at
//! U+0378, U+0379, U+1AFF, and U+E0001; the phrases
//! here are touchstone's own output, so this module
//! carries CPython's table (repr_table) and emit's
//! exact float rendering.

use flighttape::value::{Text, Value};

use crate::emit::py_float;
use crate::repr_table::NON_PRINTABLE;

fn is_printable(scalar: u32) -> bool {
    NON_PRINTABLE
        .binary_search_by(|&(lo, hi)| {
            if scalar < lo {
                std::cmp::Ordering::Greater
            } else if scalar > hi {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Equal
            }
        })
        .is_err()
}

/// The decoded scalars of a text: surrogate pairs as
/// their astral scalar, lone surrogates at face value.
fn scalars(text: &Text) -> Vec<u32> {
    let units = text.units();
    let mut out = Vec::new();
    let mut index = 0;
    while index < units.len() {
        let unit = units[index];
        if (0xD800..=0xDBFF).contains(&unit)
            && index + 1 < units.len()
            && (0xDC00..=0xDFFF).contains(&units[index + 1])
        {
            out.push(crate::util::decode_pair(unit, units[index + 1]));
            index += 2;
        } else {
            out.push(unit as u32);
            index += 1;
        }
    }
    out
}

/// CPython's repr of a string: single quotes unless
/// the text holds a single quote and no double quote,
/// the quote and backslash escaped, the whitespace
/// escapes, printable characters as themselves, and
/// everything else as \xXX, \uXXXX, or \UXXXXXXXX.
pub fn repr_text(text: &Text) -> String {
    let values = scalars(text);
    let quote = if values.contains(&(b'\'' as u32)) && !values.contains(&(b'"' as u32)) {
        '"'
    } else {
        '\''
    };
    let mut out = String::new();
    out.push(quote);
    for scalar in values {
        let ch = char::from_u32(scalar);
        match ch {
            Some(c) if c == quote => {
                out.push('\\');
                out.push(c);
            }
            Some('\\') => out.push_str("\\\\"),
            Some('\n') => out.push_str("\\n"),
            Some('\r') => out.push_str("\\r"),
            Some('\t') => out.push_str("\\t"),
            Some(c) if is_printable(scalar) => out.push(c),
            _ => {
                if scalar < 0x100 {
                    out.push_str(&format!("\\x{scalar:02x}"));
                } else if scalar < 0x10000 {
                    out.push_str(&format!("\\u{scalar:04x}"));
                } else {
                    out.push_str(&format!("\\U{scalar:08x}"));
                }
            }
        }
    }
    out.push(quote);
    out
}

/// CPython's repr of any JSON value, recursively: the
/// form the unknown event type and judged measure
/// phrases use.
pub fn repr_value(value: &Value) -> String {
    match value {
        Value::Null => "None".to_string(),
        Value::Bool(true) => "True".to_string(),
        Value::Bool(false) => "False".to_string(),
        Value::Int(digits) => digits.clone(),
        Value::Float(number) => py_float(*number),
        Value::Str(text) => repr_text(text),
        Value::Array(items) => {
            let rendered: Vec<String> = items.iter().map(repr_value).collect();
            format!("[{}]", rendered.join(", "))
        }
        Value::Object(entries) => {
            let rendered: Vec<String> = entries
                .iter()
                .map(|(key, item)| format!("{}: {}", repr_text(key), repr_value(item)))
                .collect();
            format!("{{{}}}", rendered.join(", "))
        }
    }
}
