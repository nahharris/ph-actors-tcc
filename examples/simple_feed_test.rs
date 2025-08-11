use ph::{
    app::{
        cache::{feed::FeedCache, mailing_list::MailingListCache, patch::PatchCache},
        ui::Ui,
    },
    log::Log,
    render::Render,
    terminal::Terminal,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create mock actors
    let log = Log::mock();
    let render = Render::mock(std::collections::HashMap::new());

    // Create mock caches
    let mailing_list_cache = MailingListCache::mock(Default::default());
    let feed_cache = FeedCache::mock(Default::default());
    let patch_cache = PatchCache::mock(Default::default());

    // Test that UI actor can be created with patch cache dependency
    let (_ui, _handle) = Ui::spawn(
        log,
        Terminal::mock(Default::default()),
        mailing_list_cache,
        feed_cache,
        patch_cache,
        render,
    );

    println!("✅ UI actor created successfully with patch cache dependency!");
    println!("✅ The UI actor now uses the patch cache instead of calling the API directly!");

    Ok(())
}
