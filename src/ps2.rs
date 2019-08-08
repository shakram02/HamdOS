use spin::Mutex;
use x86_64::instructions::port::Port;

use lazy_static::lazy_static;

use crate::print;

const CMD_READ_CONFIG_BYTE: u8 = 0x20;
const CMD_WRITE_CONFIG_BYTE: u8 = 0x60;
const CMD_ENABLE_FIRST_PORT: u8 = 0xAE;
const CMD_DISABLE_FIRST_PORT: u8 = 0xAD;
const CMD_ENABLE_SECOND_PORT: u8 = 0xA8;
const CMD_DISABLE_SECOND_PORT: u8 = 0xA7;
const CMD_TEST_PS2_CONTROLLER: u8 = 0xAA;
const CMD_TEST_PS2_FIRST_PORT: u8 = 0xAB;
const CMD_TEST_PS2_SECOND_PORT: u8 = 0xA9;
const CMD_DISABLE_SCANNING: u8 = 0xF5;
const PORT_NO_COMMAND: u16 = 0x64;
const PORT_NO_DATA: u16 = 0x60;
const REPLY_CONTROLLER_TEST_PASS: u8 = 0x55;
const REPLY_DEVICE_ACK: u8 = 0xFA;

lazy_static! {
    static ref PS2: Mutex<Ps2Controller> = Mutex::new(Ps2Controller::new());
}

pub fn init() {
    // PS/2 controller initialization https://wiki.osdev.org/%228042%22_PS/2_Controller
    let controller: &mut Ps2Controller = &mut PS2.lock();
    controller.disable_first_device(); // Turns out to be the keyboard
    controller.disable_second_device();
    controller.flush_output_buffer();
    controller.disable_irqs_and_translation();
    controller.perform_self_test();
    controller.perform_interface_tests();
    controller.enable_devices_and_translation();
    controller.enable_irqs();
}

enum CommandDestination {
    Device,
    Controller,
}

struct Ps2Controller {
    command_port: Port<u8>,
    data_port: Port<u8>,
    is_dual_channel: bool,
}

impl Ps2Controller {
    pub const fn new() -> Ps2Controller {
        Ps2Controller {
            command_port: Port::new(PORT_NO_COMMAND),
            data_port: Port::new(PORT_NO_DATA),
            is_dual_channel: false,
        }
    }

    fn disable_first_device(&mut self) {
        self.write_command(CMD_DISABLE_FIRST_PORT);
    }

    fn disable_second_device(&mut self) {
        self.write_command(CMD_DISABLE_SECOND_PORT);
    }

    fn can_read_data(&mut self) -> bool {
        (self.read_status() & 0x1) != 0
    }

    fn can_write_data(&mut self) -> bool {
        (self.read_status() & 0x1 << 1) != 0
    }

    fn get_write_destination(&mut self) -> CommandDestination {
        if (self.read_status() & 0x1 << 3) != 0 {
            CommandDestination::Controller
        } else { CommandDestination::Device }
    }

    /// Disables interrupt for all devices and clears translation
    /// Represents step #5 in PS/2 controller initialization
    /// Also determines if the PS/2 controller is dual channel
    /// by setting is_dual_channel
    fn disable_irqs_and_translation(&mut self) {
        let mut config = self.read_configuration_byte();
        // Disable first and second device interrupts and disable translation [0,1,6]
        config = config & 0b10111100;

        // Check for second port enabled
        // Second PS/2 port clock (1 = disabled, 0 = enabled, only if 2 PS/2 ports supported)
        if (config & 0x10) != 0 {
            self.is_dual_channel = true;
        }

        self.write_configuration_byte(config);
    }

    /// Enable any PS/2 port that exists and works.
    /// If you're using IRQs (recommended), also enable interrupts
    /// for any PS/2 ports in the Controller Configuration Byte.
    fn enable_devices_and_translation(&mut self) {
        // If any device failed, we would've paniced
        self.write_command(CMD_ENABLE_FIRST_PORT);
        self.write_command(CMD_ENABLE_SECOND_PORT);
        let mut config = self.read_configuration_byte();
        config |= 0b01000000;
        self.write_command(CMD_WRITE_CONFIG_BYTE);
        self.write_data(config);
    }

    fn enable_irqs(&mut self) {
        let mut config = self.read_configuration_byte();
        // IRQs are on bits 0,1 and enable translation 6
        config |= if self.is_dual_channel {
            0b00000011  // Enable First and Second ports
        } else {
            0b00000001  // Enable First port
        };

        self.write_command(CMD_WRITE_CONFIG_BYTE);
        self.write_data(config);
    }

    /// Performs controller self test and panics
    /// if the test failed
    fn perform_self_test(&mut self) {
        let config = self.read_configuration_byte();

        self.write_command(CMD_TEST_PS2_CONTROLLER);
        let first_port_status = self.read_data();
        if first_port_status != REPLY_CONTROLLER_TEST_PASS {
            panic!("PS/2 controller is malfunctioning")
        }

        // For the devices that may reset
        self.write_configuration_byte(config)
    }

    /// Performs device test and panics
    /// if the test failed
    fn perform_interface_tests(&mut self) {
        self.write_command(CMD_TEST_PS2_FIRST_PORT);
        if self.read_data() != 0 {
            panic!("First PS/2 device failed")
        }

        if !self.is_dual_channel { return; }

        self.write_command(CMD_TEST_PS2_SECOND_PORT);
        let response = self.read_data();
        if response!= 0 {
            panic!("Second PS/2 device failed")
        }
    }

    fn read_configuration_byte(&mut self) -> u8 {
        self.write_command(CMD_READ_CONFIG_BYTE);
        self.read_data()
    }

    fn write_configuration_byte(&mut self, config: u8) {
        self.write_command(CMD_WRITE_CONFIG_BYTE);
        self.write_data(config);
    }

    fn write_command(&mut self, cmd: u8) {
        self.wait_till_input_buffer_empty();
        self.write_direct(cmd)
    }

    fn read_status(&mut self) -> u8 {
        self.wait_till_out_buffer_full();
        self.read_direct()
    }

    fn wait_till_out_buffer_full(&mut self) {
        while (self.read_direct() & 0x1) == 0 {}
    }

    fn wait_till_input_buffer_empty(&mut self) {
        while (self.read_direct() & 0x1 << 1) != 0 {}
    }

    fn read_direct(&mut self) -> u8 {
        unsafe { self.command_port.read() }
    }

    fn write_direct(&mut self, cmd: u8) {
        unsafe { self.command_port.write(cmd); }
    }

    fn write_data(&mut self, data: u8) {
        self.wait_till_input_buffer_empty();
        unsafe { self.data_port.write(data) }
    }

    fn read_data(&mut self) -> u8 {
        self.wait_till_out_buffer_full();
        unsafe { self.data_port.read() }
    }

    /// Forces flush of controller's output buffer
    fn flush_output_buffer(&mut self) {
        unsafe {
            while (self.command_port.read() & 0x1) != 0 {
                self.data_port.read();
            }
        }
    }
}
