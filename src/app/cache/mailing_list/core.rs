use super::message::Message;
use crate::ArcPath;
use crate::api::lore::{LoreApi, LoreMailingList};
use crate::{
    app::config::{Config, PathOpt},
    fs::Fs,
    log::Log,
};
use anyhow::Context;
use serde::{Deserialize, Serialize};

const SCOPE: &str = "app.cache.mailing_list";

#[derive(Debug, Default, Serialize, Deserialize)]
/// Structure for persisting the mailing list cache to disk.
pub struct CacheData {
    /// Cached mailing lists
    pub mailing_lists: Vec<LoreMailingList>,
}

/// Core implementation for the MailingListCache actor.
pub struct Core {
    /// Lore API actor for fetching mailing lists
    pub lore: LoreApi,
    /// Filesystem actor for persistence
    pub fs: Fs,
    /// Config actor for config path
    pub config: Config,
    /// Log actor for logging operations
    pub log: Log,
    /// Cached mailing lists
    pub mailing_lists: Vec<LoreMailingList>,
}

impl Core {
    /// Creates a new core instance for MailingListCache.
    pub fn new(lore: LoreApi, fs: Fs, config: Config, log: Log) -> Self {
        Self {
            lore,
            fs,
            config,
            log,
            mailing_lists: Vec::new(),
        }
    }

