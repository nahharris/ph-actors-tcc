use crate::api::lore::{LoreApi, LorePatchMetadata};
use crate::{ArcPath, ArcStr, fs::Fs, app::config::Config};
use super::message::Message;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Default, Serialize, Deserialize)]
/// Structure for persisting the patch metadata cache to disk.
pub struct CacheData {
    /// Cached patch metadata per mailing list
    pub patch_cache: HashMap<ArcStr, Vec<LorePatchMetadata>>,
}

/// Core implementation for the PatchMetaState actor.
pub struct Core {
    /// Lore API actor for fetching patch metadata
    pub lore: LoreApi,
    /// Filesystem actor for persistence
    pub fs: Fs,
    /// Config actor for config path
    pub config: Config,
    /// Path to the cache file (from config)
    pub cache_path: ArcPath,
    /// Cached patch metadata per mailing list
    pub patch_cache: HashMap<ArcStr, Vec<LorePatchMetadata>>,
}

impl Core {
    /// Creates a new core instance for PatchMetaState.
    pub fn new(lore: LoreApi, fs: Fs, config: Config, cache_path: ArcPath) -> Self {
        Self {
            lore,
            fs,
            config,
            cache_path,
            patch_cache: HashMap::new(),
        }
    }

    /// Spawns the actor and returns the interface and task handle.
    pub fn spawn(mut self) -> (super::PatchMetaState, tokio::task::JoinHandle<()>) {
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
                    Message::InvalidateCache => {
                        self.patch_cache.clear();
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
                }
            }
        });
        (super::PatchMetaState::Actual(tx), handle)
    }

    /// Returns the number of cached patch metadata items for a given mailing list.
    pub fn len(&self, list: ArcStr) -> usize {
        self.patch_cache.get(&list).map(|v| v.len()).unwrap_or(0)
    }

    /// Checks if the cache is still valid for a given mailing list by comparing the last_update field of the 0th item.
    pub async fn is_cache_valid(&self, list: ArcStr) -> anyhow::Result<bool> {
        if let Some(cached) = self.patch_cache.get(&list) {
            if let Some(first) = cached.get(0) {
                let remote_page = self.lore.get_patch_feed_page(list.clone(), 0).await?;
                if let Some(page) = remote_page {
                    if let Some(remote_first) = page.items.get(0) {
                        return Ok(remote_first.last_update == first.last_update);
                    }
                }
            }
        }
        Ok(false)
    }

    /// Fetches a single patch metadata item by index for a given mailing list (demand-driven).
    pub async fn get(&mut self, list: ArcStr, index: usize) -> anyhow::Result<Option<LorePatchMetadata>> {
        let cache = self.patch_cache.entry(list.clone()).or_insert_with(Vec::new);
        while cache.len() <= index {
            let min_index = cache.len();
            let page = self.lore.get_patch_feed_page(list.clone(), min_index).await?;
            if let Some(page) = page {
                if page.items.is_empty() {
                    break;
                }
                cache.extend(page.items);
            } else {
                break;
            }
        }
        Ok(cache.get(index).cloned())
    }

    /// Fetches a slice of patch metadata items by range for a given mailing list (demand-driven).
    pub async fn get_slice(&mut self, list: ArcStr, range: std::ops::Range<usize>) -> anyhow::Result<Vec<LorePatchMetadata>> {
        let cache = self.patch_cache.entry(list.clone()).or_insert_with(Vec::new);
        let end = range.end;
        while cache.len() < end {
            let min_index = cache.len();
            let page = self.lore.get_patch_feed_page(list.clone(), min_index).await?;
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
    pub async fn persist_cache(&self) -> anyhow::Result<()> {
        let cache = CacheData {
            patch_cache: self.patch_cache.clone(),
        };
        let toml = toml::to_string(&cache)?;
        let file = self.fs.open_file(self.cache_path.clone()).await?;
        file.write().await.write_all(toml.as_bytes()).await?;
        Ok(())
    }

    /// Loads the cache from the filesystem (TOML).
    pub async fn load_cache(&mut self) -> anyhow::Result<()> {
        let file = self.fs.open_file(self.cache_path.clone()).await?;
        let mut contents = String::new();
        file.write().await.read_to_string(&mut contents).await?;
        let cache: CacheData = toml::from_str(&contents)?;
        self.patch_cache = cache.patch_cache;
        Ok(())
    }
} 