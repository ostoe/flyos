#![no_std]
#![no_main]
#![reexport_test_harness_main = "test_main"]
#![feature(custom_test_frameworks)]
#![test_runner(flyos::test_runner)]


use core::panic::PanicInfo;
use bootloader::entry_point;
use bootloader::BootInfo;
use flyos::memory::BootInfoFrameAllocator;
// use flyos::memory::translate_addr;
use flyos::{test_panic_handler,print, println, serial_println };
use x86_64::structures::paging::FrameAllocator;
use x86_64::structures::paging::Page;
use x86_64::structures::paging::PageTable;
use x86_64::structures::paging::Size4KiB;
use x86_64::structures::paging::Translate;


entry_point!(kernel_main);


fn kernel_main(_boot_info: &'static BootInfo) -> ! {
// 我们使用no_mangle标记这个函数，来对它禁用名称重整（name mangling）——这确保Rust编译器输出一个名为_start的函数；否则，编译器可能最终生成名为_ZN3blog_os4_start7hb173fedf945531caE的函数，无法让链接器正确辨别。
// #[no_mangle] 
// pub extern "C" fn _start(bootinfo: &'static bootloader::BootInfo) -> ! {

    // use blog_os::allocator;
    // use blog_os::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    println!("Hello World{}", "!");
    flyos::init();
    // serial_println!("had init bootInfo: {:#?}", _boot_info);
    let phys_mem_offset = VirtAddr::new(_boot_info.physical_memory_offset);
    let mut mapper = unsafe { flyos::memory::init(phys_mem_offset) };
    
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&_boot_info.memory_map)
    };

    // 
    // let mut frame_allocator = flyos::memory::EmptyFrameAllocator;
    //new page
    // let page = Page::containing_address(VirtAddr::new(0));
    let page = Page::containing_address(VirtAddr::new(0));
    flyos::memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);
    let page_ptr:  *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e)};
    // let l4_table = unsafe { active_level_4_page_table(phys_mem_offset) };
    let addresses = [
        // the identity-mapped vga buffer page
        0xb8000,  // 是虚拟地址，但是这个地址比较特殊，会做恒等映射，物理地址也是这个。
        // some code page
        0x201008,
        // some stack page
        0x0100_0020_1a10,
        // virtual address mapped to physical address 0
        _boot_info.physical_memory_offset,
    ];
    for &address in &addresses {
        let virt = VirtAddr::new(address);
        let phys = unsafe { mapper.translate_addr(virt) /* before: translate_addr(virt, phys_mem_offset)  */};
        println!("{:?} -> {:?}", virt, phys);
    }

    // for (i, entry) in l4_table.iter().enumerate() {
    //     if !entry.is_unused() {
    //         println!("L4 Entry {}: {:?}", i, entry);

    //         let phys = entry.frame().unwrap().start_address();
    //         let virt = phys.as_u64() + _boot_info.physical_memory_offset; // 读取三级页表
    //         let ptr = VirtAddr::new(virt).as_mut_ptr();
    //         let l3_table: &PageTable =  unsafe { *ptr };

    //         for (i, l3_entry) in l3_table.iter().enumerate() {
    //             if !l3_entry.is_unused() {
    //                 println!("L3 Entry {}: {:?}", i, l3_entry);

    //                 let phys = entry.frame().unwrap().start_address();
    //                 let virt = phys.as_u64() + _boot_info.physical_memory_offset; // 读取三级页表
    //                 let ptr = VirtAddr::new(virt).as_mut_ptr();
    //                 let l2_table: &PageTable =  unsafe { *ptr };
    //                 for (i, l2_entry) in l2_table.iter().enumerate() {
    //                     if !l2_entry.is_unused() {
    //                         println!("L2 Entry {}: {:?}", i, l2_entry);

    //                     }
    //                 }
    //             }
    //         }

    //     }
    // }
    /*
     use x86_64::registers::control::Cr3;
    let (level_4_page_table, cr3_flags) = Cr3::read();
    // bootloader已经做了四级分页了，是可以直接读取标志位的，但是采用的是**递归页表**的方式，不是linux的方式
    // 可以看到地址为0x1000 也就是说四级页表活动在这个地方
    // 这是物理地址，内核需要定期访问，但无法直接放问到该地址，需要其他的解决方案，
    println!("level 4 table at: {:?}", level_4_page_table.start_address());
    println!("Cr3 Content: {:?}, flags: {:?}", level_4_page_table, cr3_flags);
     */

    // case 0x3
    let level_4_table_pointer = 0xffff_ffff_ffff_f000 as *const PageTable;
    let level_4_table = unsafe {&*level_4_table_pointer};
    for i in 0..10 {
        // 如果有页表条目，是可以看到 相关标识位的。
        // let entry = ;
        println!("Entry {}: {:?}", i, level_4_table[i]);
    }


    
    let ptr = 0x2035f8 as *mut u32;
    unsafe {let x = *ptr;}
    // unsafe {*ptr = 42;}

    // 触发了缺页异常，如果没处理，就会触发双重异常；
    unsafe {
        *(0xdeadbee8 as *mut u64) = 42;
    };
    // x86_64::instructions::interrupts::int3();

    // fn stack_overflow() {stack_overflow();}
    // stack_overflow();

    // let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    // let mut mapper = unsafe { memory::init(phys_mem_offset) };
    // let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    // allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");
    #[cfg(test)]
    test_main();
    println!("---end---");
    panic!("info"); // 这里panic以后也会进入hlt_loop()
    // let mut executor = Executor::new();
    // executor.spawn(Task::new(example_task()));
    // executor.spawn(Task::new(keyboard::print_keypresses()));
    // executor.run();
    flyos::hlt_loop();         

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
