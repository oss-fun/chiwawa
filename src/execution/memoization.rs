use crate::execution::value::Val;
use lru::LruCache;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LocalAccessTracker {
    pub bitset_low: u64,
    // Bitset for locals 64-127 (only allocated if needed)
    pub bitset_high: Option<u64>,
    pub versions: Vec<(u32, u64)>,
}

impl LocalAccessTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn track_access(&mut self, index: u32, version: u64) {
        if index < 64 {
            self.bitset_low |= 1u64 << index;
        } else if index < 128 {
            let high = self.bitset_high.get_or_insert(0);
            *high |= 1u64 << (index - 64);
        }
        // Only store version if not already tracked
        if !self.versions.iter().any(|(idx, _)| *idx == index) {
            self.versions.push((index, version));
        }
    }

    pub fn is_accessed(&self, index: u32) -> bool {
        if index < 64 {
            (self.bitset_low & (1u64 << index)) != 0
        } else if index < 128 {
            self.bitset_high
                .map(|high| (high & (1u64 << (index - 64))) != 0)
                .unwrap_or(false)
        } else {
            self.versions.iter().any(|(idx, _)| *idx == index)
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GlobalAccessTracker {
    pub bitset: u64,
    // Extended tracking for more globals
    pub extended: Option<HashSet<u32>>,
}

// Lightweight tracking for memory chunk accesses
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MemoryChunkTracker {
    // Index 0: chunks 0-63, Index 1: chunks 64-127, etc.
    pub bitsets: Vec<u64>,
    pub extended: Option<HashSet<u32>>,
}

impl GlobalAccessTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn track_access(&mut self, index: u32) {
        if index < 64 {
            self.bitset |= 1u64 << index;
        } else {
            self.extended.get_or_insert_with(HashSet::new).insert(index);
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = u32> + '_ {
        (0..64)
            .filter(move |i| (self.bitset & (1u64 << i)) != 0)
            .map(|i| i as u32)
            .chain(self.extended.iter().flat_map(|set| set.iter().copied()))
    }
}

impl MemoryChunkTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn track_access(&mut self, chunk_idx: u32) {
        let bitset_idx = (chunk_idx / 64) as usize;
        let bit_pos = chunk_idx % 64;

        if bitset_idx >= self.bitsets.len() {
            if bitset_idx > 16 {
                // Arbitrary threshold: 16 * 64 = 1024 chunks
                self.extended
                    .get_or_insert_with(HashSet::new)
                    .insert(chunk_idx);
                return;
            }
            self.bitsets.resize(bitset_idx + 1, 0);
        }

        self.bitsets[bitset_idx] |= 1u64 << bit_pos;
    }

    pub fn iter(&self) -> impl Iterator<Item = u32> + '_ {
        self.bitsets
            .iter()
            .enumerate()
            .flat_map(|(idx, &bitset)| {
                (0..64)
                    .filter(move |i| (bitset & (1u64 << i)) != 0)
                    .map(move |i| (idx * 64 + i) as u32)
            })
            .chain(self.extended.iter().flat_map(|set| set.iter().copied()))
    }
}

// Configuration constants
const CACHE_EXECUTION_THRESHOLD: u32 = 10; // Number of executions before caching
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
    pub written_chunks_tracker: MemoryChunkTracker, // Bitset-based chunk tracking
    pub chunk_versions: Vec<(u32, u64)>,            // Version snapshot when cached
    pub written_globals_bitset_low: u64,            // Bitset for first 64 globals
    pub written_globals_bitset_high: Option<u64>,   // Bitset for globals 64-127
    pub written_globals_extended: Option<Vec<u32>>, // For globals >= 128
    pub global_versions: Vec<(u32, u64)>,           // Global version snapshot when cached
    pub accessed_locals_bitset_low: u64,            // Bitset for first 64 locals
    pub accessed_locals_bitset_high: Option<u64>,   // Bitset for locals 64-127
    pub accessed_locals_versions: Vec<(u32, u64)>,  // Version info for accessed locals
}

#[derive(Clone, Debug)]
pub enum BlockCacheValue {
    CachedResult(CachedBlock),
    NonCacheable,
    ExecutionCount(u32),
}

#[derive(Debug, Default)]
pub struct CacheStats {
    pub hit_count: AtomicU64,
    pub invalidation_by_memory: AtomicU64,
    pub invalidation_by_global: AtomicU64,
    pub eviction_count: AtomicU64,
    pub store_count: AtomicU64,
}

