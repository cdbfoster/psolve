use std::cell::UnsafeCell;
use std::fmt;
use std::ptr;

#[repr(transparent)]
pub struct Volatile<T>(UnsafeCell<T>);

impl<T> Volatile<T> {
    pub fn new(value: T) -> Self {
        Self(UnsafeCell::new(value))
    }
}

impl<T> Volatile<T>
where
    T: Copy,
{
    pub fn read(&self) -> T {
        unsafe { ptr::read_volatile(self.0.get()) }
    }

    pub fn write(&self, value: T) {
        unsafe {
            ptr::write_volatile(self.0.get(), value);
        }
    }
}

impl<T> fmt::Debug for Volatile<T>
where
    T: Copy + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Volatile").field(&self.read()).finish()
    }
}
