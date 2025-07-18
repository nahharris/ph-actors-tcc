use ph::{
    api::LoreApi,
    app::{
        config::{Config, PathOpt},
        state::mailing_list::MailingListState,
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
    let ml = MailingListState::spawn(lore, fs, config.clone());

    println!(
        "Mailing lists cache at: {}",
        config.path(PathOpt::CachePath).await.to_str().unwrap()
    );
    ml.load_cache().await?;
    println!("Mailing lists cache valid: {}", ml.is_cache_valid().await?);
    let lists = ml.get_slice(0..3).await?;
    println!("Mailing lists length: {}", ml.len().await);
    let list_203 = ml.get(203).await?.unwrap();
    println!("Mailing list 203: {:#?}", list_203);
    println!("Mailing lists length: {}", ml.len().await);
    println!("Mailing lists: {:#?}", lists);
    ml.persist_cache().await?;
    Ok(())
}
