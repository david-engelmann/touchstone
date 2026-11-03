//! Shared constructions: argument and path text in
//! CPython's filesystem decoding, the JSON text
//! scans the loader's refusals turn on, and the
//! exact integer arithmetic the suggestion rule's
//! comparisons need (the Python computes them over
//! integers of any size, and its float pipeline must
//! be matched bit for bit, not approximated).

use std::cmp::Ordering;

use flighttape::value::Text;

use crate::emit::exact_decimal;

/// Decode OS bytes the way CPython decodes file system
/// and argument strings: valid UTF-8 decodes normally,
/// and each byte that cannot decode becomes the
/// surrogateescape unit 0xDC00 plus the byte.
pub fn fsdecode_units(bytes: &[u8]) -> Vec<u16> {
    let mut units = Vec::new();
    let mut rest = bytes;
    loop {
        match std::str::from_utf8(rest) {
            Ok(text) => {
                units.extend(text.encode_utf16());
                return units;
            }
            Err(error) => {
                let valid = error.valid_up_to();
                let head = std::str::from_utf8(&rest[..valid]).expect("the valid prefix");
                units.extend(head.encode_utf16());
                match error.error_len() {
                    Some(len) => {
                        for byte in &rest[valid..valid + len] {
                            units.push(0xDC00 + *byte as u16);
                        }
                        rest = &rest[valid + len..];
                    }
                    None => {
                        for byte in &rest[valid..] {
                            units.push(0xDC00 + *byte as u16);
                        }
                        return units;
                    }
                }
            }
        }
    }
}

/// The scalar a surrogate pair encodes.
pub fn decode_pair(high: u16, low: u16) -> u32 {
    0x10000 + (((high as u32) - 0xD800) << 10) + ((low as u32) - 0xDC00)
}

/// The inverse: encode text units back to OS bytes,
/// mapping surrogateescape units back to their bytes.
/// A lone surrogate outside the escape range cannot
/// be encoded; it renders as the UTF-8 replacement,
/// a case argument strings from the OS never reach
/// (their surrogates come only from fsdecode).
#[cfg(unix)]
pub fn fsencode_bytes(units: &[u16]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut index = 0;
    while index < units.len() {
        let unit = units[index];
        if (0xDC80..=0xDCFF).contains(&unit) {
            out.push((unit - 0xDC00) as u8);
            index += 1;
        } else if (0xD800..=0xDBFF).contains(&unit)
            && index + 1 < units.len()
            && (0xDC00..=0xDFFF).contains(&units[index + 1])
        {
            let scalar = decode_pair(unit, units[index + 1]);
            let mut buf = [0u8; 4];
            out.extend_from_slice(
                char::from_u32(scalar)
                    .expect("a valid pair")
                    .encode_utf8(&mut buf)
                    .as_bytes(),
            );
            index += 2;
        } else if (0xD800..=0xDFFF).contains(&unit) {
            out.extend_from_slice("\u{FFFD}".as_bytes());
            index += 1;
        } else {
            let mut buf = [0u8; 4];
            out.extend_from_slice(
                char::from_u32(unit as u32)
                    .expect("a BMP scalar")
                    .encode_utf8(&mut buf)
                    .as_bytes(),
            );
            index += 1;
        }
    }
    out
}

