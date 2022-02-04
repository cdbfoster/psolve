use std::fmt;
use std::mem::{self, MaybeUninit};
use std::ptr;

pub struct Arena {
    buffer: *mut u8,
    cur: *mut u8,
    capacity: usize,
    len: usize,
}

impl Arena {
    pub fn with_capacity(bytes: usize) -> Self {
        let mut vec = Vec::with_capacity(bytes);

        let arena = Self {
            buffer: vec.as_mut_ptr(),
            cur: vec.as_mut_ptr(),
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

    pub fn allocate<T>(&mut self, n: usize) -> Result<*mut MaybeUninit<T>, Error> {
        let offset = self.cur.align_offset(mem::align_of::<T>());
        let size = n * mem::size_of::<T>();
        let new_len = self.len + offset + size;

        if self.capacity >= new_len {
            let ptr = unsafe { self.cur.offset(offset as isize) };
            self.cur = unsafe { ptr.offset(size as isize) };
            self.len = new_len;
            Ok(ptr as *mut MaybeUninit<T>)
        } else {
            Err(Error::OutOfMemory)
        }
    }
}

impl Drop for Arena {
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

pub struct DummyArena {
    cur: *mut u8,
    capacity: Option<usize>,
    len: usize,
}

impl DummyArena {
    pub fn with_capacity(bytes: usize) -> Self {
        Self {
            cur: ptr::null_mut(),
            capacity: Some(bytes),
            len: 0,
        }
    }

    pub fn infinite() -> Self {
        Self {
            cur: ptr::null_mut(),
            capacity: None,
            len: 0,
        }
    }

    pub fn capacity(&self) -> Option<usize> {
        self.capacity
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn allocate<T>(&mut self, n: usize) -> Result<(), Error> {
        let offset = self.cur.align_offset(mem::align_of::<T>());
        let size = n * mem::size_of::<T>();
        let new_len = self.len + offset + size;

        if self.capacity.is_none() || self.capacity.unwrap() >= new_len {
            let ptr = self.cur.wrapping_offset(offset as isize);
            self.cur = ptr.wrapping_offset(size as isize);
            self.len = new_len;
            Ok(())
        } else {
            Err(Error::OutOfMemory)
        }
    }
}
