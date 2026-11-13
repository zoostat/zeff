//! The engine worker, natively (Engine and Project Specification, section 8.3, point 7: "The
//! engine runs in its own worker (a Web Worker in the browser, a thread natively), so the
//! interface never freezes"). A run is started on a thread of its own; the one who started it
//! is told what the run prints and where it stands as it goes, and can cancel it. The browser
//! has no threads of this kind: there the same run is hosted by a Web Worker (step 33e).
//!
//! A session of the console or of an interface is a script that grows, run again from its
//! file (`crate::session`), so a worker that runs scripts serves it too.

use crate::control::{Control, Event};
use crate::fs::FileSystem;
use crate::project::Project;
use crate::run::{Outcome, RunOptions, run_script_under};
use std::sync::mpsc::{Receiver, channel};
use std::thread::JoinHandle;

/// The stack of the thread a run is on: that of `zeff run`, so that a script nested as deep as
/// the parser allows runs here as it does there.
pub const STACK_SIZE: usize = 64 << 20;

/// What a run sends to the one who started it.
#[derive(Debug)]
pub enum Message {
    /// A step of the run, or a piece of what it printed and said.
    Event(Event),
    /// The run has ended: its outcome, whole. Nothing follows.
    Done(Box<Outcome>),
}

/// A run on a thread of its own.
#[derive(Debug)]
pub struct Run {
    control: Control,
    messages: Receiver<Message>,
    thread: Option<JoinHandle<()>>,
}

impl Run {
    /// The handle of the run, to cancel it from anywhere.
    pub fn control(&self) -> Control {
        self.control.clone()
    }

    /// Cancel the run: it stops before its next step with Z0705, and its outcome follows.
    pub fn cancel(&self) {
        self.control.cancel();
    }

    /// The next message, waited for; `None` once the run has ended and all has been read.
    pub fn next(&self) -> Option<Message> {
        self.messages.recv().ok()
    }

    /// The next message if one is there, without waiting: what an interface asks between two
    /// frames.
    pub fn try_next(&self) -> Option<Message> {
        self.messages.try_recv().ok()
    }

    /// Wait for the run to end and give its outcome; the messages not read are passed over.
    pub fn wait(mut self) -> Outcome {
        let mut outcome = None;
        while let Ok(message) = self.messages.recv() {
            if let Message::Done(done) = message {
                outcome = Some(*done);
            }
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
        // A thread that ended without an outcome failed outside any statement, where the
        // interpreter's own guard does not reach.
        outcome.unwrap_or_else(lost)
    }
}

/// The outcome of a run whose thread was lost.
fn lost() -> Outcome {
    let mut outcome = Outcome {
        output: crate::interpreter::Output::default(),
        exit_code: 2,
    };
    outcome.output.stderr.push_str(
        "error[Z9001]: internal error: this is a bug in Zeff, not in your script (the engine worker stopped without an answer)\n",
    );
    outcome
}

/// Start a run of the text of `script` (its name in the project) on a thread of its own. The
/// file system goes to that thread with it.
pub fn start<F>(
    fs: F,
    project: Project,
    script: impl Into<String>,
    text: impl Into<String>,
    options: RunOptions,
) -> std::io::Result<Run>
where
    F: FileSystem + Send + 'static,
{
    let (script, text) = (script.into(), text.into());
    let control = Control::new();
    let held = control.clone();
    let (sender, messages) = channel();
    let thread = std::thread::Builder::new()
        .name("zeff-engine".to_string())
        .stack_size(STACK_SIZE)
        .spawn(move || {
            let tell = sender.clone();
            let listener = move |event: &Event| {
                // The one who started the run may have gone: the run goes on to its end.
                let _ = tell.send(Message::Event(event.clone()));
            };
            let outcome = run_script_under(
                &fs,
                &project,
                &script,
                &text,
                &options,
                &held,
                Some(&listener),
            );
            let _ = sender.send(Message::Done(Box::new(outcome)));
        })?;
    Ok(Run {
        control,
        messages,
        thread: Some(thread),
    })
}