/// Render text for a stderr message the way CPython's
/// stderr does (its error handler is backslashreplace):
/// valid text passes through, and every lone surrogate
/// becomes its \uXXXX escape.
pub fn stderr_render(units: &[u16]) -> String {
    let mut out = String::new();
    let mut index = 0;
    while index < units.len() {
        let unit = units[index];
        if (0xD800..=0xDBFF).contains(&unit)
            && index + 1 < units.len()
            && (0xDC00..=0xDFFF).contains(&units[index + 1])
        {
            let scalar = decode_pair(unit, units[index + 1]);
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

/// Render text the way CPython's stdout does (its
/// error handler is surrogateescape): a lone
/// surrogate in DC80 to DCFF writes as its raw byte,
/// and any other lone surrogate fails the write.
pub fn stdout_render(units: &[u16]) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut index = 0;
    while index < units.len() {
        let unit = units[index];
        if (0xD800..=0xDBFF).contains(&unit)
            && index + 1 < units.len()
            && (0xDC00..=0xDFFF).contains(&units[index + 1])
        {
            let scalar = decode_pair(unit, units[index + 1]);
            let mut buf = [0u8; 4];
            out.extend_from_slice(
                char::from_u32(scalar)
                    .expect("a valid pair")
                    .encode_utf8(&mut buf)
                    .as_bytes(),
            );
            index += 2;
        } else if (0xDC80..=0xDCFF).contains(&unit) {
            out.push((unit - 0xDC00) as u8);
            index += 1;
        } else if (0xD800..=0xDFFF).contains(&unit) {
            return None;
        } else {
            let mut buf = [0u8; 4];
            out.extend_from_slice(
                char::from_u32(unit as u32)
                    .expect("a BMP scalar")
                    .encode_utf8(&mut buf)
                    .as_bytes(),
            );
            index += 1;
        }
    }
    Some(out)
}

/// Normalize a path's display form the way pathlib
/// does when a Path is printed: empty and dot
/// components drop, duplicate slashes collapse (a
/// leading pair of slashes is preserved), a trailing
/// slash drops, and an empty result is a dot. Dot-dot
/// components are kept; nothing is resolved.
pub fn normalize_path_display(units: &[u16]) -> Vec<u16> {
    let slash = '/' as u16;
    let absolute = units.first() == Some(&slash);
    let double = absolute && units.get(1) == Some(&slash) && units.get(2) != Some(&slash);
    let mut parts: Vec<&[u16]> = Vec::new();
    for part in units.split(|unit| *unit == slash) {
        if part.is_empty() || part == ['.' as u16] {
            continue;
        }
        parts.push(part);
    }
    let mut out: Vec<u16> = Vec::new();
    if double {
        out.extend_from_slice(&[slash, slash]);
    } else if absolute {
        out.push(slash);
    }
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            out.push(slash);
        }
        out.extend_from_slice(part);
    }
    if out.is_empty() {
        out.push('.' as u16);
    }
    out
}

/// A message under construction, held as text units
/// so a name the Python inserts raw (an experiment
/// id, a path) keeps its exact units until the
/// message is rendered for its destination: stderr
/// renders backslashreplace, stdout surrogateescape.
#[derive(Clone, Debug, PartialEq)]
pub struct Msg(pub Vec<u16>);

impl Default for Msg {
    fn default() -> Msg {
        Msg::new()
    }
}

impl Msg {
    pub fn new() -> Msg {
        Msg(Vec::new())
    }

    pub fn str(text: &str) -> Msg {
        let mut msg = Msg::new();
        msg.push_str(text);
        msg
    }

    pub fn push_str(&mut self, text: &str) {
        self.0.extend(text.encode_utf16());
    }

    pub fn push_text(&mut self, text: &Text) {
        self.0.extend_from_slice(text.units());
    }

    pub fn push_msg(&mut self, other: &Msg) {
        self.0.extend_from_slice(&other.0);
    }

    pub fn render_stderr(&self) -> String {
        stderr_render(&self.0)
    }

    pub fn render_stdout(&self) -> Option<Vec<u8>> {
        stdout_render(&self.0)
    }
}

/// Join messages with a separator, the way the
/// Python joins defect strings.
pub fn join_msgs(items: &[Msg], separator: &str) -> Msg {
    let mut out = Msg::new();
    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            out.push_str(separator);
        }
        out.push_msg(item);
    }
    out
}

