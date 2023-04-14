
/*
| Bit(s) | Value            |
|--------|------------------|
| 0-7    | ASCII code point |
| 8-11   | Foreground color |
| 12-14  | Background color |
| 15     | Blink            |
 */

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
    chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT]
}


pub struct Writer {
    column_position: usize,
    raw_position: usize,
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
            let row =  self.raw_position;
            // let row = BUFFER_HEIGHT - 1;
            let col = self.column_position;
            let color_code = self.color_code;
            self.buffer.chars[row][col] = ScreenChar { //这里等价于直接写内存字段；
                ascii_character: byte,
                color_code,
            };
            self.column_position += 1;
           } 
        }
    }

    fn new_line(&mut self) {
        self.raw_position += 1;
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

    
}

pub fn print_something() {
    let mut writer = Writer {
        raw_position: 3,
        column_position: 0,
        color_code: ColorCode::new(Color::LightRed, Color::DarkGray),
        buffer: unsafe {
            &mut *(0xb8000 as *mut Buffer) // 直接使用指针；???  VGA缓冲区内存 写法不明白
        },
    };

    writer.write_byte(b'H');
    writer.write_string("ello ");
    writer.write_string("Wörld!");
    writer.write_string("-----\nfsfsgsfsa")

}