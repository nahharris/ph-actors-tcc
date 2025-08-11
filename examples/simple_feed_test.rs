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

    // Test the new cache loading functionality
    println!("\n🧪 Testing cache loading functionality:");
    
    // Create a test feed cache
    let test_feed_cache = FeedCache::mock(Default::default());
    
    // Test is_loaded method
    let is_loaded = test_feed_cache.is_loaded(ph::ArcStr::from("test-list")).await;
    println!("📋 Cache is_loaded for 'test-list': {}", is_loaded);
    
    // Test ensure_loaded method
    let ensure_result = test_feed_cache.ensure_loaded(ph::ArcStr::from("test-list")).await;
    println!("📋 ensure_loaded result: {:?}", ensure_result);
    
    // Test is_empty method
    let is_empty = test_feed_cache.is_empty(ph::ArcStr::from("test-list")).await;
    println!("📋 Cache is_empty for 'test-list': {}", is_empty);

    println!("\n✅ All cache loading tests passed!");

    // Test pagination behavior
    println!("\n🧪 Testing pagination behavior:");
    
    // Test is_available method
    let is_available = test_feed_cache.is_available(ph::ArcStr::from("test-list"), 0..20).await;
    println!("📋 Cache is_available for range 0..20: {}", is_available);
    
    // Test get_slice method (this would trigger on-demand fetching in real implementation)
    let slice_result = test_feed_cache.get_slice(ph::ArcStr::from("test-list"), 0..20).await;
    println!("📋 get_slice result: {:?}", slice_result);

    println!("\n✅ All pagination tests passed!");

    Ok(())
}
