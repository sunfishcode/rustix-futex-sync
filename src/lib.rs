#![doc = include_str!("../README.md")]
#![no_std]
#![cfg_attr(doc_cfg, feature(doc_cfg))]

// Re-export this so that our users can use the same version we do.
pub use lock_api;

pub type RawCondvar = condvar::MovableCondvar;

#[repr(transparent)]
pub struct Condvar(RawCondvar);

impl Condvar {
    #[inline]
    pub const fn new() -> Self {
        Self(condvar::MovableCondvar::new())
    }

    #[inline]
    pub fn wait<'a, T>(&self, guard: MutexGuard<'a, T>) -> MutexGuard<'a, T> {
        unsafe {
            self.0.wait(MutexGuard::mutex(&guard).raw());
        }
        guard
    }

    #[inline]
    pub fn wait_timeout<'a, T>(
        &self,
        guard: MutexGuard<'a, T>,
        dur: core::time::Duration,
    ) -> (MutexGuard<'a, T>, bool) {
        let result = unsafe { self.0.wait_timeout(MutexGuard::mutex(&guard).raw(), dur) };
        (guard, result)
    }

    #[inline]
    pub fn notify_one(&self) {
        self.0.notify_one()
    }

    #[inline]
    pub fn notify_all(&self) {
        self.0.notify_all()
    }
}

// The following is derived from Rust's
// library/std/src/sys/unix/locks/mod.rs at revision
// 6fd7e9010db6be7605241c39eab7c5078ee2d5bd.

// Export convenient `Mutex` and `RwLock` types.
pub type Mutex<T> = lock_api::Mutex<RawMutex, T>;
pub type RwLock<T> = lock_api::RwLock<RawRwLock, T>;
pub type MutexGuard<'a, T> = lock_api::MutexGuard<'a, RawMutex, T>;
pub type RwLockReadGuard<'a, T> = lock_api::RwLockReadGuard<'a, RawRwLock, T>;
pub type RwLockWriteGuard<'a, T> = lock_api::RwLockWriteGuard<'a, RawRwLock, T>;
#[cfg(feature = "atomic_usize")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "atomic_usize")))]
pub type ReentrantMutex<G, T> = lock_api::ReentrantMutex<RawMutex, G, T>;
#[cfg(feature = "atomic_usize")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "atomic_usize")))]
pub type ReentrantMutexGuard<'a, G, T> = lock_api::ReentrantMutexGuard<'a, RawMutex, G, T>;

// std's implementation code.
mod condvar;
mod futex;
mod futex_rwlock;
mod wait_wake;

// Use the raw lock types from std's implementation.
use futex::MovableMutex;
use futex_rwlock::MovableRwLock;

// Encapsulate the std lock types to hide this detail.
#[repr(transparent)]
pub struct RawMutex(MovableMutex);
#[repr(transparent)]
pub struct RawRwLock(MovableRwLock);

// Implement the raw lock traits for our wrappers.

unsafe impl lock_api::RawMutex for RawMutex {
    type GuardMarker = lock_api::GuardNoSend;

    const INIT: Self = Self(MovableMutex::new());

    fn lock(&self) {
        self.0.lock()
    }

    fn try_lock(&self) -> bool {
        self.0.try_lock()
    }

    unsafe fn unlock(&self) {
        self.0.unlock()
    }
}

unsafe impl lock_api::RawRwLock for RawRwLock {
    type GuardMarker = lock_api::GuardNoSend;

    const INIT: Self = Self(MovableRwLock::new());

    fn lock_shared(&self) {
        self.0.read()
    }

    fn try_lock_shared(&self) -> bool {
        self.0.try_read()
    }

    unsafe fn unlock_shared(&self) {
        self.0.read_unlock()
    }

    fn lock_exclusive(&self) {
        self.0.write()
    }

    fn try_lock_exclusive(&self) -> bool {
        self.0.try_write()
    }

    unsafe fn unlock_exclusive(&self) {
        self.0.write_unlock()
    }
}
