pub fn strip_ansi(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut in_escape = false;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if let Some('[') = chars.peek() {
                chars.next(); // consume '['
                in_escape = true;
                continue;
            }
        }
        if in_escape {
            if c.is_ascii_alphabetic() {
                in_escape = false;
            }
            continue;
        }
        result.push(c);
    }
    result
}

pub fn compress_logs(text: &str, log_line_cap: usize) -> String {
    let stripped = strip_ansi(text);
    let mut output = String::with_capacity(stripped.len() / 5);

    let mut last_line: Option<String> = None;
    let mut dup_count = 0;

    let write_dup_msg = |out: &mut String, count: usize| {
        if count > 0 {
            out.push_str(&format!("[consecutive duplicate lines omitted: {}]\n", count));
        }
    };

    for line in stripped.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed == "---" {
            continue;
        }

        // Process line content: strip Base64 or HTML, or truncate
        let processed = if trimmed.len() > 200
            && trimmed
                .chars()
                .all(|c| c.is_alphanumeric() || c == '+' || c == '/' || c == '=')
        {
            "[BASE64_DATA_STRIPPED]".to_string()
        } else if trimmed.starts_with('<') && trimmed.ends_with('>') && trimmed.len() > 100 {
            "[HTML_STRIPPED]".to_string()
        } else if trimmed.len() > log_line_cap {
            format!("{}... [TRUNCATED]", &trimmed[..log_line_cap])
        } else {
            trimmed.to_string()
        };

        if let Some(ref prev) = last_line {
            if prev == &processed {
                dup_count += 1;
                continue;
            } else {
                write_dup_msg(&mut output, dup_count);
                dup_count = 0;
            }
        }

        output.push_str(&processed);
        output.push('\n');
        last_line = Some(processed);
    }

    write_dup_msg(&mut output, dup_count);
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_ansi() {
        assert_eq!(strip_ansi("\x1b[31;1mHello\x1b[0m World"), "Hello World");
        assert_eq!(strip_ansi("Normal text"), "Normal text");
    }

    #[test]
    fn test_compress_logs() {
        let log = "\
info: starting cycle
info: starting cycle
info: starting cycle
trimmed base64:
SGVsbG8gV29ybGQgZnJvbSBHWk1PIGNvbXByZXNzaW9uIGxheWVyIHdyaXR0ZW4gaW4gUnVzdC4=
SGVsbG8gV29ybGQgZnJvbSBHWk1PIGNvbXByZXNzaW9uIGxheWVyIHdyaXR0ZW4gaW4gUnVzdC4=
info: done
";
        let compressed = compress_logs(log, 500);
        let expected = "\
info: starting cycle
[consecutive duplicate lines omitted: 2]
trimmed base64:
SGVsbG8gV29ybGQgZnJvbSBHWk1PIGNvbXByZXNzaW9uIGxheWVyIHdyaXR0ZW4gaW4gUnVzdC4=
[consecutive duplicate lines omitted: 1]
info: done
";
        assert_eq!(compressed, expected);
    }
}
