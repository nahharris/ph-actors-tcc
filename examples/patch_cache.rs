use ph::{
    api::LoreApi,
    app::{
        cache::{feed::FeedCache, mailing_list::MailingListCache},
        config::{Config, PathOpt},
    },
    fs::Fs,
    log::Log,
    net::Net,
};

use anyhow::Context;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::mock(Default::default());
    let log = Log::mock();
    let fs = Fs::spawn();
    let net = Net::spawn(config.clone(), log.clone()).await;
    let lore = LoreApi::spawn(net);
    let ml = MailingListCache::spawn(lore.clone(), fs.clone(), config.clone(), log.clone()).await?;
    let pm = FeedCache::spawn(lore, fs.clone(), config.clone(), log.clone()).await?;

    let cache_path = config.path(PathOpt::CachePath).await;
    if !cache_path.exists() {
        fs.mkdir(cache_path.clone()).await?;
    }

    println!("Cache path: {}", cache_path.to_str().unwrap());

    // Load mailing list cache
    let load_ml_result = ml.load().await.context("Loading mailing list cache");
    let ml_valid = load_ml_result.is_ok() && !ml.is_empty().await;
    println!("Mailing lists cache valid: {}", ml_valid);
    if !ml_valid {
        if let Err(e) = load_ml_result {
            println!("Error loading mailing list cache: {e}");
        }
        ml.invalidate().await?;
        // Refresh the cache since it's empty
        ml.refresh().await?;
    }

    let list = ml.get(0).await?.expect("Mailing list 0 not found");
    println!("Mailing list 0: {list:#?}");
    ml.persist().await?;

    // Load feed cache for the mailing list
    let load_feed_result = pm
        .load(list.name.clone())
        .await
        .context("Loading feed cache");
    let feed_valid = load_feed_result.is_ok() && !pm.is_empty(list.name.clone()).await;
    println!("Feed cache valid: {}", feed_valid);
    if !feed_valid {
        if let Err(e) = load_feed_result {
            println!("Error loading feed cache: {e}");
        }
        pm.invalidate(list.name.clone()).await?;
        // Refresh the cache since it's empty
        pm.refresh(list.name.clone()).await?;
    }

    let patch_meta = pm
        .get(list.name.clone(), 0)
        .await?
        .expect("Patch meta 0, 0 not found");
    println!("Patch meta {}, 0: {patch_meta:#?}", list.name);
    pm.persist(list.name.clone()).await?;

    Ok(())
}
