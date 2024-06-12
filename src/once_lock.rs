//! The following is derived from Rust's
//! library/std/src/sync/once_lock.rs at revision
//! ee04e0f35ed516e4f1cc6a12c28838eaf98a16d1.

use core::cell::UnsafeCell;
use core::fmt;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::panic::{RefUnwindSafe, UnwindSafe};
use crate::Once;

/// A synchronization primitive which can nominally be written to only once.
///
/// This type is a thread-safe [`OnceCell`], and can be used in statics.
/// In many simple cases, you can use [`LazyLock<T, F>`] instead to get the benefits of this type
/// with less effort: `LazyLock<T, F>` "looks like" `&T` because it initializes with `F` on deref!
/// Where OnceLock shines is when LazyLock is too simple to support a given case, as LazyLock
/// doesn't allow additional inputs to its function after you call [`LazyLock::new(|| ...)`].
///
/// [`OnceCell`]: core::cell::OnceCell
/// [`LazyLock<T, F>`]: https://doc.rust-lang.org/std/sync/struct.LazyLock.html
/// [`LazyLock::new(|| ...)`]: https://doc.rust-lang.org/std/sync/struct.LazyLock.html#method.new
///
/// # Examples
///
/// Writing to a `OnceLock` from a separate thread:
///
/// ```
/// use rustix_futex_sync::OnceLock;
///
/// static CELL: OnceLock<usize> = OnceLock::new();
///
/// // `OnceLock` has not been written to yet.
/// assert!(CELL.get().is_none());
///
/// // Spawn a thread and write to `OnceLock`.
/// std::thread::spawn(|| {
///     let value = CELL.get_or_init(|| 12345);
///     assert_eq!(value, &12345);
/// })
/// .join()
/// .unwrap();
///
/// // `OnceLock` now contains the value.
/// assert_eq!(
///     CELL.get(),
///     Some(&12345),
/// );
/// ```
///
/// You can use `OnceLock` to implement a type that requires "append-only" logic:
///
/// ```
/// use std::sync::atomic::{AtomicU32, Ordering};
/// use rustix_futex_sync::OnceLock;
/// use std::thread;
///
/// struct OnceList<T> {
///     data: OnceLock<T>,
///     next: OnceLock<Box<OnceList<T>>>,
/// }
/// impl<T> OnceList<T> {
///     const fn new() -> OnceList<T> {
///         OnceList { data: OnceLock::new(), next: OnceLock::new() }
///     }
///     fn push(&self, value: T) {
///         // FIXME: this impl is concise, but is also slow for long lists or many threads.
///         // as an exercise, consider how you might improve on it while preserving the behavior
///         if let Err(value) = self.data.set(value) {
///             let next = self.next.get_or_init(|| Box::new(OnceList::new()));
///             next.push(value)
///         };
///     }
///     fn contains(&self, example: &T) -> bool
///     where
///         T: PartialEq,
///     {
///         self.data.get().map(|item| item == example).filter(|v| *v).unwrap_or_else(|| {
///             self.next.get().map(|next| next.contains(example)).unwrap_or(false)
///         })
///     }
/// }
///
/// // Let's exercise this new Sync append-only list by doing a little counting
/// static LIST: OnceList<u32> = OnceList::new();
/// static COUNTER: AtomicU32 = AtomicU32::new(0);
///
/// let vec = (0..thread::available_parallelism().unwrap().get()).map(|_| thread::spawn(|| {
///     while let i @ 0..=1000 = COUNTER.fetch_add(1, Ordering::Relaxed) {
///         LIST.push(i);
///     }
/// })).collect::<Vec<thread::JoinHandle<_>>>();
/// vec.into_iter().for_each(|handle| handle.join().unwrap());
///
/// for i in 0..=1000 {
///     assert!(LIST.contains(&i));
/// }
///
/// ```
//#[stable(feature = "once_cell", since = "1.70.0")]
pub struct OnceLock<T> {
    once: Once,
    // Whether or not the value is initialized is tracked by `once.is_completed()`.
    value: UnsafeCell<MaybeUninit<T>>,
    /// `PhantomData` to make sure dropck understands we're dropping T in our Drop impl.
    ///
    /// ```compile_fail,E0597
    /// use rustix_futex_sync::OnceLock;
    ///
    /// struct A<'a>(&'a str);
    ///
    /// impl<'a> Drop for A<'a> {
    ///     fn drop(&mut self) {}
    /// }
    ///
    /// let cell = OnceLock::new();
    /// {
    ///     let s = String::new();
    ///     let _ = cell.set(A(&s));
    /// }
    /// ```
    _marker: PhantomData<T>,
}

