/*
    Small helper for setting up global tables such as transposition tables/ history tables/ magics etc,
    Safety:
    Dont call get without having called init
    Dont call init twice
*/

use std::cell::UnsafeCell;

unsafe impl<T> std::marker::Sync for GlobalTable<T> {}

pub struct GlobalTable<T> {
    val: UnsafeCell<Vec<T>>,
}

impl<T> GlobalTable<T>
where
    T: Copy,
{
    pub const fn new() -> GlobalTable<T> {
        GlobalTable {
            val: UnsafeCell::new(Vec::new()),
        }
    }

    pub unsafe fn init(&self, mut input: Vec<T>) {
        let vec = unsafe { &mut (*self.val.get()) };
        vec.append(&mut input);
    }

    pub unsafe fn get(&self, index: usize) -> Option<&T> {
        unsafe { &(*self.val.get()) }.get(index)
    }

    pub unsafe fn get_unchecked(&self, index: usize) -> &T {
        unsafe { &(*self.val.get()) }.get_unchecked(index)
    }

    pub fn len(&self) -> usize {
        unsafe { (*self.val.get()).len() }
    }
}
