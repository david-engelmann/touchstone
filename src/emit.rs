//! The registry's rendering: Python's json.dumps in
//! both shapes the Python writes, a compact line for
//! appended events (the default separators) and an
//! indent of two for the views, both with
//! ensure_ascii=False (VL-16). The bytes follow the
//! destination's law, which the caller picks: stdout
//! writes with surrogateescape, so a lone surrogate
//! in DC80 to DCFF writes as its raw byte and any
//! other lone surrogate fails; a file write encodes
//! strict UTF-8, so every lone surrogate fails.

use flighttape::value::{Text, Value};

/// A document carrying text the destination cannot
/// write: the CLI reports the cannot-write error for
/// a view, and the append path meets the Python's
/// own uncaught failure (named in the dossier).
#[derive(Debug, PartialEq)]
pub struct EmitError;

/// Which write law the rendered bytes must satisfy.
#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    /// sys.stdout: surrogateescape bytes pass raw.
    Stdout,
    /// A file opened with encoding utf-8: strict.
    Strict,
}

/// Python's float repr, which is also the JSON
/// encoder's rendering: the shortest digits that
/// round trip, presented fixed inside the band
/// -4 <= exp10 < 16 and scientifically outside it,
/// with a signed two-digit exponent. The digits are
/// computed exactly in this module: neither Rust's
/// own float formatting (which rounds exact decimal
/// ties up where CPython rounds to even) nor the
/// flighttape crate's renderer (whose digit sequence
/// for an exactly terminating expansion can run one
/// digit past the shortest, as at 2^-24) is CPython's
/// sequence in every case.
pub fn py_float(value: f64) -> String {
    if value.is_nan() {
        return "nan".to_string();
    }
    if value.is_infinite() {
        return if value > 0.0 {
            "inf".to_string()
        } else {
            "-inf".to_string()
        };
    }
    if value == 0.0 {
        return if value.is_sign_negative() {
            "-0.0".to_string()
        } else {
            "0.0".to_string()
        };
    }
    let sign = if value < 0.0 { "-" } else { "" };
    let (digits, point) = shortest_digits(value.abs());
    let count = digits.len() as i64;
    let exp10 = point - 1;
    if !(-4..16).contains(&exp10) {
        let mantissa = if count == 1 {
            digits
        } else {
            format!("{}.{}", &digits[..1], &digits[1..])
        };
        return format!("{sign}{mantissa}e{exp10:+03}");
    }
    if point <= 0 {
        format!("{sign}0.{}{digits}", "0".repeat((-point) as usize))
    } else if count <= point {
        format!("{sign}{digits}{}.0", "0".repeat((point - count) as usize))
    } else {
        format!(
            "{sign}{}.{}",
            &digits[..point as usize],
            &digits[point as usize..]
        )
    }
}

/// The exact decimal expansion of a positive finite
/// nonzero double: a digit string and a point, the
/// value being 0.digits * 10^point. The expansion is
/// finite (the double is m * 2^e), built digit by
/// digit with small integer arithmetic. Callers use
/// it where a comparison must be exact rather than
/// shortest (the suggestion rule's margins).
pub fn exact_decimal(value: f64) -> (String, i64) {
    let bits = value.to_bits();
    let biased = ((bits >> 52) & 0x7FF) as i32;
    let fraction = bits & ((1u64 << 52) - 1);
    let (mantissa, exponent) = if biased == 0 {
        (fraction, -1074)
    } else {
        (fraction + (1u64 << 52), biased - 1075)
    };
    let mut digits: Vec<u8> = mantissa
        .to_string()
        .bytes()
        .map(|byte| byte - b'0')
        .collect();
    let point: i64 = if exponent >= 0 {
        for _ in 0..exponent {
            multiply_small(&mut digits, 2);
        }
        digits.len() as i64
    } else {
        for _ in 0..-exponent {
            multiply_small(&mut digits, 5);
        }
        digits.len() as i64 + exponent as i64
    };
    while digits.len() > 1 && *digits.last().expect("nonempty") == 0 {
        digits.pop();
    }
    (digits.iter().map(|d| (b'0' + d) as char).collect(), point)
}

