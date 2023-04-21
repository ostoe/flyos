#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(flyos::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::{boxed::Box, rc::Rc, vec, vec::Vec};
use bootloader::{entry_point, BootInfo};
use flyos::serial_println;
use core::panic::PanicInfo;

entry_point!(main);


fn main(_boot_info: &'static BootInfo) -> ! {
    use flyos::allocator;
    use flyos::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;
    flyos::init();
    // 创建映射实例，
    let phys_mem_offset = VirtAddr::new(_boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    // 初始化内存分配器
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&_boot_info.memory_map)
    };
    // 初始化堆 分配器！
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("初始化100kib堆失败");
    
    test_main();
    loop {

    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    flyos::test_panic_handler(info)
}

#[test_case]
fn simple_allocation() {
    serial_println!("simple allocation.");
    let heap_value = Box::new(41);
    assert_eq!(*heap_value, 41);
    serial_println!("[OK]");
}

#[test_case]
fn large_vec() {
    serial_println!("large vec");
    let n = 4096;
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i);
    }
    assert_eq!(vec.iter().sum::<u64>(), (n-1)*n/2);
    serial_println!("[ok]");
}

#[test_case]
fn many_boxes() {
    serial_println!("many boxes");
    for i in 0..10_000 {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
    serial_println!("[ok]");
}