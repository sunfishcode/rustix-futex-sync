//! Polyfills for `lock_api` traits for when we don't have the real `lock_api`
//! crate.

pub struct GuardNoSend(());

pub unsafe trait RawMutex {
    type GuardMarker;

    const INIT: Self;

    fn lock(&self);
    fn try_lock(&self) -> bool;
    unsafe fn unlock(&self);

    #[inline]
    fn is_locked(&self) -> bool {
        let acquired_lock = self.try_lock();
        if acquired_lock {
            unsafe {
                self.unlock();
            }
        }
        !acquired_lock
    }
}

pub unsafe trait RawRwLock {
    type GuardMarker;

    const INIT: Self;

    fn lock_shared(&self);
    fn try_lock_shared(&self) -> bool;
    unsafe fn unlock_shared(&self);
    fn lock_exclusive(&self);
    fn try_lock_exclusive(&self) -> bool;
    unsafe fn unlock_exclusive(&self);

    #[inline]
    fn is_locked(&self) -> bool {
        let acquired_lock = self.try_lock_exclusive();
        if acquired_lock {
            unsafe {
                self.unlock_exclusive();
            }
        }
        !acquired_lock
    }

    fn is_locked_exclusive(&self) -> bool {
        let acquired_lock = self.try_lock_shared();
        if acquired_lock {
            unsafe {
                self.unlock_shared();
            }
        }
        !acquired_lock
    }
}
