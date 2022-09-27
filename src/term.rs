use std::ffi::c_uint;

#[repr(C)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct TermSize {
    pub width: c_uint,
    pub height: c_uint,
}

impl TermSize {
    pub fn get() -> Self {
        unsafe {
            get_term_size()
        }
    }
}

pub fn put(ch: char) {
    unsafe {
        term_putchar(ch as u8);
    }
}

pub fn put_line(line: &[u8]) {
    unsafe {
        term_put_line(line.as_ptr(), line.len());
    }
}

extern "C" {
    fn get_term_size() -> TermSize;
    fn term_putchar(c: u8);
    fn term_put_line(line: *const u8, len: usize);
}
