use libc::{
    c_void, mmap, memcpy, munmap, MAP_ANONYMOUS, MAP_FAILED, MAP_PRIVATE, PROT_EXEC, PROT_READ, PROT_WRITE, 
};
use std::ptr;

pub struct CodeVec {
    buffer: *mut u8,
    length: usize,
    capacity: usize,
    page_size: usize,
}

unsafe fn alloc_buffer(size: usize) -> Option<*mut u8> {
    let buffer = mmap(
        ptr::null_mut(),
        size,
        PROT_READ | PROT_WRITE | PROT_EXEC,
        MAP_PRIVATE | MAP_ANONYMOUS,
        -1,
        0,
    );

    if buffer == MAP_FAILED {
        None
    } else {
        Some(buffer as *mut u8)
    }
}

impl CodeVec {
    pub fn new(page_size: usize) -> Self {
        let buffer = unsafe {alloc_buffer(page_size)}.expect("memory allocation failed");

        CodeVec {
            buffer,
            page_size,
            capacity: page_size,
            length: 0
        }
    }

    pub fn push(&mut self, byte: u8) {
        if self.length == self.capacity {
            self.capacity *= 2;
            unsafe {
                let buffer = alloc_buffer(self.capacity).expect("memory allocation failed");
                memcpy(buffer as *mut c_void, self.buffer as *const c_void, self.length);
                munmap(self.buffer as *mut c_void, self.length);
                self.buffer = buffer;
            }
        }

        unsafe {
            *self.buffer.offset(self.length as isize) = byte;
        }

        self.length += 1;
    }

    /// the memory returned from `as_ptr` should be suitable for direct execution
    pub fn as_ptr(&self) -> *const u8 {
        self.buffer
    }
}

impl Drop for CodeVec {
    fn drop(&mut self) {
        unsafe {
            munmap(self.buffer as *mut c_void, self.capacity);
        }
    }
}