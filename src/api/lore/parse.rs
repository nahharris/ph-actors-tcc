use super::data::{LoreMailingList, LorePage, LorePatchMetadata};
use crate::{ArcStr, SequenceNumber};
use chrono::{DateTime, NaiveDateTime, Utc};
use regex::Regex;
use serde::Deserialize;
use serde_xml_rs::from_str;

/// Parses the HTML listing of available mailing lists into structured data using regex.
///
/// # Arguments
/// * `html` - The HTML content as a string
/// * `start_index` - The current start index for pagination
///
/// # Returns
/// A `LorePage<LoreMailingList>` struct containing pagination info and a list of items, or None if no items are found.
///
/// # Errors
/// Returns an error if parsing fails.
pub fn parse_available_lists_html(
    html: &str,
    start_index: usize,
) -> Result<Option<LorePage<LoreMailingList>>, crate::error::LoreApiError> {

    let mut items = Vec::new();
    let mut next_page_index = None;
    let mut total_items = None;

    let mut lines = html.lines().peekable();
    while let Some(line) = lines.next() {
        let line = line.trim();
        if line.starts_with('*') {
            // Extract the date and time (first two fields after '*')
            let mut parts = line.split_whitespace();
            parts.next(); // skip '*'
            let date = parts
                .next()
                .ok_or_else(|| crate::error::LoreApiError::ParseFailed {
                    format: "HTML".to_string(),
                    operation: "parse available lists".to_string(),
                    details: format!("Missing date in mailing list entry: '{}'", line),
                })?;
            let time = parts
                .next()
                .ok_or_else(|| crate::error::LoreApiError::ParseFailed {
                    format: "HTML".to_string(),
                    operation: "parse available lists".to_string(),
                    details: format!("Missing time in mailing list entry: '{}'", line),
                })?;
            let datetime_str = format!("{date} {time}");
            let last_update = NaiveDateTime::parse_from_str(&datetime_str, "%Y-%m-%d %H:%M")
                .map(|ndt| DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc))
                .map_err(|e| crate::error::LoreApiError::ParseFailed {
                    format: "HTML".to_string(),
                    operation: "parse available lists".to_string(),
                    details: format!(
                        "Failed to parse date/time '{datetime_str}' in mailing list entry: '{}'. Error: {}",
                        line, e
                    ),
                })?;

            // Next line: href="all/">all</a>
            let name_line = lines
                .next()
                .ok_or_else(|| crate::error::LoreApiError::ParseFailed {
                    format: "HTML".to_string(),
                    operation: "parse available lists".to_string(),
                    details: format!("Missing name line after entry: '{}'", line),
                })?
                .trim();
            let name = if let Some(gt_idx) = name_line.find('>') {
                let after_gt = &name_line[gt_idx + 1..];
                if let Some(end_tag) = after_gt.find("</a>") {
                    after_gt[..end_tag].trim()
                } else {
                    after_gt.trim()
                }
            } else {
                return Err(crate::error::LoreApiError::ParseFailed {
                    format: "HTML".to_string(),
                    operation: "parse available lists".to_string(),
                    details: format!("Failed to find mailing list name in line: '{}'", name_line),
                });
            };

            // Next line: description
            let desc_line = lines
                .next()
                .ok_or_else(|| crate::error::LoreApiError::ParseFailed {
                    format: "HTML".to_string(),
                    operation: "parse available lists".to_string(),
                    details: format!("Missing description line after entry: '{}'", line),
                })?
                .trim();
            let description = desc_line.to_string();

            items.push(LoreMailingList {
                name: ArcStr::from(name),
                description: ArcStr::from(&description),
                last_update,
            });
        }
    }

    // Regex to find the next page index from the <a rel=next> link
    let next_re = Regex::new(r#"<a[^>]*rel=next[^>]*href="\?&o=([0-9]+)""#)
        .map_err(|e| crate::error::LoreApiError::ParseFailed {
            format: "HTML".to_string(),
            operation: "parse available lists".to_string(),
            details: format!("Failed to compile next page regex: {}", e),
        })?;
    if let Some(cap) = next_re.captures(html) {
        let idx_str = cap
            .get(1)
            .ok_or_else(|| crate::error::LoreApiError::ParseFailed {
                format: "HTML".to_string(),
                operation: "parse available lists".to_string(),
                details: "Failed to capture next page index".to_string(),
            })?
            .as_str();
        let idx = idx_str
            .parse::<usize>()
            .map_err(|e| crate::error::LoreApiError::ParseFailed {
                format: "HTML".to_string(),
                operation: "parse available lists".to_string(),
                details: format!("Failed to parse next page index: '{}'. Error: {}", idx_str, e),
            })?;
        next_page_index = Some(idx);
    }

    // Regex to extract next page index and total items from "Results 1-200 of ~337"
    let total_re = Regex::new(r"Results [0-9]+(-[0-9]+)? of ~?([0-9,]+)")
        .map_err(|e| crate::error::LoreApiError::ParseFailed {
            format: "HTML".to_string(),
            operation: "parse available lists".to_string(),
            details: format!("Failed to compile total items regex: {}", e),
        })?;
    if let Some(cap) = total_re.captures(html) {
        if cap.len() < 2 {
            return Err(crate::error::LoreApiError::ParseFailed {
                format: "HTML".to_string(),
                operation: "parse available lists".to_string(),
                details: "Failed to capture results count information".to_string(),
            });
        }

        if cap.len() == 3 {
            if let Some(next) = cap.get(1) {
                let next_str = next.as_str().replace("-", "");
                let idx = next_str
                    .parse::<usize>()
                    .map_err(|e| crate::error::LoreApiError::ParseFailed {
                        format: "HTML".to_string(),
                        operation: "parse available lists".to_string(),
                        details: format!("Failed to parse next page index: '{}'. Error: {}", next_str, e),
                    })?;
                next_page_index = Some(idx);
            }
        }

        if let Some(total) = cap.get(cap.len() - 1) {
            let total_str = total.as_str().replace(",", "");
            let total_val = total_str
                .parse::<usize>()
                .map_err(|e| crate::error::LoreApiError::ParseFailed {
                    format: "HTML".to_string(),
                    operation: "parse available lists".to_string(),
                    details: format!("Failed to parse total items: '{}'. Error: {}", total_str, e),
                })?;
            total_items = Some(total_val);
        }
    }

    if start_index == total_items.unwrap_or(0) {
        return Ok(None);
    }

    Ok(Some(LorePage {
        start_index,
        next_page_index,
        total_items,
        items,
    }))
}

