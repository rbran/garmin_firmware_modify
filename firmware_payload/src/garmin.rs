use core::mem::transmute;

#[allow(dead_code)]
pub fn open_fd(filename: *const u8, flag: u32) -> i32 {
    unsafe {
        transmute::<usize, extern "C" fn(*const u8, u32) -> i32>(
            0x8b800 + 1,
        )(filename, flag)
    }
}

#[allow(dead_code)]
pub fn close_fd(fd: i32) -> u32 {
    unsafe {
        transmute::<usize, extern "C" fn(i32) -> u32>(
            0x8b3f0 + 1,
        )(fd)
    }
}

#[allow(dead_code)]
pub fn read_fd(fd: i32, buf: *const u8, size: u32) -> i32 {
    unsafe {
        transmute::<usize, extern "C" fn(i32, *const u8, u32) -> i32>(
            0x8b9a8 + 1,
        )(fd, buf, size)
    }
}

#[allow(dead_code)]
pub fn write_fd(fd: i32, buf: *const u8, size: u32) -> u32 {
    unsafe {
        transmute::<usize, extern "C" fn(i32, *const u8, u32) -> u32>(
            0x8bbf0 + 1,
        )(fd, buf, size)
    }
}

#[allow(dead_code)]
pub fn memcpy(src: *const u8, dst: *mut u8, size: u32) -> *mut u8 {
    unsafe {
        transmute::<usize, extern "C" fn(*const u8, *mut u8, u32) -> *mut u8>(
            0xeefbc + 1,
        )(src, dst, size)
    }
}

#[allow(dead_code)]
pub fn memset(s: *mut u8, c: u32, n: u32) -> *mut u8 {
    unsafe {
        transmute::<usize, extern "C" fn(*mut u8, u32, u32) -> *mut u8>(
            0xef1b8 + 1,
        )(s, c, n)
    }
}

#[allow(dead_code)]
pub fn move_file(src: *const u8, dst: *const u8) -> u32 {
    unsafe {
        transmute::<usize, extern "C" fn(*const u8, *const u8) -> u32>(
            0x8ba9c + 1,
        )(src, dst)
    }
}

#[allow(dead_code)]
pub fn delete_file(file: *const u8) -> u32 {
    unsafe {
        transmute::<usize, extern "C" fn(*const u8) -> u32>(
            0x8bb74 + 1,
        )(file)
    }
}