/// The container nesting depths of a JSON text,
/// counted over brackets outside strings: the
/// maximum depth of any container, and the maximum
/// depth at which an object opens. The Python
/// reader's recursion turns on both measures (the
/// registry loader's hook spends stack as each
/// object closes, so a deep object refuses where a
/// deep array still parses). A scalar text has
/// depth 0.
pub fn json_text_depths(text: &str) -> (usize, usize) {
    let mut depth = 0usize;
    let mut max = 0usize;
    let mut max_object = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for ch in text.chars() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '[' => {
                depth += 1;
                max = max.max(depth);
            }
            '{' => {
                depth += 1;
                max = max.max(depth);
                max_object = max_object.max(depth);
            }
            ']' | '}' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    (max, max_object)
}

/// What a registry line's own parse refuses beyond
/// the crate reader's acceptance (VL-15): duplicate
/// object keys at any level, and the NaN and
/// Infinity constants the Python's reader is told
/// to reject. The scan runs only on lines the crate
/// reader already accepted, so it may assume the
/// grammar held; every refusal it finds surfaces as
/// the same not-valid-JSON defect as a parse error.
pub struct LineScan {
    pub duplicate_key: bool,
    pub constant: bool,
}

pub fn scan_line(text: &str) -> LineScan {
    let chars: Vec<char> = text.chars().collect();
    let mut scan = LineScan {
        duplicate_key: false,
        constant: false,
    };
    let mut pos = 0usize;
    scan_value(&chars, &mut pos, &mut scan);
    scan
}

fn skip_ws(chars: &[char], pos: &mut usize) {
    while *pos < chars.len() && matches!(chars[*pos], ' ' | '\t' | '\n' | '\r') {
        *pos += 1;
    }
}

fn scan_string(chars: &[char], pos: &mut usize) -> Vec<u16> {
    // chars[*pos] is the opening quote.
    let mut units = Vec::new();
    *pos += 1;
    while *pos < chars.len() {
        let ch = chars[*pos];
        *pos += 1;
        match ch {
            '"' => break,
            '\\' => {
                if *pos >= chars.len() {
                    break;
                }
                let esc = chars[*pos];
                *pos += 1;
                match esc {
                    'u' => {
                        let mut unit = 0u16;
                        for _ in 0..4 {
                            if *pos < chars.len() {
                                unit = unit * 16 + chars[*pos].to_digit(16).unwrap_or(0) as u16;
                                *pos += 1;
                            }
                        }
                        units.push(unit);
                    }
                    'n' => units.push('\n' as u16),
                    'r' => units.push('\r' as u16),
                    't' => units.push('\t' as u16),
                    'b' => units.push('\u{8}' as u16),
                    'f' => units.push('\u{C}' as u16),
                    other => units.push(other as u16),
                }
            }
            _ => {
                let mut buf = [0u16; 2];
                units.extend_from_slice(ch.encode_utf16(&mut buf));
            }
        }
    }
    units
}

fn scan_value(chars: &[char], pos: &mut usize, scan: &mut LineScan) {
    skip_ws(chars, pos);
    if *pos >= chars.len() {
        return;
    }
    match chars[*pos] {
        '{' => {
            *pos += 1;
            let mut keys: Vec<Vec<u16>> = Vec::new();
            loop {
                skip_ws(chars, pos);
                if *pos >= chars.len() {
                    return;
                }
                if chars[*pos] == '}' {
                    *pos += 1;
                    return;
                }
                if chars[*pos] == '"' {
                    let key = scan_string(chars, pos);
                    if keys.contains(&key) {
                        scan.duplicate_key = true;
                    } else {
                        keys.push(key);
                    }
                }
                skip_ws(chars, pos);
                if *pos < chars.len() && chars[*pos] == ':' {
                    *pos += 1;
                }
                scan_value(chars, pos, scan);
                skip_ws(chars, pos);
                if *pos < chars.len() && chars[*pos] == ',' {
                    *pos += 1;
                }
            }
        }
        '[' => {
            *pos += 1;
            loop {
                skip_ws(chars, pos);
                if *pos >= chars.len() {
                    return;
                }
                if chars[*pos] == ']' {
                    *pos += 1;
                    return;
                }
                scan_value(chars, pos, scan);
                skip_ws(chars, pos);
                if *pos < chars.len() && chars[*pos] == ',' {
                    *pos += 1;
                }
            }
        }
        '"' => {
            scan_string(chars, pos);
        }
        _ => {
            // A bare token: a number or one of the
            // literal words. Only NaN and the two
            // Infinities are refusals.
            let mut word = String::new();
            while *pos < chars.len()
                && (chars[*pos].is_ascii_alphanumeric() || matches!(chars[*pos], '+' | '-' | '.'))
            {
                word.push(chars[*pos]);
                *pos += 1;
            }
            if word == "NaN" || word == "Infinity" || word == "-Infinity" {
                scan.constant = true;
            }
        }
    }
}