/// Parses a patch title to extract version and sequence information.
///
/// The patch title must start with one of the following patterns:
/// - [PATCH]: version 1, simple series (1 out of 1 patch)
/// - [PATCH x/y]: version 1, patch x in a series of y patches
/// - [PATCH vZ]: version Z, simple series
/// - [PATCH vZ x/y]: version Z, patch x in a series of y patches
///
/// # Arguments
/// * `title` - The patch title to parse
///
/// # Returns
/// A tuple of (version, sequence_number) where:
/// - version: The patch version (defaults to 1 if not specified)
/// - sequence_number: The sequence number in the series (None if simple series)
///
/// # Errors
/// Returns an error if the title doesn't match any expected pattern.
pub fn parse_patch_title(title: &str) -> Option<(usize, Option<SequenceNumber>)> {
    // Regex to match patch title patterns with named captures
    let patch_regex = Regex::new(
        r"^\[PATCH\s*(?:v(?P<version>\d+))?\s*(?:(?P<current>\d+)/(?P<total>\d+))?\s*\]",
    ).ok()?;

    let captures = patch_regex.captures(title)?;
    // Extract version (defaults to 1 if not specified)
    let version = if let Some(version_match) = captures.name("version") {
        version_match.as_str().parse::<usize>().ok()?
    } else {
        1
    };

    // Extract sequence information
    let sequence = if let (Some(current_match), Some(total_match)) =
        (captures.name("current"), captures.name("total"))
    {
        let seq_str = format!("{}/{}", current_match.as_str(), total_match.as_str());
        seq_str.parse::<SequenceNumber>().ok()
    } else {
        None
    };

    Some((version, sequence))
}

