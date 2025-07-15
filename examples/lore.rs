use ph::api::lore::LoreApi;
use ph::app::config::Config;
use ph::log::Log;
use ph::net::Net;

#[tokio::main]
async fn main() {
    let config = Config::mock(Default::default());
    let log = Log::mock();
    let net = Net::spawn(config, log).await;

    let lore = LoreApi::spawn(net);

    let lists = lore.get_available_lists().await.unwrap();
    println!("Total mailing lists: {}", lists.len());
    let item = lists.get(0).unwrap();
    let name = item.name.clone();
    println!("First mailing list: {}", name);
    let patch_feed = lore.get_patch_feed_page(name, 0).await.unwrap();
    for item in patch_feed.unwrap().items {
        println!(
            "[{}] {}<{}> - {}",
            item.datetime, item.author, item.email, item.title
        );
    }
}
