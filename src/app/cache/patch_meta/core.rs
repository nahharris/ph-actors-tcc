use super::message::Message;
use crate::api::lore::{LoreApi, LorePatchMetadata};
use crate::{
    ArcPath, ArcStr,
    app::config::{Config, PathOpt},
    fs::Fs,
    log::Log,
};
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const SCOPE: &str = "app.cache.patch_meta";

#[derive(Debug, Default, Serialize, Deserialize)]
/// Structure for persisting the patch metadata cache to disk.
pub struct CacheData {
    /// Cached patch metadata per mailing list
    pub patch_cache: HashMap<ArcStr, Vec<LorePatchMetadata>>,
}

/// Core implementation for the PatchMetaCache actor.
pub struct Core {
    /// Lore API actor for fetching patch metadata
    pub lore: LoreApi,
    /// Filesystem actor for persistence
    pub fs: Fs,
    /// Config actor for config path
    pub config: Config,
    /// Log actor for logging operations
    pub log: Log,
    /// Cached patch metadata per mailing list
    pub patch_cache: HashMap<ArcStr, Vec<LorePatchMetadata>>,
}

impl Core {
    /// Creates a new core instance for PatchMetaCache.
    pub fn new(lore: LoreApi, fs: Fs, config: Config, log: Log) -> Self {
        Self {
            lore,
            fs,
            config,
            log,
            patch_cache: Default::default(),
        }
    }

