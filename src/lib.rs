#![doc = include_str!("../README.md")]
#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

// Re-export this so that our users can use the same version we do.
#[cfg(feature = "lock_api")]
pub use lock_api;

// If we don't have the real `lock_api` crate, use our polyfills.
#[cfg(not(feature = "lock_api"))]
pub mod lock_api;

#[cfg(feature = "lock_api")]
pub use condvar::WaitTimeoutResult;
pub use once::OnceState;

// Non-shared API.

pub type Once = generic::Once<false>;
#[cfg(feature = "lock_api")]
pub type Condvar = generic::Condvar<false>;
pub type RawCondvar = generic::RawCondvar<false>;
pub type RawMutex = generic::RawMutex<false>;
pub type RawRwLock = generic::RawRwLock<false>;
pub type OnceLock<T> = generic::OnceLock<T, false>;
#[cfg(feature = "lock_api")]
pub type Mutex<T> = generic::Mutex<T, false>;
#[cfg(feature = "lock_api")]
pub type RwLock<T> = generic::RwLock<T, false>;
#[cfg(feature = "lock_api")]
pub type MutexGuard<'a, T> = generic::MutexGuard<'a, T, false>;
#[cfg(feature = "lock_api")]
pub type MappedMutexGuard<'a, T> = generic::MappedMutexGuard<'a, T, false>;
#[cfg(feature = "lock_api")]
pub type RwLockReadGuard<'a, T> = generic::RwLockReadGuard<'a, T, false>;
#[cfg(feature = "lock_api")]
pub type RwLockWriteGuard<'a, T> = generic::RwLockWriteGuard<'a, T, false>;
#[cfg(feature = "lock_api")]
pub type MappedRwLockReadGuard<'a, T> = generic::MappedRwLockReadGuard<'a, T, false>;
#[cfg(feature = "lock_api")]
pub type MappedRwLockWriteGuard<'a, T> = generic::MappedRwLockWriteGuard<'a, T, false>;
#[cfg(feature = "lock_api")]
#[cfg(feature = "atomic_usize")]
#[cfg_attr(docsrs, doc(cfg(feature = "atomic_usize")))]
pub type ReentrantMutex<G, T> = generic::ReentrantMutex<G, T, false>;
#[cfg(feature = "lock_api")]
#[cfg(feature = "atomic_usize")]
#[cfg_attr(docsrs, doc(cfg(feature = "atomic_usize")))]
pub type ReentrantMutexGuard<'a, G, T> = generic::ReentrantMutexGuard<'a, G, T, false>;

/// Shared-memory API.
///
/// The types in this module behave the same as the types defined at the top
/// level of this crate, except that they don't set the `FUTEX_PRIVATE_FLAG`
/// flag, so they can be used on memory shared with other processes.
///
/// See [the Linux documentation] for more information about
/// `FUTEX_PRIVATE_FLAG`.
///
/// [the Linux documentation]: https://man7.org/linux/man-pages/man2/futex.2.html
#[cfg(feature = "shm")]
#[cfg_attr(docsrs, doc(cfg(feature = "shm")))]
pub mod shm {
    use crate::generic;

    pub type Once = generic::Once<true>;
    #[cfg(feature = "lock_api")]
    pub type Condvar = generic::Condvar<true>;
    pub type RawCondvar = generic::RawCondvar<true>;
    pub type RawMutex = generic::RawMutex<true>;
    pub type RawRwLock = generic::RawRwLock<true>;
    pub type OnceLock<T> = generic::OnceLock<T, true>;
    #[cfg(feature = "lock_api")]
    pub type Mutex<T> = generic::Mutex<T, true>;
    #[cfg(feature = "lock_api")]
    pub type RwLock<T> = generic::RwLock<T, true>;
    #[cfg(feature = "lock_api")]
    pub type MutexGuard<'a, T> = generic::MutexGuard<'a, T, true>;
    #[cfg(feature = "lock_api")]
    pub type MappedMutexGuard<'a, T> = generic::MappedMutexGuard<'a, T, true>;
    #[cfg(feature = "lock_api")]
    pub type RwLockReadGuard<'a, T> = generic::RwLockReadGuard<'a, T, true>;
    #[cfg(feature = "lock_api")]
    pub type RwLockWriteGuard<'a, T> = generic::RwLockWriteGuard<'a, T, true>;
    #[cfg(feature = "lock_api")]
    pub type MappedRwLockReadGuard<'a, T> = generic::MappedRwLockReadGuard<'a, T, true>;
    #[cfg(feature = "lock_api")]
    pub type MappedRwLockWriteGuard<'a, T> = generic::MappedRwLockWriteGuard<'a, T, true>;
    #[cfg(feature = "lock_api")]
    #[cfg(feature = "atomic_usize")]
    #[cfg_attr(docsrs, doc(cfg(feature = "atomic_usize")))]
    pub type ReentrantMutex<G, T> = generic::ReentrantMutex<G, T, true>;
    #[cfg(feature = "lock_api")]
    #[cfg(feature = "atomic_usize")]
    #[cfg_attr(docsrs, doc(cfg(feature = "atomic_usize")))]
    pub type ReentrantMutexGuard<'a, G, T> = generic::ReentrantMutexGuard<'a, G, T, true>;
}

