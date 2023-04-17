#![no_std]
#![no_main]
#![reexport_test_harness_main = "test_main"]
#![feature(custom_test_frameworks)]
#![test_runner(flyos::test_runner)]


use core::panic::PanicInfo;
use flyos::{test_panic_handler,print, println, serial_println };



#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) ->! {
    serial_println!("i'm panic!");
    flyos::test_panic_handler(info);
}


// 我们使用no_mangle标记这个函数，来对它禁用名称重整（name mangling）——这确保Rust编译器输出一个名为_start的函数；否则，编译器可能最终生成名为_ZN3blog_os4_start7hb173fedf945531caE的函数，无法让链接器正确辨别。
#[no_mangle] 
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    #[cfg(test)]
    test_main();

    for x in 0..8 {
        // vga_buffer::WRITER.lock().write_str("test---01\n");
        println!("test--{}\n", x);
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
    print!("i'm panic!");

    println!("{}", info);
    loop{
    }
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
