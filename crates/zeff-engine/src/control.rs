//! The control of a run (Engine and Project Specification, section 8.3, point 7: "Long
//! operations report progress and can be cancelled between steps"). A run that is given a
//! [`Control`] says where it stands as it goes, and looks at it between its steps: before each
//! statement of the script and before each pass of a loop. When the run has been cancelled it
//! stops there with Z0705 (Language Reference, section 11); what ran before stays.
//!
//! The handle can be cloned and sent to another thread, which is how a run is cancelled from
//! outside: by the engine worker (`crate::worker`) for an interface, and by the interrupt of
//! `zeff run`.

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
    /// What a statement printed and said, when it did either, before its `Finished`; and,
    /// before the first statement, what the check of the script said. The pieces of a run, in
    /// order, are the whole of its standard output and of its messages.
    Output { stdout: String, stderr: String },
    /// A pass of a loop (`for`, `[for ...]` or `iterate`) is about to run: which, counted from
    /// 1, of how many where the loop knows, and the line of the script's statement the loop
    /// runs under. A long loop does not tell every pass: about a hundred of them, the first
    /// and the last among them, so that a listener is not flooded.
    Pass {
        pass: usize,
        of: Option<usize>,
        line: u32,
    },
}

/// What a run tells as it goes: each event as it happens, on the thread of the run. It need
/// not be sent to another thread, so it may write to what the caller holds.
pub type Listener<'a> = &'a dyn Fn(&Event);

/// The handle by which a run is cancelled. A clone is the same run's handle, and may be held
/// by another thread.
#[derive(Clone, Default)]
pub struct Control {
    cancelled: Arc<AtomicBool>,
    /// Asked as well, where the flag lives outside the engine: in the browser the interface
    /// sets it in memory it shares with the worker, since a worker that is computing reads no
    /// message (step 33e1).
    asked: Option<fn() -> bool>,
}

impl Control {
    pub fn new() -> Control {
        Control::default()
    }

    /// A handle that is also cancelled when `asked` says so.
    pub fn asking(asked: fn() -> bool) -> Control {
        Control {
            cancelled: Arc::default(),
            asked: Some(asked),
        }
    }

    /// Cancel the run: it stops before its next step.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst) || self.asked.is_some_and(|asked| asked())
    }
}

impl fmt::Debug for Control {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Control")
            .field("cancelled", &self.is_cancelled())
            .finish()
    }
}
