#[test]
fn mutex() {
    use rustix_futex_sync::Mutex;

    let m = Mutex::new(0);
    let mut locked = m.lock();
    *locked = 1;
    assert!(m.try_lock().is_none());
    drop(locked);
    let mut locked = m.try_lock().unwrap();
    assert!(m.try_lock().is_none());
    *locked = 2;
    drop(locked);
    let inner = m.into_inner();
    assert_eq!(inner, 2);
}

#[test]
fn rw_lock() {
    use rustix_futex_sync::RwLock;

    let m = RwLock::new(0);
    let locked = m.read();
    assert_eq!(*locked, 0);
    assert!(m.try_read().is_some());
    assert!(m.try_write().is_none());
    drop(locked);
    let mut locked = m.write();
    *locked = 1;
    assert!(m.try_read().is_none());
    assert!(m.try_write().is_none());
    drop(locked);
    let locked = m.try_read().unwrap();
    assert_eq!(*locked, 1);
    drop(locked);
    let mut locked = m.try_write().unwrap();
    *locked = 2;
    drop(locked);
    let locked = m.try_read().unwrap();
    assert_eq!(*locked, 2);
    drop(locked);
    let inner = m.into_inner();
    assert_eq!(inner, 2);
}

#[test]
fn condvar() {
    use rustix_futex_sync::{Condvar, Mutex};
    use std::sync::Arc;
    use std::thread;

    let pair = Arc::new((Mutex::new(false), Condvar::new()));
    let pair2 = pair.clone();

    // Inside of our lock, spawn a new thread, and then wait for it to start
    thread::spawn(move || {
        let (lock, cvar) = &*pair2;
        let mut started = lock.lock();
        *started = true;
        cvar.notify_one();
    });

    // wait for the thread to start up
    let (lock, cvar) = &*pair;
    let mut started = lock.lock();
    if !*started {
        started = cvar.wait(started);
    }
    drop(started);
}
