use super::data::MailingListData;
use super::message::Message;
use crate::ArcPath;
use crate::api::lore::{LoreApi, LoreMailingList};
use crate::app::config::Config;
use crate::fs::Fs;
use crate::log::Log;
use anyhow::Context;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

const BUFFER_SIZE: usize = 100;
const SCOPE: &str = "app.cache.mailing_list";

/// Core implementation for the Mailing List Actor.
pub struct Core {
    /// Lore API actor for fetching mailing lists
    lore: LoreApi,
    /// Filesystem actor for persistence
    fs: Fs,
    /// Config actor for configuration
    config: Config,
    /// Log actor for logging
    log: Log,
    /// Internal state
    data: MailingListData,
}

impl Core {
    /// Creates a new Core instance.
    pub async fn new(lore: LoreApi, fs: Fs, config: Config, log: Log) -> anyhow::Result<Self> {
        let cache_dir = config.path(crate::app::config::PathOpt::CachePath).await;
        let cache_path = ArcPath::from(&cache_dir.join("mailing_lists.toml"));
        let data = MailingListData::new(cache_path);

        Ok(Self {
            lore,
            fs,
            config,
            log,
            data,
        })
    }

    /// Spawns the actor and returns the public interface and join handle.
    pub fn spawn(self) -> (super::MailingListCache, JoinHandle<()>) {
        let (tx, mut rx) = mpsc::channel(BUFFER_SIZE);
        let handle = tokio::spawn(async move {
            let mut core = self;

            // Load cache on startup
            if let Err(e) = core.load_cache().await {
                core.log
                    .error(SCOPE, &format!("Failed to load cache: {}", e));
            }

            while let Some(message) = rx.recv().await {
                match message {
                    Message::Get { index, tx } => {
                        let result = core.handle_get(index).await;
                        let _ = tx.send(result);
                    }
                    Message::GetSlice { range, tx } => {
                        let result = core.handle_get_slice(range).await;
                        let _ = tx.send(result);
                    }
                    Message::Refresh { tx } => {
                        let result = core.handle_refresh().await;
                        let _ = tx.send(result);
                    }
                    Message::Invalidate { tx } => {
                        let result = core.handle_invalidate().await;
                        let _ = tx.send(result);
                    }
                    Message::IsAvailable { range, tx } => {
                        let result = core.handle_is_available(range);
                        let _ = tx.send(result);
                    }
                    Message::Len { tx } => {
                        let result = core.data.lists.len();
                        let _ = tx.send(result);
                    }
                    Message::Persist { tx } => {
                        let result = core.persist_cache().await;
                        let _ = tx.send(result);
                    }
                    Message::Load { tx } => {
                        let result = core.load_cache().await;
                        let _ = tx.send(result);
                    }
                }
            }
        });

        (super::MailingListCache::Actual(tx), handle)
    }

    /// Handles getting a single mailing list by index.
    async fn handle_get(&mut self, index: usize) -> anyhow::Result<Option<LoreMailingList>> {
        Ok(self.data.lists.get(index).cloned())
    }

    /// Handles getting a slice of mailing lists by range.
    async fn handle_get_slice(
        &mut self,
        range: std::ops::Range<usize>,
    ) -> anyhow::Result<Vec<LoreMailingList>> {
        if range.start >= self.data.lists.len() {
            return Ok(Vec::new());
        }

        let end = range.end.min(self.data.lists.len());
        Ok(self.data.lists[range.start..end].to_vec())
    }

    /// Handles refreshing the cache.
    async fn handle_refresh(&mut self) -> anyhow::Result<()> {
        self.refresh_cache().await
    }

    /// Handles invalidating the cache.
    async fn handle_invalidate(&mut self) -> anyhow::Result<()> {
        self.data.lists.clear();
        self.data.last_updated = None;
        self.persist_cache().await
    }

    /// Handles checking if a range is available.
    fn handle_is_available(&self, range: std::ops::Range<usize>) -> bool {
        range.end <= self.data.lists.len()
    }

    /// Checks if the cache is still valid.
    async fn is_cache_valid(&self) -> anyhow::Result<bool> {
        if self.data.lists.is_empty() {
            return Ok(false);
        }

        // Get the first page to check the 0-th item's updated time
        let first_page = self.lore.get_available_lists_page(0).await?;
        let api_last_updated =
            first_page.and_then(|page| page.items.first().map(|item| item.last_update));

        Ok(self.data.is_cache_valid(api_last_updated))
    }

    /// Refreshes the cache by fetching all mailing lists and sorting them.
    async fn refresh_cache(&mut self) -> anyhow::Result<()> {
        self.log.info(SCOPE, "Refreshing mailing list cache");

        let mut all_lists = Vec::new();
        let mut min_index = 0;

        loop {
            let page = self.lore.get_available_lists_page(min_index).await?;
            match page {
                Some(page) => {
                    let items_len = page.items.len();
                    all_lists.extend(page.items);
                    min_index = page.next_page_index.unwrap_or(min_index + items_len);

                    if page.next_page_index.is_none() {
                        break;
                    }
                }
                None => break,
            }
        }

        // Sort alphabetically
        all_lists.sort_by(|a, b| a.name.cmp(&b.name));

        // Update internal state
        self.data.lists = all_lists;
        self.data.update_last_updated();

        // Persist to disk
        self.persist_cache().await?;

        self.log.info(
            SCOPE,
            &format!("Cached {} mailing lists", self.data.lists.len()),
        );
        Ok(())
    }

    /// Persists the cache to the filesystem.
    async fn persist_cache(&self) -> anyhow::Result<()> {
        let cache_data = self.data.to_cache_data();
        let content =
            toml::to_string_pretty(&cache_data).context("Failed to serialize cache data")?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = self.data.cache_path.parent() {
            self.fs
                .mkdir(ArcPath::from(parent))
                .await
                .context("Failed to create cache directory")?;
        }

        // Write the file
        let mut file = self
            .fs
            .write_file(self.data.cache_path.clone())
            .await
            .context("Failed to open cache file for writing")?;

        use tokio::io::AsyncWriteExt;
        file.write_all(content.as_bytes())
            .await
            .context("Failed to write cache file")?;

        Ok(())
    }

    /// Loads the cache from the filesystem.
    async fn load_cache(&mut self) -> anyhow::Result<()> {
        // Check if file exists by trying to read it
        let file = match self.fs.read_file(self.data.cache_path.clone()).await {
            Ok(file) => file,
            Err(_) => return Ok(()), // File doesn't exist, that's ok
        };

        // Read the content
        use tokio::io::AsyncReadExt;
        let mut content = String::new();
        let mut file = file;
        file.read_to_string(&mut content)
            .await
            .context("Failed to read cache file content")?;

        let cache_data: super::data::CacheData =
            toml::from_str(&content).context("Failed to deserialize cache data")?;

        self.data.from_cache_data(cache_data);

        self.log.info(
            SCOPE,
            &format!("Loaded {} mailing lists from cache", self.data.lists.len()),
        );
        Ok(())
    }
}