/// An exact integer: a sign and a magnitude held as
/// decimal digits, most significant first, no leading
/// zeros; zero has an empty magnitude and no sign.
#[derive(Clone, Debug, PartialEq)]
pub struct Big {
    pub neg: bool,
    pub mag: Vec<u8>,
}

impl Big {
    pub fn zero() -> Big {
        Big {
            neg: false,
            mag: Vec::new(),
        }
    }

    /// From the crate's canonical integer form (an
    /// optional minus and digits).
    pub fn from_int_str(text: &str) -> Big {
        let (neg, digits) = match text.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, text),
        };
        let mut mag: Vec<u8> = digits.bytes().map(|byte| byte - b'0').collect();
        let lead = mag.iter().take_while(|digit| **digit == 0).count();
        mag.drain(..lead.min(mag.len()));
        if mag.is_empty() {
            return Big::zero();
        }
        Big { neg, mag }
    }

    pub fn is_zero(&self) -> bool {
        self.mag.is_empty()
    }

    fn cmp_mag(a: &[u8], b: &[u8]) -> Ordering {
        if a.len() != b.len() {
            return a.len().cmp(&b.len());
        }
        a.cmp(b)
    }

    pub fn compare(&self, other: &Big) -> Ordering {
        match (self.neg, other.neg) {
            (false, true) => Ordering::Greater,
            (true, false) => Ordering::Less,
            (false, false) => Big::cmp_mag(&self.mag, &other.mag),
            (true, true) => Big::cmp_mag(&other.mag, &self.mag),
        }
    }

    fn add_mag(a: &[u8], b: &[u8]) -> Vec<u8> {
        let mut out = Vec::with_capacity(a.len().max(b.len()) + 1);
        let mut carry = 0u16;
        let mut i = a.len();
        let mut j = b.len();
        while i > 0 || j > 0 || carry > 0 {
            let mut sum = carry;
            if i > 0 {
                i -= 1;
                sum += a[i] as u16;
            }
            if j > 0 {
                j -= 1;
                sum += b[j] as u16;
            }
            out.push((sum % 10) as u8);
            carry = sum / 10;
        }
        out.reverse();
        let lead = out.iter().take_while(|digit| **digit == 0).count();
        out.drain(..lead.min(out.len()));
        out
    }

    /// a - b, for a >= b as magnitudes.
    fn sub_mag(a: &[u8], b: &[u8]) -> Vec<u8> {
        let mut out = vec![0u8; a.len()];
        let mut borrow = 0i16;
        let mut i = a.len();
        let mut j = b.len();
        while i > 0 {
            i -= 1;
            let mut digit = a[i] as i16 - borrow;
            if j > 0 {
                j -= 1;
                digit -= b[j] as i16;
            }
            if digit < 0 {
                digit += 10;
                borrow = 1;
            } else {
                borrow = 0;
            }
            out[i] = digit as u8;
        }
        let lead = out.iter().take_while(|digit| **digit == 0).count();
        out.drain(..lead.min(out.len()));
        out
    }

    pub fn add(&self, other: &Big) -> Big {
        if self.neg == other.neg {
            let mag = Big::add_mag(&self.mag, &other.mag);
            if mag.is_empty() {
                return Big::zero();
            }
            return Big { neg: self.neg, mag };
        }
        match Big::cmp_mag(&self.mag, &other.mag) {
            Ordering::Equal => Big::zero(),
            Ordering::Greater => Big {
                neg: self.neg,
                mag: Big::sub_mag(&self.mag, &other.mag),
            },
            Ordering::Less => Big {
                neg: other.neg,
                mag: Big::sub_mag(&other.mag, &self.mag),
            },
        }
    }

    pub fn sub(&self, other: &Big) -> Big {
        let flipped = Big {
            neg: !other.neg && !other.is_zero(),
            mag: other.mag.clone(),
        };
        self.add(&flipped)
    }

    pub fn mul(&self, other: &Big) -> Big {
        if self.is_zero() || other.is_zero() {
            return Big::zero();
        }
        let mut acc = vec![0u32; self.mag.len() + other.mag.len()];
        for (i, a) in self.mag.iter().enumerate() {
            for (j, b) in other.mag.iter().enumerate() {
                acc[i + j + 1] += *a as u32 * *b as u32;
            }
        }
        for k in (1..acc.len()).rev() {
            acc[k - 1] += acc[k] / 10;
            acc[k] %= 10;
        }
        let mut mag: Vec<u8> = acc.iter().map(|digit| *digit as u8).collect();
        let lead = mag.iter().take_while(|digit| **digit == 0).count();
        mag.drain(..lead.min(mag.len()));
        Big {
            neg: self.neg != other.neg,
            mag,
        }
    }

    pub fn mul_small(&self, factor: u32) -> Big {
        if factor == 0 || self.is_zero() {
            return Big::zero();
        }
        let mut mag = self.mag.clone();
        let mut carry = 0u32;
        for digit in mag.iter_mut().rev() {
            let product = *digit as u32 * factor + carry;
            *digit = (product % 10) as u8;
            carry = product / 10;
        }
        let mut prefix = Vec::new();
        while carry > 0 {
            prefix.push((carry % 10) as u8);
            carry /= 10;
        }
        prefix.reverse();
        prefix.extend(mag);
        Big {
            neg: self.neg,
            mag: prefix,
        }
    }

    /// Multiply by 10^zeros: zeros append to the
    /// magnitude. Zero stays zero.
    pub fn shifted(&self, zeros: usize) -> Big {
        if self.is_zero() || zeros == 0 {
            return self.clone();
        }
        let mut mag = self.mag.clone();
        mag.extend(std::iter::repeat_n(0u8, zeros));
        Big { neg: self.neg, mag }
    }

    pub fn abs(&self) -> Big {
        Big {
            neg: false,
            mag: self.mag.clone(),
        }
    }

    fn digit_string(&self) -> String {
        self.mag.iter().map(|d| (b'0' + d) as char).collect()
    }
}

