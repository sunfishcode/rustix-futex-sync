//! rustix_futex_sync documents some details of the representation of some of
//! its public types.

use core::mem::{align_of, size_of, transmute};
use rustix_futex_sync::shm::lock_api::{RawMutex as _, RawRwLock as _};
use rustix_futex_sync::shm::{Condvar, Once, RawCondvar, RawMutex, RawRwLock};

#[test]
fn repr_raw_mutex() {
    assert_eq!(size_of::<RawMutex>(), size_of::<u32>());
    assert_eq!(align_of::<RawMutex>(), align_of::<u32>());
    unsafe {
        assert_eq!(transmute::<RawMutex, u32>(RawMutex::INIT), 0_u32);
    }
}

#[test]
fn repr_raw_condvar() {
    assert_eq!(size_of::<RawCondvar>(), size_of::<u32>());
    assert_eq!(align_of::<RawCondvar>(), align_of::<u32>());
    unsafe {
        assert_eq!(transmute::<RawCondvar, u32>(RawCondvar::new()), 0_u32);
    }
}

#[test]
fn repr_condvar() {
    assert_eq!(size_of::<Condvar>(), size_of::<u32>());
    assert_eq!(align_of::<Condvar>(), align_of::<u32>());
    unsafe {
        assert_eq!(transmute::<Condvar, u32>(Condvar::new()), 0_u32);
    }
}

#[test]
fn repr_once() {
    assert_eq!(size_of::<Once>(), size_of::<u32>());
    assert_eq!(align_of::<Once>(), align_of::<u32>());
    unsafe {
        assert_eq!(transmute::<Once, u32>(Once::new()), 0_u32);
    }
}

#[test]
fn repr_raw_rwlock() {
    assert_eq!(size_of::<RawRwLock>(), size_of::<[u32; 2]>());
    assert_eq!(align_of::<RawRwLock>(), align_of::<[u32; 2]>());
    unsafe {
        assert_eq!(
            transmute::<RawRwLock, [u32; 2]>(RawRwLock::INIT),
            [0_u32; 2]
        );
    }
}

// Test that the types are FFI-safe.
#[allow(dead_code)]
#[deny(improper_ctypes)]
extern "C" {
    fn use_raw_mutex(x: RawMutex);
    fn use_raw_rwlock(x: RawRwLock);
    fn use_condvar(x: Condvar);
    fn use_raw_condvar(x: RawCondvar);
    fn use_once(x: Once);
}
