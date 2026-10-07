//! Instruction execution statistics collection.

use crate::instrument::instruction_name;

/// Number of handler-index slots. Must exceed the largest `HANDLER_IDX_*`
/// (currently 0x104); 0x105 leaves the counter array indexable by every index.
const HANDLER_SLOTS: usize = 0x105;

/// Collects per-instruction execution counts.
#[derive(Debug)]
pub struct ExecutionStats {
    total: u64,
    counts: Box<[u64]>,
}

impl ExecutionStats {
    /// Creates a new statistics collector.
    pub fn new() -> Self {
        Self {
            total: 0,
            counts: vec![0u64; HANDLER_SLOTS].into_boxed_slice(),
        }
    }

    /// Records execution of a single instruction by handler index.
    #[inline]
    pub fn record_instruction(&mut self, handler_index: usize) {
        self.total += 1;
        self.counts[handler_index] += 1;
    }

    /// Prints statistics summary to stderr.
    pub fn report(&self) {
        let total = self.total;

        eprintln!("=== Execution Statistics ===");
        eprintln!("Total instructions executed: {}", total);

        if total == 0 {
            eprintln!("=======================");
            return;
        }

        // Collect non-zero instruction counts and sort descending
        let mut counts: Vec<(usize, u64)> = self
            .counts
            .iter()
            .enumerate()
            .filter(|(_, &count)| count > 0)
            .map(|(idx, &count)| (idx, count))
            .collect();

        counts.sort_by(|a, b| b.1.cmp(&a.1));

        eprintln!("\nTop Instructions:");
        let top_n = 20.min(counts.len());
        for (idx, count) in counts.iter().take(top_n) {
            let name = instruction_name(*idx);
            let percentage = (*count as f64 / total as f64) * 100.0;
            eprintln!(
                "  {:25} {:12} ({:5.1}%)",
                format!("{}:", name),
                count,
                percentage
            );
        }

        if counts.len() > top_n {
            eprintln!("  ... and {} more", counts.len() - top_n);
        }

        eprintln!("=======================");
    }
}
