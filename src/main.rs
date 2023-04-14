#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod vga_buffer;

static HELLO: &[u8] = b"Hello World!";
#[no_mangle] 
// 我们使用no_mangle标记这个函数，来对它禁用名称重整（name mangling）——这确保Rust编译器输出一个名为_start的函数；否则，编译器可能最终生成名为_ZN3blog_os4_start7hb173fedf945531caE的函数，无法让链接器正确辨别。
pub extern "C" fn _start() -> ! {
    vga_buffer::print_something();
    let vga_buffer = 0xb8000 as *mut u8; // 注意此处 *mut，裸指针，指向内存0xb8000这片区域！
    for (i, &byte) in HELLO.iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte;  // 0-7 bit
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb; // 淡青色！因为是一个内存块的不同位； 8-15 bit
        }
    }
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop{

    }
}

