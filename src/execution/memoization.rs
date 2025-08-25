use crate::execution::value::Val;
use lru::LruCache;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};

// Configuration constants
const CACHE_EXECUTION_THRESHOLD: u32 = 3; // Number of executions before caching
const CACHE_SIZE: usize = 5000; // Maximum number of cached blocks

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
    pub written_chunks: std::collections::HashSet<u32>, // Chunks written during execution
    pub chunk_versions: Vec<(u32, u64)>,                // Version snapshot when cached
    pub written_globals: std::collections::HashSet<u32>, // Globals written during execution
    pub global_versions: Vec<(u32, u64)>,               // Global version snapshot when cached
}

#[derive(Clone, Debug)]
pub enum BlockCacheValue {
    CachedResult(CachedBlock),
    NonCacheable,
}

#[derive(Debug, Default)]
pub struct CacheStats {
    pub hit_count: AtomicU64,
    pub invalidation_by_memory: AtomicU64,
    pub invalidation_by_global: AtomicU64,
    pub eviction_count: AtomicU64,
    pub store_count: AtomicU64,
}

impl CacheStats {
    pub fn report(&self) {
        let hits = self.hit_count.load(Ordering::Relaxed);
        let stores = self.store_count.load(Ordering::Relaxed);
        let mem_invalidations = self.invalidation_by_memory.load(Ordering::Relaxed);
        let global_invalidations = self.invalidation_by_global.load(Ordering::Relaxed);
        let evictions = self.eviction_count.load(Ordering::Relaxed);

        let total_accesses = hits + stores;
        let hit_rate = if total_accesses > 0 {
            (hits as f64 / total_accesses as f64) * 100.0
        } else {
            0.0
        };

        eprintln!("=== Cache Statistics ===");
        eprintln!("Cache hits: {} ({:.1}% hit rate)", hits, hit_rate);
        eprintln!("Blocks cached: {}", stores);
        eprintln!(
            "Invalidations: {} (memory: {}, global: {})",
            mem_invalidations + global_invalidations,
            mem_invalidations,
            global_invalidations
        );
        eprintln!("Evictions: {}", evictions);
        eprintln!("Chunk size: {} bytes", crate::execution::mem::CHUNK_SIZE);
        eprintln!("=======================");
    }
}

#[derive(Debug)]
pub struct BlockMemoizationCache {
    cache: LruCache<BlockCacheKey, BlockCacheValue>,
    write_patterns: HashMap<(usize, usize), std::collections::HashSet<u32>>, // (start_ip, end_ip) -> written_chunks
    global_write_patterns: HashMap<(usize, usize), std::collections::HashSet<u32>>, // (start_ip, end_ip) -> written_globals
    cached_blocks: std::collections::HashSet<(usize, usize)>, // Track which blocks have cache entries
    block_execution_counts: HashMap<(usize, usize), u32>, // Track execution count for each block
    pub stats: CacheStats,
}

impl BlockMemoizationCache {
    pub fn new() -> Self {
        Self {
            cache: LruCache::new(std::num::NonZeroUsize::new(CACHE_SIZE).unwrap()),
            write_patterns: HashMap::new(),
            global_write_patterns: HashMap::new(),
            cached_blocks: std::collections::HashSet::new(),
            block_execution_counts: HashMap::new(),
            stats: CacheStats::default(),
        }
    }

    fn get(&mut self, key: &BlockCacheKey) -> Option<&BlockCacheValue> {
        self.cache.get(key)
    }

    fn insert(&mut self, key: BlockCacheKey, value: BlockCacheValue) {
        if let Some(_) = self.cache.push(key, value) {
            self.stats.eviction_count.fetch_add(1, Ordering::Relaxed);
        }
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
        &mut self,
        start_ip: usize,
        end_ip: usize,
        stack: &[Val],
        locals: &[Val],
        current_chunk_versions: &[(u32, u64)],
        current_global_versions: &[(u32, u64)],
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

        let cached_value = self.get(&key).cloned();

        if let Some(value) = cached_value {
            match value {
                BlockCacheValue::CachedResult(cached_block) => {
                    // Check if any accessed chunks have changed
                    let current_versions_map: std::collections::HashMap<u32, u64> =
                        current_chunk_versions.iter().cloned().collect();

                    for &(chunk, cached_version) in &cached_block.chunk_versions {
                        if let Some(&current_version) = current_versions_map.get(&chunk) {
                            if current_version != cached_version {
                                self.stats
                                    .invalidation_by_memory
                                    .fetch_add(1, Ordering::Relaxed);
                                return None; // Chunk version changed, cache invalid
                            }
                        }
                    }

                    // Check if any accessed globals have changed
                    let current_global_versions_map: std::collections::HashMap<u32, u64> =
                        current_global_versions.iter().cloned().collect();

                    for &(global_idx, cached_version) in &cached_block.global_versions {
                        if let Some(&current_version) = current_global_versions_map.get(&global_idx)
                        {
                            if current_version != cached_version {
                                self.stats
                                    .invalidation_by_global
                                    .fetch_add(1, Ordering::Relaxed);
                                return None; // Global version changed, cache invalid
                            }
                        }
                    }

                    self.stats.hit_count.fetch_add(1, Ordering::Relaxed);
                    Some(cached_block.result)
                }
                BlockCacheValue::NonCacheable => None,
            }
        } else {
            None
        }
    }

    pub fn store_block(
        &mut self,
        start_ip: usize,
        end_ip: usize,
        input_stack: &[Val],
        locals: &[Val],
        written_chunks: Vec<(u32, u64)>,
        output_stack: Vec<Val>,
        written_globals: std::collections::HashSet<u32>,
        global_versions: Vec<(u32, u64)>,
    ) {
        // Increment execution count
        let count = self
            .block_execution_counts
            .entry((start_ip, end_ip))
            .or_insert(0);
        *count += 1;

        // Only cache blocks that have been executed multiple times
        if *count < CACHE_EXECUTION_THRESHOLD {
            // Still track write patterns for future use
            let written_chunk_set: std::collections::HashSet<u32> =
                written_chunks.iter().map(|&(chunk, _)| chunk).collect();
            self.write_patterns
                .insert((start_ip, end_ip), written_chunk_set);
            self.global_write_patterns
                .insert((start_ip, end_ip), written_globals);
            return;
        }

        let stack_hash = Self::compute_stack_hash(input_stack);
        let locals_hash = Self::compute_locals_hash(locals);
        let key = BlockCacheKey {
            start_ip,
            end_ip,
            stack_hash,
            locals_hash,
        };

        // Extract chunk indices for written chunks tracking
        let written_chunk_set: std::collections::HashSet<u32> =
            written_chunks.iter().map(|&(chunk, _)| chunk).collect();

        // Store write patterns separately for reuse
        self.write_patterns
            .insert((start_ip, end_ip), written_chunk_set.clone());
        self.global_write_patterns
            .insert((start_ip, end_ip), written_globals.clone());

        // Record that this block range now has cache entries
        self.cached_blocks.insert((start_ip, end_ip));

        let cached_block = CachedBlock {
            result: output_stack,
            written_chunks: written_chunk_set,
            chunk_versions: written_chunks,
            written_globals,
            global_versions,
        };
        let value = BlockCacheValue::CachedResult(cached_block);
        self.insert(key, value);
        self.stats.store_count.fetch_add(1, Ordering::Relaxed);
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

    pub fn get_global_write_pattern(
        &self,
        start_ip: usize,
        end_ip: usize,
    ) -> Option<&std::collections::HashSet<u32>> {
        self.global_write_patterns.get(&(start_ip, end_ip))
    }
}
