use ph::{
    api::lore::LoreApi,
    app::{
        cache::feed::FeedCache,
        config::{Config, PathOpt},
    },
    fs::Fs,
    log::Log,
    net::Net,
};


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::mock(Default::default());
    let log = Log::mock();
    let fs = Fs::spawn();
    let net = Net::spawn(config.clone(), log.clone()).await;
    let lore = LoreApi::spawn(net);
    let feed_cache = FeedCache::spawn(lore, fs.clone(), config.clone(), log.clone()).await?;

    let cache_path = config.path(PathOpt::CachePath).await;
    if !cache_path.exists() {
        fs.mkdir(cache_path.clone()).await?;
    }

    println!("Cache path: {}", cache_path.to_str().unwrap());

    // Test with a test mailing list
    let test_list = "amd-gfx";
    println!("Testing feed cache for list: {}", test_list);

    // Check initial state
    let initial_len = feed_cache.len(test_list.into()).await;
    println!("Initial cache length: {}", initial_len);

    // Try to load cache
    let load_result = feed_cache.load(test_list.into()).await;
    println!("Load result: {:?}", load_result);

    // Check length after load
    let after_load_len = feed_cache.len(test_list.into()).await;
    println!("After load cache length: {}", after_load_len);

    // Try to refresh the cache
    println!("Refreshing cache...");
    let refresh_result = feed_cache.refresh(test_list.into()).await;
    println!("Refresh result: {:?}", refresh_result);

    // Check length after refresh
    let after_refresh_len = feed_cache.len(test_list.into()).await;
    println!("After refresh cache length: {}", after_refresh_len);

    // Try to get some items
    if after_refresh_len > 0 {
        println!("Getting first item...");
        let first_item = feed_cache.get(test_list.into(), 0).await?;
        println!("First item: {:?}", first_item);

        println!("Getting slice 0..5...");
        let slice = feed_cache.get_slice(test_list.into(), 0..5).await?;
        println!("Slice length: {}", slice.len());
        for (i, item) in slice.iter().enumerate() {
            println!("  {}: {}", i, item.title);
        }
    } else {
        println!("Cache is empty after refresh!");
    }

    // Test persistence
    println!("Persisting cache...");
    let persist_result = feed_cache.persist(test_list.into()).await;
    println!("Persist result: {:?}", persist_result);

    Ok(())
}