/// Types and traits with a `const SHM: bool>` generic paramters.
///
/// These are the generic types that are parameterized on whether they support
/// shared memory or not. They are aliased as non-parameterized types in the
/// top-level crate and in the `shm` module for better ergonomics.
pub mod generic {
    #[cfg(feature = "lock_api")]
    pub use crate::condvar::Condvar;
    pub use crate::futex_condvar::Condvar as RawCondvar;
    pub use crate::once::Once;
    pub use crate::once_lock::OnceLock;
    pub use crate::raw_mutex::RawMutex;
    pub use crate::raw_rwlock::RawRwLock;

    #[cfg(feature = "lock_api")]
    pub type Mutex<T, const SHM: bool> = lock_api::Mutex<RawMutex<SHM>, T>;
    #[cfg(feature = "lock_api")]
    pub type RwLock<T, const SHM: bool> = lock_api::RwLock<RawRwLock<SHM>, T>;
    #[cfg(feature = "lock_api")]
    pub type MutexGuard<'a, T, const SHM: bool> = lock_api::MutexGuard<'a, RawMutex<SHM>, T>;
    #[cfg(feature = "lock_api")]
    pub type MappedMutexGuard<'a, T, const SHM: bool> =
        lock_api::MappedMutexGuard<'a, RawMutex<SHM>, T>;
    #[cfg(feature = "lock_api")]
    pub type RwLockReadGuard<'a, T, const SHM: bool> =
        lock_api::RwLockReadGuard<'a, RawRwLock<SHM>, T>;
    #[cfg(feature = "lock_api")]
    pub type RwLockWriteGuard<'a, T, const SHM: bool> =
        lock_api::RwLockWriteGuard<'a, RawRwLock<SHM>, T>;
    #[cfg(feature = "lock_api")]
    pub type MappedRwLockReadGuard<'a, T, const SHM: bool> =
        lock_api::MappedRwLockReadGuard<'a, RawRwLock<SHM>, T>;
    #[cfg(feature = "lock_api")]
    pub type MappedRwLockWriteGuard<'a, T, const SHM: bool> =
        lock_api::MappedRwLockWriteGuard<'a, RawRwLock<SHM>, T>;
    #[cfg(feature = "lock_api")]
    #[cfg(feature = "atomic_usize")]
    #[cfg_attr(docsrs, doc(cfg(feature = "atomic_usize")))]
    pub type ReentrantMutex<G, T, const SHM: bool> = lock_api::ReentrantMutex<RawMutex<SHM>, G, T>;
    #[cfg(feature = "lock_api")]
    #[cfg(feature = "atomic_usize")]
    #[cfg_attr(docsrs, doc(cfg(feature = "atomic_usize")))]
    pub type ReentrantMutexGuard<'a, G, T, const SHM: bool> =
        lock_api::ReentrantMutexGuard<'a, RawMutex<SHM>, G, T>;
}

// std's implementation code.
#[cfg(feature = "lock_api")]
mod condvar;
mod futex_condvar;
mod futex_mutex;
mod futex_once;
mod futex_rwlock;
mod once;
mod once_lock;
mod raw_mutex;
mod raw_rwlock;
mod wait_wake;
