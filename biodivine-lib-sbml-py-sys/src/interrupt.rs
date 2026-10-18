//! Long operations that Python signal handlers can interrupt.
//!
//! This is the XML DOM layer's implementation, unchanged: the SBML library reads its cancellation
//! triggers through the same `cancel-this` crate, so the argument (one worker thread on the main
//! thread only, signals checked between waits, a handler never runs with a document lock held)
//! carries over as it is.
//!
//! A Python signal handler (the one behind Ctrl+C, or one installed with `signal.signal`) only runs
//! when the main thread calls into the interpreter. A long Rust operation that released the GIL
//! would therefore postpone every signal until it returns. The usual fix — consulting
//! `Python::check_signals` from inside the operation — runs the handlers in the middle of the
//! operation, while the document lock is held, so a handler that touches the same document
//! deadlocks.
//!
//! [`run_interruptible`] keeps the two apart instead. On the main thread, a long operation runs on
//! a scoped worker thread under a [`CancelAtomic`] trigger, while the calling thread waits without
//! the GIL and checks for signals between waits. A handler therefore always runs on the calling
//! thread, which holds no document lock; if it raises, the trigger is cancelled, the worker stops
//! at its next cancellation check, and the handler's own exception is propagated unchanged.
//!
//! Short operations, and operations on threads other than the main thread (which never run signal
//! handlers), simply run with the GIL released.

use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::Duration;

use biodivine_lib_sbml::SbmlResult;
use cancel_this::{self, CancelAtomic};
use pyo3::prelude::*;

use crate::error::to_py_err;

/// Below this size (in input bytes or document nodes) an operation is over before a signal could
/// usefully interrupt it, so it is not worth a worker thread.
const INTERRUPTIBLE_SIZE: usize = 100_000;

/// How long the calling thread waits for the worker before it checks for signals again.
const SIGNAL_CHECK_INTERVAL: Duration = Duration::from_millis(20);

/// Runs `work` with the GIL released, letting Python signal handlers interrupt it.
///
/// `size` is a rough measure of the work (input bytes, or nodes of the document), used only to
/// decide whether the operation is long enough to run on a worker thread.
///
/// # Errors
///
/// Returns the exception a signal handler raised, or the translation of the error `work`
/// returned.
///
/// # Panics
///
/// A panic of `work` is resumed on the calling thread, where PyO3 turns it into a Python
/// `PanicException`.
pub(crate) fn run_interruptible<T, F>(py: Python<'_>, size: usize, work: F) -> PyResult<T>
where
    T: Send,
    F: FnOnce() -> SbmlResult<T> + Send,
{
    if size < INTERRUPTIBLE_SIZE || !is_main_thread(py)? {
        return py.detach(work).map_err(to_py_err);
    }

    let trigger = CancelAtomic::new();
    let worker_trigger = trigger.clone();
    std::thread::scope(|scope| {
        let (finished, receiver) = mpsc::channel::<()>();
        let worker = scope.spawn(move || {
            let result = cancel_this::on_atomic(worker_trigger, work);
            // The calling thread waits on the receiver for as long as the worker runs, so the
            // message can only be lost if the receiver is gone, i.e. if nobody waits for it.
            let _ = finished.send(());
            result
        });

        let mut receiver = receiver;
        loop {
            let (returned, done) = py.detach(move || {
                let done = !matches!(
                    receiver.recv_timeout(SIGNAL_CHECK_INTERVAL),
                    Err(RecvTimeoutError::Timeout)
                );
                (receiver, done)
            });
            receiver = returned;
            if done {
                break;
            }
            if let Err(error) = py.check_signals() {
                trigger.cancel();
                // The worker stops at its next cancellation check; whatever it returns is moot.
                let _ = py.detach(move || worker.join());
                return Err(error);
            }
        }

        match py.detach(move || worker.join()) {
            Ok(result) => result.map_err(to_py_err),
            Err(panic) => std::panic::resume_unwind(panic),
        }
    })
}

/// Whether the current thread is the main thread of the interpreter, the only one on which
/// signal handlers run.
fn is_main_thread(py: Python<'_>) -> PyResult<bool> {
    let threading = py.import("threading")?;
    let current = threading.call_method0("current_thread")?;
    let main = threading.call_method0("main_thread")?;
    Ok(current.is(&main))
}
