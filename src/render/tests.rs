use std::collections::HashMap;

use super::Render;
use crate::{ArcStr, app::config::Renderer};

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
    assert!(delta_args.contains(&"--side-by-side=false"));
}

#[tokio::test]
async fn test_mock_render() {
    let render = Render::mock(HashMap::from([(ArcStr::from("test patch content"), ArcStr::from("rendered content"))]));

    let result = render.render_patch(ArcStr::from("test patch content")).await.unwrap();
    assert_eq!(result, ArcStr::from("rendered content"));
}
