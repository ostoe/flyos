#![no_std]
#![no_main]
#![reexport_test_harness_main = "test_main"]
#![feature(custom_test_frameworks)]
#![test_runner(flyos::test_runner)]


use core::panic::PanicInfo;
use bootloader::entry_point;
use bootloader::BootInfo;
use flyos::{test_panic_handler,print, println, serial_println };


entry_point!(kernel_main);



fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // use blog_os::allocator;
    // use blog_os::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    println!("Hello World{}", "!");
    flyos::init();
    println!("had init");
    x86_64::instructions::interrupts::int3();

    // let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    // let mut mapper = unsafe { memory::init(phys_mem_offset) };
    // let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    // allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    #[cfg(test)]
    test_main();
    println!("---end---");
    panic!("info")
    // let mut executor = Executor::new();
    // executor.spawn(Task::new(example_task()));
    // executor.spawn(Task::new(keyboard::print_keypresses()));
    // executor.run();
}

// 我们使用no_mangle标记这个函数，来对它禁用名称重整（name mangling）——这确保Rust编译器输出一个名为_start的函数；否则，编译器可能最终生成名为_ZN3blog_os4_start7hb173fedf945531caE的函数，无法让链接器正确辨别。
// #[no_mangle] 
// pub extern "C" fn _start(bootinfo: &'static bootloader::BootInfo) -> ! {
fn kernel_main1(boot_info: &'static BootInfo) -> ! {

    println!("Hello World{}", "!");

    flyos::init();
    // fn stack_overflow() {
    //     stack_overflow();
    // }
    // stack_overflow();
    x86_64::instructions::interrupts::int3();

    #[cfg(test)]
    test_main();

    use x86_64::registers::control::Cr3;
    use x86_64::structures::paging::PageTable;
    let level_4_table_ptr = 0xffff_ffff_ffff_f000 as *const PageTable;
    let (level4_page_table, _) = Cr3::read();
    // println!("Level 4 page table at: {:?}", level4_page_table.start_address());
    for i in 0..10 {
        // vga_buffer::WRITER.lock().write_str("test---01\n");
        // println!("Entry: {}: {:}", i, level_4_table_ptr[i]); //打印页表一个条目的每个字段。
        println!("test--{}\n", i);
    }
    // let a = 1.0/3.0; // ok; 但是不能打印

    print!("fdsf {}", "fsgs"); // 不支持浮点数？？？TODO
    println!("hello {}", 123432);
    // TODO 像python那样的print，参数随便写，不用 {}
    // panic!("I'm panic!");

    loop {

    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    flyos::hlt_loop()
}



#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) ->! {
    serial_println!("i'm panic!");
    flyos::test_panic_handler(info);
}


#[test_case]
fn trivial_assertion() {
    serial_println!("trivial assertion..");
    assert_eq!(1, 1);
    // panic!("hello");
    serial_println!("[OK!]")
}

/*isa-debug-exit设备使用的就是端口映射I/O。其中， iobase
 参数指定了设备对应的端口地址（在x86中，0xf4是一个通常未被使用的端口），
 而iosize则指定了端口的大小（0x04代表4字节）。 */
