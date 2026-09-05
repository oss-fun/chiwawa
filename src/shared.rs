//! Reference-counted pointer for state that crosses thread boundaries.
//!
//! wasi-threads gives every thread its own module instance and shares only the
//! linear memory, so the parsed [`Module`](crate::structure::module::Module)
//! and [`MemAddr`](crate::execution::mem::MemAddr) are the only values sent to
//! another thread. They hold `Shared`, which is `Arc` under the `threads`
//! feature and `Rc` otherwise so single-threaded builds keep non-atomic
//! reference counts.

#[cfg(feature = "threads")]
pub type Shared<T> = std::sync::Arc<T>;

#[cfg(not(feature = "threads"))]
pub type Shared<T> = std::rc::Rc<T>;
