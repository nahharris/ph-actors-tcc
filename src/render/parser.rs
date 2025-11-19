use crate::ArcStr;

/// Cleans an mbox file by removing all headers except Subject, From, and Date,
/// and preserving the body (commit message and diff).
///
/// # Arguments
/// * `mbox` - The raw mbox content as an ArcStr
///
/// # Returns
/// A cleaned version with only useful headers and the body
pub fn clean_mbox(mbox: ArcStr) -> ArcStr {
    let lines: Vec<&str> = mbox.lines().collect();
    let mut result = String::new();
    let mut current_header: Option<(&str, String)> = None;
    let mut body_start = 0;
    let mut start_idx = 0;

    // Skip mbox separator line if present (starts with "From " followed by space)
    // Mbox format: "From email@domain.com date"
    if !lines.is_empty() && lines[0].starts_with("From ") {
        // Check if it's a separator line (has space after "From " and looks like email/date)
        let first_line = lines[0];
        if first_line.len() > 5 {
            let after_from = &first_line[5..];
            // Mbox separator typically has email-like pattern or date
            if after_from.contains('@')
                || after_from
                    .chars()
                    .next()
                    .map_or(false, |c| c.is_alphabetic())
            {
                start_idx = 1;
            }
        }
    }

    // Find where headers end (blank line)
    for (i, line) in lines.iter().enumerate().skip(start_idx) {
        // Check if this is a blank line (end of headers)
        if line.trim().is_empty() {
            // Save the current header before ending
            if let Some((header_name, value)) = current_header.take() {
                if should_keep_header(header_name) {
                    result.push_str(header_name);
                    result.push_str(": ");
                    result.push_str(&value);
                    result.push_str("\n");
                }
            }
            body_start = i + 1;
            break;
        }

        // Check if this is a continuation line (starts with whitespace)
        // Note: continuation lines in email headers start with space or tab
        // We need to check this before checking for a new header
        if let Some((_header_name, value)) = current_header.as_mut() {
            if line.starts_with(' ') || line.starts_with('\t') {
                // Continuation line - append to current header value
                value.push_str("\n");
                value.push_str(line);
                continue;
            }
        }

        // New header line - save the previous header first
        if let Some((header_name, value)) = current_header.take() {
            // Save the previous header if it's one we want to keep
            if should_keep_header(header_name) {
                result.push_str(header_name);
                result.push_str(": ");
                result.push_str(&value);
                result.push_str("\n");
            }
        }

        // Parse new header
        if let Some(colon_pos) = line.find(':') {
            let header_name = &line[..colon_pos].trim();
            let header_value = line[colon_pos + 1..].trim();
            current_header = Some((header_name, header_value.to_string()));
        }
    }

    // Handle the last header if we're still in headers
    if let Some((header_name, value)) = current_header {
        if should_keep_header(header_name) {
            result.push_str(header_name);
            result.push_str(": ");
            result.push_str(&value);
            result.push_str("\n");
        }
    }

    // Add blank line before body
    if !result.is_empty() {
        result.push_str("\n");
    }

    // Add the body (everything after the blank line)
    if body_start < lines.len() {
        let body_lines = &lines[body_start..];
        result.push_str(&body_lines.join("\n"));
        // Ensure the output ends with a newline if the original content had one
        // This is important for tools like delta that expect proper line endings
        let mbox_str: &str = mbox.as_ref();
        if !mbox_str.is_empty() && mbox_str.ends_with('\n') {
            result.push('\n');
        }
    }

    ArcStr::from(result)
}