impl<T> OnceLock<T> {
    /// Creates a new empty cell.
    #[inline]
    #[must_use]
    //#[stable(feature = "once_cell", since = "1.70.0")]
    //#[rustc_const_stable(feature = "once_cell", since = "1.70.0")]
    pub const fn new() -> OnceLock<T> {
        OnceLock {
            once: Once::new(),
            value: UnsafeCell::new(MaybeUninit::uninit()),
            _marker: PhantomData,
        }
    }

    /// Gets the reference to the underlying value.
    ///
    /// Returns `None` if the cell is empty, or being initialized. This
    /// method never blocks.
    #[inline]
    //#[stable(feature = "once_cell", since = "1.70.0")]
    pub fn get(&self) -> Option<&T> {
        if self.is_initialized() {
            // Safe b/c checked is_initialized
            Some(unsafe { self.get_unchecked() })
        } else {
            None
        }
    }

    /// Gets the mutable reference to the underlying value.
    ///
    /// Returns `None` if the cell is empty. This method never blocks.
    #[inline]
    //#[stable(feature = "once_cell", since = "1.70.0")]
    pub fn get_mut(&mut self) -> Option<&mut T> {
        if self.is_initialized() {
            // Safe b/c checked is_initialized and we have a unique access
            Some(unsafe { self.get_unchecked_mut() })
        } else {
            None
        }
    }

    /// Sets the contents of this cell to `value`.
    ///
    /// May block if another thread is currently attempting to initialize the cell. The cell is
    /// guaranteed to contain a value when set returns, though not necessarily the one provided.
    ///
    /// Returns `Ok(())` if the cell's value was set by this call.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustix_futex_sync::OnceLock;
    ///
    /// static CELL: OnceLock<i32> = OnceLock::new();
    ///
    /// fn main() {
    ///     assert!(CELL.get().is_none());
    ///
    ///     std::thread::spawn(|| {
    ///         assert_eq!(CELL.set(92), Ok(()));
    ///     }).join().unwrap();
    ///
    ///     assert_eq!(CELL.set(62), Err(62));
    ///     assert_eq!(CELL.get(), Some(&92));
    /// }
    /// ```
    #[inline]
    //#[stable(feature = "once_cell", since = "1.70.0")]
    pub fn set(&self, value: T) -> Result<(), T> {
        match self.try_insert(value) {
            Ok(_) => Ok(()),
            Err((_, value)) => Err(value),
        }
    }

    /// Sets the contents of this cell to `value` if the cell was empty, then
    /// returns a reference to it.
    ///
    /// May block if another thread is currently attempting to initialize the cell. The cell is
    /// guaranteed to contain a value when set returns, though not necessarily the one provided.
    ///
    /// Returns `Ok(&value)` if the cell was empty and `Err(&current_value, value)` if it was full.
    ///
    /// # Examples
    ///
    /// ```
    /// //#![feature(once_cell_try_insert)]
    ///
    /// use rustix_futex_sync::OnceLock;
    ///
    /// static CELL: OnceLock<i32> = OnceLock::new();
    ///
    /// fn main() {
    ///     assert!(CELL.get().is_none());
    ///
    ///     std::thread::spawn(|| {
    ///         assert_eq!(CELL.try_insert(92), Ok(&92));
    ///     }).join().unwrap();
    ///
    ///     assert_eq!(CELL.try_insert(62), Err((&92, 62)));
    ///     assert_eq!(CELL.get(), Some(&92));
    /// }
    /// ```
    #[inline]
    //#[unstable(feature = "once_cell_try_insert", issue = "116693")]
    pub fn try_insert(&self, value: T) -> Result<&T, (&T, T)> {
        let mut value = Some(value);
        let res = self.get_or_init(|| value.take().unwrap());
        match value {
            None => Ok(res),
            Some(value) => Err((res, value)),
        }
    }