    /// Spawns the actor and returns the interface and task handle.
    pub fn spawn(mut self) -> (super::PatchMetaCache, tokio::task::JoinHandle<()>) {
        let (tx, mut rx) = tokio::sync::mpsc::channel(32);
        let handle = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                match msg {
                    Message::Get { list, index, tx } => {
                        let res = self.get(list, index).await;
                        let _ = tx.send(res);
                    }
                    Message::GetSlice { list, range, tx } => {
                        let res = self.get_slice(list, range).await;
                        let _ = tx.send(res);
                    }
                    Message::InvalidateCache { list } => {
                        self.patch_cache.remove(&list);
                    }
                    Message::PersistCache { tx } => {
                        let res = self.persist_cache().await;
                        let _ = tx.send(res);
                    }
                    Message::LoadCache { tx } => {
                        let res = self.load_cache().await;
                        let _ = tx.send(res);
                    }
                    Message::Len { list, tx } => {
                        let _ = tx.send(self.len(list));
                    }
                    Message::IsCacheValid { list, tx } => {
                        let res = self.is_cache_valid(list).await;
                        let _ = tx.send(res);
                    }
                    Message::ContainsRange { list, range, tx } => {
                        let contains = self.contains_range(&list, &range);
                        let _ = tx.send(contains);
                    }
                }
            }
        });
        (super::PatchMetaCache::Actual(tx), handle)
    }

    /// Returns the number of cached patch metadata items for a given mailing list.
    fn len(&self, list: ArcStr) -> usize {
        self.patch_cache.get(&list).map(|v| v.len()).unwrap_or(0)
    }

    /// Checks if the cache contains the given range for a mailing list without fetching new data.
    fn contains_range(&self, list: &ArcStr, range: &std::ops::Range<usize>) -> bool {
        self.patch_cache
            .get(list)
            .map(|v| v.len() >= range.end)
            .unwrap_or(false)
    }

    /// Checks if the cache is still valid for a given mailing list by comparing the last_update field of the 0th item.
    async fn is_cache_valid(&self, list: ArcStr) -> anyhow::Result<bool> {
        if let Some(cached) = self.patch_cache.get(&list) {
            if let Some(first) = cached.first() {
                let remote_page = self.lore.get_patch_feed_page(list.clone(), 0).await?;
                if let Some(page) = remote_page {
                    if let Some(remote_first) = page.items.first() {
                        return Ok(remote_first.last_update == first.last_update);
                    }
                }
            }
        }
        Ok(false)
    }

    /// Fetches a single patch metadata item by index for a given mailing list (demand-driven).
    async fn get(
        &mut self,
        list: ArcStr,
        index: usize,
    ) -> anyhow::Result<Option<LorePatchMetadata>> {
        let cache = self.patch_cache.entry(list.clone()).or_default();
        let initial_count = cache.len();

        while cache.len() <= index {
            let min_index = cache.len();

            self.log.info(
                SCOPE,
                &format!(
                    "Cache miss for '{}': fetching page starting at index {}",
                    list, min_index
                ),
            );

            let page = self
                .lore
                .get_patch_feed_page(list.clone(), min_index)
                .await?;
            if let Some(page) = page {
                if page.items.is_empty() {
                    self.log.info(
                        SCOPE,
                        &format!("No more patches available for '{}' from API", list),
                    );
                    break;
                }
                let new_items = page.items.len();
                cache.extend(page.items);

                self.log.info(
                    SCOPE,
                    &format!(
                        "Fetched {} new patches for '{}' from API (total: {})",
                        new_items,
                        list,
                        cache.len()
                    ),
                );
            } else {
                self.log
                    .info(SCOPE, &format!("API returned no page data for '{}'", list));
                break;
            }
        }

        if cache.len() > initial_count {
            let fetched_count = cache.len() - initial_count;
            self.log.info(
                SCOPE,
                &format!(
                    "Cache for '{}' expanded by {} items to serve request",
                    list, fetched_count
                ),
            );
        }

        Ok(cache.get(index).cloned())
    }

    /// Fetches a slice of patch metadata items by range for a given mailing list (demand-driven).
    async fn get_slice(
        &mut self,
        list: ArcStr,
        range: std::ops::Range<usize>,
    ) -> anyhow::Result<Vec<LorePatchMetadata>> {
        let cache = self.patch_cache.entry(list.clone()).or_default();
        let end = range.end;
        while cache.len() < end {
            let min_index = cache.len();
            let page = self
                .lore
                .get_patch_feed_page(list.clone(), min_index)
                .await?;
            if let Some(page) = page {
                if page.items.is_empty() {
                    break;
                }
                cache.extend(page.items);
            } else {
                break;
            }
        }
        Ok(cache.get(range).map(|s| s.to_vec()).unwrap_or_default())
    }

    /// Persists the cache to the filesystem as TOML.
    async fn persist_cache(&self) -> anyhow::Result<()> {
        let list_count = self.patch_cache.len();
        let total_patches: usize = self.patch_cache.values().map(|v| v.len()).sum();

        self.log.info(
            SCOPE,
            &format!(
                "Persisting patch metadata cache with {} lists and {} total patches",
                list_count, total_patches
            ),
        );

        let cache = CacheData {
            patch_cache: self.patch_cache.clone(),
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

        let mut file = self.fs.write_file(cache_path.clone()).await?;
        use tokio::io::AsyncWriteExt;
        file.write_all(toml.as_bytes()).await?;

        self.log.info(
            SCOPE,
            &format!(
                "Successfully persisted patch metadata cache to {}",
                cache_path.display()
            ),
        );
        Ok(())
    }

    /// Loads the cache from the filesystem (TOML).
    async fn load_cache(&mut self) -> anyhow::Result<()> {
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
            &format!("Loading patch metadata cache from {}", cache_path.display()),
        );

        let mut file = self.fs.read_file(cache_path).await?;
        let mut contents = String::new();
        use tokio::io::AsyncReadExt;
        file.read_to_string(&mut contents).await?;
        let cache: CacheData = toml::from_str(&contents)?;

        let list_count = cache.patch_cache.len();
        let total_patches: usize = cache.patch_cache.values().map(|v| v.len()).sum();
        self.patch_cache = cache.patch_cache;

        self.log.info(
            SCOPE,
            &format!(
                "Successfully loaded {} lists with {} total patches from cache",
                list_count, total_patches
            ),
        );
        Ok(())
    }

    async fn cache_path(&self) -> ArcPath {
        self.config
            .path(PathOpt::CachePath)
            .await
            .join("patch_meta.toml")
            .as_path()
            .into()
    }
}
