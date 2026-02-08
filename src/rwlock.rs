use core::{
    sync::atomic::{Ordering, AtomicI32},
    ops::{Deref, DerefMut},
    cell::UnsafeCell
};
use std::thread::yield_now;

pub struct RwLock<T> {
    state: AtomicI32,
    value: UnsafeCell<T>
}
impl<T> RwLock<T> {
    pub const fn new(value: T) -> Self {
        Self {
            state: AtomicI32::new(0),
            value: UnsafeCell::new(value)
        }
    }
    pub fn read(&self) -> RwLockReadGuard<'_, T> {
        while let Err(_) = self.state.fetch_update(
            Ordering::Relaxed,
            Ordering::Relaxed,
            |state| if state == -1 {None} else {Some(state + 1)}
        ) {yield_now();}
        RwLockReadGuard {
            lock: self
        }
    }
    pub fn write(&self) -> RwLockWriteGuard<'_, T> {
        while let Err(_) = self.state.fetch_update(
            Ordering::Relaxed,
            Ordering::Relaxed,
            |state| if state == 0 {Some(-1)} else {None}
        ) {yield_now();}
        RwLockWriteGuard {
            lock: self
        }
    }
}
unsafe impl<T: Send> Send for RwLock<T> {}
unsafe impl<T: Sync> Sync for RwLock<T> {}

pub struct RwLockReadGuard<'a, T> {
    lock: &'a RwLock<T>
}
impl<'a, T> Deref for RwLockReadGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.value.get() }
    }
}
impl<'a, T> Drop for RwLockReadGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.state.fetch_sub(1, Ordering::Relaxed);
    }
}

pub struct RwLockWriteGuard<'a, T> {
    lock: &'a RwLock<T>
}
impl<'a, T> Deref for RwLockWriteGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.value.get() }
    }
}
impl<'a, T> DerefMut for RwLockWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.value.get() }
    }
}
impl<'a, T> Drop for RwLockWriteGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.state.store(0, Ordering::Relaxed);
    }
}