/// Integer division and remainder of non-negative
/// magnitudes, by decimal long division.
fn divmod_mag(a: &Big, b: &Big) -> (Big, Big) {
    debug_assert!(!a.neg && !b.neg && !b.is_zero());
    if Big::cmp_mag(&a.mag, &b.mag) == Ordering::Less {
        return (Big::zero(), a.clone());
    }
    let mut quotient: Vec<u8> = Vec::new();
    let mut remainder = Big::zero();
    for digit in a.mag.iter() {
        remainder = remainder.shifted(1);
        if *digit > 0 {
            remainder = remainder.add(&Big {
                neg: false,
                mag: vec![*digit],
            });
        }
        let mut q = 0u8;
        for cand in (1..=9u8).rev() {
            let product = b.mul_small(cand as u32);
            if Big::cmp_mag(&product.mag, &remainder.mag) != Ordering::Greater {
                q = cand;
                remainder = Big {
                    neg: false,
                    mag: Big::sub_mag(&remainder.mag, &product.mag),
                };
                break;
            }
        }
        quotient.push(q);
    }
    let lead = quotient.iter().take_while(|digit| **digit == 0).count();
    quotient.drain(..lead.min(quotient.len()));
    (
        Big {
            neg: false,
            mag: quotient,
        },
        remainder,
    )
}

