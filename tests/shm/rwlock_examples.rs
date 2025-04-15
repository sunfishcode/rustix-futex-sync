//! The following is derived from the documentation tests in Rust's
//! library/std/src/sync/rwlock.rs at revision
//! 0cd57725f9d624aed3cdaa3852a1f80f550ae275.

#[test]
fn rwlock_example_0() {
    use rustix_futex_sync::shm::RwLock;

    let lock = RwLock::new(5);

    // many reader locks can be held at once
    {
        let r1 = lock.read();
        let r2 = lock.read();
        assert_eq!(*r1, 5);
        assert_eq!(*r2, 5);
    } // read locks are dropped at this point

    // only one write lock may be held, however
    {
        let mut w = lock.write();
        *w += 1;
        assert_eq!(*w, 6);
    } // write lock is dropped here
}

#[test]
fn rwlock_example_1() {
    use rustix_futex_sync::shm::RwLock;

    let _lock = RwLock::new(5);
}

#[test]
fn rwlock_example_2() {
    use std::sync::Arc;
    use rustix_futex_sync::shm::RwLock;
    use std::thread;

    let lock = Arc::new(RwLock::new(1));
    let c_lock = Arc::clone(&lock);

    let n = lock.read();
    assert_eq!(*n, 1);

    thread::spawn(move || {
        let _r = c_lock.read();
    }).join().unwrap();
}

#[test]
fn rwlock_example_3() {
    use rustix_futex_sync::shm::RwLock;

    let lock = RwLock::new(1);

    match lock.try_read() {
        Some(n) => assert_eq!(*n, 1),
        None => unreachable!(),
    };
}

#[test]
fn rwlock_example_4() {
    use rustix_futex_sync::shm::RwLock;

    let lock = RwLock::new(1);

    let mut n = lock.write();
    *n = 2;

    assert!(lock.try_read().is_none());
}

#[test]
fn rwlock_example_5() {
    use rustix_futex_sync::shm::RwLock;

    let lock = RwLock::new(1);

    let n = lock.read();
    assert_eq!(*n, 1);

    assert!(lock.try_write().is_none());
}

#[test]
fn rwlock_example_6() {
    use std::sync::Arc;
    use rustix_futex_sync::shm::RwLock;
    use std::thread;

    let lock = Arc::new(RwLock::new(0));
    let c_lock = Arc::clone(&lock);

    let _ = thread::spawn(move || {
    let _lock = c_lock.write();
        panic!(); // the lock gets poisoned
    }).join();
    //assert_eq!(lock.is_poisoned(), true);
}

#[test]
fn rwlock_example_7() {
    use rustix_futex_sync::shm::RwLock;

    let lock = RwLock::new(String::new());
    {
        let mut s = lock.write();
        *s = "modified".to_owned();
    }
    assert_eq!(lock.into_inner(), "modified");
}

#[test]
fn rwlock_example_8() {
    use rustix_futex_sync::shm::RwLock;

    let mut lock = RwLock::new(0);
    *lock.get_mut() = 10;
    assert_eq!(*lock.read(), 10);
}
