use std::sync::OnceLock;

/// Global runtime context carrying CLI flags.
/// Initialized once at startup and immutable thereafter.
#[derive(Debug, Clone)]
pub struct RunContext {
    pub dry_run: bool,
    pub no_tui: bool,
    pub verbose: bool,
    pub parallel: usize,
    pub max_retries: u32,
}

static CONTEXT: OnceLock<RunContext> = OnceLock::new();

/// Initialize the global context. Must be called exactly once from main.
pub fn init(ctx: RunContext) {
    CONTEXT.set(ctx).expect("RunContext already initialized");
}

/// Get a reference to the global context.
pub fn get() -> &'static RunContext {
    CONTEXT.get().expect("RunContext not initialized")
}

/// Convenience: is dry-run mode active?
pub fn is_dry_run() -> bool {
    get().dry_run
}