    /// Gets the contents of the cell, initializing it with `f` if the cell
    /// was empty.
    ///
    /// Many threads may call `get_or_init` concurrently with different
    /// initializing functions, but it is guaranteed that only one function
    /// will be executed.
    ///
    /// # Panics
    ///
    /// If `f` panics, the panic is propagated to the caller, and the cell
    /// remains uninitialized.
    ///
    /// It is an error to reentrantly initialize the cell from `f`. The
    /// exact outcome is unspecified. Current implementation deadlocks, but
    /// this may be changed to a panic in the future.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustix_futex_sync::OnceLock;
    ///
    /// let cell = OnceLock::new();
    /// let value = cell.get_or_init(|| 92);
    /// assert_eq!(value, &92);
    /// let value = cell.get_or_init(|| unreachable!());
    /// assert_eq!(value, &92);
    /// ```
    #[inline]
    //#[stable(feature = "once_cell", since = "1.70.0")]
    pub fn get_or_init<F>(&self, f: F) -> &T
    where
        F: FnOnce() -> T,
    {
        //match self.get_or_try_init(|| Ok::<T, !>(f())) {
        match self.get_or_try_init(|| Ok::<T, ()>(f())) {
            Ok(val) => val,
            Err(()) => panic!(),
        }
    }

    /// Gets the mutable reference of the contents of the cell, initializing
    /// it with `f` if the cell was empty.
    ///
    /// Many threads may call `get_mut_or_init` concurrently with different
    /// initializing functions, but it is guaranteed that only one function
    /// will be executed.
    ///
    /// # Panics
    ///
    /// If `f` panics, the panic is propagated to the caller, and the cell
    /// remains uninitialized.
    ///
    /// # Examples
    ///
    /// ```
    /// //#![feature(once_cell_get_mut)]
    ///
    /// use rustix_futex_sync::OnceLock;
    ///
    /// let mut cell = OnceLock::new();
    /// let value = cell.get_mut_or_init(|| 92);
    /// assert_eq!(*value, 92);
    ///
    /// *value += 2;
    /// assert_eq!(*value, 94);
    ///
    /// let value = cell.get_mut_or_init(|| unreachable!());
    /// assert_eq!(*value, 94);
    /// ```
    #[inline]
    //#[unstable(feature = "once_cell_get_mut", issue = "121641")]
    pub fn get_mut_or_init<F>(&mut self, f: F) -> &mut T
    where
        F: FnOnce() -> T,
    {
        //match self.get_mut_or_try_init(|| Ok::<T, !>(f())) {
        match self.get_mut_or_try_init(|| Ok::<T, ()>(f())) {
            Ok(val) => val,
            Err(()) => panic!(),
        }
    }

    /// Gets the contents of the cell, initializing it with `f` if
    /// the cell was empty. If the cell was empty and `f` failed, an
    /// error is returned.
    ///
    /// # Panics
    ///
    /// If `f` panics, the panic is propagated to the caller, and
    /// the cell remains uninitialized.
    ///
    /// It is an error to reentrantly initialize the cell from `f`.
    /// The exact outcome is unspecified. Current implementation
    /// deadlocks, but this may be changed to a panic in the future.
    ///
    /// # Examples
    ///
    /// ```
    /// //#![feature(once_cell_try)]
    ///
    /// use rustix_futex_sync::OnceLock;
    ///
    /// let cell = OnceLock::new();
    /// assert_eq!(cell.get_or_try_init(|| Err(())), Err(()));
    /// assert!(cell.get().is_none());
    /// let value = cell.get_or_try_init(|| -> Result<i32, ()> {
    ///     Ok(92)
    /// });
    /// assert_eq!(value, Ok(&92));
    /// assert_eq!(cell.get(), Some(&92))
    /// ```
    #[inline]
    //#[unstable(feature = "once_cell_try", issue = "109737")]
    pub fn get_or_try_init<F, E>(&self, f: F) -> Result<&T, E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        // Fast path check
        // NOTE: We need to perform an acquire on the state in this method
        // in order to correctly synchronize `LazyLock::force`. This is
        // currently done by calling `self.get()`, which in turn calls
        // `self.is_initialized()`, which in turn performs the acquire.
        if let Some(value) = self.get() {
            return Ok(value);
        }
        self.initialize(f)?;