/// The shortest round-trip digit sequence of a
/// positive finite nonzero double, and the position
/// of the decimal point: the value is
/// 0.digits * 10^point. The shortest length whose
/// rounding of the exact expansion parses back to
/// the same double wins, the closer candidate first
/// and an exact tie going to the even last digit.
/// Parsing back goes through Rust's correctly
/// rounded reader, so the true (asymmetric, at
/// powers of two) rounding interval is honored by
/// construction.
fn shortest_digits(value: f64) -> (String, i64) {
    let (exact, point) = exact_decimal(value);
    for len in 1..=exact.len() {
        let low = &exact[..len];
        let rest = &exact[len..];
        let high = increment_digits(low);
        let (high_digits, high_point) = match &high {
            Some(high) => (high.clone(), point),
            None => ("1".to_string(), point + 1),
        };
        let low_ok = parses_back(low, point, value);
        let high_ok = parses_back(&high_digits, high_point, value);
        match (low_ok, high_ok) {
            (false, false) => continue,
            (true, false) => return (low.to_string(), point),
            (false, true) => return (high_digits, high_point),
            (true, true) => {
                // The closer candidate wins; the tail
                // against the half decides, and an
                // exact half goes to the even last
                // digit.
                let cmp = cmp_tail_to_half(rest);
                let pick_high = match cmp {
                    std::cmp::Ordering::Greater => true,
                    std::cmp::Ordering::Less => false,
                    std::cmp::Ordering::Equal => (low.as_bytes()[len - 1] - b'0') % 2 == 1,
                };
                return if pick_high {
                    (high_digits, high_point)
                } else {
                    (low.to_string(), point)
                };
            }
        }
    }
    unreachable!("the exact digits themselves round trip")
}

/// Multiply a most-significant-first digit vector by
/// a small factor, in place.
fn multiply_small(digits: &mut Vec<u8>, factor: u8) {
    let mut carry = 0u16;
    for digit in digits.iter_mut().rev() {
        let product = *digit as u16 * factor as u16 + carry;
        *digit = (product % 10) as u8;
        carry = product / 10;
    }
    let mut prefix = Vec::new();
    while carry > 0 {
        prefix.push((carry % 10) as u8);
        carry /= 10;
    }
    prefix.reverse();
    let tail = std::mem::take(digits);
    *digits = prefix;
    digits.extend(tail);
}

/// A digit string plus one, or None when every digit
/// was 9 (the result gains a digit: 1 followed by
/// zeros, which the caller renders as "1" with the
/// point moved).
fn increment_digits(digits: &str) -> Option<String> {
    let mut bytes: Vec<u8> = digits.bytes().collect();
    let mut index = bytes.len();
    loop {
        if index == 0 {
            return None;
        }
        index -= 1;
        if bytes[index] == b'9' {
            bytes[index] = b'0';
        } else {
            bytes[index] += 1;
            return Some(String::from_utf8(bytes).expect("digits are ascii"));
        }
    }
}

/// Compare a dropped tail to exactly one half: the
/// tail is the fraction of one unit in the last kept
/// digit, in decimal digits.
fn cmp_tail_to_half(rest: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    if rest.is_empty() {
        return Ordering::Less;
    }
    match rest.as_bytes()[0] {
        b'0'..=b'4' => Ordering::Less,
        b'6'..=b'9' => Ordering::Greater,
        _ => {
            if rest[1..].chars().all(|ch| ch == '0') {
                Ordering::Equal
            } else {
                Ordering::Greater
            }
        }
    }
}

/// Whether the decimal digits at the given point
/// parse back to exactly this double.
fn parses_back(digits: &str, point: i64, value: f64) -> bool {
    let text = if digits.len() == 1 {
        format!("{digits}e{}", point - 1)
    } else {
        format!("{}.{}e{}", &digits[..1], &digits[1..], point - 1)
    };
    match text.parse::<f64>() {
        Ok(parsed) => parsed.to_bits() == value.to_bits(),
        Err(_) => false,
    }
}

/// Append one text as a JSON string in bytes. Valid
/// scalars escape only the quote, the backslash, and
/// the control characters, as the encoder does. In
/// Stdout mode a surrogateescape unit writes as its
/// raw byte; any other lone surrogate (and every
/// lone surrogate in Strict mode) is an EmitError.
fn escape_string(text: &Text, mode: Mode, out: &mut Vec<u8>) -> Result<(), EmitError> {
    out.push(b'"');
    let units = text.units();
    let mut index = 0;
    while index < units.len() {
        let unit = units[index];
        if (0xD800..=0xDBFF).contains(&unit)
            && index + 1 < units.len()
            && (0xDC00..=0xDFFF).contains(&units[index + 1])
        {
            let scalar = crate::util::decode_pair(unit, units[index + 1]);
            push_scalar(scalar, out);
            index += 2;
            continue;
        }
        if mode == Mode::Stdout && (0xDC80..=0xDCFF).contains(&unit) {
            out.push((unit - 0xDC00) as u8);
            index += 1;
            continue;
        }
        if (0xD800..=0xDFFF).contains(&unit) {
            return Err(EmitError);
        }
        match unit {
            0x22 => out.extend_from_slice(b"\\\""),
            0x5C => out.extend_from_slice(b"\\\\"),
            0x08 => out.extend_from_slice(b"\\b"),
            0x09 => out.extend_from_slice(b"\\t"),
            0x0A => out.extend_from_slice(b"\\n"),
            0x0C => out.extend_from_slice(b"\\f"),
            0x0D => out.extend_from_slice(b"\\r"),
            0x00..=0x1F => {
                out.extend_from_slice(format!("\\u{unit:04x}").as_bytes());
            }
            _ => push_scalar(unit as u32, out),
        }
        index += 1;
    }
    out.push(b'"');
    Ok(())
}

