//! Synchronization types for FreeRTOS tasks.
//!
//! Types implemented here are specificially designed to mimick the standard library.

use core::{cell::UnsafeCell, fmt::Debug, future::Future, sync::atomic::AtomicU8};

use snafu::Snafu;

const MUTEX_STATUS_OPEN: u8 = 0b0;
const MUTEX_STATUS_LOCKED: u8 = 0b1;
const MUTEX_STATUS_POISONED: u8 = 0b10;

/// The basic mutex type.
/// Mutexes are used to share variables between tasks safely.
pub struct Mutex<T> {
    status: AtomicU8,
    data: UnsafeCell<T>,
}
unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    /// Creates a new mutex.
    pub fn new(data: T) -> Self {
        Self {
            status: AtomicU8::new(0),
            data: UnsafeCell::new(data),
        }
    }

    pub fn poll_lock(&self) -> Result<Option<MutexGuard<T>>, MutexError> {
        let status = self.status.load(core::sync::atomic::Ordering::Acquire);
        if status & MUTEX_STATUS_POISONED != 0 {
            return Err(MutexError::Poisoned);
        }

        if status & MUTEX_STATUS_LOCKED != 0 {
            return Ok(None);
        }

        self.status
            .store(MUTEX_STATUS_LOCKED, core::sync::atomic::Ordering::Release);

        Ok(Some(MutexGuard { mutex: self }))
    }

    /// Locks the mutex so that it cannot be locked in another task at the same time.
    /// Blocks the current task until the lock is acquired.
    pub fn lock_blocking(&self) -> Result<MutexGuard<T>, MutexError> {
        let status = self.status.load(core::sync::atomic::Ordering::Acquire);
        if status & MUTEX_STATUS_POISONED != 0 {
            return Err(MutexError::Poisoned);
        }

        if status & MUTEX_STATUS_LOCKED != 0 {
            loop {
                let status = self.status.load(core::sync::atomic::Ordering::Acquire);
                if status & MUTEX_STATUS_POISONED != 0 {
                    return Err(MutexError::Poisoned);
                }

                if status & MUTEX_STATUS_LOCKED == 0 {
                    break;
                }
            }
        }

        Ok(MutexGuard { mutex: self })
    }

    pub fn lock(&self) -> MutexLockFuture<T> {
        MutexLockFuture { mutex: self }
    }

    pub fn into_inner(self) -> T {
        let data = self.data;
        data.into_inner()
    }

    pub fn get_mut(&mut self) -> &mut T {
        self.data.get_mut()
    }
}

impl<T> Debug for Mutex<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut d = f.debug_struct("Mutex");
        match self.poll_lock() {
            Ok(Some(guard)) => d.field("data", &&*guard),
            Ok(None) => d.field("data", &"<locked>"),
            Err(_) => d.field("data", &"<poisoned>"),
        };
        d.finish_non_exhaustive()
    }
}

impl<T> Default for Mutex<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T> From<T> for Mutex<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

#[derive(Debug)]
pub struct MutexLockFuture<'a, T> {
    mutex: &'a Mutex<T>,
}
impl<'a, T> Future for MutexLockFuture<'a, T> {
    type Output = Result<MutexGuard<'a, T>, MutexError>;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        match self.mutex.poll_lock() {
            Ok(Some(guard)) => core::task::Poll::Ready(Ok(guard)),
            Ok(None) => {
                cx.waker().wake_by_ref();
                core::task::Poll::Pending
            }
            Err(err) => core::task::Poll::Ready(Err(err)),
        }
    }
}

/// Allows the user to access the data from a locked mutex.
/// Dereference to get the inner data.
pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
}

impl<T> core::ops::Deref for MutexGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<T> core::ops::DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        // TODO: This currently does not check for if the thread is panicking, so mutexes cannot be poisoned.
        self.mutex
            .status
            .store(MUTEX_STATUS_OPEN, core::sync::atomic::Ordering::Release);
    }
}

#[derive(Snafu, Debug)]
pub enum MutexError {
    #[snafu(display("Mutex poisoned"))]
    Poisoned,
}