        debug_assert!(self.is_initialized());

        // SAFETY: The inner value has been initialized
        Ok(unsafe { self.get_unchecked() })
    }

    /// Gets the mutable reference of the contents of the cell, initializing
    /// it with `f` if the cell was empty. If the cell was empty and `f` failed,
    /// an error is returned.
    ///
    /// # Panics
    ///
    /// If `f` panics, the panic is propagated to the caller, and
    /// the cell remains uninitialized.
    ///
    /// # Examples
    ///
    /// ```
    /// //#![feature(once_cell_get_mut)]
    ///
    /// use rustix_futex_sync::OnceLock;
    ///
    /// let mut cell: OnceLock<u32> = OnceLock::new();
    ///
    /// // Failed initializers do not change the value
    /// assert!(cell.get_mut_or_try_init(|| "not a number!".parse()).is_err());
    /// assert!(cell.get().is_none());
    ///
    /// let value = cell.get_mut_or_try_init(|| "1234".parse());
    /// assert_eq!(value, Ok(&mut 1234));
    /// *value.unwrap() += 2;
    /// assert_eq!(cell.get(), Some(&1236))
    /// ```
    #[inline]
    //#[unstable(feature = "once_cell_get_mut", issue = "121641")]
    pub fn get_mut_or_try_init<F, E>(&mut self, f: F) -> Result<&mut T, E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        if self.get().is_none() {
            self.initialize(f)?;
        }
        debug_assert!(self.is_initialized());
        // SAFETY: The inner value has been initialized
        Ok(unsafe { self.get_unchecked_mut() })
    }

    /// Consumes the `OnceLock`, returning the wrapped value. Returns
    /// `None` if the cell was empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustix_futex_sync::OnceLock;
    ///
    /// let cell: OnceLock<String> = OnceLock::new();
    /// assert_eq!(cell.into_inner(), None);
    ///
    /// let cell = OnceLock::new();
    /// cell.set("hello".to_string()).unwrap();
    /// assert_eq!(cell.into_inner(), Some("hello".to_string()));
    /// ```
    #[inline]
    //#[stable(feature = "once_cell", since = "1.70.0")]
    pub fn into_inner(mut self) -> Option<T> {
        self.take()
    }

    /// Takes the value out of this `OnceLock`, moving it back to an uninitialized state.
    ///
    /// Has no effect and returns `None` if the `OnceLock` hasn't been initialized.
    ///
    /// Safety is guaranteed by requiring a mutable reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustix_futex_sync::OnceLock;
    ///
    /// let mut cell: OnceLock<String> = OnceLock::new();
    /// assert_eq!(cell.take(), None);
    ///
    /// let mut cell = OnceLock::new();
    /// cell.set("hello".to_string()).unwrap();
    /// assert_eq!(cell.take(), Some("hello".to_string()));
    /// assert_eq!(cell.get(), None);
    /// ```
    #[inline]
    //#[stable(feature = "once_cell", since = "1.70.0")]
    pub fn take(&mut self) -> Option<T> {
        if self.is_initialized() {
            self.once = Once::new();
            // SAFETY: `self.value` is initialized and contains a valid `T`.
            // `self.once` is reset, so `is_initialized()` will be false again
            // which prevents the value from being read twice.
            unsafe { Some((&mut *self.value.get()).assume_init_read()) }
        } else {
            None
        }
    }

    #[inline]
    fn is_initialized(&self) -> bool {
        self.once.is_completed()
    }

    #[cold]
    fn initialize<F, E>(&self, f: F) -> Result<(), E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        let mut res: Result<(), E> = Ok(());
        let slot = &self.value;

        // Ignore poisoning from other threads
        // If another thread panics, then we'll be able to run our closure
        self.once.call_once_force(|p| {
            match f() {
                Ok(value) => {
                    unsafe { (&mut *slot.get()).write(value) };
                }
                Err(e) => {
                    res = Err(e);

                    // The call failed, so reset the once state back to
                    // incomplete.
                    p.set_incomplete();

                    /*
                    // Treat the underlying `Once` as poisoned since we
                    // failed to initialize our value. Calls
                    p.poison();
                    */
                }
            }
        });
        res
    }

    /// # Safety
    ///
    /// The value must be initialized
    #[inline]
    unsafe fn get_unchecked(&self) -> &T {
        debug_assert!(self.is_initialized());
        (&*self.value.get()).assume_init_ref()
    }

    /// # Safety
    ///
    /// The value must be initialized
    #[inline]
    unsafe fn get_unchecked_mut(&mut self) -> &mut T {
        debug_assert!(self.is_initialized());
        (&mut *self.value.get()).assume_init_mut()
    }
}

