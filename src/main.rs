#![no_std]
#![no_main]
#![reexport_test_harness_main = "test_main"]

use core::{panic::PanicInfo, fmt::Write};

mod vga_buffer;

static HELLO: &[u8] = b"Hello World!";
#[no_mangle] 
// 我们使用no_mangle标记这个函数，来对它禁用名称重整（name mangling）——这确保Rust编译器输出一个名为_start的函数；否则，编译器可能最终生成名为_ZN3blog_os4_start7hb173fedf945531caE的函数，无法让链接器正确辨别。
pub extern "C" fn _start() -> ! {

    for x in 0..100 {
        // vga_buffer::WRITER.lock().write_str("test---01\n");
        write!(&mut vga_buffer::WRITER.lock(), "test--{}\n", x);
    }
    let a = 1.0/3.0;

    print!("fdsf {}", "fsgs"); // 不支持浮点数？？？TODO
    println!("hello {}", 123432);
    // TODO 像python那样的print，参数随便写，不用 {}
    panic!("Some panic message");
    loop {


    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    loop{

    }
}



#[feature(custom_test_frameworks)]
#[test_runner(crate::test_runner)]
#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
}



#[no_mangle]
pub extern "C" fn _star() -> ! {
    println!("Hello World{}", "!");

    #[cfg(test)]
    trivial_assertion();
    loop {
        
    }
}

#[test_case]
fn trivial_assertion() {
    print!("trivial assertion..");
    assert_eq!(1, 1);
    println!("[OK!]")
}