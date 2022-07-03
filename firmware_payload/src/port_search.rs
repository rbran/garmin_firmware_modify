#![no_std]
#![no_main]

use core::mem::transmute;
use core::panic::PanicInfo;

mod garmin;
use garmin::*;

//Trigger file
const TRI_FILE: *const u8 = "0:\\a.tri\x00".as_ptr();

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
        unsafe {
            const GPIOA_PDDR: *mut u32 =
                unsafe { transmute::<usize, *mut u32>(0x400ff014) };

            const GPIOA_PTOR: *mut u32 =
                unsafe{ transmute::<usize, *mut u32>(0x400ff00c) };
            const GPIOA_PSOR: *mut u32 =
                unsafe{ transmute::<usize, *mut u32>(0x400ff004) };
            const GPIOA_PCOR: *mut u32 =
                unsafe{ transmute::<usize, *mut u32>(0x400ff008) };

            ////porta0-3
            const PORTA_BASE: u32 = 0x40049000;

            //save the gpio pddr
            let old_pddr = *GPIOA_PDDR;

            const PINS_BITS: u32 = 0xffffffff;
            const PINS_NUM: usize = 32;

            //set the gpio to output
            *GPIOA_PDDR = old_pddr | PINS_BITS;

            //save old pcr values
            //new pcr
            const NEW_PCR: u32 = 0 << 6 | //drive strength high
                       0b001 << 8 | //alternative 1 gpio
                       0b0000 << 16 | //diable interrupt
                       0 << 24; //dma ignore interrup
            let mut old_pcr = [0; PINS_NUM];
            for (i, old_pcr) in old_pcr.iter_mut().enumerate() {
                let pcr =
                    transmute::<u32, *mut u32>(PORTA_BASE + (i as u32 * 4));
                *old_pcr = *pcr;
                *pcr = NEW_PCR;
            }

            //set all pins to high
            *GPIOA_PSOR = PINS_BITS;
            sleep(10);
            //set all pins to low
            *GPIOA_PCOR = PINS_BITS;
            sleep(10);

            for i in 0..PINS_NUM {
                // pin h/l in a identifiable delay
                let id = (i + 1) * 2;
                *GPIOA_PSOR = 1 << i;
                sleep(id);
                *GPIOA_PCOR = 1 << i;
                sleep(id);

                //cycle h/l in a identifiable number or times
                for _ in 0..id {
                    *GPIOA_PTOR = 1 << i;
                    sleep(2);
                }
                *GPIOA_PCOR = 1 << i;
                sleep(2);

                // pin h/l in a identifiable delay
                *GPIOA_PSOR = 1 << i;
                sleep(id);
                *GPIOA_PCOR = 1 << i;
                sleep(id);
            }

            //invert gpio HIGH/LOW
            //for i in 0..8192 {
            //    let mut pins = 1;
            //    for pin in 1..32 { //avoid 0 div
            //        if i % pin == 0 {
            //            pins |= 1 << pin;
            //        }
            //    }
            //    *GPIOA_PTOR = pins;
            //    sleep(1);
            //}

            //restore PORTA PCR
            for (i, old_pcr) in old_pcr.iter().enumerate() {
                let pcr =
                    transmute::<u32, *mut u32>(PORTA_BASE + (i as u32 * 4));
                *pcr = *old_pcr;
            }

            //restore GPIO PDDR
            *GPIOA_PDDR = old_pddr;
        }
    }
    0
}

fn sleep(cycles: usize) {
    for _ in 0..cycles {
        sleep_30000();
    }
}

#[allow(dead_code)]
fn sleep_30000() {
    unsafe {
        transmute::<usize, extern "C" fn()>(0x69098 + 1)();
    }
}

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}
