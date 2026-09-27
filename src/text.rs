//! Ruby's string behaviour where a seed depends on it.
//!
//! A room's population is seeded from a checksum of its name, so the name has
//! to be folded exactly as Ruby folds it: `String#downcase` never applies the
//! final-sigma rule, `[[:space:]]` is Unicode whitespace, `\s` and `strip`
//! are ASCII.

/// Ruby's `String#downcase`: full Unicode lowercasing, one character at a
/// time, so "Σ" always becomes "σ" (Rust's `str::to_lowercase` would give a
/// final "ς").
pub fn ruby_downcase(text: &str) -> String {
    text.chars().flat_map(char::to_lowercase).collect()
}

/// The characters Ruby's `String#strip` removes: NUL and ASCII whitespace.
pub fn is_ruby_strip(c: char) -> bool {
    matches!(c, '\0' | ' ' | '\t' | '\n' | '\u{0b}' | '\u{0c}' | '\r')
}

/// The characters a Ruby `\s` matches: ASCII whitespace only.
pub fn is_ruby_space(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\u{0b}' | '\u{0c}' | '\r')
}

pub fn ruby_strip(text: &str) -> &str {
    text.trim_matches(is_ruby_strip)
}

/// Every run of Unicode whitespace (`[[:space:]]+`) becomes one space.
pub fn squish_spaces(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_space = false;
    for c in text.chars() {
        if c.is_whitespace() {
            if !in_space {
                out.push(' ');
            }
            in_space = true;
        } else {
            out.push(c);
            in_space = false;
        }
    }
    out
}

/// `WorldSeed.natural_key`: lowercased, whitespace squeezed, stripped, and
/// one leading "the", "a" or "an" (with the ASCII spaces after it) removed.
pub fn natural_key(name: &str) -> String {
    let folded = squish_spaces(&ruby_downcase(name));
    let folded = ruby_strip(&folded);
    ruby_strip(strip_leading_article(folded)).to_string()
}

/// `\A(?:the|a|an)\s+`. The alternation tries "the", then "a", then "an",
/// and each needs at least one space after it.
fn strip_leading_article(text: &str) -> &str {
    for article in ["the", "a", "an"] {
        if let Some(rest) = text.strip_prefix(article) {
            let trimmed = rest.trim_start_matches(is_ruby_space);
            if trimmed.len() < rest.len() {
                return trimmed;
            }
        }
    }
    text
}

/// Zlib's CRC-32 (the IEEE polynomial, reflected), as `Zlib.crc32` computes.
pub fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xedb8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

/// Every run of ASCII whitespace (`\s+`) becomes one space.
pub fn squish_ascii(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_space = false;
    for c in text.chars() {
        if is_ruby_space(c) {
            if !in_space {
                out.push(' ');
            }
            in_space = true;
        } else {
            out.push(c);
            in_space = false;
        }
    }
    out
}

/// Rails' `blank?` for a string: empty, or nothing but whitespace.
pub fn is_blank(text: &str) -> bool {
    text.chars().all(char::is_whitespace)
}

/// Ruby's awk-style `String#split` with no pattern: runs of ASCII whitespace
/// separate words, and there are no empty words.
pub fn split_words(text: &str) -> Vec<&str> {
    text.split(is_ruby_space)
        .filter(|word| !word.is_empty())
        .collect()
}

/// Ruby's `String#inspect` for a UTF-8 string: quoted, with a backslash in
/// front of what would otherwise end or interpolate it.
pub fn inspect(text: &str) -> String {
    let mut out = String::from("\"");
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            '\u{0c}' => out.push_str("\\f"),
            '\u{0b}' => out.push_str("\\v"),
            '\u{07}' => out.push_str("\\a"),
            '\u{08}' => out.push_str("\\b"),
            '\u{1b}' => out.push_str("\\e"),
            '#' if matches!(chars.peek(), Some('{' | '$' | '@')) => out.push_str("\\#"),
            c if c.is_control() => out.push_str(&format!("\\u{:04X}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Ruby's `String#upcase_first`.
pub fn upcase_first(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// Ruby's `String#casecmp?`: equal once both are case-folded.
pub fn casecmp(a: &str, b: &str) -> bool {
    ruby_downcase(a) == ruby_downcase(b)
}
