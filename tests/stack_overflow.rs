#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use lazy_static::lazy_static;
use core::panic::PanicInfo;
use ham_dos::{exit_qemu, serial_println,serial_print, QemuExitCode};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault
                .set_handler_fn(test_double_fault_handler)
                .set_stack_index(ham_dos::gdt::DOUBLE_FAULT_IST_INDEX);
        }

		idt
    };
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_print!("stack_overflow... ");
    ham_dos::gdt::init();

    init_test_idt();

    // trigger a stack overflow
    stack_overflow();
    panic!("Execution continued after stack overflow");
}

#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    ham_dos::test_panic_handler(info)
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow();
}

fn init_test_idt() {
    TEST_IDT.load();
}

extern "x86-interrupt" fn test_double_fault_handler(
    _stack_frame: &mut InterruptStackFrame,
    _error_code: u64,
) {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);

    loop {}
}