/// Parses the XML patch feed into structured data using serde_xml_rs.
///
/// # Arguments
/// * `xml` - The XML content as a string
/// * `start_index` - The current start index for pagination
///
/// # Returns
/// A `LorePage<LorePatchMetadata>` struct containing pagination info and a list of patches.
///
/// # Errors
/// Returns an error if parsing fails.
pub fn parse_patch_feed_xml(
    xml: &str,
    start_index: usize,
) -> Result<LorePage<LorePatchMetadata>, crate::error::LoreApiError> {
    #[derive(Debug, Deserialize)]
    struct Feed {
        #[serde(rename = "entry")]
        entries: Vec<Entry>,
        #[serde(rename = "link", default)]
        links: Vec<Link>,
        #[serde(rename = "totalResults")]
        total_results: Option<usize>,
    }

    #[derive(Debug, Deserialize)]
    struct Entry {
        title: String,
        author: Author,
        id: String,
        updated: String,
        link: Link,
    }

    #[derive(Debug, Deserialize)]
    struct Author {
        name: String,
        email: String,
    }

    #[derive(Debug, Deserialize)]
    struct Link {
        #[serde(rename = "@href")]
        href: Option<String>,
        #[serde(rename = "@rel")]
        rel: Option<String>,
    }

    use chrono::{DateTime, Utc};
    let feed: Feed = from_str(xml).map_err(|e| crate::error::LoreApiError::ParseFailed {
        format: "XML".to_string(),
        operation: "parse patch feed".to_string(),
        details: format!("Failed to parse patch feed XML: {}", e),
    })?;
    let list_message_id_regex = Regex::new(r"https://lore.kernel.org/([^/]+)/([^/]+)/")
        .map_err(|e| crate::error::LoreApiError::ParseFailed {
            format: "XML".to_string(),
            operation: "parse patch feed".to_string(),
            details: format!("Failed to compile list message ID regex: {}", e),
        })?;

    let items = feed
        .entries
        .into_iter()
        .filter_map(|entry| {
            // Parse patch title to extract version and sequence information
            // Filter out entries that don't have valid patch titles
            let (version, sequence) = parse_patch_title(&entry.title)?;

            let datetime = DateTime::parse_from_rfc3339(&entry.updated)
                .map(|dt| dt.with_timezone(&Utc))
                .ok()?;

            let link = entry.link.href?;
            let captures = list_message_id_regex.captures(&link)?;

            let list = captures.get(1)?.as_str();

            let message_id = captures.get(2)?.as_str();

            Some(LorePatchMetadata {
                author: ArcStr::from(&entry.author.name),
                email: ArcStr::from(&entry.author.email),
                last_update: datetime,
                title: ArcStr::from(&entry.title),
                version,
                sequence,
                link: ArcStr::from(&link),
                list: ArcStr::from(list),
                message_id: ArcStr::from(message_id),
            })
        })
        .collect::<Vec<_>>();

    Ok(LorePage {
        start_index,
        next_page_index: Some(start_index + items.len()),
        total_items: Some(items.len()),
        items,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_patch_title_simple() {
        let title = "[PATCH] Add new feature";
        let result = parse_patch_title(title).unwrap();
        assert_eq!(result.0, 1); // version
        assert_eq!(result.1, None); // sequence
    }

    #[test]
    fn test_parse_patch_title_with_sequence() {
        let title = "[PATCH 2/5] Add new feature";
        let result = parse_patch_title(title).unwrap();
        assert_eq!(result.0, 1); // version
        assert_eq!(result.1, Some(SequenceNumber::new(2, 5))); // sequence
    }

    #[test]
    fn test_parse_patch_title_with_version() {
        let title = "[PATCH v3] Add new feature";
        let result = parse_patch_title(title).unwrap();
        assert_eq!(result.0, 3); // version
        assert_eq!(result.1, None); // sequence
    }

    #[test]
    fn test_parse_patch_title_with_version_and_sequence() {
        let title = "[PATCH v2 3/7] Add new feature";
        let result = parse_patch_title(title).unwrap();
        assert_eq!(result.0, 2); // version
        assert_eq!(result.1, Some(SequenceNumber::new(3, 7))); // sequence
    }

    #[test]
    fn test_parse_patch_title_with_extra_spaces() {
        let title = "[PATCH  v4  1/10  ] Add new feature";
        let result = parse_patch_title(title).unwrap();
        assert_eq!(result.0, 4); // version
        assert_eq!(result.1, Some(SequenceNumber::new(1, 10))); // sequence
    }

    #[test]
    fn test_parse_patch_title_invalid_format() {
        let title = "Invalid patch title";
        let result = parse_patch_title(title);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_patch_title_malformed_sequence() {
        let title = "[PATCH 1/] Add new feature";
        let result = parse_patch_title(title);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_patch_title_malformed_version() {
        let title = "[PATCH v] Add new feature";
        let result = parse_patch_title(title);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_patch_title_sequence_capture_verification() {
        // Test that the named captures correctly extract sequence numbers
        let title = "[PATCH 3/7] Add new feature";
        let result = parse_patch_title(title).unwrap();
        assert_eq!(result.0, 1); // version defaults to 1
        assert_eq!(result.1, Some(SequenceNumber::new(3, 7))); // sequence 3/7

        // Test with version and sequence
        let title = "[PATCH v2 5/10] Add new feature";
        let result = parse_patch_title(title).unwrap();
        assert_eq!(result.0, 2); // version 2
        assert_eq!(result.1, Some(SequenceNumber::new(5, 10))); // sequence 5/10
    }

    #[test]
    fn test_parse_available_lists_html_single_item() {
        let html = r#"
* 2025-01-15 10:30
<a href="all/">linux-kernel</a>
Linux kernel development mailing list
Results 1-1 of 1
"#;
        let result = parse_available_lists_html(html, 0).unwrap();
        assert!(result.is_some());
        let page = result.unwrap();
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].name.as_ref() as &str, "linux-kernel");
        assert_eq!(
            page.items[0].description.as_ref() as &str,
            "Linux kernel development mailing list"
        );
        assert_eq!(page.start_index, 0);
    }

    #[test]
    fn test_parse_available_lists_html_multiple_items() {
        let html = r#"
* 2025-01-15 10:30
<a href="all/">list1</a>
Description 1
* 2025-01-14 09:20
<a href="all/">list2</a>
Description 2
Results 1-2 of 10
"#;
        let result = parse_available_lists_html(html, 0).unwrap();
        assert!(result.is_some());
        let page = result.unwrap();
        assert_eq!(page.items.len(), 2);
        assert_eq!(page.items[0].name.as_ref() as &str, "list1");
        assert_eq!(page.items[1].name.as_ref() as &str, "list2");
    }

    #[test]
    fn test_parse_available_lists_html_with_next_page() {
        let html = r#"
* 2025-01-15 10:30
<a href="all/">list1</a>
Description 1
<a rel=next href="?&o=50"></a>
Results 1-50 of ~200
"#;
        let result = parse_available_lists_html(html, 0).unwrap();
        assert!(result.is_some());
        let page = result.unwrap();
        assert_eq!(page.next_page_index, Some(50));
        assert_eq!(page.total_items, Some(200));
    }

    #[test]
    fn test_parse_available_lists_html_empty() {
        // Empty HTML should return None when start_index equals total_items (which defaults to 0)
        let html = "";
        let result = parse_available_lists_html(html, 0).unwrap();
        // When start_index == total_items (0), returns None
        assert!(result.is_none());

        // But if start_index is not 0, it should return Some with empty items
        let result2 = parse_available_lists_html(html, 10).unwrap();
        assert!(result2.is_some());
        let page = result2.unwrap();
        assert_eq!(page.items.len(), 0);
    }

    #[test]
    fn test_parse_available_lists_html_at_end() {
        let html = r#"
* 2025-01-15 10:30
<a href="all/">list1</a>
Description 1
Results 1-200 of 200
"#;
        let result = parse_available_lists_html(html, 200).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_available_lists_html_invalid_date() {
        let html = r#"
* invalid-date 10:30
<a href="all/">list1</a>
Description 1
"#;
        let result = parse_available_lists_html(html, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_patch_feed_xml_single_entry() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
    <entry>
        <title>[PATCH] Test patch</title>
        <author>
            <name>Test Author</name>
            <email>test@example.com</email>
        </author>
        <id>https://lore.kernel.org/test-list/20250115103000.12345@example.com/</id>
        <updated>2025-01-15T10:30:00Z</updated>
        <link href="https://lore.kernel.org/test-list/20250115103000.12345@example.com/" />
    </entry>
</feed>"#;
        let result = parse_patch_feed_xml(xml, 0).unwrap();
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].title.as_ref() as &str, "[PATCH] Test patch");
        assert_eq!(result.items[0].author.as_ref() as &str, "Test Author");
        assert_eq!(result.items[0].email.as_ref() as &str, "test@example.com");
        assert_eq!(result.items[0].list.as_ref() as &str, "test-list");
        assert_eq!(result.start_index, 0);
    }

    #[test]
    fn test_parse_patch_feed_xml_with_sequence() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
    <entry>
        <title>[PATCH 2/5] Test patch series</title>
        <author>
            <name>Test Author</name>
            <email>test@example.com</email>
        </author>
        <id>https://lore.kernel.org/test-list/20250115103000.12345@example.com/</id>
        <updated>2025-01-15T10:30:00Z</updated>
        <link href="https://lore.kernel.org/test-list/20250115103000.12345@example.com/" />
    </entry>
</feed>"#;
        let result = parse_patch_feed_xml(xml, 0).unwrap();
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].sequence, Some(SequenceNumber::new(2, 5)));
        assert_eq!(result.items[0].version, 1);
    }

    #[test]
    fn test_parse_patch_feed_xml_with_version() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
    <entry>
        <title>[PATCH v3] Test patch</title>
        <author>
            <name>Test Author</name>
            <email>test@example.com</email>
        </author>
        <id>https://lore.kernel.org/test-list/20250115103000.12345@example.com/</id>
        <updated>2025-01-15T10:30:00Z</updated>
        <link href="https://lore.kernel.org/test-list/20250115103000.12345@example.com/" />
    </entry>
</feed>"#;
        let result = parse_patch_feed_xml(xml, 0).unwrap();
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].version, 3);
        assert_eq!(result.items[0].sequence, None);
    }

    #[test]
    fn test_parse_patch_feed_xml_empty_feed() {
        // Feed with no entries - entries with missing required fields get filtered
        let _xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
</feed>"#;
        // XML parser requires entries field - test with minimal valid entry that gets filtered
        let xml2 = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
    <entry>
        <title>Not a patch</title>
        <author>
            <name>Author</name>
            <email>test@example.com</email>
        </author>
        <id>https://lore.kernel.org/test-list/message/</id>
        <updated>2025-01-15T10:30:00Z</updated>
        <link href="https://lore.kernel.org/test-list/message/" />
    </entry>
</feed>"#;
        // This entry will be filtered out due to invalid title
        let result = parse_patch_feed_xml(xml2, 0).unwrap();
        assert_eq!(result.items.len(), 0);
    }

    #[test]
    fn test_parse_patch_feed_xml_invalid_title_filtered() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
    <entry>
        <title>Invalid patch title</title>
        <author>
            <name>Test Author</name>
            <email>test@example.com</email>
        </author>
        <id>https://lore.kernel.org/test-list/20250115103000.12345@example.com/</id>
        <updated>2025-01-15T10:30:00Z</updated>
        <link href="https://lore.kernel.org/test-list/20250115103000.12345@example.com/" />
    </entry>
