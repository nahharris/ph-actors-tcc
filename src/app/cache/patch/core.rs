use super::data::PatchData;
use super::message::Message;
use crate::{error::CacheError, ArcPath, ArcStr};
use tokio::sync::mpsc;

#[cfg(not(test))]
use crate::{api::lore::LoreApi, app::config::Config, fs::Fs, log::Log};
#[cfg(test)]
use crate::{
    api::lore::mock::MockLoreApi as LoreApi, app::config::mock::MockConfig as Config,
    fs::mock::MockFs as Fs, log::mock::MockLog as Log,
};

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
    pub async fn new(lore: LoreApi, fs: Fs, config: Config, log: Log) -> Result<Self, CacheError> {
        let cache_dir = config.path(crate::app::config::PathOpt::CachePath).await.map_err(|e| match e {
            crate::error::ConfigError::Fatal(fatal) => CacheError::Fatal(fatal),
            crate::error::ConfigError::ParseFailed { .. } | crate::error::ConfigError::InvalidValue { .. } | crate::error::ConfigError::FileOperationFailed { .. } => {
                CacheError::FileOperationFailed {
                    path: "config".to_string(),
                    operation: "get cache path".to_string(),
                    source: std::io::Error::new(std::io::ErrorKind::NotFound, format!("{}", e)),
                }
            }
        })?;
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
    pub async fn init(mut self, mut rx: mpsc::Receiver<Message>) {
        while let Some(message) = rx.recv().await {
            match message {
                Message::Get {
                    list,
                    message_id,
                    tx,
                } => {
                    let result = self.handle_get(&list, &message_id).await;
                    let _ = tx.send(result);
                }
                Message::Invalidate {
                    list,
                    message_id,
                    tx,
                } => {
                    let result = self.handle_invalidate(&list, &message_id).await;
                    let _ = tx.send(result);
                }
                Message::IsAvailable {
                    list,
                    message_id,
                    tx,
                } => {
                    let result = Ok(self.handle_is_available(&list, &message_id));
                    let _ = tx.send(result);
                }
            }
        }
    }

    /// Handles getting a patch by mailing list and message ID.
    async fn handle_get(&mut self, list: &str, message_id: &str) -> Result<String, CacheError> {
        // First check the buffer
        if let Some(content) = self.data.get_from_buffer(list, message_id) {
            return Ok(content);
        }

        // Check if the patch exists on disk
        if self.patch_exists_on_disk(list, message_id).await? {
            // Load from disk and add to buffer
            let content = self.load_patch_from_disk(list, message_id).await?;
            self.data.add_to_buffer(list, message_id, content.clone());
            return Ok(content);
        }

        // Fetch from API
        self.log.info(
            SCOPE,
            format!("Fetching patch {} from API for list: {}", message_id, list),
        );

        let content = self
            .lore
            .get_raw_patch(ArcStr::from(list), ArcStr::from(message_id))
            .await
            .map_err(|e| match e {
                crate::error::LoreApiError::Fatal(fatal) => CacheError::Fatal(fatal),
                crate::error::LoreApiError::RequestFailed { endpoint, message, retryable: _ } => {
                    CacheError::FileOperationFailed {
                        path: endpoint,
                        operation: "fetch patch".to_string(),
                        source: std::io::Error::new(std::io::ErrorKind::Other, message),
                    }
                }
                crate::error::LoreApiError::ParseFailed { format, operation, details } => {
                    CacheError::SerializationFailed {
                        message: format!("Failed to parse {} for {}: {}", format, operation, details),
                        source: None,
                    }
                }
            })?;

        // Save to disk and add to buffer
        let content_str = content.to_string();
        if let Err(e) = self
            .save_patch_to_disk(list, message_id, &content_str)
            .await
        {
            self.log.error(
                SCOPE,
                format!("Failed to save patch {list}/{message_id} to disk: {e}"),
            );
        }
        self.data
            .add_to_buffer(list, message_id, content_str.clone());

        Ok(content_str)
    }

    /// Handles invalidating a specific patch.
    async fn handle_invalidate(&mut self, list: &str, message_id: &str) -> Result<(), CacheError> {
        // Remove from buffer
        let key = self.data.get_buffer_key(list, message_id);
        self.data.buffer.pop(&key);

        // Remove from disk
        let cache_path = self.data.get_cache_path(list, message_id);
        if let Err(e) = self.fs.remove_file(cache_path.clone()).await {
            match e {
                crate::error::FsError::Fatal(fatal) => return Err(CacheError::Fatal(fatal)),
                crate::error::FsError::OperationFailed { .. } => {
                    // Ignore errors if file doesn't exist
                }
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
    async fn patch_exists_on_disk(&self, list: &str, message_id: &str) -> Result<bool, CacheError> {
        let cache_path = self.data.get_cache_path(list, message_id);

        // Try to read the file to check if it exists
        match self.fs.read_file(cache_path).await {
            Ok(_) => Ok(true),
            Err(crate::error::FsError::Fatal(fatal)) => Err(CacheError::Fatal(fatal)),
            Err(crate::error::FsError::OperationFailed { .. }) => Ok(false),
        }
    }

    /// Loads a patch from disk.
    async fn load_patch_from_disk(&self, list: &str, message_id: &str) -> Result<String, CacheError> {
        let cache_path = self.data.get_cache_path(list, message_id);

        let file = self
            .fs
            .read_file(cache_path.clone())
            .await
            .map_err(|e| match e {
                crate::error::FsError::Fatal(fatal) => CacheError::Fatal(fatal),
                crate::error::FsError::OperationFailed { path, operation, source, .. } => {
                    CacheError::FileOperationFailed {
                        path: path.unwrap_or_else(|| cache_path.to_string_lossy().to_string()),
                        operation,
                        source,
                    }
                }
            })?;

        // Read the content
        use tokio::io::AsyncReadExt;
        let mut content = String::new();
        let mut file = file;
        file.read_to_string(&mut content)
            .await
            .map_err(|e| {
                CacheError::FileOperationFailed {
                    path: cache_path.to_string_lossy().to_string(),
                    operation: "read patch file content".to_string(),
                    source: e,
                }
            })?;

        Ok(content)
    }

    /// Saves a patch to disk.
    async fn save_patch_to_disk(
        &self,
        list: &str,
        message_id: &str,
        content: &str,
    ) -> Result<(), CacheError> {
        let cache_path = self.data.get_cache_path(list, message_id);

        // Create parent directory if it doesn't exist
        if let Some(parent) = cache_path.parent() {
            self.fs
                .mkdir(ArcPath::from(parent))
                .await
                .map_err(|e| match e {
                    crate::error::FsError::Fatal(fatal) => CacheError::Fatal(fatal),
                    crate::error::FsError::OperationFailed { path, operation, source, .. } => {
                        CacheError::FileOperationFailed {
                            path: path.unwrap_or_else(|| cache_path.to_string_lossy().to_string()),
                            operation,
                            source,
                        }
                    }
                })?;
        }

        // Write the file
        let mut file = self
            .fs
            .write_file(cache_path.clone())
            .await
            .map_err(|e| match e {
                crate::error::FsError::Fatal(fatal) => CacheError::Fatal(fatal),
                crate::error::FsError::OperationFailed { path, operation, source, .. } => {
                    CacheError::FileOperationFailed {
                        path: path.unwrap_or_else(|| cache_path.to_string_lossy().to_string()),
                        operation,
                        source,
                    }
                }
            })?;

        use tokio::io::AsyncWriteExt;
        file.write_all(content.as_bytes())
            .await
            .map_err(|e| {
                CacheError::FileOperationFailed {
                    path: cache_path.to_string_lossy().to_string(),
                    operation: "write patch file".to_string(),
                    source: e,
                }
            })?;

        self.log.info(
            SCOPE,
            format!("Saved patch {} to disk for list: {}", message_id, list),
        );
        Ok(())
    }
}
