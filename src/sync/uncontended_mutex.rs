use super::{Mutex, MutexGuard};

/// A mutex that panics immediately upon any contention.
///
/// This is intended to be used for implementing shared buffers that are
/// specified without requirements on thread safety, as a safer alternative to
/// `static mut`. By panicking when the lock is already held, we can reliably
/// panic when something bad would otherwise happen, without waiting or
/// masking the user's erroneous usage of the contents.
pub struct UncontendedMutex<T> {
    inner: Mutex<T>,
}

impl<T> UncontendedMutex<T> {
    pub const fn new(content: T) -> Self {
        Self {
            inner: Mutex::new(content),
        }
    }

    pub fn lock(&self) -> MutexGuard<T> {
        // The error message reflects the misuse that UncontendedMutex is intended to protect against
        self.inner
            .try_lock()
            .expect("attempted unsafe multithreaded access")
    }
}
