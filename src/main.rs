#![no_std]
#![no_main]
#![feature(asm)]
#![reexport_test_harness_main = "test_main"]

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // The custom test frameworks feature generates a main function that
    // calls test_runner, but this function is ignored because we use
    // the #[no_main] attribute and provide our own entry point.
    #[cfg(test)]
    test_main(); // Generated

    unsafe {
        asm!("hlt");
    }
    panic!("DIE!");
}
