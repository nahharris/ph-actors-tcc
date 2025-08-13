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

    let patch_string = format!("{}", test_patch);
    let render = Render::mock(HashMap::from([(
        ArcStr::from(patch_string),
        ArcStr::from("rendered content"),
    )]));

    let result = render.render_patch(test_patch).await.unwrap();
    assert_eq!(result, ArcStr::from("rendered content"));
}
