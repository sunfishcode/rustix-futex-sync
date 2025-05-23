//! The following is derived from Rust's
//! library/std/src/sys/sync/once/futex.rs at revision
//! 22a5267c83a3e17f2b763279eb24bb632c45dc6b.

use core::cell::Cell;
use crate as public;
use core::sync::atomic::{
    AtomicU32,
    Ordering::{Acquire, Relaxed, Release},
};
//use super::once::ExclusiveState;
use super::wait_wake::{futex_wait, futex_wake_all};

// On some platforms, the OS is very nice and handles the waiter queue for us.
// This means we only need one atomic value with 5 states:

/// No initialization has run yet, and no thread is currently using the Once.
const INCOMPLETE: u32 = 0;
/// Some thread has previously attempted to initialize the Once, but it panicked,
/// so the Once is now poisoned. There are no other threads currently accessing
/// this Once.
const POISONED: u32 = 1;
/// Some thread is currently attempting to run initialization. It may succeed,
/// so all future threads need to wait for it to finish.
const RUNNING: u32 = 2;
/// Some thread is currently attempting to run initialization and there are threads
/// waiting for it to finish.
const QUEUED: u32 = 3;
/// Initialization has completed and all future calls should finish immediately.
const COMPLETE: u32 = 4;

// Threads wait by setting the state to QUEUED and calling `futex_wait` on the state
// variable. When the running thread finishes, it will wake all waiting threads using
// `futex_wake_all`.

pub struct OnceState {
    //poisoned: bool,
    set_state_to: Cell<u32>,
}

impl OnceState {
/*
    #[inline]
    pub fn is_poisoned(&self) -> bool {
        self.poisoned
    }

    #[inline]
    pub fn poison(&self) {
        self.set_state_to.set(POISONED);
    }
*/

    #[inline]
    pub fn set_incomplete(&self) {
        self.set_state_to.set(INCOMPLETE);
    }
}

struct CompletionGuard<'a, const SHM: bool> {
    state: &'a AtomicU32,
    set_state_on_drop_to: u32,
}

impl<'a, const SHM: bool> Drop for CompletionGuard<'a, SHM> {
    fn drop(&mut self) {
        // Use release ordering to propagate changes to all threads checking
        // up on the Once. `futex_wake_all` does its own synchronization, hence
        // we do not need `AcqRel`.
        if self.state.swap(self.set_state_on_drop_to, Release) == QUEUED {
            futex_wake_all::<SHM>(&self.state);
        }
    }
}

#[repr(transparent)]
pub struct Once<const SHM: bool> {
    state: AtomicU32,
}

impl<const SHM: bool> Once<SHM> {
    #[inline]
    pub const fn new() -> Once<SHM> {
        Once { state: AtomicU32::new(INCOMPLETE) }
    }

    #[inline]
    pub fn is_completed(&self) -> bool {
        // Use acquire ordering to make all initialization changes visible to the
        // current thread.
        self.state.load(Acquire) == COMPLETE
    }

    /*
    #[inline]
    pub(crate) fn state(&mut self) -> ExclusiveState {
        match *self.state.get_mut() {
            INCOMPLETE => ExclusiveState::Incomplete,
            POISONED => ExclusiveState::Poisoned,
            COMPLETE => ExclusiveState::Complete,
            _ => unreachable!("invalid Once state"),
        }
    }
    */

    // This uses FnMut to match the API of the generic implementation. As this
    // implementation is quite light-weight, it is generic over the closure and
    // so avoids the cost of dynamic dispatch.
    #[cold]
    #[track_caller]
    pub fn call(&self, ignore_poisoning: bool, f: &mut impl FnMut(&public::OnceState)) {
        let mut state = self.state.load(Acquire);
        loop {
            match state {
                POISONED if !ignore_poisoning => {
                    // Panic to propagate the poison.
                    panic!("Once instance has previously been poisoned");
                }
                INCOMPLETE | POISONED => {
                    // Try to register the current thread as the one running.
                    if let Err(new) =
                        self.state.compare_exchange_weak(state, RUNNING, Acquire, Acquire)
                    {
                        state = new;
                        continue;
                    }
                    // `waiter_queue` will manage other waiting threads, and
                    // wake them up on drop.
                    let mut waiter_queue =
                        //CompletionGuard { state: &self.state, set_state_on_drop_to: POISONED };
                        CompletionGuard::<SHM> { state: &self.state, set_state_on_drop_to: INCOMPLETE };
                    // Run the function, letting it know if we're poisoned or not.
                    let f_state = public::OnceState {
                        inner: OnceState {
                            //poisoned: state == POISONED,
                            set_state_to: Cell::new(COMPLETE),
                        },
                    };
                    f(&f_state);
                    waiter_queue.set_state_on_drop_to = f_state.inner.set_state_to.get();
                    return;
                }
                RUNNING | QUEUED => {
                    // Set the state to QUEUED if it is not already.
                    if state == RUNNING {
                        if let Err(new) = self.state.compare_exchange_weak(RUNNING, QUEUED, Relaxed, Acquire) {
                            state = new;
                            continue;
                        }
                    }

                    futex_wait::<SHM>(&self.state, QUEUED, None);
                    state = self.state.load(Acquire);
                }
                COMPLETE => return,
                _ => unreachable!("state is never set to invalid values"),
            }
        }
    }
}
