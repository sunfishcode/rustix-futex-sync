//! This test is derived from `parking_lot` at
//! <https://github.com/Amanieu/parking_lot/blob/master/tests/issue_392.rs>
//! at revision 64b711461ed528de7c05868e5024f21e602cd4e0.

// rustix_futex_sync doesn't currently support upgrading
/*
use rustix_futex_sync::RwLock;

struct Lock(RwLock<i32>);

#[test]
fn issue_392() {
    let lock = Lock(RwLock::new(0));
    let mut rl = lock.0.upgradable_read();
    rl.with_upgraded(|_| {
        println!("lock upgrade");
    });
    rl.with_upgraded(|_| {
        println!("lock upgrade");
    });
}
*/
