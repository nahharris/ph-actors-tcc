use ph::{
    api::LoreApi,
    app::{
        config::{Config, PathOpt},
        state::{mailing_list::MailingListState, patch_meta::PatchMetaState},
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
    let ml = MailingListState::spawn(lore.clone(), fs.clone(), config.clone());
    let pm = PatchMetaState::spawn(lore, fs.clone(), config.clone());

    let cache_path = config.path(PathOpt::CachePath).await;
    if !cache_path.exists() {
        fs.mkdir(cache_path.clone()).await?;
    }

    println!("Cache path: {}", cache_path.to_str().unwrap());

    ml.load_cache()
        .await
        .context("Loading mailing list cache")?;
    pm.load_cache().await.context("Loading patch meta cache")?;

    let valid = ml.is_cache_valid().await?;
    println!("Mailing lists cache valid: {}", valid);
    if !valid {
        ml.invalidate_cache().await;
    }

    let list = ml.get(0).await?.expect("Mailing list 0 not found");
    println!("Mailing list 0: {list:#?}");
    ml.persist_cache().await?;

    let valid = pm.is_cache_valid(list.name.clone()).await?;
    println!("Patch meta cache valid: {}", valid);
    if !valid {
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
