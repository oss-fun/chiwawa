use crate::execution::stack::ProcessedInstr;
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
    pub accessed_pages: Vec<(u32, u64)>, // (page_index, version)
}

#[derive(Clone, Debug)]
pub enum BlockCacheValue {
    CachedResult(Vec<Val>),
    NonCacheable,
}

#[derive(Debug)]
pub struct BlockMemoizationCache {
    cache: HashMap<BlockCacheKey, BlockCacheValue>,
    max_entries: usize,
}

impl BlockMemoizationCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
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
        }

        self.cache.insert(key, value);
    }

    fn compute_stack_hash(stack: &[Val]) -> u64 {
        let mut hasher = DefaultHasher::new();
        stack.hash(&mut hasher);
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
        accessed_pages: &[(u32, u64)],
    ) -> Option<Vec<Val>> {
        let stack_hash = Self::compute_stack_hash(stack);
        let locals_hash = Self::compute_locals_hash(locals);
        let key = BlockCacheKey {
            start_ip,
            end_ip,
            stack_hash,
            locals_hash,
            accessed_pages: accessed_pages.to_vec(),
        };
        self.get(&key).and_then(|value| match value {
            BlockCacheValue::CachedResult(result) => Some(result.clone()),
            BlockCacheValue::NonCacheable => None,
        })
    }

    pub fn store_block(
        &mut self,
        start_ip: usize,
        end_ip: usize,
        input_stack: &[Val],
        locals: &[Val],
        accessed_pages: Vec<(u32, u64)>,
        output_stack: Vec<Val>,
    ) {
        let stack_hash = Self::compute_stack_hash(input_stack);
        let locals_hash = Self::compute_locals_hash(locals);
        let key = BlockCacheKey {
            start_ip,
            end_ip,
            stack_hash,
            locals_hash,
            accessed_pages,
        };
        let value = BlockCacheValue::CachedResult(output_stack);
        self.insert(key, value);
    }
}

fn is_vm_mutable_instruction(handler_index: usize) -> bool {
    use crate::execution::stack::*;

    match handler_index {
        // Memory stores (mutate memory state)
        HANDLER_IDX_I32_STORE
        | HANDLER_IDX_I64_STORE
        | HANDLER_IDX_F32_STORE
        | HANDLER_IDX_F64_STORE
        | HANDLER_IDX_I32_STORE8
        | HANDLER_IDX_I32_STORE16
        | HANDLER_IDX_I64_STORE8
        | HANDLER_IDX_I64_STORE16
        | HANDLER_IDX_I64_STORE32 => true,

        // Store superinstructions
        HANDLER_IDX_I32_STORE_I32_CONST
        | HANDLER_IDX_I64_STORE_I64_CONST
        | HANDLER_IDX_I32_CONST_I64_STORE
        | HANDLER_IDX_I32_CONST_F32_STORE
        | HANDLER_IDX_I32_CONST_F64_STORE
        | HANDLER_IDX_I64_CONST_I32_STORE
        | HANDLER_IDX_I64_CONST_I64_STORE
        | HANDLER_IDX_I64_CONST_F32_STORE
        | HANDLER_IDX_I64_CONST_F64_STORE
        | HANDLER_IDX_I32_STORE8_CONST
        | HANDLER_IDX_I32_STORE16_CONST
        | HANDLER_IDX_I64_STORE8_CONST
        | HANDLER_IDX_I64_STORE16_CONST
        | HANDLER_IDX_I64_STORE32_CONST => true,

        HANDLER_IDX_GLOBAL_SET => true,

        // Local variable mutations (only writes are mutable)
        HANDLER_IDX_LOCAL_SET
        | HANDLER_IDX_LOCAL_TEE
        | HANDLER_IDX_LOCAL_SET_I32_CONST
        | HANDLER_IDX_LOCAL_SET_I64_CONST
        | HANDLER_IDX_LOCAL_SET_F32_CONST
        | HANDLER_IDX_LOCAL_SET_F64_CONST => true,

        HANDLER_IDX_CALL | HANDLER_IDX_CALL_INDIRECT => true,

        HANDLER_IDX_MEMORY_SIZE
        | HANDLER_IDX_MEMORY_GROW
        | HANDLER_IDX_MEMORY_COPY
        | HANDLER_IDX_MEMORY_FILL
        | HANDLER_IDX_MEMORY_INIT => true,

        HANDLER_IDX_TABLE_SET | HANDLER_IDX_TABLE_FILL => true,

        // Control flow instructions (cannot be cached due to branching)
        HANDLER_IDX_IF
        | HANDLER_IDX_ELSE
        | HANDLER_IDX_BR
        | HANDLER_IDX_BR_IF
        | HANDLER_IDX_BR_TABLE
        | HANDLER_IDX_RETURN
        | HANDLER_IDX_UNREACHABLE => true,

        // Nested block structures (cannot be cached)
        HANDLER_IDX_BLOCK | HANDLER_IDX_LOOP | HANDLER_IDX_END => true,

        _ => false,
    }
}

pub fn is_vm_immutable_block(instructions: &[ProcessedInstr]) -> bool {
    for instr in instructions {
        if is_vm_mutable_instruction(instr.handler_index) {
            return false;
        }
    }
    true
}
