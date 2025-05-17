use core::cell::{RefCell, RefMut};

pub struct UPSafeCell<T> {
    inner: RefCell<T>,
}

unsafe impl<T> Sync for UPSafeCell<T> {}

impl<T> UPSafeCell<T> {
    // User is responsible to guarantee that inner struct is only used in
    // uniprocessor
    pub unsafe fn new(value: T) -> Self {
        UPSafeCell {
            inner: RefCell::new(value),
        }
    }

    // Always get mutable reference, so it will panic
    // if the data has been borrow twice
    pub fn exclusive_access(&self) -> RefMut<'_, T> {
        self.inner.borrow_mut()
    }
}
