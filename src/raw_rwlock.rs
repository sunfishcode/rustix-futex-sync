use crate::lock_api;

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
pub struct RawRwLock<const SHM: bool>(crate::futex_rwlock::RwLock<SHM>);

unsafe impl<const SHM: bool> lock_api::RawRwLock for RawRwLock<SHM> {
    type GuardMarker = lock_api::GuardNoSend;

    const INIT: Self = Self(crate::futex_rwlock::RwLock::new());

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
