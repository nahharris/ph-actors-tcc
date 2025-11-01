use super::*;
use crate::app::config::mock::MockConfig;
use crate::shell::mock::MockShell;
use crate::{
    ArcSlice, ArcStr,
    app::config::{Renderer, RendererOpt},
    shell::data::{Command, Result, Status},
};

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

#[tokio::test]
async fn test_render_with_bat_renderer() {
    let mut mock_shell = MockShell::new();
    let mut mock_config = MockConfig::new();
    let patch_content = ArcStr::from("diff --git a/file.c b/file.c\n--- a/file.c");
    let rendered_content = ArcStr::from("rendered patch with bat");
    let expected_rendered_content = rendered_content.clone();

    mock_config
        .expect_renderer()
        .with(mockall::predicate::eq(RendererOpt::PatchRenderer))
        .times(1)
        .returning(|_| Renderer::Bat);

    mock_shell
        .expect_execute()
        .withf({
            let patch_content = patch_content.clone();
            move |program, args, stdin| {
                <ArcStr as AsRef<str>>::as_ref(program) == "bat"
                    && args
                        .iter()
                        .any(|a| <ArcStr as AsRef<str>>::as_ref(a) == "--language=diff")
                    && stdin.as_ref().map(|s| {
                        <ArcStr as AsRef<str>>::as_ref(s)
                            == <ArcStr as AsRef<str>>::as_ref(&patch_content)
                    }) == Some(true)
            }
        })
        .times(1)
        .returning(move |_, _, _| {
            Ok(Result::new(
                rendered_content.clone(),
                ArcStr::from(""),
                Status::Success(0),
                Command::new(ArcStr::from("bat"), ArcSlice::from([]), None),
            ))
        });

    let render = Render::spawn(mock_shell, mock_config);
    let result = render.render_patch(patch_content.clone()).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), expected_rendered_content);
}

#[tokio::test]
async fn test_render_with_delta_renderer() {
    let mut mock_shell = MockShell::new();
    let mut mock_config = MockConfig::new();
    let patch_content = ArcStr::from("diff --git a/file.c b/file.c\n--- a/file.c");
    let rendered_content = ArcStr::from("rendered patch with delta");

    mock_config
        .expect_renderer()
        .with(mockall::predicate::eq(RendererOpt::PatchRenderer))
        .times(1)
        .returning(|_| Renderer::Delta);

    mock_shell
        .expect_execute()
        .withf({
            let patch_content = patch_content.clone();
            move |program, args, stdin| {
                <ArcStr as AsRef<str>>::as_ref(program) == "delta"
                    && args
                        .iter()
                        .any(|a| <ArcStr as AsRef<str>>::as_ref(a) == "--paging=never")
                    && stdin.as_ref().map(|s| {
                        <ArcStr as AsRef<str>>::as_ref(s)
                            == <ArcStr as AsRef<str>>::as_ref(&patch_content)
                    }) == Some(true)
            }
        })
        .times(1)
        .returning({
            let rendered_content = rendered_content.clone();
            move |_, _, _| {
                Ok(Result::new(
                    rendered_content.clone(),
                    ArcStr::from(""),
                    Status::Success(0),
                    Command::new(ArcStr::from("delta"), ArcSlice::from([]), None),
                ))
            }
        });

    let expected_content = rendered_content.clone();
    let render = Render::spawn(mock_shell, mock_config);
    let result = render.render_patch(patch_content.clone()).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), expected_content);
}

#[tokio::test]
async fn test_render_with_none_renderer() {
    let mock_shell = MockShell::new();
    let mut mock_config = MockConfig::new();
    let patch_content = ArcStr::from("diff --git a/file.c b/file.c\n--- a/file.c");

    mock_config
        .expect_renderer()
        .with(mockall::predicate::eq(RendererOpt::PatchRenderer))
        .times(1)
        .returning(|_| Renderer::None);

    // Shell should never be called when renderer is None
    let render = Render::spawn(mock_shell, mock_config);
    let result = render.render_patch(patch_content.clone()).await;

    assert!(result.is_ok());
    // Should return content as-is when renderer is None
    assert_eq!(result.unwrap(), patch_content.clone());
}

#[tokio::test]
async fn test_render_shell_error_handling() {
    let mut mock_shell = MockShell::new();
    let mut mock_config = MockConfig::new();
    let patch_content = ArcStr::from("diff --git a/file.c b/file.c");

    mock_config
        .expect_renderer()
        .with(mockall::predicate::eq(RendererOpt::PatchRenderer))
        .times(1)
        .returning(|_| Renderer::Bat);

    mock_shell.expect_execute().times(1).returning(|_, _, _| {
        Ok(Result::new(
            ArcStr::from(""),
            ArcStr::from("bat: command not found"),
            Status::Success(1),
            Command::new(ArcStr::from("bat"), ArcSlice::from([]), None),
        ))
    });

    let render = Render::spawn(mock_shell, mock_config);
    let result = render.render_patch(patch_content).await;

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("bat"));
    assert!(error_msg.contains("status: Success(1)"));
    assert!(error_msg.contains("stderr: bat: command not found"));
}

#[tokio::test]
async fn test_render_command_with_stdin() {
    let mut mock_shell = MockShell::new();
    let mut mock_config = MockConfig::new();
    let patch_content = ArcStr::from("patch content to be piped");

    mock_config
        .expect_renderer()
        .with(mockall::predicate::eq(RendererOpt::PatchRenderer))
        .times(1)
        .returning(|_| Renderer::Bat);

    mock_shell
        .expect_execute()
        .withf({
            let patch_content = patch_content.clone();
            move |program, _, stdin| {
                <ArcStr as AsRef<str>>::as_ref(program) == "bat"
                    && stdin.as_ref().map(|s| {
                        <ArcStr as AsRef<str>>::as_ref(s)
                            == <ArcStr as AsRef<str>>::as_ref(&patch_content)
                    }) == Some(true)
            }
        })
        .times(1)
        .returning(|_, _, stdin| {
            let content = stdin.unwrap_or_default();
            Ok(Result::new(
                content,
                ArcStr::from(""),
                Status::Success(0),
                Command::new(ArcStr::from("bat"), ArcSlice::from([]), None),
            ))
        });

    let render = Render::spawn(mock_shell, mock_config);
    let result = render.render_patch(patch_content.clone()).await;

    assert!(result.is_ok());
    // Verify content was passed correctly
    assert_eq!(result.unwrap(), patch_content);
}
