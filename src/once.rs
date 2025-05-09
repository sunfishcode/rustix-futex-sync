//! The following is derived from Rust's
//! library/std/src/sync/once.rs at revision
//! 22a5267c83a3e17f2b763279eb24bb632c45dc6b.
//!
//! A "once initialization" primitive
//!
//! This primitive is meant to be used to run one-time initialization. An
//! example use case would be for initializing an FFI library.

/*
#[cfg(all(test, not(target_os = "emscripten")))]
mod tests;
*/

use core::fmt;
use core::panic::{RefUnwindSafe, UnwindSafe};
use super::futex_once as sys;

/// A synchronization primitive which can be used to run a one-time global
/// initialization. Useful for one-time initialization for FFI or related
/// functionality. This type can only be constructed with [`Once::new()`].
///
/// # Examples
///
/// ```
/// use rustix_futex_sync::Once;
///
/// static START: Once = Once::new();
///
/// START.call_once(|| {
///     // run initialization here
/// });
/// ```
//#[stable(feature = "rust1", since = "1.0.0")]
#[repr(transparent)]
pub struct Once<const SHM: bool> {
    inner: sys::Once<SHM>,
}

//#[stable(feature = "sync_once_unwind_safe", since = "1.59.0")]
impl<const SHM: bool> UnwindSafe for Once<SHM> {}

//#[stable(feature = "sync_once_unwind_safe", since = "1.59.0")]
impl<const SHM: bool> RefUnwindSafe for Once<SHM> {}

/// State yielded to [`Once::call_once_force()`]’s closure parameter. The state
/// can be used to query the poison status of the [`Once`].
//#[stable(feature = "once_poison", since = "1.51.0")]
pub struct OnceState {
    pub(crate) inner: sys::OnceState,
}

/*
pub(crate) enum ExclusiveState {
    Incomplete,
    Poisoned,
    Complete,
}
*/

/*
/// Initialization value for static [`Once`] values.
///
/// # Examples
///
/// ```
/// use rustix_futex_sync::{Once, ONCE_INIT};
///
/// static START: Once = ONCE_INIT;
/// ```
//#[stable(feature = "rust1", since = "1.0.0")]
#[deprecated(
    since = "1.38.0",
    note = "the `new` function is now preferred",
    suggestion = "Once::new()"
)]
pub const ONCE_INIT: Once = Once::new();
*/

impl<const SHM: bool> Once<SHM> {
    /// Creates a new `Once` value.
    #[inline]
    //#[stable(feature = "once_new", since = "1.2.0")]
    //#[rustc_const_stable(feature = "const_once_new", since = "1.32.0")]
    #[must_use]
    pub const fn new() -> Self {
        Once { inner: sys::Once::new() }
    }

    /// Performs an initialization routine once and only once. The given closure
    /// will be executed if this is the first time `call_once` has been called,
    /// and otherwise the routine will *not* be invoked.
    ///
    /// This method will block the calling thread if another initialization
    /// routine is currently running.
    ///
    /// When this function returns, it is guaranteed that some initialization
    /// has run and completed (it might not be the closure specified). It is also
    /// guaranteed that any memory writes performed by the executed closure can
    /// be reliably observed by other threads at this point (there is a
    /// happens-before relation between the closure and code executing after the
    /// return).
    ///
    /// If the given closure recursively invokes `call_once` on the same [`Once`]
    /// instance, the exact behavior is not specified: allowed outcomes are
    /// a panic or a deadlock.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustix_futex_sync::Once;
    ///
    /// static mut VAL: usize = 0;
    /// static INIT: Once = Once::new();
    ///
    /// // Accessing a `static mut` is unsafe much of the time, but if we do so
    /// // in a synchronized fashion (e.g., write once or read all) then we're
    /// // good to go!
    /// //
    /// // This function will only call `expensive_computation` once, and will
    /// // otherwise always return the value returned from the first invocation.
    /// fn get_cached_val() -> usize {
    ///     unsafe {
    ///         INIT.call_once(|| {
    ///             VAL = expensive_computation();
    ///         });
    ///         VAL
    ///     }
    /// }
    ///
    /// fn expensive_computation() -> usize {
    ///     // ...
    /// # 2
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// The closure `f` will only be executed once even if this is called
    /// concurrently amongst many threads. If that closure panics, however, then
    /// it will *poison* this [`Once`] instance, causing all future invocations of
    /// `call_once` to also panic.
    ///
    /// This is similar to [poisoning with mutexes][poison].
    ///
    /// [poison]: struct.Mutex.html#poisoning
    #[inline]
    //#[stable(feature = "rust1", since = "1.0.0")]
    #[track_caller]
    pub fn call_once<F>(&self, f: F)
    where
        F: FnOnce(),
    {
        // Fast path check
        if self.inner.is_completed() {
            return;
        }

        let mut f = Some(f);
        self.inner.call(false, &mut |_| f.take().unwrap()());
    }

