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
    use super::mock::MockRender;

    let mut mock_render = MockRender::new();
    mock_render
        .expect_render_patch()
        .times(1)
        .returning(|_| Ok(ArcStr::from("rendered content")));

    let result = mock_render
        .render_patch(ArcStr::from("test patch content"))
        .await
        .unwrap();
    assert_eq!(result, ArcStr::from("rendered content"));
}
