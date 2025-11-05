use super::*;
use crate::{ArcStr, net::mock::MockNet};
use std::collections::HashMap;

fn create_sample_patch_feed_xml() -> ArcStr {
    ArcStr::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <totalResults>42</totalResults>
  <entry>
    <title>[PATCH] Test patch title</title>
    <author>
      <name>Test Author</name>
      <email>test@example.com</email>
    </author>
    <id>https://lore.kernel.org/test-list/20231201.123456.1-1@example.com/</id>
    <updated>2023-12-01T12:34:56Z</updated>
    <link href="https://lore.kernel.org/test-list/20231201.123456.1-1@example.com/" />
  </entry>
  <link rel="next" href="?x=A&amp;q=((s:patch+OR+s:rfc)+AND+NOT+s:re:)&amp;o=25" />
</feed>"#,
    )
}

fn create_sample_available_lists_html() -> ArcStr {
    ArcStr::from(
        r#"
    * 2023-12-01 12:34
    <a href="all/">test-list</a>
    Test list description
    * 2023-12-02 13:45
    <a href="all/">another-list</a>
    Another list description
    <a rel=next href="?&o=25"></a>
    Results 1-25 of ~50
"#,
    )
}

#[tokio::test]
async fn test_lore_api_get_patch_feed_page_success() {
    let mut mock_net = MockNet::new();
    let target_list = ArcStr::from("test-list");
    let min_index = 0;

    let expected_url = format!(
        "https://lore.kernel.org/{}/?x=A&q=((s:patch+OR+s:rfc)+AND+NOT+s:re:)&o={}",
        target_list, min_index
    );

    mock_net
        .expect_get()
        .with(
            mockall::predicate::eq(ArcStr::from(&expected_url)),
            mockall::predicate::function(|headers: &Option<HashMap<ArcStr, ArcStr>>| {
                headers
                    .as_ref()
                    .map(|h| {
                        h.get(&ArcStr::from("Accept"))
                            .map(|v| {
                                <ArcStr as AsRef<str>>::as_ref(v)
                                    == "text/html,application/xhtml+xml,application/xml"
                            })
                            .unwrap_or(false)
                    })
                    .unwrap_or(false)
            }),
        )
        .times(1)
        .returning(|_, _| Ok(create_sample_patch_feed_xml()));

    let lore_api = LoreApi::spawn(mock_net);
    let result = lore_api.get_patch_feed_page(target_list, min_index).await;

    assert!(result.is_ok());
    let page = result.unwrap();
    assert!(page.is_some());
    let page = page.unwrap();
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.start_index, min_index);
}

#[tokio::test]
async fn test_lore_api_get_patch_feed_page_end_of_feed() {
    let mut mock_net = MockNet::new();
    let target_list = ArcStr::from("test-list");
    let min_index = 0;

    mock_net
        .expect_get()
        .times(1)
        .returning(|_, _| Ok(ArcStr::from("</feed>")));

    let lore_api = LoreApi::spawn(mock_net);
    let result = lore_api.get_patch_feed_page(target_list, min_index).await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_lore_api_get_patch_feed_page_no_results() {
    let mut mock_net = MockNet::new();
    let target_list = ArcStr::from("test-list");
    let min_index = 0;

    mock_net
        .expect_get()
        .times(1)
        .returning(|_, _| Ok(ArcStr::from("[No results found]")));

    let lore_api = LoreApi::spawn(mock_net);
    let result = lore_api.get_patch_feed_page(target_list, min_index).await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_lore_api_get_available_lists_page_single_page() {
    let mut mock_net = MockNet::new();
    let min_index = 0;

    let expected_url = format!("https://lore.kernel.org/?&o={}", min_index);

    mock_net
        .expect_get()
        .with(
            mockall::predicate::eq(ArcStr::from(&expected_url)),
            mockall::predicate::function(|headers: &Option<HashMap<ArcStr, ArcStr>>| {
                headers
                    .as_ref()
                    .map(|h| {
                        h.get(&ArcStr::from("Accept"))
                            .map(|v| {
                                <ArcStr as AsRef<str>>::as_ref(v)
                                    == "text/html,application/xhtml+xml,application/xml"
                            })
                            .unwrap_or(false)
                    })
                    .unwrap_or(false)
            }),
        )
        .times(1)
        .returning(|_, _| Ok(create_sample_available_lists_html()));

    let lore_api = LoreApi::spawn(mock_net);
    let result = lore_api.get_available_lists_page(min_index).await;

    assert!(result.is_ok());
    let page = result.unwrap();
    assert!(page.is_some());
    let page = page.unwrap();
    assert_eq!(page.items.len(), 2);
    assert_eq!(page.items[0].name, ArcStr::from("test-list"));
    assert_eq!(page.items[1].name, ArcStr::from("another-list"));
}

#[tokio::test]
async fn test_lore_api_get_available_lists_multiple_pages() {
    let mut mock_net = MockNet::new();

    let page1_html = ArcStr::from(
        r#"
    * 2023-12-01 12:34
    <a href="all/">list1</a>
    List 1 description
    <a rel=next href="?&o=25"></a>
    Results 1-25 of ~50
"#,
    );

    let page2_html = ArcStr::from(
        r#"
    * 2023-12-02 13:45
    <a href="all/">list2</a>
    List 2 description
    Results 26 of 50
"#,
    );

    mock_net
        .expect_get()
        .with(
            mockall::predicate::eq(ArcStr::from("https://lore.kernel.org/?&o=0")),
            mockall::predicate::always(),
        )
        .times(1)
        .returning(move |_, _| Ok(page1_html.clone()));

    mock_net
        .expect_get()
        .with(
            mockall::predicate::eq(ArcStr::from("https://lore.kernel.org/?&o=25")),
            mockall::predicate::always(),
        )
        .times(1)
        .returning(move |_, _| Ok(page2_html.clone()));

    let lore_api = LoreApi::spawn(mock_net);
    let result = lore_api.get_available_lists().await;

    assert!(result.is_ok());
    let lists = result.unwrap();
    assert_eq!(lists.len(), 2);
    assert_eq!(lists[0].name, ArcStr::from("list1"));
    assert_eq!(lists[1].name, ArcStr::from("list2"));
}

#[tokio::test]
async fn test_lore_api_get_patch_html() {
    let mut mock_net = MockNet::new();
    let target_list = ArcStr::from("test-list");
    let message_id = ArcStr::from("20231201.123456.1-1@example.com");
    let html_content = ArcStr::from("<html><body>Patch HTML</body></html>");
    let expected_html_content = html_content.clone();

    let expected_url = format!("https://lore.kernel.org/{}/{}/", target_list, message_id);

    mock_net
        .expect_get()
        .with(
            mockall::predicate::eq(ArcStr::from(&expected_url)),
            mockall::predicate::function(|headers: &Option<HashMap<ArcStr, ArcStr>>| {
                headers
                    .as_ref()
                    .map(|h| {
                        h.get(&ArcStr::from("Accept"))
                            .map(|v| {
                                <ArcStr as AsRef<str>>::as_ref(v)
                                    == "text/html,application/xhtml+xml,application/xml"
                            })
                            .unwrap_or(false)
                    })
                    .unwrap_or(false)
            }),
        )
        .times(1)
        .returning(move |_, _| Ok(html_content.clone()));

    let lore_api = LoreApi::spawn(mock_net);
    let result = lore_api.get_patch_html(target_list, message_id).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), expected_html_content);
}