/// The correctly rounded double of the exact rational
/// num / den (den positive and nonzero): the value
/// CPython's integer true division produces. The
/// quotient's decimal digits are taken by long
/// division; as many are taken as the rounding needs
/// (a truncated and an incremented reading that agree
/// on a double settle it, and an exact remainder
/// settles it at once).
pub fn rational_to_f64(num: &Big, den: &Big) -> f64 {
    if num.is_zero() {
        return 0.0;
    }
    let sign = if num.neg { -1.0 } else { 1.0 };
    let a = num.abs();
    let b = den.abs();
    // The decimal exponent: 10^e <= a/b < 10^(e+1).
    let mut e: i64 = a.mag.len() as i64 - b.mag.len() as i64;
    let at_least = |e: i64| -> bool {
        if e >= 0 {
            Big::cmp_mag(&a.mag, &b.shifted(e as usize).mag) != Ordering::Less
        } else {
            Big::cmp_mag(&a.shifted((-e) as usize).mag, &b.mag) != Ordering::Less
        }
    };
    if !at_least(e) {
        e -= 1;
    }
    let mut want = 25usize;
    loop {
        // q = floor(a * 10^(want-1-e) / b), exactly
        // `want` digits, with the remainder as sticky.
        let shift = want as i64 - 1 - e;
        let (dividend, divisor) = if shift >= 0 {
            (a.shifted(shift as usize), b.clone())
        } else {
            (a.clone(), b.shifted((-shift) as usize))
        };
        let (q, r) = divmod_mag(&dividend, &divisor);
        let digits = q.digit_string();
        debug_assert_eq!(digits.len(), want);
        let parse = |digits: &str| -> f64 {
            let text = format!("0.{}e{}", digits, e + 1);
            text.parse::<f64>().expect("a decimal reading")
        };
        let low = parse(&digits);
        if r.is_zero() {
            return sign * low;
        }
        let bumped = {
            let mut bytes: Vec<u8> = digits.bytes().collect();
            let mut i = bytes.len();
            let mut done = false;
            while i > 0 {
                i -= 1;
                if bytes[i] == b'9' {
                    bytes[i] = b'0';
                } else {
                    bytes[i] += 1;
                    done = true;
                    break;
                }
            }
            if done {
                String::from_utf8(bytes).expect("digits")
            } else {
                "1".to_string()
            }
        };
        let high = if bumped == "1" {
            format!("0.1e{}", e + 2)
                .parse::<f64>()
                .expect("a decimal reading")
        } else {
            parse(&bumped)
        };
        if low == high {
            return sign * low;
        }
        want *= 4;
        if want > 6400 {
            // Far past any double midpoint's decimal
            // length; the low reading stands.
            return sign * low;
        }
    }
}

/// Compare a finite double against an exact integer,
/// exactly, the way CPython compares a float with an
/// int.
pub fn cmp_f64_int(x: f64, digits: &str) -> Ordering {
    let m = Big::from_int_str(digits);
    if x == 0.0 {
        return match (m.neg, m.is_zero()) {
            (_, true) => Ordering::Equal,
            (false, false) => Ordering::Less,
            (true, false) => Ordering::Greater,
        };
    }
    let x_neg = x < 0.0;
    if x_neg != m.neg {
        return if x_neg {
            Ordering::Less
        } else {
            Ordering::Greater
        };
    }
    let ord = cmp_f64_abs_int(x.abs(), &m.abs());
    if x_neg {
        ord.reverse()
    } else {
        ord
    }
}