    /// Spawns the actor and returns the interface and task handle.
    pub fn spawn(mut self) -> (super::MailingListCache, tokio::task::JoinHandle<()>) {
        let (tx, mut rx) = tokio::sync::mpsc::channel(32);
        let handle = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                match msg {
                    Message::Get { index, tx } => {
                        let res = self.get(index).await;
                        let _ = tx.send(res);
                    }
                    Message::GetSlice { range, tx } => {
                        let res = self.get_slice(range).await;
                        let _ = tx.send(res);
                    }
                    Message::InvalidateCache => {
                        self.mailing_lists.clear();
                    }
                    Message::PersistCache { tx } => {
                        let res = self.persist_cache().await;
                        let _ = tx.send(res);
                    }
                    Message::LoadCache { tx } => {
                        let res = self.load_cache().await;
                        let _ = tx.send(res);
                    }
                    Message::Len { tx } => {
                        let _ = tx.send(self.len());
                    }
                    Message::IsCacheValid { tx } => {
                        let res = self.is_cache_valid().await;
                        let _ = tx.send(res);
                    }
                    Message::ContainsRange { range, tx } => {
                        let contains = self.contains_range(&range);
                        let _ = tx.send(contains);
                    }
                }
            }
        });
        (super::MailingListCache::Actual(tx), handle)
    }

    /// Returns the number of cached mailing lists.
    fn len(&self) -> usize {
        self.mailing_lists.len()
    }

    /// Checks if the cache contains the given range without fetching new data.
    fn contains_range(&self, range: &std::ops::Range<usize>) -> bool {
        self.mailing_lists.len() >= range.end
    }

    /// Checks if the cache is still valid by comparing the last_update field of the 0th item.
    async fn is_cache_valid(&self) -> anyhow::Result<bool> {
        if let Some(first) = self.mailing_lists.first() {
            let remote_page = self.lore.get_available_lists_page(0).await?;
            if let Some(page) = remote_page {
                if let Some(remote_first) = page.items.first() {
                    return Ok(remote_first.last_update == first.last_update);
                }
            }
        }
        Ok(false)
    }

    /// Fetches a single mailing list by index (demand-driven).
    async fn get(&mut self, index: usize) -> anyhow::Result<Option<LoreMailingList>> {
        let initial_count = self.mailing_lists.len();

        while self.mailing_lists.len() <= index {
            let min_index = self.mailing_lists.len();

            self.log.info(
                SCOPE,
                &format!("Cache miss: fetching page starting at index {}", min_index),
            );

            let page = self.lore.get_available_lists_page(min_index).await?;
            if let Some(page) = page {
                if page.items.is_empty() {
                    self.log
                        .info(SCOPE, "No more mailing lists available from API");
                    break;
                }
                let new_items = page.items.len();
                self.mailing_lists.extend(page.items);

                self.log.info(
                    SCOPE,
                    &format!(
                        "Fetched {} new mailing lists from API (total: {})",
                        new_items,
                        self.mailing_lists.len()
                    ),
                );
            } else {
                self.log.info(SCOPE, "API returned no page data");
                break;
            }
        }

        if self.mailing_lists.len() > initial_count {
            let fetched_count = self.mailing_lists.len() - initial_count;
            self.log.info(
                SCOPE,
                &format!("Cache expanded by {} items to serve request", fetched_count),
            );
        }

        Ok(self.mailing_lists.get(index).cloned())
    }

    /// Fetches a slice of mailing lists by range (demand-driven).
    async fn get_slice(
        &mut self,
        range: std::ops::Range<usize>,
    ) -> anyhow::Result<Vec<LoreMailingList>> {
        let end = range.end;
        while self.mailing_lists.len() < end {
            let min_index = self.mailing_lists.len();
            let page = self.lore.get_available_lists_page(min_index).await?;
            if let Some(page) = page {
                if page.items.is_empty() {
                    break;
                }
                self.mailing_lists.extend(page.items);
            } else {
                break;
            }
        }
        Ok(self
            .mailing_lists
            .get(range)
            .map(|s| s.to_vec())
            .unwrap_or_default())
    }

    /// Persists the cache to the filesystem as TOML.
    async fn persist_cache(&self) -> anyhow::Result<()> {
        let cache_count = self.mailing_lists.len();
        self.log.info(
            SCOPE,
            &format!("Persisting mailing list cache with {} items", cache_count),
        );

        let cache = CacheData {
            mailing_lists: self.mailing_lists.clone(),
        };
        let toml = toml::to_string(&cache)?;
        let cache_path = self.cache_path().await;

        // Create parent directory if it doesn't exist
        if let Some(parent) = cache_path.parent() {
            self.fs
                .mkdir(ArcPath::from(parent))
                .await
                .context("Creating cache directory")?;
        }

        let mut file = self
            .fs
            .write_file(cache_path.clone())
            .await
            .context("Opening cache file for writing")?;
        use tokio::io::AsyncWriteExt;
        file.write_all(toml.as_bytes())
            .await
            .context("Writing cache file")?;

        self.log.info(
            SCOPE,
            &format!(
                "Successfully persisted mailing list cache to {}",
                cache_path.display()
            ),
        );
        Ok(())
    }

    /// Loads the cache from the filesystem (TOML).
    async fn load_cache(&mut self) -> anyhow::Result<()> {
        // Ensure the cache file exists before opening it
        let cache_path = self.cache_path().await;
        if !cache_path.exists() {
            self.log.info(
                SCOPE,
                &format!("No existing cache file found at {}", cache_path.display()),
            );
            return Ok(());
        }

        self.log.info(
            SCOPE,
            &format!("Loading mailing list cache from {}", cache_path.display()),
        );

        let mut file = self
            .fs
            .read_file(cache_path)
            .await
            .context("Opening cache file for reading")?;
        let mut contents = String::new();
        use tokio::io::AsyncReadExt;
        file.read_to_string(&mut contents)
            .await
            .context("Reading cache file")?;
        let cache: CacheData = toml::from_str(&contents).context("Deserializing cache file")?;
        let loaded_count = cache.mailing_lists.len();
        self.mailing_lists = cache.mailing_lists;

        self.log.info(
            SCOPE,
            &format!(
                "Successfully loaded {} mailing lists from cache",
                loaded_count
            ),
        );
        Ok(())
    }

    async fn cache_path(&self) -> ArcPath {
        self.config
            .path(PathOpt::CachePath)
            .await
            .join("mailing_lists.toml")
            .as_path()
            .into()
    }
}