fn push_scalar(scalar: u32, out: &mut Vec<u8>) {
    let mut buf = [0u8; 4];
    let encoded = char::from_u32(scalar)
        .expect("a valid scalar")
        .encode_utf8(&mut buf);
    out.extend_from_slice(encoded.as_bytes());
}

fn write_number(value: &Value, out: &mut Vec<u8>) {
    match value {
        Value::Int(digits) => out.extend_from_slice(digits.as_bytes()),
        Value::Float(number) => {
            if number.is_nan() {
                out.extend_from_slice(b"NaN");
            } else if number.is_infinite() {
                out.extend_from_slice(if *number > 0.0 {
                    b"Infinity"
                } else {
                    b"-Infinity"
                });
            } else {
                out.extend_from_slice(py_float(*number).as_bytes());
            }
        }
        _ => unreachable!("a number value"),
    }
}

/// json.dumps with the default separators and no
/// indent: the appended event line, without its
/// trailing newline.
pub fn dumps_line(value: &Value, mode: Mode) -> Result<Vec<u8>, EmitError> {
    let mut out = Vec::new();
    write_compact(value, mode, &mut out)?;
    Ok(out)
}

fn write_compact(value: &Value, mode: Mode, out: &mut Vec<u8>) -> Result<(), EmitError> {
    match value {
        Value::Null => out.extend_from_slice(b"null"),
        Value::Bool(true) => out.extend_from_slice(b"true"),
        Value::Bool(false) => out.extend_from_slice(b"false"),
        Value::Int(_) | Value::Float(_) => write_number(value, out),
        Value::Str(text) => escape_string(text, mode, out)?,
        Value::Array(items) => {
            out.push(b'[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.extend_from_slice(b", ");
                }
                write_compact(item, mode, out)?;
            }
            out.push(b']');
        }
        Value::Object(entries) => {
            out.push(b'{');
            for (index, (key, item)) in entries.iter().enumerate() {
                if index > 0 {
                    out.extend_from_slice(b", ");
                }
                escape_string(key, mode, out)?;
                out.extend_from_slice(b": ");
                write_compact(item, mode, out)?;
            }
            out.push(b'}');
        }
    }
    Ok(())
}

/// json.dumps with an indent of two: the view
/// document, trailing newline included.
pub fn dumps_view(value: &Value, mode: Mode) -> Result<Vec<u8>, EmitError> {
    let mut out = Vec::new();
    write_indented(value, 0, mode, &mut out)?;
    out.push(b'\n');
    Ok(out)
}

fn push_indent(out: &mut Vec<u8>, depth: usize) {
    for _ in 0..depth {
        out.extend_from_slice(b"  ");
    }
}

fn write_indented(
    value: &Value,
    depth: usize,
    mode: Mode,
    out: &mut Vec<u8>,
) -> Result<(), EmitError> {
    match value {
        Value::Null => out.extend_from_slice(b"null"),
        Value::Bool(true) => out.extend_from_slice(b"true"),
        Value::Bool(false) => out.extend_from_slice(b"false"),
        Value::Int(_) | Value::Float(_) => write_number(value, out),
        Value::Str(text) => escape_string(text, mode, out)?,
        Value::Array(items) => {
            if items.is_empty() {
                out.extend_from_slice(b"[]");
                return Ok(());
            }
            out.extend_from_slice(b"[\n");
            for (index, item) in items.iter().enumerate() {
                push_indent(out, depth + 1);
                write_indented(item, depth + 1, mode, out)?;
                if index + 1 < items.len() {
                    out.push(b',');
                }
                out.push(b'\n');
            }
            push_indent(out, depth);
            out.push(b']');
        }
        Value::Object(entries) => {
            if entries.is_empty() {
                out.extend_from_slice(b"{}");
                return Ok(());
            }
            out.extend_from_slice(b"{\n");
            for (index, (key, item)) in entries.iter().enumerate() {
                push_indent(out, depth + 1);
                escape_string(key, mode, out)?;
                out.extend_from_slice(b": ");
                write_indented(item, depth + 1, mode, out)?;
                if index + 1 < entries.len() {
                    out.push(b',');
                }
                out.push(b'\n');
            }
            push_indent(out, depth);
            out.push(b'}');
        }
    }
    Ok(())
}
