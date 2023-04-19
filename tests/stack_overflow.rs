#![feature(abi_x86_interrupt)]

#![no_std]
#![no_main]

use core::panic::PanicInfo;

use flyos::{serial_println, exit_qemu};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};


lazy_static::lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault
                .set_handler_fn(test_double_fault_handler)
                .set_stack_index(flyos::gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt
    };
}

pub extern "x86-interrupt" fn test_double_fault_handler(
    stack_frame: InterruptStackFrame, error_code: u64) -> !{
        serial_println!("[ok]!");
        exit_qemu(flyos::QemuExitCode::Success);
        loop {}
}

#[no_mangle]
pub extern "C" fn _start() {
    serial_println!("stack overflow");

    flyos::gdt::init();
    init_test_gdt();

    stack_overflow();

    panic!("Execution continued after stack overflow");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    flyos::test_panic_handler(info)
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow();
    volatile::Volatile::new(0).read();
}

fn init_test_gdt() {
    TEST_IDT.load();
}