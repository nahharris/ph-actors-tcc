use std::collections::HashMap;

use super::Render;
use crate::{ArcStr, SequenceNumber, api::lore::data::LorePatch, app::config::Renderer};
use chrono::Utc;

#[tokio::test]
async fn test_renderer_program_names() {
    assert_eq!(Renderer::Bat.program_name(), "bat");
    assert_eq!(Renderer::Delta.program_name(), "delta");
}

#[tokio::test]
async fn test_renderer_default_args() {
    let bat_args = Renderer::Bat.default_args();
    assert!(bat_args.contains(&"--language=diff"));
    assert!(bat_args.contains(&"--paging=never"));
    assert!(bat_args.contains(&"--style=numbers"));

    let delta_args = Renderer::Delta.default_args();
    assert!(delta_args.contains(&"--paging=never"));
}

#[tokio::test]
async fn test_mock_render() {
    let test_patch = LorePatch {
        title: ArcStr::from("test patch content"),
        version: 1,
        sequence: SequenceNumber::new(1, 1),
        from: ArcStr::from("test@example.com"),
        to: vec![ArcStr::from("list@example.com")],
        cc: vec![],
        bcc: vec![],
        date: Utc::now(),
        message: ArcStr::from("test message"),
        diff: ArcStr::from("test diff"),
    };

    let diff_content = format!("{}", test_patch.diff);
    let render = Render::mock(HashMap::from([(
        ArcStr::from(diff_content),
        ArcStr::from("rendered content"),
    )]));

    let result = render.render_patch(test_patch).await.unwrap();
    assert_eq!(result, ArcStr::from("rendered content"));
}

#[tokio::test]
async fn test_patch_content_formatting() {
    let test_patch = LorePatch {
        title: ArcStr::from("test patch"),
        version: 1,
        sequence: SequenceNumber::new(1, 1),
        from: ArcStr::from("test@example.com"),
        to: vec![ArcStr::from("list@example.com")],
        cc: vec![],
        bcc: vec![],
        date: Utc::now(),
        message: ArcStr::from("test message"),
        diff: ArcStr::from("--- a/file.txt\n+++ b/file.txt\n@@ -1,1 +1,1 @@\n-old\n+new"),
    };

    // Test that the formatted content includes email headers and message but not diff
    let formatted_content = format!("{}", test_patch);
    assert!(formatted_content.contains("From: test@example.com"));
    assert!(formatted_content.contains("To: list@example.com"));
    assert!(formatted_content.contains("test message"));
    assert!(!formatted_content.contains("--- a/file.txt")); // Diff is no longer included

    // Test that the diff content is just the diff portion
    let diff_content = format!("{}", test_patch.diff);
    assert!(!diff_content.contains("From: test@example.com"));
    assert!(!diff_content.contains("To: list@example.com"));
    assert!(!diff_content.contains("test message"));
    assert!(diff_content.contains("--- a/file.txt"));
    assert!(diff_content.contains("+++ b/file.txt"));
}