// Why do we need `T: Send`?
// Thread A creates a `OnceLock` and shares it with
// scoped thread B, which fills the cell, which is
// then destroyed by A. That is, destructor observes
// a sent value.
//#[stable(feature = "once_cell", since = "1.70.0")]
unsafe impl<T: Sync + Send> Sync for OnceLock<T> {}
//#[stable(feature = "once_cell", since = "1.70.0")]
unsafe impl<T: Send> Send for OnceLock<T> {}

//#[stable(feature = "once_cell", since = "1.70.0")]
impl<T: RefUnwindSafe + UnwindSafe> RefUnwindSafe for OnceLock<T> {}
//#[stable(feature = "once_cell", since = "1.70.0")]
impl<T: UnwindSafe> UnwindSafe for OnceLock<T> {}

//#[stable(feature = "once_cell", since = "1.70.0")]
impl<T> Default for OnceLock<T> {
    /// Creates a new empty cell.
    ///
    /// # Example
    ///
    /// ```
    /// use rustix_futex_sync::OnceLock;
    ///
    /// fn main() {
    ///     assert_eq!(OnceLock::<()>::new(), OnceLock::default());
    /// }
    /// ```
    #[inline]
    fn default() -> OnceLock<T> {
        OnceLock::new()
    }
}

//#[stable(feature = "once_cell", since = "1.70.0")]
impl<T: fmt::Debug> fmt::Debug for OnceLock<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_tuple("OnceLock");
        match self.get() {
            Some(v) => d.field(v),
            None => d.field(&format_args!("<uninit>")),
        };
        d.finish()
    }
}

//#[stable(feature = "once_cell", since = "1.70.0")]
impl<T: Clone> Clone for OnceLock<T> {
    #[inline]
    fn clone(&self) -> OnceLock<T> {
        let cell = Self::new();
        if let Some(value) = self.get() {
            match cell.set(value.clone()) {
                Ok(()) => (),
                Err(_) => unreachable!(),
            }
        }
        cell
    }
}

//#[stable(feature = "once_cell", since = "1.70.0")]
impl<T> From<T> for OnceLock<T> {
    /// Create a new cell with its contents set to `value`.
    ///
    /// # Example
    ///
    /// ```
    /// use rustix_futex_sync::OnceLock;
    ///
    /// # fn main() -> Result<(), i32> {
    /// let a = OnceLock::from(3);
    /// let b = OnceLock::new();
    /// b.set(3)?;
    /// assert_eq!(a, b);
    /// Ok(())
    /// # }
    /// ```
    #[inline]
    fn from(value: T) -> Self {
        let cell = Self::new();
        match cell.set(value) {
            Ok(()) => cell,
            Err(_) => unreachable!(),
        }
    }
}

//#[stable(feature = "once_cell", since = "1.70.0")]
impl<T: PartialEq> PartialEq for OnceLock<T> {
    #[inline]
    fn eq(&self, other: &OnceLock<T>) -> bool {
        self.get() == other.get()
    }
}

//#[stable(feature = "once_cell", since = "1.70.0")]
impl<T: Eq> Eq for OnceLock<T> {}

//#[stable(feature = "once_cell", since = "1.70.0")]
/*unsafe*/ impl</*#[may_dangle]*/ T> Drop for OnceLock<T> {
    #[inline]
    fn drop(&mut self) {
        if self.is_initialized() {
            // SAFETY: The cell is initialized and being dropped, so it can't
            // be accessed again. We also don't touch the `T` other than
            // dropping it, which validates our usage of #[may_dangle].
            unsafe { (&mut *self.value.get()).assume_init_drop() };
        }
    }
}

/*
#[cfg(test)]
mod tests;
*/
