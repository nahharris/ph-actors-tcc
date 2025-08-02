use ph::api::lore::LoreApi;
use ph::app::config::Config;
use ph::log::Log;
use ph::net::Net;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::mock(Default::default());
    let log = Log::mock();
    let net = Net::spawn(config, log).await;

    let lore = LoreApi::spawn(net);

    let lists = match lore.get_available_lists().await {
        Ok(lists) => lists,
        Err(e) => {
            println!("Error getting mailing lists: {e}");
            return Err(e);
        }
    };
    println!("Total mailing lists: {}", lists.len());
    let item = lists.get(1).unwrap();
    let name = item.name.clone();
    println!("First mailing list: {name}");

    let patch_feed = match lore.get_patch_feed_page(name.clone(), 0).await {
        Ok(Some(patch_feed)) => patch_feed,
        Ok(None) => {
            println!("No patch feed found");
            return Err(anyhow::anyhow!("No patch feed found for {name}"));
        }
        Err(e) => {
            println!("Error getting patch feed: {e}");
            return Err(e);
        }
    };

    let meta = &patch_feed.items[0];
    println!("First patch from feed {}: {:#?}", name, meta);

    let raw_patch = match lore
        .get_patch_metadata(meta.list.clone(), meta.message_id.clone())
        .await
    {
        Ok(raw_patch) => raw_patch,
        Err(e) => {
            println!("Error getting raw patch: {e}");
            return Err(e);
        }
    };
    println!("Raw patch: {raw_patch}");

    Ok(())
}
