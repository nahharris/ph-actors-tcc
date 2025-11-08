use ph::{
    ArcPath,
    api::lore::LoreApi,
    app::{
        App,
        cache::{feed::FeedCache, mailing_list::MailingListCache, patch::PatchCache},
        config::Config,
        ui::Ui,
    },
    env::Env,
    fs::Fs,
    log::Log,
    net::Net,
    render::Render,
    shell::Shell,
    terminal::Terminal,
    utils::install_panic_hook,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    install_panic_hook()?;

    // Build actors in dependency order
    let env = Env::spawn();
    let fs = Fs::spawn();

    // Get config directory from environment or use default
    let home = std::env::var("HOME")
        .ok()
        .or_else(|| std::env::var("USERPROFILE").ok()) // Windows
        .unwrap_or_else(|| ".".to_string());
    let config_dir = std::path::Path::new(&home)
        .join(".config")
        .join("patch-hub");
    let config_path = ArcPath::from(&config_dir.join("config.toml"));
    let config = Config::spawn(env.clone(), fs.clone(), config_path);
    config.load().await.ok(); // Try to load, ignore errors if file doesn't exist

    let log = Log::spawn(fs.clone(), config.clone())
        .await
        .map_err(Box::new)?;
    let net = Net::spawn(config.clone(), log.clone());
    let shell = Shell::spawn(log.clone()).await.map_err(Box::new)?;
    let render = Render::spawn(shell.clone(), config.clone());
    let lore = LoreApi::spawn(net);

    let mailing_list_cache =
        MailingListCache::spawn(lore.clone(), fs.clone(), config.clone(), log.clone())
            .await
            .map_err(Box::new)?;
    let feed_cache = FeedCache::spawn(lore.clone(), fs.clone(), config.clone(), log.clone())
        .await
        .map_err(Box::new)?;
    let patch_cache = PatchCache::spawn(lore.clone(), fs.clone(), config.clone(), log.clone())
        .await
        .map_err(Box::new)?;

    let (terminal, terminal_handle) = Terminal::spawn(log.clone());
    let ui = Ui::spawn(
        log.clone(),
        terminal.clone(),
        mailing_list_cache.clone(),
        feed_cache.clone(),
        patch_cache.clone(),
        render.clone(),
    );

    // Spawn the App actor
    let (_app, app_handle) = App::spawn(
        log,
        mailing_list_cache,
        feed_cache,
        terminal,
        terminal_handle,
        ui,
    );

    // Wait for the application to finish (it will exit when the terminal exits)
    // The app will handle its own shutdown when the terminal closes
    let _ = app_handle.await;

    Ok(())
}
