//! The following is derived from Rust's
//! library/std/src/sys/sync/condvar/futex.rs at revision
//! 22a5267c83a3e17f2b763279eb24bb632c45dc6b.

use core::sync::atomic::{AtomicU32, Ordering::Relaxed};
use super::wait_wake::{futex_wait, futex_wake, futex_wake_all};
use core::time::Duration;
use super::generic::RawMutex;
use super::lock_api::RawMutex as _;

#[repr(transparent)]
pub struct Condvar<const SHM: bool> {
    // The value of this atomic is simply incremented on every notification.
    // This is used by `.wait()` to not miss any notifications after
    // unlocking the mutex and before waiting for notifications.
    futex: AtomicU32,
}

impl<const SHM: bool> Condvar<SHM> {
    #[inline]
    pub const fn new() -> Self {
        Self { futex: AtomicU32::new(0) }
    }

    // All the memory orderings here are `Relaxed`,
    // because synchronization is done by unlocking and locking the mutex.

    pub fn notify_one(&self) {
        self.futex.fetch_add(1, Relaxed);
        futex_wake::<SHM>(&self.futex);
    }

    pub fn notify_all(&self) {
        self.futex.fetch_add(1, Relaxed);
        futex_wake_all::<SHM>(&self.futex);
    }

    pub unsafe fn wait(&self, mutex: &RawMutex<SHM>) {
        self.wait_optional_timeout(mutex, None);
    }

    pub unsafe fn wait_timeout(&self, mutex: &RawMutex<SHM>, timeout: Duration) -> bool {
        self.wait_optional_timeout(mutex, Some(timeout))
    }

    unsafe fn wait_optional_timeout(&self, mutex: &RawMutex<SHM>, timeout: Option<Duration>) -> bool {
        // Examine the notification counter _before_ we unlock the mutex.
        let futex_value = self.futex.load(Relaxed);

        // Unlock the mutex before going to sleep.
        mutex.unlock();

        // Wait, but only if there hasn't been any
        // notification since we unlocked the mutex.
        let r = futex_wait::<SHM>(&self.futex, futex_value, timeout);

        // Lock the mutex again.
        mutex.lock();

        r
    }
}
