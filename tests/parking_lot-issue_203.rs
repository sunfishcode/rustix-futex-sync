//! This test is derived from `parking_lot` at
//! <https://github.com/Amanieu/parking_lot/blob/master/tests/issue_203.rs>
//! at revision 699325c9a7119694f009372c7a4f83c8fa8384a6.

use rustix_futex_sync::RwLock;
use std::thread;

struct Bar(RwLock<()>);

impl Drop for Bar {
    fn drop(&mut self) {
        let _n = self.0.write();
    }
}

thread_local! {
    static B: Bar = Bar(RwLock::new(()));
}

#[test]
fn main() {
    thread::spawn(|| {
        B.with(|_| ());

        let a = RwLock::new(());
        let _a = a.read();
    })
    .join()
    .unwrap();
}
