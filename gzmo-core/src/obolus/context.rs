//! Thread-local call context for Obolus process attribution.

use std::cell::RefCell;

/// Per-call metadata attached to instrumented gateway completions.
#[derive(Debug, Clone, Default)]
pub struct ObolusCallContext {
    pub process: String,
    pub task_kind: Option<String>,
    pub caller: String,
    pub correlation_id: Option<String>,
    pub action_id: Option<String>,
}

thread_local! {
    static CURRENT: RefCell<Option<ObolusCallContext>> = const { RefCell::new(None) };
}

/// RAII guard — installs context for the current thread until dropped.
pub struct CallContextGuard {
    prev: Option<ObolusCallContext>,
}

impl CallContextGuard {
    pub fn new(ctx: ObolusCallContext) -> Self {
        let mut prev = None;
        CURRENT.with(|cell| {
            prev = cell.borrow().clone();
            *cell.borrow_mut() = Some(ctx);
        });
        Self { prev }
    }
}

impl Drop for CallContextGuard {
    fn drop(&mut self) {
        CURRENT.with(|cell| {
            *cell.borrow_mut() = self.prev.take();
        });
    }
}

/// Read the active call context for this thread, if any.
pub fn current_call_context() -> Option<ObolusCallContext> {
    CURRENT.with(|c| c.borrow().clone())
}

/// Run `f` with `ctx` installed; restores the previous context afterward.
pub fn with_call_context<R>(ctx: ObolusCallContext, f: impl FnOnce() -> R) -> R {
    let _guard = CallContextGuard::new(ctx);
    f()
}
