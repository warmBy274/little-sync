use core::{
    sync::atomic::{Ordering, AtomicBool},
    ops::{Deref, DerefMut},
    cell::UnsafeCell
};
use std::thread::yield_now;

pub struct Mutex<T> {
    locked: AtomicBool,
    value: UnsafeCell<T>
}
impl<T> Mutex<T> {
    pub fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            value: UnsafeCell::new(value)
        }
    }
    pub fn lock(&self) -> MutexGuard<'_, T> {
        while let Err(_) = self.locked.fetch_update(
            Ordering::Relaxed,
            Ordering::Relaxed,
            |locked| if locked {None} else {Some(true)}
        ) {yield_now();}
        MutexGuard {
            mutex: self
        }
    }
}
unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T: Sync> Sync for Mutex<T> {}

pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>
}
impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.value.get() }
    }
}
impl<'a, T> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mutex.value.get() }
    }
}
impl<'a, T> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        self.mutex.locked.store(false, Ordering::Relaxed);
    }
}