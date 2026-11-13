//! The control of a run (Engine and Project Specification, section 8.3, point 7: "Long
//! operations report progress and can be cancelled between steps"). A run that is given a
//! [`Control`] says where it stands as it goes, and looks at it between its steps: before each
//! statement of the script and before each pass of a loop. When the run has been cancelled it
//! stops there with Z0705 (Language Reference, section 11); what ran before stays.
//!
//! The handle can be cloned and sent to another thread, which is how the interface cancels a
//! run from outside (the engine worker, step 33b).

use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Where a run stands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// A statement of the script is about to run: which, counted from 1, of how many, and the
    /// line it begins on.
    Started {
        statement: usize,
        of: usize,
        line: u32,
    },
    /// The statement ran, or stopped with an error.
    Finished {
        statement: usize,
        of: usize,
        line: u32,
        failed: bool,
    },
    /// A pass of a loop (`for`, `[for ...]` or `iterate`) is about to run: which, counted from
    /// 1, of how many where the loop knows, and the line of the script's statement the loop
    /// runs under.
    Pass {
        pass: usize,
        of: Option<usize>,
        line: u32,
    },
}

/// What a listener is given: each event as it happens, on the thread of the run.
pub type Listener = Arc<dyn Fn(&Event) + Send + Sync>;

/// The handle of a run: its listener, and the flag by which it is cancelled. A clone is the
/// same run's handle.
#[derive(Clone, Default)]
pub struct Control {
    cancelled: Arc<AtomicBool>,
    listener: Option<Listener>,
}

impl Control {
    /// A handle with no listener: the run can be cancelled, and reports to no one.
    pub fn new() -> Control {
        Control::default()
    }

    /// A handle whose listener is told each event.
    pub fn with_listener(listener: impl Fn(&Event) + Send + Sync + 'static) -> Control {
        Control {
            cancelled: Arc::default(),
            listener: Some(Arc::new(listener)),
        }
    }

    /// Cancel the run: it stops before its next step.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    pub(crate) fn report(&self, event: Event) {
        if let Some(listener) = &self.listener {
            listener(&event);
        }
    }
}

impl fmt::Debug for Control {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Control")
            .field("cancelled", &self.is_cancelled())
            .field("listener", &self.listener.is_some())
            .finish()
    }
}
