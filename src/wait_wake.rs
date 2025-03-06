//! The following is derived from Rust's
//! library/std/src/sys/pal/unix/futex.rs at revision
//! b58f647d5488dce73bba517907c44af2c2a618c4.

use core::num::NonZeroU32;
use core::sync::atomic::AtomicU32;
use core::time::Duration;
use rustix::thread::futex;
use rustix::time::{ClockId, Timespec};

/// Wait for a futex_wake operation to wake us.
///
/// Returns directly if the futex doesn't hold the expected value.
///
/// Returns false on timeout, and true in all other cases.
pub fn futex_wait(futex: &AtomicU32, expected: u32, timeout: Option<Duration>) -> bool {
    // Calculate the timeout as an absolute timespec.
    //
    // Overflows are rounded up to an infinite timeout (None).
    let timespec = timeout.and_then(|d| {
        Some({
            let now = rustix::time::clock_gettime(ClockId::Monotonic);
            let plus = Duration::new(now.tv_sec as u64, now.tv_nsec as _).checked_add(d)?;
            Timespec {
                tv_sec: plus.as_secs() as i64,
                tv_nsec: plus.subsec_nanos() as _,
            }
        })
    });

    futex_wait_timespec(futex, expected, timespec.as_ref())
}

/// Like [`futex_wait`], but takes a [`Timespec`] for an optional time on the
/// [`ClockId::Monotonic`] clock to wake up at.
///
/// This allows callers that don't need the timeout to pass `None` and avoid
/// statically depending on `clock_gettime`.
pub fn futex_wait_timespec(futex: &AtomicU32, expected: u32, timespec: Option<&Timespec>) -> bool {
    use core::sync::atomic::Ordering::Relaxed;

    loop {
        // No need to wait if the value already changed.
        if futex.load(Relaxed) != expected {
            return true;
        }

        let r =
            // Use `FUTEX_WAIT_BITSET` rather than `FUTEX_WAIT` to be able to
            // give an absolute time rather than a relative time.
            futex::wait_bitset(
                futex,
                futex::Flags::PRIVATE,
                expected,
                timespec,
 // A full bitmask, to make it behave like a regular `FUTEX_WAIT`.
                NonZeroU32::MAX
            )
        ;

        match r {
            Err(rustix::io::Errno::TIMEDOUT) => return false,
            Err(rustix::io::Errno::INTR) => continue,
            _ => return true,
        }
    }
}

/// Wake up one thread that's blocked on futex_wait on this futex.
///
/// Returns true if this actually woke up such a thread,
/// or false if no thread was waiting on this futex.
pub fn futex_wake(futex: &AtomicU32) -> bool {
    match futex::wake(futex, futex::Flags::PRIVATE, 1) {
        Err(_) | Ok(0) => false,
        _ => true,
    }
}

/// Wake up all threads that are waiting on futex_wait on this futex.
pub fn futex_wake_all(futex: &AtomicU32) {
    futex::wake(futex, futex::Flags::PRIVATE, i32::MAX as u32).ok();
}
