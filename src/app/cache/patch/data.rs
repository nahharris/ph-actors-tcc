use crate::ArcPath;
use crate::ArcStr;
use lru::LruCache;
use std::num::NonZeroUsize;

/// Represents cached patch content with available formats.
///
/// This structure holds the different representations of a patch that can be
/// fetched from the Lore API: raw text for applying patches and displaying content,
/// and metadata JSON for programmatic access.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LorePatch {
    /// The message ID of the patch (used as the cache key)
    pub message_id: ArcStr,
    /// The mailing list this patch belongs to
    pub list: ArcStr,
    /// Raw patch content (for applying patches and display)
    pub raw_content: Option<ArcStr>,
    /// Metadata in JSON format (for programmatic access)
    pub metadata: Option<ArcStr>,
}

impl LorePatch {
    /// Creates a new empty patch with the given message ID and list.
    pub fn new(message_id: ArcStr, list: ArcStr) -> Self {
        Self {
            message_id,
            list,
            raw_content: None,
            metadata: None,
        }
    }

    /// Sets the raw content for this patch.
    pub fn with_raw(mut self, raw_content: ArcStr) -> Self {
        self.raw_content = Some(raw_content);
        self
    }

    /// Sets the metadata for this patch.
    pub fn with_metadata(mut self, metadata: ArcStr) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Returns true if the patch has raw content.
    pub fn has_raw(&self) -> bool {
        self.raw_content.is_some()
    }

    /// Returns true if the patch has metadata.
    pub fn has_metadata(&self) -> bool {
        self.metadata.is_some()
    }

    /// Returns true if the patch has any content.
    pub fn has_any_content(&self) -> bool {
        self.has_raw() || self.has_metadata()
    }
}

/// Internal state for the Patch Actor.
pub struct PatchData {
    /// Small in-memory buffer for fast access to recently used patches
    pub buffer: LruCache<String, String>,
    /// Directory for cache files
    pub cache_dir: ArcPath,
}

impl PatchData {
    /// Creates a new PatchData instance.
    pub fn new(cache_dir: ArcPath) -> Self {
        // Use a small buffer size for memory efficiency
        let buffer = LruCache::new(NonZeroUsize::new(50).unwrap());

        Self { buffer, cache_dir }
    }

    /// Gets the cache file path for a specific patch.
    pub fn get_cache_path(&self, list: &str, message_id: &str) -> ArcPath {
        ArcPath::from(
            &self
                .cache_dir
                .join(list)
                .join(format!("{}.mbox", message_id)),
        )
    }

    /// Gets the buffer key for a patch.
    pub fn get_buffer_key(&self, list: &str, message_id: &str) -> String {
        format!("{}:{}", list, message_id)
    }

    /// Adds a patch to the buffer.
    pub fn add_to_buffer(&mut self, list: &str, message_id: &str, content: String) {
        let key = self.get_buffer_key(list, message_id);
        self.buffer.put(key, content);
    }

    /// Gets a patch from the buffer.
    pub fn get_from_buffer(&mut self, list: &str, message_id: &str) -> Option<String> {
        let key = self.get_buffer_key(list, message_id);
        self.buffer.get(&key).cloned()
    }

    /// Checks if a patch is in the buffer.
    pub fn is_in_buffer(&self, list: &str, message_id: &str) -> bool {
        let key = self.get_buffer_key(list, message_id);
        self.buffer.contains(&key)
    }
}
