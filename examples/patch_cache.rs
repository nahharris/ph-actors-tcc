use ph::{
    api::LoreApi,
    app::{
        cache::{mailing_list::MailingListCache, patch_meta::PatchMetaCache},
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
    let ml = MailingListCache::spawn(lore.clone(), fs.clone(), config.clone());
    let pm = PatchMetaCache::spawn(lore, fs.clone(), config.clone());

    let cache_path = config.path(PathOpt::CachePath).await;
    if !cache_path.exists() {
        fs.mkdir(cache_path.clone()).await?;
    }

    println!("Cache path: {}", cache_path.to_str().unwrap());

    let load_ml_cache = ml.load_cache().await.context("Loading mailing list cache");
    let load_pm_cache = pm.load_cache().await.context("Loading patch meta cache");

    let valid = load_ml_cache.is_ok() && ml.is_cache_valid().await.is_ok();
    println!("Mailing lists cache valid: {}", valid);
    if !valid {
        if let Err(e) = load_ml_cache {
            println!("Error loading mailing list cache: {e}");
        }
        ml.invalidate_cache().await;
    }

    let list = ml.get(0).await?.expect("Mailing list 0 not found");
    println!("Mailing list 0: {list:#?}");
    ml.persist_cache().await?;

    let valid = load_pm_cache.is_ok() && pm.is_cache_valid(list.name.clone()).await.is_ok();
    println!("Patch meta cache valid: {}", valid);
    if !valid {
        if let Err(e) = load_pm_cache {
            println!("Error loading patch meta cache: {e}");
        }
        pm.invalidate_cache(list.name.clone()).await;
    }

    let patch_meta = pm
        .get(list.name.clone(), 0)
        .await?
        .expect("Patch meta 0, 0 not found");
    println!("Patch meta {}, 0: {patch_meta:#?}", list.name);
    pm.persist_cache().await?;

    Ok(())
}
