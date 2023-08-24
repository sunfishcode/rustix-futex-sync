//! The following is derived from Rust's
//! library/std/src/sync/rwlock/tests.rs at revision
//! 72a25d05bf1a4b155d74139ef700ff93af6d8e22.

use rand::{self, Rng};
use rustix_futex_sync::RwLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::thread;

#[derive(Eq, PartialEq, Debug)]
struct NonCopy(i32);

#[test]
fn smoke() {
    let l = RwLock::new(());
    drop(l.read());
    drop(l.write());
    drop((l.read(), l.read()));
    drop(l.write());
}

#[test]
fn frob() {
    const N: u32 = 10;
    const M: usize = if cfg!(miri) { 100 } else { 1000 };

    let r = Arc::new(RwLock::new(()));

    let (tx, rx) = channel::<()>();
    for _ in 0..N {
        let tx = tx.clone();
        let r = r.clone();
        thread::spawn(move || {
            let mut rng = rand::thread_rng();
            for _ in 0..M {
                if rng.gen_bool(1.0 / (N as f64)) {
                    drop(r.write());
                } else {
                    drop(r.read());
                }
            }
            drop(tx);
        });
    }
    drop(tx);
    let _ = rx.recv();
}

/*
#[test]
#[cfg_attr(all(target_arch = "arm", not(feature = "unwinding")), ignore)]
fn test_rw_arc_poison_wr() {
    let arc = Arc::new(RwLock::new(1));
    let arc2 = arc.clone();
    let _: Result<(), _> = thread::spawn(move || {
        let _lock = arc2.write();
        panic!();
    })
    .join();
    assert!(arc.read().is_err());
}

#[test]
#[cfg_attr(all(target_arch = "arm", not(feature = "unwinding")), ignore)]
fn test_rw_arc_poison_ww() {
    let arc = Arc::new(RwLock::new(1));
    assert!(!arc.is_poisoned());
    let arc2 = arc.clone();
    let _: Result<(), _> = thread::spawn(move || {
        let _lock = arc2.write();
        panic!();
    })
    .join();
    assert!(arc.write().is_err());
    assert!(arc.is_poisoned());
}

#[test]
#[cfg_attr(all(target_arch = "arm", not(feature = "unwinding")), ignore)]
fn test_rw_arc_no_poison_rr() {
    let arc = Arc::new(RwLock::new(1));
    let arc2 = arc.clone();
    let _: Result<(), _> = thread::spawn(move || {
        let _lock = arc2.read();
        panic!();
    })
    .join();
    let lock = arc.read();
    assert_eq!(*lock, 1);
}
#[test]
#[cfg_attr(all(target_arch = "arm", not(feature = "unwinding")), ignore)]
fn test_rw_arc_no_poison_rw() {
    let arc = Arc::new(RwLock::new(1));
    let arc2 = arc.clone();
    let _: Result<(), _> = thread::spawn(move || {
        let _lock = arc2.read();
        panic!()
    })
    .join();
    let lock = arc.write();
    assert_eq!(*lock, 1);
}
*/

#[test]
fn test_rw_arc() {
    let arc = Arc::new(RwLock::new(0));
    let arc2 = arc.clone();
    let (tx, rx) = channel();

    thread::spawn(move || {
        let mut lock = arc2.write();
        for _ in 0..10 {
            let tmp = *lock;
            *lock = -1;
            thread::yield_now();
            *lock = tmp + 1;
        }
        tx.send(()).unwrap();
    });

    // Readers try to catch the writer in the act
    let mut children = Vec::new();
    for _ in 0..5 {
        let arc3 = arc.clone();
        children.push(thread::spawn(move || {
            let lock = arc3.read();
            assert!(*lock >= 0);
        }));
    }

    // Wait for children to pass their asserts
    for r in children {
        assert!(r.join().is_ok());
    }

    // Wait for writer to finish
    rx.recv().unwrap();
    let lock = arc.read();
    assert_eq!(*lock, 10);
}

#[test]
#[cfg_attr(all(target_arch = "arm", not(feature = "unwinding")), ignore)]
fn test_rw_arc_access_in_unwind() {
    let arc = Arc::new(RwLock::new(1));
    let arc2 = arc.clone();
    let _ = thread::spawn(move || -> () {
        struct Unwinder {
            i: Arc<RwLock<isize>>,
        }
        impl Drop for Unwinder {
            fn drop(&mut self) {
                let mut lock = self.i.write();
                *lock += 1;
            }
        }
        let _u = Unwinder { i: arc2 };
        panic!();
    })
    .join();
    let lock = arc.read();
    assert_eq!(*lock, 2);
}

#[test]
fn test_rwlock_unsized() {
    let rw: &RwLock<[i32]> = &RwLock::new([1, 2, 3]);
    {
        let b = &mut *rw.write();
        b[0] = 4;
        b[2] = 5;
    }
    let comp: &[i32] = &[4, 2, 5];
    assert_eq!(&*rw.read(), comp);
}

#[test]
fn test_rwlock_try_write() {
    let lock = RwLock::new(0isize);
    let read_guard = lock.read();

    let write_result = lock.try_write();
    match write_result {
        None => (),
        Some(_) => assert!(
            false,
            "try_write should not succeed while read_guard is in scope"
        ),
    }

    drop(read_guard);
}

#[test]
fn test_into_inner() {
    let m = RwLock::new(NonCopy(10));
    assert_eq!(m.into_inner(), NonCopy(10));
}

#[test]
fn test_into_inner_drop() {
    struct Foo(Arc<AtomicUsize>);
    impl Drop for Foo {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    let num_drops = Arc::new(AtomicUsize::new(0));
    let m = RwLock::new(Foo(num_drops.clone()));
    assert_eq!(num_drops.load(Ordering::SeqCst), 0);
    {
        let _inner = m.into_inner();
        assert_eq!(num_drops.load(Ordering::SeqCst), 0);
    }
    assert_eq!(num_drops.load(Ordering::SeqCst), 1);
}

/*
#[test]
#[cfg_attr(all(target_arch = "arm", not(feature = "unwinding")), ignore)]
fn test_into_inner_poison() {
    let m = Arc::new(RwLock::new(NonCopy(10)));
    let m2 = m.clone();
    let _ = thread::spawn(move || {
        let _lock = m2.write().unwrap();
        panic!("test panic in inner thread to poison RwLock");
    })
    .join();

    assert!(m.is_poisoned());
    match Arc::try_unwrap(m).unwrap().into_inner() {
        Err(e) => assert_eq!(e.into_inner(), NonCopy(10)),
        Ok(x) => panic!("into_inner of poisoned RwLock is Ok: {x:?}"),
    }
}
*/

#[test]
fn test_get_mut() {
    let mut m = RwLock::new(NonCopy(10));
    *m.get_mut() = NonCopy(20);
    assert_eq!(m.into_inner(), NonCopy(20));
}

/*
#[test]
#[cfg_attr(all(target_arch = "arm", not(feature = "unwinding")), ignore)]
fn test_get_mut_poison() {
    let m = Arc::new(RwLock::new(NonCopy(10)));
    let m2 = m.clone();
    let _ = thread::spawn(move || {
        let _lock = m2.write();
        panic!("test panic in inner thread to poison RwLock");
    })
    .join();

    assert!(m.is_poisoned());
    match Arc::try_unwrap(m).unwrap().get_mut() {
        Err(e) => assert_eq!(*e.into_inner(), NonCopy(10)),
        Ok(x) => panic!("get_mut of poisoned RwLock is Ok: {x:?}"),
    }
}
*/
