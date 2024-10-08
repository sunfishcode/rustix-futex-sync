#![doc = include_str!("../README.md")]
#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

// Re-export this so that our users can use the same version we do.
#[cfg(feature = "lock_api")]
pub use lock_api;

// If we don't have the real `lock_api` crate, use our polyfills.
#[cfg(not(feature = "lock_api"))]
pub mod lock_api;

// Export convenient `Mutex` and `RwLock` types.
#[cfg(feature = "lock_api")]
pub type Mutex<T> = lock_api::Mutex<RawMutex, T>;
#[cfg(feature = "lock_api")]
pub type RwLock<T> = lock_api::RwLock<RawRwLock, T>;
#[cfg(feature = "lock_api")]
pub type MutexGuard<'a, T> = lock_api::MutexGuard<'a, RawMutex, T>;
#[cfg(feature = "lock_api")]
pub type MappedMutexGuard<'a, T> = lock_api::MappedMutexGuard<'a, RawMutex, T>;
#[cfg(feature = "lock_api")]
pub type RwLockReadGuard<'a, T> = lock_api::RwLockReadGuard<'a, RawRwLock, T>;
#[cfg(feature = "lock_api")]
pub type RwLockWriteGuard<'a, T> = lock_api::RwLockWriteGuard<'a, RawRwLock, T>;
#[cfg(feature = "lock_api")]
pub type MappedRwLockReadGuard<'a, T> = lock_api::MappedRwLockReadGuard<'a, RawRwLock, T>;
#[cfg(feature = "lock_api")]
pub type MappedRwLockWriteGuard<'a, T> = lock_api::MappedRwLockWriteGuard<'a, RawRwLock, T>;
#[cfg(feature = "lock_api")]
#[cfg(feature = "atomic_usize")]
#[cfg_attr(docsrs, doc(cfg(feature = "atomic_usize")))]
pub type ReentrantMutex<G, T> = lock_api::ReentrantMutex<RawMutex, G, T>;
#[cfg(feature = "lock_api")]
#[cfg(feature = "atomic_usize")]
#[cfg_attr(docsrs, doc(cfg(feature = "atomic_usize")))]
pub type ReentrantMutexGuard<'a, G, T> = lock_api::ReentrantMutexGuard<'a, RawMutex, G, T>;

// Export the once types.
pub use once::{Once, OnceState};
pub use once_lock::OnceLock;

// Export the condvar types.
#[cfg(feature = "lock_api")]
pub use condvar::{Condvar, WaitTimeoutResult};

// Export the raw condvar types.
pub use futex_condvar::Condvar as RawCondvar;

// std's implementation code.
#[cfg(feature = "lock_api")]
mod condvar;
mod futex_condvar;
mod futex_mutex;
mod futex_once;
mod futex_rwlock;
mod once;
mod once_lock;
mod wait_wake;

/// An implementation of [`lock_api::RawMutex`].
///
/// All of this `RawMutex`'s methods are in its implementation of
/// [`lock_api::RawMutex`]. To import that trait without conflicting
/// with this `RawMutex` type, use:
///
/// ```
/// use rustix_futex_sync::lock_api::RawMutex as _;
/// ```
#[repr(transparent)]
pub struct RawMutex(futex_mutex::Mutex);

/// An implementation of [`lock_api::RawRwLock`].
///
/// All of this `RawRwLock`'s methods are in its implementation of
/// [`lock_api::RawRwLock`]. To import that trait without conflicting
/// with this `RawRwLock` type, use:
///
/// ```
/// use rustix_futex_sync::lock_api::RawRwLock as _;
/// ```
#[repr(C)]
pub struct RawRwLock(futex_rwlock::RwLock);

// Implement the raw lock traits for our wrappers.

unsafe impl lock_api::RawMutex for RawMutex {
    type GuardMarker = lock_api::GuardNoSend;

    const INIT: Self = Self(futex_mutex::Mutex::new());

    #[inline]
    fn lock(&self) {
        self.0.lock()
    }

    #[inline]
    fn try_lock(&self) -> bool {
        self.0.try_lock()
    }

    #[inline]
    unsafe fn unlock(&self) {
        self.0.unlock()
    }
}

unsafe impl lock_api::RawRwLock for RawRwLock {
    type GuardMarker = lock_api::GuardNoSend;

    const INIT: Self = Self(futex_rwlock::RwLock::new());

    #[inline]
    fn lock_shared(&self) {
        self.0.read()
    }

    #[inline]
    fn try_lock_shared(&self) -> bool {
        self.0.try_read()
    }

    #[inline]
    unsafe fn unlock_shared(&self) {
        self.0.read_unlock()
    }

    #[inline]
    fn lock_exclusive(&self) {
        self.0.write()
    }

    #[inline]
    fn try_lock_exclusive(&self) -> bool {
        self.0.try_write()
    }

    #[inline]
    unsafe fn unlock_exclusive(&self) {
        self.0.write_unlock()
    }
}
