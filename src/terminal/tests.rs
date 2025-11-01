use super::*;
use crate::{
    ArcPath, ArcStr, api::lore::LoreMailingList, app::config::mock::MockConfig, fs::mock::MockFs,
    log::Log,
};
use chrono::Utc;

async fn create_test_log() -> Log {
    let temp_dir = tempfile::tempdir().unwrap();
    let log_dir = ArcPath::from(temp_dir.path());

    let mut mock_fs = MockFs::new();
    let mut mock_config = MockConfig::new();

    mock_config
        .expect_log_level()
        .returning(|| crate::log::LogLevel::Info);
    mock_config
        .expect_usize()
        .with(mockall::predicate::eq(crate::app::config::USizeOpt::MaxAge))
        .returning(|_| 0);
    mock_config
        .expect_path()
        .with(mockall::predicate::eq(crate::app::config::PathOpt::LogDir))
        .returning(move |_| log_dir.clone());

    mock_fs.expect_mkdir().returning(|_| Ok(()));
    mock_fs.expect_write_file().times(2).returning(|_| {
        let file = tempfile::tempfile().unwrap();
        Ok(tokio::fs::File::from_std(file))
    });

    Log::spawn(mock_fs, mock_config).await.unwrap()
}

#[tokio::test]
async fn test_terminal_show_screen() {
    let log = create_test_log().await;

    let (terminal, _handle) = Terminal::spawn(log);

    // Test showing different screen types
    terminal
        .show(Screen::Loading(ArcStr::from("Test loading")))
        .await;

    terminal
        .show(Screen::Error(ArcStr::from("Test error")))
        .await;

    // Test Lists screen
    let items = vec![LoreMailingList {
        name: ArcStr::from("test-list"),
        description: ArcStr::from("Test description"),
        last_update: Utc::now(),
    }];
    terminal
        .show(Screen::Lists {
            items,
            page: 0,
            selected: 0,
        })
        .await;

    // Cleanup
    terminal.quit().await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_terminal_get_ui_event_empty_queue() {
    let log = create_test_log().await;

    let (terminal, _handle) = Terminal::spawn(log);

    // Get event from empty queue should return None
    let event = terminal.get_ui_event().await;
    assert_eq!(event, None);

    // Cleanup
    terminal.quit().await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_terminal_clear_ui_events() {
    let log = create_test_log().await;

    let (terminal, _handle) = Terminal::spawn(log);

    // Clear events should complete without error
    terminal.clear_ui_events().await;

    // Cleanup
    terminal.quit().await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_terminal_quit() {
    let log = create_test_log().await;

    let (terminal, _handle) = Terminal::spawn(log);

    // Quit should complete successfully
    terminal.quit().await;

    // Give a bit of time for cleanup
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_terminal_show_all_screen_variants() {
    let log = create_test_log().await;

    let (terminal, _handle) = Terminal::spawn(log);

    // Test all screen variants
    terminal
        .show(Screen::Loading(ArcStr::from("Loading...")))
        .await;

    terminal.show(Screen::Error(ArcStr::from("Error!"))).await;

    // Lists screen
    let lists = vec![
        LoreMailingList {
            name: ArcStr::from("list1"),
            description: ArcStr::from("Description 1"),
            last_update: Utc::now(),
        },
        LoreMailingList {
            name: ArcStr::from("list2"),
            description: ArcStr::from("Description 2"),
            last_update: Utc::now(),
        },
    ];
    terminal
        .show(Screen::Lists {
            items: lists,
            page: 1,
            selected: 0,
        })
        .await;

    // Patch screen
    terminal
        .show(Screen::Patch {
            title: ArcStr::from("Test Patch"),
            content: ArcStr::from("Patch content here"),
        })
        .await;

    // Cleanup
    terminal.quit().await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}