    /// Performs the same function as [`call_once()`] except ignores poisoning.
    ///
    /// Unlike [`call_once()`], if this [`Once`] has been poisoned (i.e., a previous
    /// call to [`call_once()`] or [`call_once_force()`] caused a panic), calling
    /// [`call_once_force()`] will still invoke the closure `f` and will _not_
    /// result in an immediate panic. If `f` panics, the [`Once`] will remain
    /// in a poison state. If `f` does _not_ panic, the [`Once`] will no
    /// longer be in a poison state and all future calls to [`call_once()`] or
    /// [`call_once_force()`] will be no-ops.
    ///
    /// The closure `f` is yielded a [`OnceState`] structure which can be used
    /// to query the poison status of the [`Once`].
    ///
    /// [`call_once()`]: Once::call_once
    /// [`call_once_force()`]: Once::call_once_force
    ///
    /// # Examples
    ///
    /// ```
    /// use rustix_futex_sync::Once;
    /// use std::thread;
    ///
    /// static INIT: Once = Once::new();
    ///
    /// // poison the once
    /// let handle = thread::spawn(|| {
    ///     INIT.call_once(|| panic!());
    /// });
    /// assert!(handle.join().is_err());
    ///
    /// /*
    /// // poisoning propagates
    /// let handle = thread::spawn(|| {
    ///     INIT.call_once(|| {});
    /// });
    /// assert!(handle.join().is_err());
    /// */
    ///
    /// // call_once_force will still run and reset the poisoned state
    /// INIT.call_once_force(|state| {
    ///     //assert!(state.is_poisoned());
    /// });
    ///
    /// // once any success happens, we stop propagating the poison
    /// INIT.call_once(|| {});
    /// ```
    #[inline]
    //#[stable(feature = "once_poison", since = "1.51.0")]
    pub fn call_once_force<F>(&self, f: F)
    where
        F: FnOnce(&OnceState),
    {
        // Fast path check
        if self.inner.is_completed() {
            return;
        }

        let mut f = Some(f);
        self.inner.call(true, &mut |p| f.take().unwrap()(p));
    }

    /// Returns `true` if some [`call_once()`] call has completed
    /// successfully. Specifically, `is_completed` will return false in
    /// the following situations:
    ///   * [`call_once()`] was not called at all,
    ///   * [`call_once()`] was called, but has not yet completed,
    ///   * the [`Once`] instance is poisoned
    ///
    /// This function returning `false` does not mean that [`Once`] has not been
    /// executed. For example, it may have been executed in the time between
    /// when `is_completed` starts executing and when it returns, in which case
    /// the `false` return value would be stale (but still permissible).
    ///
    /// [`call_once()`]: Once::call_once
    ///
    /// # Examples
    ///
    /// ```
    /// use rustix_futex_sync::Once;
    ///
    /// static INIT: Once = Once::new();
    ///
    /// assert_eq!(INIT.is_completed(), false);
    /// INIT.call_once(|| {
    ///     assert_eq!(INIT.is_completed(), false);
    /// });
    /// assert_eq!(INIT.is_completed(), true);
    /// ```
    ///
    /// ```
    /// use rustix_futex_sync::Once;
    /// use std::thread;
    ///
    /// static INIT: Once = Once::new();
    ///
    /// assert_eq!(INIT.is_completed(), false);
    /// let handle = thread::spawn(|| {
    ///     INIT.call_once(|| panic!());
    /// });
    /// assert!(handle.join().is_err());
    /// assert_eq!(INIT.is_completed(), false);
    /// ```
    //#[stable(feature = "once_is_completed", since = "1.43.0")]
    #[inline]
    pub fn is_completed(&self) -> bool {
        self.inner.is_completed()
    }

    /*
    /// Returns the current state of the `Once` instance.
    ///
    /// Since this takes a mutable reference, no initialization can currently
    /// be running, so the state must be either "incomplete", "poisoned" or
    /// "complete".
    #[inline]
    pub(crate) fn state(&mut self) -> ExclusiveState {
        self.inner.state()
    }
    */
}

//#[stable(feature = "std_debug", since = "1.16.0")]
impl<const SHM: bool> fmt::Debug for Once<SHM> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Once").finish_non_exhaustive()
    }
}

impl OnceState {
/*
    /// Returns `true` if the associated [`Once`] was poisoned prior to the
    /// invocation of the closure passed to [`Once::call_once_force()`].
    ///
    /// # Examples
    ///
    /// A poisoned [`Once`]:
    ///
    /// ```
    /// use rustix_futex_sync::Once;
    /// use std::thread;
    ///
    /// static INIT: Once = Once::new();
    ///
    /// // poison the once
    /// let handle = thread::spawn(|| {
    ///     INIT.call_once(|| panic!());
    /// });
    /// assert!(handle.join().is_err());
    ///
    /// INIT.call_once_force(|state| {
    ///     assert!(state.is_poisoned());
    /// });
    /// ```
    ///
    /// An unpoisoned [`Once`]:
    ///
    /// ```
    /// use rustix_futex_sync::Once;
    ///
    /// static INIT: Once = Once::new();
    ///
    /// INIT.call_once_force(|state| {
    ///     assert!(!state.is_poisoned());
    /// });
    //#[stable(feature = "once_poison", since = "1.51.0")]
    #[inline]
    pub fn is_poisoned(&self) -> bool {
        self.inner.is_poisoned()
    }

    /// Poison the associated [`Once`] without explicitly panicking.
    // NOTE: This is currently only exposed for `OnceLock`.
    #[inline]
    pub(crate) fn poison(&self) {
        self.inner.poison();
    }
*/

    /// Mark the associated [`Once`] as incomplete.
    // NOTE: This is currently only exposed for `OnceLock`.
    #[inline]
    pub(crate) fn set_incomplete(&self) {
        self.inner.set_incomplete();
    }
}

//#[stable(feature = "std_debug", since = "1.16.0")]
impl fmt::Debug for OnceState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OnceState")
            /* .field("poisoned", &self.is_poisoned()) */.finish()
    }
}