/// Checks if a header should be kept in the cleaned output.
///
/// # Arguments
/// * `header_name` - The name of the header (case-insensitive)
///
/// # Returns
/// `true` if the header should be kept, `false` otherwise
fn should_keep_header(header_name: &str) -> bool {
    let lower = header_name.to_lowercase();
    lower == "subject" || lower == "from" || lower == "date"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_mbox_keeps_required_headers() {
        let mbox = ArcStr::from(
            "From: test@example.com\n\
             Subject: Test Patch\n\
             Date: Mon, 1 Jan 2024 12:00:00 +0000\n\
             Message-ID: <test@example.com>\n\
             To: linux-kernel@vger.kernel.org\n\
             \n\
             This is the commit message.\n\
             \n\
             ---\n\
             diff --git a/file.c b/file.c\n\
             @@ -1,1 +1,2 @@\n\
             +new line",
        );

        let cleaned = clean_mbox(mbox);
        let cleaned_str: &str = cleaned.as_ref();

        assert!(cleaned_str.contains("From: test@example.com"));
        assert!(cleaned_str.contains("Subject: Test Patch"));
        assert!(cleaned_str.contains("Date: Mon, 1 Jan 2024 12:00:00 +0000"));
        assert!(!cleaned_str.contains("Message-ID:"));
        assert!(!cleaned_str.contains("To:"));
        assert!(cleaned_str.contains("This is the commit message."));
        assert!(cleaned_str.contains("diff --git"));
    }

    #[test]
    fn test_clean_mbox_handles_multiline_headers() {
        // Use a string that explicitly includes continuation lines with leading spaces
        let mbox = ArcStr::from(
            "From: John Doe\n <john@example.com>\nSubject: Multi-line\n Subject Header\nDate: Mon, 1 Jan 2024 12:00:00 +0000\n\nBody content",
        );

        let cleaned = clean_mbox(mbox);
        let cleaned_str: &str = cleaned.as_ref();

        // Multiline headers preserve the original whitespace from continuation lines
        assert!(cleaned_str.contains("From: John Doe"));
        assert!(cleaned_str.contains("<john@example.com>"));
        assert!(cleaned_str.contains("Subject: Multi-line"));
        assert!(cleaned_str.contains("Subject Header"));
        assert!(cleaned_str.contains("Date: Mon, 1 Jan 2024 12:00:00 +0000"));
        assert!(cleaned_str.contains("Body content"));
    }

    #[test]
    fn test_clean_mbox_case_insensitive_header_matching() {
        let mbox = ArcStr::from(
            "from: test@example.com\n\
             SUBJECT: Test Patch\n\
             DaTe: Mon, 1 Jan 2024 12:00:00 +0000\n\
             \n\
             Body",
        );

        let cleaned = clean_mbox(mbox);
        let cleaned_str: &str = cleaned.as_ref();

        assert!(cleaned_str.contains("from: test@example.com"));
        assert!(cleaned_str.contains("SUBJECT: Test Patch"));
        assert!(cleaned_str.contains("DaTe: Mon, 1 Jan 2024 12:00:00 +0000"));
    }

    #[test]
    fn test_clean_mbox_preserves_body() {
        let mbox = ArcStr::from(
            "From: test@example.com\n\
             Subject: Test\n\
             Date: Mon, 1 Jan 2024 12:00:00 +0000\n\
             \n\
             Commit message line 1\n\
             Commit message line 2\n\
             \n\
             Signed-off-by: Test <test@example.com>\n\
             \n\
             ---\n\
             diff --git a/test.c b/test.c\n\
             index 1234567..abcdefg 100644\n\
             --- a/test.c\n\
             +++ b/test.c\n\
             @@ -10,6 +10,7 @@\n\
              line 1\n\
              line 2\n\
             +new line\n\
              line 3",
        );

        let cleaned = clean_mbox(mbox);
        let cleaned_str: &str = cleaned.as_ref();

        // Check that body is preserved
        assert!(cleaned_str.contains("Commit message line 1"));
        assert!(cleaned_str.contains("Commit message line 2"));
        assert!(cleaned_str.contains("Signed-off-by: Test <test@example.com>"));
        assert!(cleaned_str.contains("diff --git"));
        assert!(cleaned_str.contains("+new line"));
    }

    #[test]
    fn test_clean_mbox_no_body() {
        let mbox = ArcStr::from(
            "From: test@example.com\n\
             Subject: Test\n\
             Date: Mon, 1 Jan 2024 12:00:00 +0000\n\
             \n",
        );

        let cleaned = clean_mbox(mbox);
        let cleaned_str: &str = cleaned.as_ref();

        assert!(cleaned_str.contains("From: test@example.com"));
        assert!(cleaned_str.contains("Subject: Test"));
        assert!(cleaned_str.contains("Date: Mon, 1 Jan 2024 12:00:00 +0000"));
    }

    #[test]
    fn test_clean_mbox_only_headers() {
        let mbox = ArcStr::from(
            "From: test@example.com\n\
             Subject: Test\n\
             Date: Mon, 1 Jan 2024 12:00:00 +0000",
        );

        let cleaned = clean_mbox(mbox);
        let cleaned_str: &str = cleaned.as_ref();

        assert!(cleaned_str.contains("From: test@example.com"));
        assert!(cleaned_str.contains("Subject: Test"));
        assert!(cleaned_str.contains("Date: Mon, 1 Jan 2024 12:00:00 +0000"));
    }

    #[test]
    fn test_clean_mbox_removes_unwanted_headers() {
        let mbox = ArcStr::from(
            "From: test@example.com\n\
             Subject: Test\n\
             Date: Mon, 1 Jan 2024 12:00:00 +0000\n\
             Message-ID: <test@example.com>\n\
             In-Reply-To: <other@example.com>\n\
             References: <ref@example.com>\n\
             List-Id: <list.example.com>\n\
             List-Unsubscribe: <unsub@example.com>\n\
             Content-Type: text/plain\n\
             \n\
             Body",
        );

        let cleaned = clean_mbox(mbox);
        let cleaned_str: &str = cleaned.as_ref();

        // Should keep these
        assert!(cleaned_str.contains("From: test@example.com"));
        assert!(cleaned_str.contains("Subject: Test"));
        assert!(cleaned_str.contains("Date: Mon, 1 Jan 2024 12:00:00 +0000"));

        // Should remove these
        assert!(!cleaned_str.contains("Message-ID:"));
        assert!(!cleaned_str.contains("In-Reply-To:"));
        assert!(!cleaned_str.contains("References:"));
        assert!(!cleaned_str.contains("List-Id:"));
        assert!(!cleaned_str.contains("List-Unsubscribe:"));
        assert!(!cleaned_str.contains("Content-Type:"));

        // Should keep body
        assert!(cleaned_str.contains("Body"));
    }

    #[test]
    fn test_clean_mbox_skips_mbox_separator() {
        // Test with mbox separator line (starts with "From " followed by email/date)
        let mbox = ArcStr::from(
            "From mboxrd@z Thu Jan  1 00:00:00 1970\n\
             From: test@example.com\n\
             Subject: Test Patch\n\
             Date: Mon, 1 Jan 2024 12:00:00 +0000\n\
             \n\
             This is the body.",
        );

        let cleaned = clean_mbox(mbox);
        let cleaned_str: &str = cleaned.as_ref();

        // Should skip the mbox separator line
        assert!(!cleaned_str.contains("From mboxrd@z"));
        // Should keep the actual From header
        assert!(cleaned_str.contains("From: test@example.com"));
        assert!(cleaned_str.contains("Subject: Test Patch"));
        assert!(cleaned_str.contains("Date: Mon, 1 Jan 2024 12:00:00 +0000"));
        assert!(cleaned_str.contains("This is the body."));
    }
}
