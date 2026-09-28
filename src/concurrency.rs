//! Concurrency and thread pool configuration utilities.

const THREAD_STACK_SIZE: usize = 64 * 1024 * 1024;

/// Configures the global Rayon thread pool with a 64MB worker thread stack size
/// matching the rustc standard, safeguarding deep AST recursions on massive source files.
///
/// This function is idempotent and safe to invoke multiple times; subsequent calls
/// after the global thread pool is built will be safe no-ops.
pub fn ensure_thread_pool_configured() {
    let _ = rayon::ThreadPoolBuilder::new()
        .stack_size(THREAD_STACK_SIZE)
        .build_global();
}
