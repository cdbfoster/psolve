use std::fmt;
use std::mem::{self, MaybeUninit};
use std::ptr;

pub struct Arena<T> {
    buffer: *mut MaybeUninit<T>,
    capacity: usize,
    len: usize,
}

impl<T> Arena<T> {
    pub fn with_capacity(capacity: usize) -> Self {
        let mut vec = Vec::with_capacity(capacity);

        let arena = Self {
            buffer: vec.as_mut_ptr(),
            capacity: vec.capacity(),
            len: 0,
        };
        mem::forget(vec);

        arena
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn allocate(&mut self, n: usize) -> Result<*mut MaybeUninit<T>, Error> {
        if self.capacity >= self.len + n {
            let ptr = unsafe { self.buffer.offset(self.len as isize) };
            self.len += n;
            Ok(ptr)
        } else {
            Err(Error::OutOfMemory)
        }
    }

    /// Unsafe because we can't guarantee that nobody still has a pointer into us,
    /// or that every T we handed out was initialized.
    pub unsafe fn drop_all(this: Self) {
        let mut vec = Vec::from_raw_parts(this.buffer, this.len, this.capacity);

        for item in &mut vec {
            ptr::drop_in_place(item.as_mut_ptr());
        }

        mem::forget(vec); // We'll let Arena's drop code deallocate the buffer.
    }
}

impl<T> Drop for Arena<T> {
    fn drop(&mut self) {
        let _vec = unsafe { Vec::from_raw_parts(self.buffer, self.len, self.capacity) };
    }
}

#[derive(Debug)]
pub enum Error {
    OutOfMemory,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}
