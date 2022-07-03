#![no_std]
#![no_main]

use core::mem::transmute;
use core::panic::PanicInfo;

mod garmin;
use garmin::*;

const TRI_FILE: *const u8 = "0:\\garmin\\debug\\payload.tri\x00".as_ptr();
const CODE_FILE: *const u8 = "0:\\code.bin\x00".as_ptr();
const OCRAM_FILE: *const u8 = "0:\\ocram.bin\x00".as_ptr();
const RAM_FILE: *const u8 = "0:\\ram.bin\x00".as_ptr();
const ROM_FILE: *const u8 = "0:\\rom.bin\x00".as_ptr();

#[no_mangle]
pub extern "C" fn _start() -> usize {
    //init the struct like the original function
    let print_stuff = unsafe { transmute::<u32, *mut u8>(0x1fff7974) };
    memset(print_stuff, 0xff, 0x3f);

    let fd = open_fd(TRI_FILE, 0x20);
    if fd >= 0 {
        close_fd(fd);
        //TODO delete file works??
        delete_file(TRI_FILE);
        //TODO 0x43 create the file?
        for &(file, start_addr, step, end_addr) in &[
            (CODE_FILE, 0x0000_0000, 1024, 0x0020_0000),
            (OCRAM_FILE, 0x3400_0000, 1024, 0x3408_0000),
            (RAM_FILE, 0x1FFC_0000, 1024, 0x2004_0000),
            (ROM_FILE, 0x1C00_0000, 1024, 0x1C00_8000),
        ] {
            let fd = open_fd(file, 0x42);
            if fd >= 0 {
                //write the first 0x200_0000 bytes to the files, 1024 each time
                let mut addr = start_addr;
                while addr < end_addr {
                    let block = unsafe {
                        transmute::<u32, *const u8>(addr)
                    };
                    if write_fd(fd, block, step) != step {
                        break;
                    }
                    addr += step;
                }
                close_fd(fd);
            }
        }
    }
    0
}

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}
