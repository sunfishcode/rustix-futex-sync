//! The following is derived from the documentation tests in Rust's
//! library/std/src/sync/mutex.rs at revision
//! e853b50a722c09c7526683316b5471528063cccd.

#[test]
fn mutex_example_0() {
    use std::sync::Arc;
    use rustix_futex_sync::shm::Mutex;
    use std::thread;
    use std::sync::mpsc::channel;

    const N: usize = 10;

    // Spawn a few threads to increment a shared variable (non-atomically), and
    // let the main thread know once all increments are done.
    //
    // Here we're using an Arc to share memory among threads, and the data inside
    // the Arc is protected with a mutex.
    let data = Arc::new(Mutex::new(0));

    let (tx, rx) = channel();
    for _ in 0..N {
        let (data, tx) = (Arc::clone(&data), tx.clone());
        thread::spawn(move || {
            // The shared state can only be accessed once the lock is held.
            // Our non-atomic increment is safe because we're the only thread
            // which can access the shared state when the lock is held.
            //
            // We unwrap() the return value to assert that we are not expecting
            // threads to ever fail while holding the lock.
            let mut data = data.lock();
            *data += 1;
            if *data == N {
                tx.send(()).unwrap();
            }
            // the lock is unlocked here when `data` goes out of scope.
        });
    }

    rx.recv().unwrap();
}

#[test]
fn mutex_example_1() {
    use std::sync::Arc;
    use rustix_futex_sync::shm::Mutex;
    use std::thread;

    const N: usize = 3;

    let data_mutex = Arc::new(Mutex::new(vec![1, 2, 3, 4]));
    let res_mutex = Arc::new(Mutex::new(0));

    let mut threads = Vec::with_capacity(N);
    (0..N).for_each(|_| {
        let data_mutex_clone = Arc::clone(&data_mutex);
        let res_mutex_clone = Arc::clone(&res_mutex);

        threads.push(thread::spawn(move || {
            // Here we use a block to limit the lifetime of the lock guard.
            let result = {
                let mut data = data_mutex_clone.lock();
                // This is the result of some important and long-ish work.
                let result = data.iter().fold(0, |acc, x| acc + x * 2);
                data.push(result);
                result
                // The mutex guard gets dropped here, together with any other values
                // created in the critical section.
            };
            // The guard created here is a temporary dropped at the end of the statement, i.e.
            // the lock would not remain being held even if the thread did some additional work.
            *res_mutex_clone.lock() += result;
        }));
    });

    let mut data = data_mutex.lock();
    // This is the result of some important and long-ish work.
    let result = data.iter().fold(0, |acc, x| acc + x * 2);
    data.push(result);
    // We drop the `data` explicitly because it's not necessary anymore and the
    // thread still has work to do. This allow other threads to start working on
    // the data immediately, without waiting for the rest of the unrelated work
    // to be done here.
    //
    // It's even more important here than in the threads because we `.join` the
    // threads after that. If we had not dropped the mutex guard, a thread could
    // be waiting forever for it, causing a deadlock.
    // As in the threads, a block could have been used instead of calling the
    // `drop` function.
    drop(data);
    // Here the mutex guard is not assigned to a variable and so, even if the
    // scope does not end after this line, the mutex is still released: there is
    // no deadlock.
    *res_mutex.lock() += result;

    threads.into_iter().for_each(|thread| {
        thread
            .join()
            .expect("The thread creating or execution failed !")
    });

    assert_eq!(*res_mutex.lock(), 800);
}

#[test]
fn mutex_example_2() {
    use rustix_futex_sync::shm::Mutex;

    let _mutex = Mutex::new(0);
}

#[test]
fn mutex_example_3() {
    use std::sync::Arc;
    use rustix_futex_sync::shm::Mutex;
    use std::thread;

    let mutex = Arc::new(Mutex::new(0));
    let c_mutex = Arc::clone(&mutex);

    thread::spawn(move || {
        *c_mutex.lock() = 10;
    }).join().expect("thread::spawn failed");
    assert_eq!(*mutex.lock(), 10);
}

#[test]
fn mutex_example_4() {
    use std::sync::Arc;
    use rustix_futex_sync::shm::Mutex;
    use std::thread;

    let mutex = Arc::new(Mutex::new(0));
    let c_mutex = Arc::clone(&mutex);

    thread::spawn(move || {
        let mut lock = c_mutex.try_lock();
        if let Some(ref mut mutex) = lock {
            **mutex = 10;
        } else {
            println!("try_lock failed");
        }
    }).join().expect("thread::spawn failed");
    assert_eq!(*mutex.lock(), 10);
}

#[test]
fn mutex_example_5() {
    use std::sync::Arc;
    use rustix_futex_sync::shm::Mutex;
    use std::thread;

    let mutex = Arc::new(Mutex::new(0));
    let c_mutex = Arc::clone(&mutex);

    let _ = thread::spawn(move || {
        let _lock = c_mutex.lock();
        panic!(); // the mutex gets poisoned
    }).join();
    //assert_eq!(mutex.is_poisoned(), true);
}

#[test]
fn mutex_example_6() {
    use rustix_futex_sync::shm::Mutex;

    let mutex = Mutex::new(0);
    assert_eq!(mutex.into_inner(), 0);
}

#[test]
fn mutex_example_7() {
    use rustix_futex_sync::shm::Mutex;

    let mut mutex = Mutex::new(0);
    *mutex.get_mut() = 10;
    assert_eq!(*mutex.lock(), 10);
}