impl Clone for CacheStats {
    fn clone(&self) -> Self {
        Self {
            hit_count: AtomicU64::new(self.hit_count.load(Ordering::Relaxed)),
            invalidation_by_memory: AtomicU64::new(
                self.invalidation_by_memory.load(Ordering::Relaxed),
            ),
            invalidation_by_global: AtomicU64::new(
                self.invalidation_by_global.load(Ordering::Relaxed),
            ),
            eviction_count: AtomicU64::new(self.eviction_count.load(Ordering::Relaxed)),
            store_count: AtomicU64::new(self.store_count.load(Ordering::Relaxed)),
        }
    }
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

#[derive(Debug, Clone)]
pub struct BlockMemoizationCache {
    cache: LruCache<BlockCacheKey, BlockCacheValue>,
    write_patterns: LruCache<(usize, usize), MemoryChunkTracker>, // LRU cache for write patterns
    global_write_patterns: LruCache<(usize, usize), GlobalAccessTracker>, // LRU cache for global write patterns
    pub stats: CacheStats,
}

const WRITE_PATTERN_CACHE_SIZE: usize = 1000;

impl BlockMemoizationCache {
    pub fn new() -> Self {
        Self {
            cache: LruCache::new(std::num::NonZeroUsize::new(CACHE_SIZE).unwrap()),
            write_patterns: LruCache::new(
                std::num::NonZeroUsize::new(WRITE_PATTERN_CACHE_SIZE).unwrap(),
            ),
            global_write_patterns: LruCache::new(
                std::num::NonZeroUsize::new(WRITE_PATTERN_CACHE_SIZE).unwrap(),
            ),
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
        local_versions: &[u64],
        current_chunk_versions: &[(u32, u64)],
        current_global_versions: &[(u32, u64)],
    ) -> Option<Vec<Val>> {
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
                    for &(chunk, cached_version) in &cached_block.chunk_versions {
                        // Direct linear search instead of HashMap creation
                        let current_version = current_chunk_versions
                            .iter()
                            .find(|(idx, _)| *idx == chunk)
                            .map(|(_, ver)| *ver);

                        if let Some(current_version) = current_version {
                            if current_version != cached_version {
                                self.stats
                                    .invalidation_by_memory
                                    .fetch_add(1, Ordering::Relaxed);
                                return None; // Chunk version changed, cache invalid
                            }
                        }
                    }

                    // Check if any accessed globals have changed
                    for &(global_idx, cached_version) in &cached_block.global_versions {
                        // Direct linear search instead of HashMap creation
                        let current_version = current_global_versions
                            .iter()
                            .find(|(idx, _)| *idx == global_idx)
                            .map(|(_, ver)| *ver);

                        if let Some(current_version) = current_version {
                            if current_version != cached_version {
                                self.stats
                                    .invalidation_by_global
                                    .fetch_add(1, Ordering::Relaxed);
                                return None; // Global version changed, cache invalid
                            }
                        }
                    }

                    // Check if any accessed locals have changed using bitsets
                    for &(local_idx, cached_version) in &cached_block.accessed_locals_versions {
                        if (local_idx as usize) < local_versions.len() {
                            let current_version = local_versions[local_idx as usize];
                            if current_version != cached_version {
                                // Local variable version changed, cache invalid
                                return None;
                            }
                        }
                    }

                    self.stats.hit_count.fetch_add(1, Ordering::Relaxed);
                    Some(cached_block.result)
                }
                BlockCacheValue::NonCacheable => None,
                BlockCacheValue::ExecutionCount(_) => None, // Still tracking executions
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
        written_globals_tracker: GlobalAccessTracker,
        global_versions: Vec<(u32, u64)>,
        accessed_locals_tracker: LocalAccessTracker,
    ) {
        let stack_hash = Self::compute_stack_hash(input_stack);
        let locals_hash = Self::compute_locals_hash(locals);
        let key = BlockCacheKey {
            start_ip,
            end_ip,
            stack_hash,
            locals_hash,
        };

        // Check if we already have this block in cache
        let execution_count = match self.cache.get(&key) {
            Some(BlockCacheValue::ExecutionCount(count)) => count + 1,
            Some(BlockCacheValue::CachedResult(_)) => {
                // Already cached, nothing to do
                return;
            }
            Some(BlockCacheValue::NonCacheable) => {
                // Non-cacheable block
                return;
            }
            None => 1, // First execution
        };

        // Only cache blocks that have been executed multiple times
        if execution_count < CACHE_EXECUTION_THRESHOLD {
            // Store execution count
            self.insert(key, BlockCacheValue::ExecutionCount(execution_count));

            // Still track write patterns for future use
            let mut written_chunks_tracker = MemoryChunkTracker::new();
            for &(chunk_idx, _) in &written_chunks {
                written_chunks_tracker.track_access(chunk_idx);
            }
            self.write_patterns
                .push((start_ip, end_ip), written_chunks_tracker);

            // Store global write patterns
            self.global_write_patterns
                .push((start_ip, end_ip), written_globals_tracker.clone());
            return;
        }

        // Create memory chunk tracker
        let mut written_chunks_tracker = MemoryChunkTracker::new();
        for &(chunk_idx, _) in &written_chunks {
            written_chunks_tracker.track_access(chunk_idx);
        }

        // Store write patterns separately for reuse
        self.write_patterns
            .push((start_ip, end_ip), written_chunks_tracker.clone());

        // Store global write patterns
        self.global_write_patterns
            .push((start_ip, end_ip), written_globals_tracker.clone());

        // Extract globals bitset and extended list
        let written_globals_extended = written_globals_tracker
            .extended
            .map(|extended| extended.into_iter().collect::<Vec<_>>());

        let cached_block = CachedBlock {
            result: output_stack,
            written_chunks_tracker,
            chunk_versions: written_chunks,
            written_globals_bitset_low: written_globals_tracker.bitset,
            written_globals_bitset_high: None, // TODO: Add support for 64-127 globals if needed
            written_globals_extended,
            global_versions,
            accessed_locals_bitset_low: accessed_locals_tracker.bitset_low,
            accessed_locals_bitset_high: accessed_locals_tracker.bitset_high,
            accessed_locals_versions: accessed_locals_tracker.versions,
        };
        let value = BlockCacheValue::CachedResult(cached_block);
        self.insert(key, value);
        self.stats.store_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_write_pattern(
        &mut self,
        start_ip: usize,
        end_ip: usize,
    ) -> Option<&MemoryChunkTracker> {
        self.write_patterns.get(&(start_ip, end_ip))
    }

    pub fn store_write_pattern(
        &mut self,
        start_ip: usize,
        end_ip: usize,
        chunks: MemoryChunkTracker,
    ) {
        self.write_patterns.push((start_ip, end_ip), chunks);
    }

    pub fn get_global_write_pattern(
        &mut self,
        start_ip: usize,
        end_ip: usize,
    ) -> Option<&GlobalAccessTracker> {
        self.global_write_patterns.get(&(start_ip, end_ip))
    }
}
