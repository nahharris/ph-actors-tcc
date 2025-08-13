use super::data::{LoreFeedItem, LoreMailingList, LorePage, LorePatch};
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
) -> anyhow::Result<Option<LorePage<LoreMailingList>>> {
    use anyhow::{Context, anyhow};

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
                .ok_or_else(|| anyhow!("Missing date in mailing list entry: '{}'", line))?;
            let time = parts
                .next()
                .ok_or_else(|| anyhow!("Missing time in mailing list entry: '{}'", line))?;
            let datetime_str = format!("{date} {time}");
            let last_update = NaiveDateTime::parse_from_str(&datetime_str, "%Y-%m-%d %H:%M")
                .map(|ndt| DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc))
                .with_context(|| {
                    format!(
                        "Failed to parse date/time '{datetime_str}' in mailing list entry: '{line}'"
                    )
                })?;

            // Next line: href="all/">all</a>
            let name_line = lines
                .next()
                .ok_or_else(|| anyhow!("Missing name line after entry: '{}'", line))?
                .trim();
            let name = if let Some(gt_idx) = name_line.find('>') {
                let after_gt = &name_line[gt_idx + 1..];
                if let Some(end_tag) = after_gt.find("</a>") {
                    after_gt[..end_tag].trim()
                } else {
                    after_gt.trim()
                }
            } else {
                return Err(anyhow!(
                    "Failed to find mailing list name in line: '{}'",
                    name_line
                ));
            };

            // Next line: description
            let desc_line = lines
                .next()
                .ok_or_else(|| anyhow!("Missing description line after entry: '{}'", line))?
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
        .context("Failed to compile next page regex")?;
    if let Some(cap) = next_re.captures(html) {
        let idx_str = cap
            .get(1)
            .ok_or_else(|| anyhow!("Failed to capture next page index"))?
            .as_str();
        let idx = idx_str
            .parse::<usize>()
            .with_context(|| format!("Failed to parse next page index: '{idx_str}'"))?;
        next_page_index = Some(idx);
    }

    // Regex to extract next page index and total items from "Results 1-200 of ~337"
    let total_re = Regex::new(r"Results [0-9]+(-[0-9]+)? of ~?([0-9,]+)")
        .context("Failed to compile total items regex")?;
    if let Some(cap) = total_re.captures(html) {
        if cap.len() < 2 {
            return Err(anyhow!("Failed to capture results count information"));
        }

        if cap.len() == 3 {
            if let Some(next) = cap.get(1) {
                let next_str = next.as_str().replace("-", "");
                let idx = next_str
                    .parse::<usize>()
                    .with_context(|| format!("Failed to parse next page index: '{next_str}'"))?;
                next_page_index = Some(idx);
            }
        }

        if let Some(total) = cap.get(cap.len() - 1) {
            let total_str = total.as_str().replace(",", "");
            let total_val = total_str
                .parse::<usize>()
                .with_context(|| format!("Failed to parse total items: '{total_str}'"))?;
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
pub fn parse_patch_title(title: &str) -> anyhow::Result<(usize, Option<SequenceNumber>, ArcStr)> {
    use anyhow::{Context, anyhow};

    // Regex to match patch title patterns with named captures
    let patch_regex = Regex::new(
        r"^\[PATCH\s*(?:v(?P<version>\d+))?\s*(?:(?P<current>\d+)/(?P<total>\d+))?\s*\](?P<title>.*)$",
    )
    .context("Failed to compile patch title regex")?;

    if let Some(captures) = patch_regex.captures(title) {
        // Extract version (defaults to 1 if not specified)
        let version = if let Some(version_match) = captures.name("version") {
            version_match.as_str().parse::<usize>().with_context(|| {
                format!(
                    "Failed to parse version number: '{}'",
                    version_match.as_str()
                )
            })?
        } else {
            1
        };

        // Extract sequence information
        let sequence = if let (Some(current_match), Some(total_match)) =
            (captures.name("current"), captures.name("total"))
        {
            let seq_str = format!("{}/{}", current_match.as_str(), total_match.as_str());
            seq_str
                .parse::<SequenceNumber>()
                .with_context(|| format!("Failed to parse sequence number: '{}'", seq_str))
                .map(Some)?
        } else {
            None
        };

        let Some(title) = captures
            .name("title")
            .map(|m| ArcStr::from(m.as_str().trim()))
        else {
            return Err(anyhow!("Failed to capture title"));
        };

        Ok((version, sequence, title))
    } else {
        // If the title doesn't match the expected pattern, skip it
        Err(anyhow!(
            "Patch title does not match expected format: '{}'",
            title
        ))
    }
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
) -> anyhow::Result<LorePage<LoreFeedItem>> {
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

    use anyhow::Context;
    use chrono::{DateTime, Utc};
    let feed: Feed = from_str(xml).context("Failed to parse patch feed XML")?;
    let list_message_id_regex = Regex::new(r"https://lore.kernel.org/([^/]+)/([^/]+)/")
        .context("Failed to compile list message ID regex")?;

    let items = feed
        .entries
        .into_iter()
        .filter_map(|entry| {
            // Parse patch title to extract version and sequence information
            let (version, sequence, title) = parse_patch_title(&entry.title).ok()?;

            let datetime = DateTime::parse_from_rfc3339(&entry.updated)
                .map(|dt| dt.with_timezone(&Utc))
                .ok()?;

            let link = entry.link.href?;
            let captures = list_message_id_regex.captures(&link)?;

            let list = captures.get(1)?.as_str();

            let message_id = captures.get(2)?.as_str();

            Some(LoreFeedItem {
                author: ArcStr::from(&entry.author.name),
                email: ArcStr::from(&entry.author.email),
                last_update: datetime,
                title,
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

pub fn parse_patch_mbox(mbox: &str) -> anyhow::Result<LorePatch> {
    use anyhow::{Context, anyhow};

    let mut lines = mbox.lines().peekable();

    // Parse email headers
    let mut from = None;
    let mut to = Vec::new();
    let mut cc = Vec::new();
    let mut bcc = Vec::new();
    let mut subject = None;
    let mut date = None;

    // Parse headers until we hit an empty line
    while let Some(line) = lines.next() {
        if line.trim().is_empty() {
            break;
        }

        if line.starts_with("From: ") {
            from = Some(ArcStr::from(line[6..].trim()));
        } else if line.starts_with("To: ") {
            let recipients = &line[4..].trim();
            to.extend(parse_email_list(recipients));
            while let Some(line) = lines.peek() {
                if line.starts_with("\t") || line.starts_with("  ") {
                    let line = lines.next().expect("Peek should have a line");
                    let recipients = &line[1..].trim();
                    to.extend(parse_email_list(recipients));
                } else {
                    break;
                }
            }
        } else if line.starts_with("CC: ") {
            let recipients = &line[4..].trim();
            cc.extend(parse_email_list(recipients));
            while let Some(line) = lines.peek() {
                if line.starts_with("\t") || line.starts_with("  ") {
                    let line = lines.next().expect("Peek should have a line");
                    let recipients = &line[1..].trim();
                    cc.extend(parse_email_list(recipients));
                } else {
                    break;
                }
            }
        } else if line.starts_with("BCC: ") {
            let recipients = &line[5..].trim();
            bcc.extend(parse_email_list(recipients));
            while let Some(line) = lines.peek() {
                if line.starts_with("\t") || line.starts_with("  ") {
                    let line = lines.next().expect("Peek should have a line");
                    let recipients = &line[1..].trim();
                    bcc.extend(parse_email_list(recipients));
                } else {
                    break;
                }
            }
        } else if line.starts_with("Subject: ") {
            subject = Some(ArcStr::from(line[9..].trim()));
        } else if line.starts_with("Date: ") {
            let date_str = line[6..].trim();
            date = Some(
                parse_email_date(date_str)
                    .with_context(|| format!("Failed to parse date: '{}'", date_str))?,
            );
        }
    }

    // Validate required fields
    let from = from.ok_or_else(|| anyhow!("Missing From header"))?;
    let subject = subject.ok_or_else(|| anyhow!("Missing Subject header"))?;
    let date = date.ok_or_else(|| anyhow!("Missing Date header"))?;

    // Parse patch title to extract version and sequence
    let (version, sequence, title) = parse_patch_title(&subject)
        .with_context(|| format!("Failed to parse patch title: '{}'", subject))?;

    // Default to 1/1 if no sequence is specified (simple patch)
    let sequence = sequence.unwrap_or_else(|| SequenceNumber::new(1, 1));

    // Collect the message body (everything until the diff starts)
    let mut message_lines = Vec::new();
    let mut diff_lines = Vec::new();
    let mut in_diff = false;

    while let Some(line) = lines.next() {
        if line.starts_with("---") && !in_diff {
            in_diff = true;
            diff_lines.push(line);
        } else if in_diff {
            diff_lines.push(line);
        } else {
            message_lines.push(line);
        }
    }

    let message = ArcStr::from(message_lines.join("\n").replace("\t", "    ").trim());
    let diff = ArcStr::from(diff_lines.join("\n").replace("\t", "    ").trim());

    Ok(LorePatch {
        title,
        version,
        sequence,
        from,
        to,
        cc,
        bcc,
        date,
        message,
        diff,
    })
}

/// Parses a comma-separated list of email addresses
fn parse_email_list(email_list: &str) -> Vec<ArcStr> {
    email_list
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(ArcStr::from)
        .collect()
}

/// Parses an email date string into a DateTime<Utc>
fn parse_email_date(date_str: &str) -> anyhow::Result<DateTime<Utc>> {
    use chrono::{NaiveDateTime, TimeZone};

    // Try different date formats commonly used in email headers
    let formats = [
        "%a, %d %b %Y %H:%M:%S %z", // RFC 2822 format
        "%d %b %Y %H:%M:%S %z",     // Without day of week
        "%a, %d %b %Y %H:%M:%S %Z", // With timezone name
        "%d %b %Y %H:%M:%S %Z",     // Without day of week, with timezone name
    ];

    for format in &formats {
        if let Ok(dt) = chrono::DateTime::parse_from_str(date_str, format) {
            return Ok(dt.with_timezone(&Utc));
        }
    }

    // If none of the standard formats work, try parsing as naive datetime
    if let Ok(naive_dt) = NaiveDateTime::parse_from_str(date_str, "%a, %d %b %Y %H:%M:%S") {
        return Ok(Utc.from_utc_datetime(&naive_dt));
    }

    Err(anyhow::anyhow!("Failed to parse date: '{}'", date_str))
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
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_patch_title_malformed_sequence() {
        let title = "[PATCH 1/] Add new feature";
        let result = parse_patch_title(title);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_patch_title_malformed_version() {
        let title = "[PATCH v] Add new feature";
        let result = parse_patch_title(title);
        assert!(result.is_err());
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
    fn test_parse_patch_mbox() {
        let mbox_content = r#"From mboxrd@z Thu Jan  1 00:00:00 1970
Return-Path: <amd-gfx-bounces@lists.freedesktop.org>
From: Chenglei Xie <Chenglei.Xie@amd.com>
To: <amd-gfx@lists.freedesktop.org>
CC: <yunru.pan@amd.com>, 
    <Shravankumar.Gande@amd.com>, 
    Chenglei Xie <Chenglei.Xie@amd.com>
Subject: [PATCH] drm/amdgpu: refactor bad_page_work for corner case handling
Date: Fri, 8 Aug 2025 10:24:46 -0400
Message-ID: <20250808142447.2280-1-Chenglei.Xie@amd.com>

When a poison is consumed on the guest before the guest receives the host's poison creation msg, a corner case may occur to have poison_handler complete processing earlier than it should to cause the guest to hang waiting for the req_bad_pages reply during a VF FLR, resulting in the VM becoming inaccessible in stress tests.

To fix this issue, this patch refactored the mailbox sequence by seperating the bad_page_work into two parts req_bad_pages_work and handle_bad_pages_work.

Signed-off-by: Chenglei Xie <Chenglei.Xie@amd.com>
---
 drivers/gpu/drm/amd/amdgpu/amdgpu_virt.h |  3 +-
 drivers/gpu/drm/amd/amdgpu/mxgpu_ai.c    | 32 +++++++++++++++++++---
 drivers/gpu/drm/amd/amdgpu/mxgpu_nv.c    | 35 +++++++++++++++++++-----
 drivers/gpu/drm/amd/amdgpu/soc15.c       |  1 -
 4 files changed, 58 insertions(+), 13 deletions(-)

diff --git a/drivers/gpu/drm/amd/amdgpu/amdgpu_virt.h b/drivers/gpu/drm/amd/amdgpu/amdgpu_virt.h
index 3da3ebb1d9a1..58accf2259b3 100644
--- a/drivers/gpu/drm/amd/amdgpu/amdgpu_virt.h
+++ b/drivers/gpu/drm/amd/amdgpu/amdgpu_virt.h
@@ -267,7 +267,8 @@ struct amdgpu_virt {
 	struct amdgpu_irq_src		rcv_irq;
 
 	struct work_struct		flr_work;
-	struct work_struct		bad_pages_work;
+	struct work_struct		req_bad_pages_work;
+	struct work_struct		handle_bad_pages_work;
 
 	struct amdgpu_mm_table		mm_table
 	const struct amdgpu_virt_ops	*ops;
"#;

        let result = parse_patch_mbox(mbox_content).unwrap();

        assert_eq!(
            result.from,
            ArcStr::from("Chenglei Xie <Chenglei.Xie@amd.com>")
        );
        assert_eq!(
            result.title,
            ArcStr::from("drm/amdgpu: refactor bad_page_work for corner case handling")
        );
        assert_eq!(result.version, 1);
        assert_eq!(result.sequence, SequenceNumber::new(1, 1));
        assert_eq!(result.to.len(), 1);
        assert_eq!(
            result.to[0],
            ArcStr::from("<amd-gfx@lists.freedesktop.org>")
        );
        assert_eq!(result.cc.len(), 3);
        assert!(result.message.contains("When a poison is consumed"));
        assert!(result.diff.contains("diff --git"));
    }

    #[test]
    fn test_parse_patch_mbox_with_sequence() {
        let mbox_content = r#"From: Test Author <test@example.com>
To: <list@example.com>
Subject: [PATCH 2/5] Add new feature
Date: Fri, 8 Aug 2025 10:24:46 -0400

This is the second patch in a series of 5.

Signed-off-by: Test Author <test@example.com>
---
 file.c | 2 +-
 1 file changed, 1 insertion(+), 1 deletion(-)

diff --git a/file.c b/file.c
index 1234567..abcdefg 100644
--- a/file.c
+++ b/file.c
@@ -1,3 +1,3 @@
-old line
+new line
 other line
"#;

        let result = parse_patch_mbox(mbox_content).unwrap();

        assert_eq!(result.from, ArcStr::from("Test Author <test@example.com>"));
        assert_eq!(result.title, ArcStr::from("Add new feature"));
        assert_eq!(result.version, 1);
        assert_eq!(result.sequence, SequenceNumber::new(2, 5));
        assert_eq!(result.to.len(), 1);
        assert_eq!(result.to[0], ArcStr::from("<list@example.com>"));
        assert!(result.message.contains("This is the second patch"));
        assert!(result.diff.contains("diff --git"));
    }
}
