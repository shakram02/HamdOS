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
            idt[InterruptIndex::Keyboard.as_usize()]
                .set_handler_fn(keyboard_interrupt_handler);
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
    Keyboard,
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

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    use x86_64::instructions::port::Port;
    use spin::Mutex;
    use pc_keyboard::{Keyboard, layouts, ScancodeSet1, DecodedKey};

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1));
    }

    let keyboard: &mut Keyboard<layouts::Us104Key, ScancodeSet1> = &mut KEYBOARD.lock();
    // Data port of PS/2 controller https://wiki.osdev.org/%228042%22_PS/2_Controller
    let mut keyboard_port = Port::new(0x60);
    let scan_code = unsafe { keyboard_port.read() };

    if let Ok(Some(key_event)) = keyboard.add_byte(scan_code) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    unsafe {
        let pics: &mut ChainedPics = &mut PICS.lock();  // Just for auto complete
        pics.notify_end_of_interrupt(Timer.as_u8());
    }
}
