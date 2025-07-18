use super::data::{LoreMailingList, LorePage, LorePatchMetadata};
use crate::ArcStr;
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
            let datetime_str = format!("{} {}", date, time);
            let last_update = NaiveDateTime::parse_from_str(&datetime_str, "%Y-%m-%d %H:%M")
                .map(|ndt| DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc))
                .with_context(|| {
                    format!(
                        "Failed to parse date/time '{}' in mailing list entry: '{}'",
                        datetime_str, line
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
            .with_context(|| format!("Failed to parse next page index: '{}'", idx_str))?;
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
                    .with_context(|| format!("Failed to parse next page index: '{}'", next_str))?;
                next_page_index = Some(idx);
            }
        }

        if let Some(total) = cap.get(cap.len() - 1) {
            let total_str = total.as_str().replace(",", "");
            let total_val = total_str
                .parse::<usize>()
                .with_context(|| format!("Failed to parse total items: '{}'", total_str))?;
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
) -> anyhow::Result<LorePage<LorePatchMetadata>> {
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

    let items = feed
        .entries
        .into_iter()
        .map(|entry| {
            let datetime = DateTime::parse_from_rfc3339(&entry.updated)
                .map(|dt| dt.with_timezone(&Utc))
                .context("Failed to parse patch datetime")?;
            Ok(LorePatchMetadata {
                author: ArcStr::from(&entry.author.name),
                email: ArcStr::from(&entry.author.email),
                last_update: datetime,
                title: ArcStr::from(&entry.title),
                link: ArcStr::from(&entry.link.href.unwrap_or_default()),
            })
        })
        .collect::<Result<Vec<_>, anyhow::Error>>()?;

    Ok(LorePage {
        start_index,
        next_page_index: Some(start_index + items.len()),
        total_items: Some(items.len()),
        items,
    })
}
