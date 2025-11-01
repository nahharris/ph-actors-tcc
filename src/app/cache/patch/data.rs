use crate::ArcPath;
use lru::LruCache;
use std::num::NonZeroUsize;

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
