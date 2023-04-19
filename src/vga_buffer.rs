
/*
| Bit(s) | Value            |
|--------|------------------|
| 0-7    | ASCII code point |
| 8-11   | Foreground color |
| 12-14  | Background color |
| 15     | Blink            |


raw write:
```rust
    // let vga_buffer = 0xb8000 as *mut u8; // 注意此处 *mut，裸指针，指向内存0xb8000这片区域！
    // for (i, &byte) in HELLO.iter().enumerate() {
    //     unsafe {
    //         *vga_buffer.offset(i as isize * 2) = byte;  // 0-7 bit
    //         *vga_buffer.offset(i as isize * 2 + 1) = 0xb; // 淡青色！因为是一个内存块的不同位； 8-15 bit
    //     }
    // }
```
 */

use spin::Mutex;
use volatile::Volatile;
use core::fmt;
use lazy_static::lazy_static;



lazy_static! {
     pub static ref WRITER: Mutex<Writer> = Mutex::new( Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::LightGreen, Color::Black),
        buffer: unsafe {
            /*CPU和外围设备通信方式，此处使用使用内存映射的方式，
            通过内存地址0xb8000访问了[VGA文本缓冲区]。
            该地址并没有映射到RAM，而是映射到了VGA设备的一部分内存上。
            */
            &mut *(0xb8000 as *mut Buffer) //
        },
    });
}
// #[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color, ) -> ColorCode {
        ColorCode((background as u8) << 4 | foreground as u8) // 8-11 bit 12-14 bit 还可以加上光标
    }
    
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(C)] //使用#[repr(C)]标记结构体；这将按C语言约定的顺序布局它的成员变量，正确地映射内存片段

struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;


//对Buffer类型，我们再次使用repr(transparent)，来确保类型和它的单个成员有相同的内存布局。
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT]
}


pub struct Writer {
    column_position: usize,
    // raw_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl  Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
           b'\n' => self.new_line(),
           byte => {
            if self.column_position >= BUFFER_WIDTH {
                self.new_line();
            }
            // let row =  self.raw_position;
            let row = BUFFER_HEIGHT - 1;
            let col = self.column_position;
            let color_code = self.color_code;
            self.buffer.chars[row][col].write( ScreenChar { //这里等价于直接写内存字段；
                ascii_character: byte,
                color_code,
            });
            self.column_position += 1;
           } 
        }
    }

    fn new_line(&mut self) {

        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let characator = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(characator);
            }
        }
        self.clear_row(BUFFER_HEIGHT-1);
        self.column_position = 0;
    }
    fn write_string(&mut self, s: &str) {
        for byte in s.as_bytes() {
            match byte {
                // #![feature(exclusive_range_pattern)]
                0x20..=0x7e | b'\n' => self.write_byte(*byte),
                _ => self.write_byte(0xfe) //不支持的编码打印一个符号 ■
            }
        }
    }
    fn clear_row(&mut self, row: usize)  {
        let blank  = ScreenChar{
            ascii_character: b' ',
             color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
    
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }

}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    // write_fmt是fmt::Write默认实现，中间调用了write方法，而该方法就是for x in xxx { self.write_str(x) }
    // 而此处的write_str就是我们实现的
    x86_64::instructions::interrupts::without_interrupts(||  {
        WRITER.lock().write_fmt(args).unwrap()
    });
   
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n")); // 有点 match switch的意思
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! printt {
    () => ($crate::print!("\n"));
    // ($($args:tt)*) =>  ($crate::print!("{}\n", format_args!($({})* ,$($arg)*)));

    // ($($args:tt)*) =>  ($crate::print!("{}\n", stringify!($($arg , )*)));
    ($($args:expr),*) => {
            $(
                $crate::print!("{} ", $arg); 
            )*
            $crate::print!("{}\n");

        };

}


#[allow(unused)]
pub fn print_something() {
    use core::fmt::Write;
    let mut writer : Mutex<Writer>= Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::LightRed, Color::Black),
        buffer: unsafe {
            &mut *(0xb8000 as *mut Buffer) // 直接使用指针；???  VGA缓冲区内存 写法不明白
        },
    });
    // let writer = WRITER;
    writer.lock().write_string("-----\nfsfsgsfsa");
    write!(&mut writer.lock(), "hefdsafdsa------").unwrap();
    // let a = 1.0f32/3.0 as f32;
    let a =  stringify!(1.0 + 1);
    write!(&mut writer.lock(), "The numbers are {} and {}", 42, a).unwrap();// .unwrap();
    writer.lock().write_string("-----\nfsfsgsfsa");
}


/* 
====================
test
====================
 */


 use crate::interrupts;
#[cfg(test)]
 use crate::{serial_print, serial_println};
 
 #[test_case]
 fn test_println_simple() {
     serial_print!("test_println... ");
     println!("test_println_simple output");
     serial_println!("[ok]");
 }

 #[test_case]
fn test_println_many() {
    serial_print!("test_println_many... ");
    for _ in 0..200 {
        println!("test_println_many output");
    }
    serial_println!("[ok]");
}


#[test_case]
fn test_println_output() {
    use core::fmt::Write;
    serial_print!("test_println_output... ");
    let s = "Some test string that fits on a single line";

    x86_64::instructions::interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s).expect("writeln failed.");
        for (i, c) in s.chars().enumerate() {
            let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screen_char.ascii_character), c);
        }
    });

    serial_println!("[ok]");
}