use crate::execution::value::Val;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct BlockCacheKey {
    pub start_ip: usize,
    pub end_ip: usize,
    pub stack_hash: u64,
    pub locals_hash: u64,
}

#[derive(Clone, Debug)]
pub struct CachedBlock {
    pub result: Vec<Val>,
    pub written_pages: std::collections::HashSet<u32>, // Pages written during execution
    pub page_versions: Vec<(u32, u64)>,                // Version snapshot when cached
}

#[derive(Clone, Debug)]
pub enum BlockCacheValue {
    CachedResult(CachedBlock),
    NonCacheable,
}

#[derive(Debug)]
pub struct BlockMemoizationCache {
    cache: HashMap<BlockCacheKey, BlockCacheValue>,
    write_patterns: HashMap<(usize, usize), std::collections::HashSet<u32>>, // (start_ip, end_ip) -> written_pages
    cached_blocks: std::collections::HashSet<(usize, usize)>, // Track which blocks have cache entries
    max_entries: usize,
}

impl BlockMemoizationCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            write_patterns: HashMap::new(),
            cached_blocks: std::collections::HashSet::new(),
            max_entries: 1000, // Reasonable limit for block cache
        }
    }

    fn get(&self, key: &BlockCacheKey) -> Option<&BlockCacheValue> {
        self.cache.get(key)
    }

    fn insert(&mut self, key: BlockCacheKey, value: BlockCacheValue) {
        // Simple eviction: clear cache when limit is reached
        if self.cache.len() >= self.max_entries {
            self.cache.clear();
            self.write_patterns.clear();
            self.cached_blocks.clear();
        }

        self.cache.insert(key, value);
    }

    fn compute_stack_hash(stack: &[Val]) -> u64 {
        let mut hasher = DefaultHasher::new();
        // Only hash the last 8 values for better performance
        const MAX_HASH_VALUES: usize = 8;
        if stack.len() <= MAX_HASH_VALUES {
            stack.hash(&mut hasher);
        } else {
            stack[stack.len() - MAX_HASH_VALUES..].hash(&mut hasher);
        }
        hasher.finish()
    }

    fn compute_locals_hash(locals: &[Val]) -> u64 {
        let mut hasher = DefaultHasher::new();
        locals.hash(&mut hasher);
        hasher.finish()
    }

    pub fn check_block(
        &self,
        start_ip: usize,
        end_ip: usize,
        stack: &[Val],
        locals: &[Val],
        current_page_versions: &[(u32, u64)],
    ) -> Option<Vec<Val>> {
        // Early exit if this block range has never been cached
        if !self.cached_blocks.contains(&(start_ip, end_ip)) {
            return None;
        }

        // Only compute hashes if block range exists in cache
        let stack_hash = Self::compute_stack_hash(stack);
        let locals_hash = Self::compute_locals_hash(locals);
        let key = BlockCacheKey {
            start_ip,
            end_ip,
            stack_hash,
            locals_hash,
        };
        self.get(&key).and_then(|value| match value {
            BlockCacheValue::CachedResult(cached_block) => {
                // Check if any accessed pages have changed
                let current_versions_map: std::collections::HashMap<u32, u64> =
                    current_page_versions.iter().cloned().collect();

                for &(page, cached_version) in &cached_block.page_versions {
                    if let Some(&current_version) = current_versions_map.get(&page) {
                        if current_version != cached_version {
                            return None; // Page version changed, cache invalid
                        }
                    }
                }
                Some(cached_block.result.clone())
            }
            BlockCacheValue::NonCacheable => None,
        })
    }

    pub fn store_block(
        &mut self,
        start_ip: usize,
        end_ip: usize,
        input_stack: &[Val],
        locals: &[Val],
        written_pages: Vec<(u32, u64)>,
        output_stack: Vec<Val>,
    ) {
        let stack_hash = Self::compute_stack_hash(input_stack);
        let locals_hash = Self::compute_locals_hash(locals);
        let key = BlockCacheKey {
            start_ip,
            end_ip,
            stack_hash,
            locals_hash,
        };

        // Extract page indices for written pages tracking
        let written_page_set: std::collections::HashSet<u32> =
            written_pages.iter().map(|&(page, _)| page).collect();

        // Store write pattern separately for reuse
        self.write_patterns
            .insert((start_ip, end_ip), written_page_set.clone());

        // Record that this block range now has cache entries
        self.cached_blocks.insert((start_ip, end_ip));

        let cached_block = CachedBlock {
            result: output_stack,
            written_pages: written_page_set,
            page_versions: written_pages,
        };
        let value = BlockCacheValue::CachedResult(cached_block);
        self.insert(key, value);
    }

    pub fn get_write_pattern(
        &self,
        start_ip: usize,
        end_ip: usize,
    ) -> Option<&std::collections::HashSet<u32>> {
        self.write_patterns.get(&(start_ip, end_ip))
    }

    pub fn store_write_pattern(
        &mut self,
        start_ip: usize,
        end_ip: usize,
        pages: std::collections::HashSet<u32>,
    ) {
        self.write_patterns.insert((start_ip, end_ip), pages);
    }
}
