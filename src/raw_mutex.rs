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
pub struct RawMutex<const SHM: bool>(crate::futex_mutex::Mutex<SHM>);

unsafe impl<const SHM: bool> lock_api::RawMutex for RawMutex<SHM> {
    type GuardMarker = lock_api::GuardNoSend;

    const INIT: Self = Self(crate::futex_mutex::Mutex::new());

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