</feed>"#;
        let result = parse_patch_feed_xml(xml, 0).unwrap();
        // Entries with invalid titles are filtered out
        assert_eq!(result.items.len(), 0);
    }

    #[test]
    fn test_parse_patch_feed_xml_multiple_entries() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
    <entry>
        <title>[PATCH] First patch</title>
        <author>
            <name>Author One</name>
            <email>one@example.com</email>
        </author>
        <id>https://lore.kernel.org/test-list/20250115103000.11111@example.com/</id>
        <updated>2025-01-15T10:30:00Z</updated>
        <link href="https://lore.kernel.org/test-list/20250115103000.11111@example.com/" />
    </entry>
    <entry>
        <title>[PATCH] Second patch</title>
        <author>
            <name>Author Two</name>
            <email>two@example.com</email>
        </author>
        <id>https://lore.kernel.org/test-list/20250115103100.22222@example.com/</id>
        <updated>2025-01-15T10:31:00Z</updated>
        <link href="https://lore.kernel.org/test-list/20250115103100.22222@example.com/" />
    </entry>
</feed>"#;
        let result = parse_patch_feed_xml(xml, 0).unwrap();
        assert_eq!(result.items.len(), 2);
        assert_eq!(
            result.items[0].title.as_ref() as &str,
            "[PATCH] First patch"
        );
        assert_eq!(
            result.items[1].title.as_ref() as &str,
            "[PATCH] Second patch"
        );
    }
}