fn cmp_f64_abs_int(x: f64, m: &Big) -> Ordering {
    // x = 0.digits * 10^point, exactly.
    let (digits, point) = exact_decimal(x);
    let d: Vec<u8> = digits.bytes().map(|byte| byte - b'0').collect();
    let int_part: Vec<u8> = if point <= 0 {
        Vec::new()
    } else if (point as usize) >= d.len() {
        let mut part = d.clone();
        part.extend(std::iter::repeat_n(0u8, point as usize - d.len()));
        part
    } else {
        d[..point as usize].to_vec()
    };
    let frac_nonzero = if point <= 0 {
        true // x is a nonzero fraction below 1 with empty int part; digits are nonzero
    } else if (point as usize) >= d.len() {
        false
    } else {
        d[point as usize..].iter().any(|digit| *digit != 0)
    };
    let lead = int_part.iter().take_while(|digit| **digit == 0).count();
    let int_part = &int_part[lead.min(int_part.len())..];
    match Big::cmp_mag(int_part, &m.mag) {
        Ordering::Equal => {
            if frac_nonzero {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        }
        other => other,
    }
}

/// The first code point of each block of ten
/// Unicode decimal digits (the Nd category): every
/// block runs 0 to 9 from its start.
const DECIMAL_DIGIT_STARTS: [u32; 68] = [
    0x30, 0x660, 0x6F0, 0x7C0, 0x966, 0x9E6, 0xA66, 0xAE6, 0xB66, 0xBE6, 0xC66, 0xCE6, 0xD66,
    0xDE6, 0xE50, 0xED0, 0xF20, 0x1040, 0x1090, 0x17E0, 0x1810, 0x1946, 0x19D0, 0x1A80, 0x1A90,
    0x1B50, 0x1BB0, 0x1C40, 0x1C50, 0xA620, 0xA8D0, 0xA900, 0xA9D0, 0xA9F0, 0xAA50, 0xABF0, 0xFF10,
    0x104A0, 0x10D30, 0x11066, 0x110F0, 0x11136, 0x111D0, 0x112F0, 0x11450, 0x114D0, 0x11650,
    0x116C0, 0x11730, 0x118E0, 0x11950, 0x11C50, 0x11D50, 0x11DA0, 0x11F50, 0x16A60, 0x16AC0,
    0x16B50, 0x1D7CE, 0x1D7D8, 0x1D7E2, 0x1D7EC, 0x1D7F6, 0x1E140, 0x1E2F0, 0x1E4F0, 0x1E950,
    0x1FBF0,
];

/// A scalar's Unicode decimal digit value, where it
/// has one: the digit set CPython's float() and
/// argparse's negative number matcher read.
pub fn decimal_digit(scalar: u32) -> Option<u32> {
    let at = DECIMAL_DIGIT_STARTS.partition_point(|start| *start <= scalar);
    if at == 0 {
        return None;
    }
    let start = DECIMAL_DIGIT_STARTS[at - 1];
    let offset = scalar - start;
    if offset < 10 {
        Some(offset)
    } else {
        None
    }
}

/// The exact overflow threshold of CPython's integer
/// conversion and true division: 2^1024 - 2^970, the
/// rounding midpoint above the largest double. An
/// exact integer or quotient reaching it rounds to
/// an infinite double and CPython raises
/// OverflowError; below it, the largest double is
/// the correctly rounded result.
pub fn overflow_threshold() -> Big {
    let mut power = Big::from_int_str("1");
    for _ in 0..970 {
        power = power.mul_small(2);
    }
    // 2^970 * (2^54 - 1) = 2^1024 - 2^970.
    power.mul(&Big::from_int_str("18014398509481983"))
}

/// CPython's float(int): the correctly rounded
/// double of an exact integer, or None where the
/// conversion raises OverflowError.
pub fn int_to_f64(digits: &str) -> Option<f64> {
    let m = Big::from_int_str(digits);
    if Big::cmp_mag(&m.mag, &overflow_threshold().mag) != Ordering::Less {
        return None;
    }
    digits.parse::<f64>().ok()
}
