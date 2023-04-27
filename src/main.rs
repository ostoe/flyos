#![no_std]
#![no_main]
#![reexport_test_harness_main = "test_main"]
#![feature(custom_test_frameworks)]
#![test_runner(flyos::test_runner)]

extern crate alloc;

use alloc::{boxed::Box, rc::Rc, vec, vec::Vec};
use bootloader::entry_point;
use bootloader::BootInfo;
use flyos::task::keyboard;
use flyos::task::simple_executor::SimpleExecutor;
use core::panic::PanicInfo;
use flyos::memory::BootInfoFrameAllocator;
// use flyos::memory::translate_addr;
use flyos::{print, println, serial_println, test_panic_handler};
use x86_64::structures::paging::{FrameAllocator, Page, PageTable, Size4KiB, Translate};



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

    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&_boot_info.memory_map) };
    // 初始化heap相关的东西
    flyos::allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed.");

    //
    // let mut frame_allocator = flyos::memory::EmptyFrameAllocator;
    //new page
    // let page = Page::containing_address(VirtAddr::new(0));
    // 这里的思路是，选一段虚拟地址,创建一个Page，这里只是逻辑上生成一个page
    let page = Page::containing_address(VirtAddr::new(0xdeadbeaf));
    // 将上面的page映射到VGA的物理地址上
    flyos::memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);
    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    // 写入内容测试！
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) };
    // let l4_table = unsafe { active_level_4_page_table(phys_mem_offset) };
    /* -------------test heap  ---------- */
    let heap_value = Box::new(41);
    println!("heap_value at {:p}", heap_value);
    let mut vec = Vec::new();
    for i in 0..1024 {
        vec.push(i);
    }
    println!("vec as {:p}", vec.as_slice());
    let reference_counted = Rc::new(vec![1, 2, 3]);
    let cloned_reference = reference_counted.clone();
    println!(
        "reference count is  {} now",
        Rc::strong_count(&cloned_reference)
    );
    core::mem::drop(reference_counted);
    println!(
        "reference count is  {} now",
        Rc::strong_count(&cloned_reference)
    );

    let addresses = [
        // the identity-mapped vga buffer page
        0xb8000, // 是虚拟地址，但是这个地址比较特殊，会做恒等映射，物理地址也是这个。
        // some code page
        0x201008,
        // some stack page
        0x0100_0020_1a10,
        // virtual address mapped to physical address 0
        _boot_info.physical_memory_offset,
    ];
    for &address in &addresses {
        let virt = VirtAddr::new(address);
        let phys = unsafe {
            mapper.translate_addr(virt) /* before: translate_addr(virt, phys_mem_offset)  */
        };
        println!("{:?} -> {:?}", virt, phys);
    }

    let mut executor = SimpleExecutor::new();
    executor.spwan(flyos::task::Task::new(example_task()));
    executor.spwan(flyos::task::Task::new(keyboard::print_keypress()));
    executor.run();

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
    // let level_4_table_pointer = 0xffff_ffff_ffff_f000 as *const PageTable;
    // let level_4_table = unsafe { &*level_4_table_pointer };
    // for i in 0..10 {
    //     // 如果有页表条目，是可以看到 相关标识位的。
    //     // let entry = ;
    //     println!("Entry {}: {:?}", i, level_4_table[i]);
    // }
        // TODO ????
    let ptr = 0x2035f8 as *mut u32;
    unsafe {
        // *ptr = 42;
        let x = *ptr;
        println!("x:::{:?}",x);
    }
    // unsafe {*ptr = 42;}
    // println!("now");
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
    // panic!("info");
    // 这里panic以后也会进入hlt_loop()
    // let mut executor = Executor::new();
    // executor.spawn(Task::new(example_task()));
    // executor.spawn(Task::new(keyboard::print_keypresses()));
    // executor.run();
    flyos::hlt_loop();
}

async fn async_number() -> u32 {
    42
}
async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    flyos::hlt_loop()
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
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
