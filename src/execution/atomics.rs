//! Parking table behind `memory.atomic.wait` and `memory.atomic.notify`.
//!
//! The wasm intrinsics that would hand these straight to the host's futex are
//! still unstable, so the waiters are tracked here instead. Nothing outside
//! chiwawa's own guests parks on these addresses, so a table of our own is
//! enough.
//!
//! Addresses are spread over a fixed set of shards. One shared condition
//! variable would mean every `notify` woke every parked thread, whatever
//! address it was waiting on, only for most of them to find no wake-up waiting
//! and park again.

use std::collections::BTreeMap;
use std::sync::{Condvar, Mutex, MutexGuard};
use std::time::{Duration, Instant};

/// Waiters parked on one guest address.
#[derive(Default)]
struct Slot {
    /// Threads currently inside `wait`.
    parked: u32,
    /// Wake-ups handed out by `notify` and not yet claimed.
    permits: u32,
}

/// One independently locked slice of the address space.
struct Shard {
    table: Mutex<BTreeMap<usize, Slot>>,
    wakeup: Condvar,
}

const SHARD_COUNT: usize = 64;

const NEW_SHARD: Shard = Shard {
    table: Mutex::new(BTreeMap::new()),
    wakeup: Condvar::new(),
};

static SHARDS: [Shard; SHARD_COUNT] = [NEW_SHARD; SHARD_COUNT];

impl Shard {
    /// The shard an address parks in. `wait` and `notify` are the only callers
    /// and both require at least 4-byte alignment, so the bottom two bits are
    /// always zero and would leave most shards empty.
    fn of(addr: usize) -> &'static Shard {
        &SHARDS[(addr >> 2) % SHARD_COUNT]
    }

    fn lock(&self) -> MutexGuard<'_, BTreeMap<usize, Slot>> {
        self.table
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn take_permit(table: &mut BTreeMap<usize, Slot>, addr: usize) -> bool {
    match table.get_mut(&addr) {
        Some(slot) if slot.permits > 0 => {
            slot.permits -= 1;
            true
        }
        _ => false,
    }
}

/// Parks until notified or `timeout_ns` elapses, unless `matches` reports that
/// the address no longer holds the expected value.
///
/// Returns the value the instruction pushes: 0 woken, 1 not-equal, 2 timed out.
/// `matches` runs under the shard lock so a `notify` cannot slip in between the
/// comparison and parking. A negative `timeout_ns` means no timeout.
pub fn wait(addr: usize, matches: impl FnOnce() -> bool, timeout_ns: i64) -> i32 {
    let shard = Shard::of(addr);
    let mut table = shard.lock();
    if !matches() {
        return 1;
    }
    table.entry(addr).or_default().parked += 1;

    let deadline = if timeout_ns < 0 {
        None
    } else {
        Some(Instant::now() + Duration::from_nanos(timeout_ns as u64))
    };

    let result = loop {
        if take_permit(&mut table, addr) {
            break 0;
        }
        let Some(deadline) = deadline else {
            table = shard
                .wakeup
                .wait(table)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            continue;
        };
        // A timed-out wait loops once more: a notify may have raced in, and
        // otherwise the deadline check above breaks out.
        let now = Instant::now();
        if now >= deadline {
            break 2;
        }
        table = shard
            .wakeup
            .wait_timeout(table, deadline - now)
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .0;
    };

    if let Some(slot) = table.get_mut(&addr) {
        slot.parked -= 1;
        if slot.parked == 0 && slot.permits == 0 {
            table.remove(&addr);
        }
    }
    result
}

/// Wakes up to `count` waiters on `addr`, returning how many were woken.
pub fn notify(addr: usize, count: u32) -> u32 {
    let shard = Shard::of(addr);
    let mut table = shard.lock();
    let Some(slot) = table.get_mut(&addr) else {
        return 0;
    };
    let woken = count.min(slot.parked.saturating_sub(slot.permits));
    if woken == 0 {
        return 0;
    }
    slot.permits += woken;
    drop(table);
    shard.wakeup.notify_all();
    woken
}
