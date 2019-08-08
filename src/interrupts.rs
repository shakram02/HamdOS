use pic8259_simple::ChainedPics;
use spin;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use lazy_static::lazy_static;

use crate::gdt;
use crate::interrupts::InterruptIndex::Timer;
use crate::print;
use crate::println;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);

            idt[InterruptIndex::Timer.as_usize()]
                .set_handler_fn(timer_interrupt_handler);
        }

        idt
    };
}

pub fn init_idt() {
    println!("LOADING IDT");
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

// Error code is always 0, we don't need it
extern "x86-interrupt" fn double_fault_handler(stack_frame: &mut InterruptStackFrame, _: u64) {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

// Interrupt controllers
pub const PIC_1_OFFSET: u8 = 32;
// First empty interrupt number
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;    // First empty interrupt number

pub static PICS: spin::Mutex<ChainedPics> = spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
enum InterruptIndex {
    Timer = PIC_1_OFFSET,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    print!(".");
    unsafe {
        PICS.lock().notify_end_of_interrupt(Timer.as_u8());
    }
}