#[tokio::test]
async fn test_lore_api_get_raw_patch() {
    let mut mock_net = MockNet::new();
    let target_list = ArcStr::from("test-list");
    let message_id = ArcStr::from("20231201.123456.1-1@example.com");
    let raw_content = ArcStr::from("diff --git a/file.c b/file.c\n--- a/file.c\n+++ b/file.c");
    let expected_raw_content = raw_content.clone();

    let expected_url = format!("https://lore.kernel.org/{}/{}/raw", target_list, message_id);

    mock_net
        .expect_get()
        .with(
            mockall::predicate::eq(ArcStr::from(&expected_url)),
            mockall::predicate::function(|headers: &Option<HashMap<ArcStr, ArcStr>>| {
                headers
                    .as_ref()
                    .map(|h| {
                        h.get(&ArcStr::from("Accept"))
                            .map(|v| <ArcStr as AsRef<str>>::as_ref(v) == "text/plain")
                            .unwrap_or(false)
                    })
                    .unwrap_or(false)
            }),
        )
        .times(1)
        .returning(move |_, _| Ok(raw_content.clone()));

    let lore_api = LoreApi::spawn(mock_net);
    let result = lore_api.get_raw_patch(target_list, message_id).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), expected_raw_content);
}

#[tokio::test]
async fn test_lore_api_get_patch_metadata() {
    let mut mock_net = MockNet::new();
    let target_list = ArcStr::from("test-list");
    let message_id = ArcStr::from("20231201.123456.1-1@example.com");
    let json_content = ArcStr::from(r#"{"id": "test-id", "title": "Test Patch"}"#);
    let expected_json_content = json_content.clone();

    let expected_url = format!(
        "https://lore.kernel.org/{}/{}/json",
        target_list, message_id
    );

    mock_net
        .expect_get()
        .with(
            mockall::predicate::eq(ArcStr::from(&expected_url)),
            mockall::predicate::function(|headers: &Option<HashMap<ArcStr, ArcStr>>| {
                headers
                    .as_ref()
                    .map(|h| {
                        h.get(&ArcStr::from("Accept"))
                            .map(|v| <ArcStr as AsRef<str>>::as_ref(v) == "application/json")
                            .unwrap_or(false)
                    })
                    .unwrap_or(false)
            }),
        )
        .times(1)
        .returning(move |_, _| Ok(json_content.clone()));

    let lore_api = LoreApi::spawn(mock_net);
    let result = lore_api.get_patch_metadata(target_list, message_id).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), expected_json_content);
}

/*#[tokio::test]
async fn test_lore_api_network_error_propagation() {
    let mut mock_net = MockNet::new();
    let target_list = ArcStr::from("test-list");
    let min_index = 0;

    mock_net
        .expect_get()
        .times(1)
        .returning(|_, _| Err(NetError::RequestFailed {
            url: "https://lore.kernel.org/test-list/".to_string(),
            method: "GET".to_string(),
            retryable: false,
            source: reqwest::Error::(reqwest::StatusCode::INTERNAL_SERVER_ERROR, "Network error"),
            message: "Network error".to_string(),
        }));

    let lore_api = LoreApi::spawn(mock_net);
    let result = lore_api.get_patch_feed_page(target_list, min_index).await;

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    // The error is wrapped with context, so check for both parts
    assert!(
        error_msg.contains("Network error") || error_msg.contains("GET patch feed failed"),
        "Error message should contain 'Network error' or 'GET patch feed failed', got: {}",
        error_msg
    );
}*/
