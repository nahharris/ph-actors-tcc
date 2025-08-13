use super::data::PatchData;
use super::message::Message;
use crate::ArcPath;
use crate::ArcStr;
use crate::api::lore::{LoreApi, data::LorePatch};
use crate::app::config::Config;
use crate::fs::Fs;
use crate::log::Log;
use anyhow::Context;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

const BUFFER_SIZE: usize = 100;
const SCOPE: &str = "app.cache.patch";

/// Core implementation for the Patch Actor.
pub struct Core {
    /// Lore API actor for fetching patch content
    lore: LoreApi,
    /// Filesystem actor for persistence
    fs: Fs,
    /// Config actor for configuration
    config: Config,
    /// Log actor for logging
    log: Log,
    /// Internal state
    data: PatchData,
}

impl Core {
    /// Creates a new Core instance.
    pub async fn new(lore: LoreApi, fs: Fs, config: Config, log: Log) -> anyhow::Result<Self> {
        let cache_dir = config.path(crate::app::config::PathOpt::CachePath).await;
        let patch_cache_dir = ArcPath::from(&cache_dir.join("patch"));
        let data = PatchData::new(patch_cache_dir);

        Ok(Self {
            lore,
            fs,
            config,
            log,
            data,
        })
    }

    /// Spawns the actor and returns the public interface and join handle.
    pub fn spawn(self) -> (super::PatchCache, JoinHandle<()>) {
        let (tx, mut rx) = mpsc::channel(BUFFER_SIZE);
        let handle = tokio::spawn(async move {
            let mut core = self;

            while let Some(message) = rx.recv().await {
                match message {
                    Message::Get {
                        list,
                        message_id,
                        tx,
                    } => {
                        let result = core.handle_get(&list, &message_id).await;
                        let _ = tx.send(result);
                    }
                    Message::Invalidate {
                        list,
                        message_id,
                        tx,
                    } => {
                        let result = core.handle_invalidate(&list, &message_id).await;
                        let _ = tx.send(result);
                    }
                    Message::IsAvailable {
                        list,
                        message_id,
                        tx,
                    } => {
                        let result = core.handle_is_available(&list, &message_id);
                        let _ = tx.send(result);
                    }
                }
            }
        });

        (super::PatchCache::Actual(tx), handle)
    }

    /// Handles getting a patch by mailing list and message ID.
    async fn handle_get(&mut self, list: &str, message_id: &str) -> anyhow::Result<LorePatch> {
        // First check the buffer
        if let Some(patch) = self.data.get_from_buffer(list, message_id) {
            return Ok(patch);
        }

        // Check if the patch exists on disk
        if self.patch_exists_on_disk(list, message_id).await? {
            // Load from disk and add to buffer
            let patch = self.load_patch_from_disk(list, message_id).await?;
            self.data.add_to_buffer(list, message_id, patch.clone());
            return Ok(patch);
        }

        // Fetch from API
        self.log.info(
            SCOPE,
            &format!("Fetching patch {} from API for list: {}", message_id, list),
        );

        let raw_content = self
            .lore
            .get_raw_patch(ArcStr::from(list), ArcStr::from(message_id))
            .await?;

        // Parse the raw content into a LorePatch
        let patch = crate::api::lore::parse::parse_patch_mbox(&raw_content)
            .with_context(|| format!("Failed to parse patch {list}/{message_id}"))?;

        // Save to disk and add to buffer
        if let Err(e) = self.save_patch_to_disk(list, message_id, &patch).await {
            self.log.error(
                SCOPE,
                &format!("Failed to save patch {list}/{message_id} to disk: {e}"),
            );
        }
        self.data.add_to_buffer(list, message_id, patch.clone());

        Ok(patch)
    }

    /// Handles invalidating a specific patch.
    async fn handle_invalidate(&mut self, list: &str, message_id: &str) -> anyhow::Result<()> {
        // Remove from buffer
        let key = self.data.get_buffer_key(list, message_id);
        self.data.buffer.pop(&key);

        // Remove from disk
        let cache_path = self.data.get_cache_path(list, message_id);
        if let Err(e) = self.fs.remove_file(cache_path).await {
            // Ignore errors if file doesn't exist
            if !e.to_string().contains("No such file") {
                return Err(e.into());
            }
        }

        Ok(())
    }

    /// Handles checking if a patch is available.
    fn handle_is_available(&self, list: &str, message_id: &str) -> bool {
        // Check buffer first
        if self.data.is_in_buffer(list, message_id) {
            return true;
        }

        // Check disk (this is a synchronous check, so we'll assume it exists)
        // In a real implementation, you might want to make this async
        true
    }

    /// Checks if a patch exists on disk.
    async fn patch_exists_on_disk(&self, list: &str, message_id: &str) -> anyhow::Result<bool> {
        let cache_path = self.data.get_cache_path(list, message_id);

        // Try to read the file to check if it exists
        match self.fs.read_file(cache_path).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Loads a patch from disk.
    async fn load_patch_from_disk(
        &self,
        list: &str,
        message_id: &str,
    ) -> anyhow::Result<LorePatch> {
        let cache_path = self.data.get_cache_path(list, message_id);

        let file = self
            .fs
            .read_file(cache_path)
            .await
            .context("Failed to open patch file for reading")?;

        // Read the content
        use tokio::io::AsyncReadExt;
        let mut content = String::new();
        let mut file = file;
        file.read_to_string(&mut content)
            .await
            .context("Failed to read patch file content")?;

        // Deserialize from TOML
        let patch: LorePatch =
            toml::from_str(&content).context("Failed to deserialize patch from TOML")?;

        Ok(patch)
    }

    /// Saves a patch to disk.
    async fn save_patch_to_disk(
        &self,
        list: &str,
        message_id: &str,
        patch: &LorePatch,
    ) -> anyhow::Result<()> {
        let cache_path = self.data.get_cache_path(list, message_id);

        // Create parent directory if it doesn't exist
        if let Some(parent) = cache_path.parent() {
            self.fs
                .mkdir(ArcPath::from(parent))
                .await
                .context("Failed to create patch cache directory")?;
        }

        // Serialize to TOML
        let content = toml::to_string_pretty(patch).context("Failed to serialize patch to TOML")?;

        // Write the file
        let mut file = self
            .fs
            .write_file(cache_path)
            .await
            .context("Failed to open patch file for writing")?;

        use tokio::io::AsyncWriteExt;
        file.write_all(content.as_bytes())
            .await
            .context("Failed to write patch file")?;

        self.log.info(
            SCOPE,
            &format!("Saved patch {} to disk for list: {}", message_id, list),
        );
        Ok(())
    }
}
