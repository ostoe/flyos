#![no_std]

#![feature(abi_x86_interrupt)]

#![feature(const_mut_refs)]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

#![feature(alloc_error_handler)]

// 在const函数中使用可变引用不稳定，添加才能编译
// #![feature(const_mut_refs)]

#![feature(alloc_layout_extra)]

// #![feature(const_in_array_repeat_expressions)]

extern crate alloc;

use core::{panic::PanicInfo, alloc::Layout};
use allocator::{bump::BumpAllocator, Locked, linked_list::LinkedListAllocator, fixed_size_block::FixedSizeBlockAllocator};
use linked_list_allocator::LockedHeap;

pub mod serial;
pub mod vga_buffer;
// pub mod should_panic;
pub mod interrupts;
pub mod gdt;
pub mod memory;
pub mod allocator;

// future async/await
pub mod task;

#[global_allocator]
// static ALLOCATOR: allocator::Dummy = allocator::Dummy;
// static ALLOCATOR: LockedHeap = LockedHeap::empty();
// static ALLOCATOR: allocator::Locked<BumpAllocator>= allocator::Locked::new(BumpAllocator::new());
// static ALLOCATOR: Locked<LinkedListAllocator> =  Locked::new(LinkedListAllocator::new());
static ALLOCATOR: Locked<FixedSizeBlockAllocator> = Locked::new(FixedSizeBlockAllocator::new());


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;
    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}



pub fn test_panic_handler(info: &PanicInfo) ->! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

pub fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
    exit_qemu(QemuExitCode::Success);
}


/// Entry point for `cargo xtest`
#[cfg(test)]
#[no_mangle]

#[cfg(test)]
use bootloader::{entry_point, BootInfo};
use pc_keyboard::layouts;

#[cfg(test)]
entry_point!(test_kernal_main);

#[cfg(test)]
pub fn test_kernal_main(_boot_info: &'static BootInfo) -> ! {
    test_main();
    init();
    hlt_loop();
}



#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

//让 CPU 停下来，直到下一个中断到达。这允许 CPU 进入休眠状态
pub fn hlt_loop() -> !{
    loop {
        x86_64::instructions::hlt();
    }
}


pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable(); // 打开了CPU的硬件定时器；能够接受中断；

}





/*
---------------------------
    test
---------------------------
 */


