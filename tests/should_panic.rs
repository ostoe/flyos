#![no_std]
#![no_main]

// #![feature(custom_test_frameworks)]
// #![test_runner(flyos::test_runner)]
// #![reexport_test_harness_main = "test_main"]


use core::panic::PanicInfo;
use flyos::{println, serial_print, serial_println, exit_qemu, QemuExitCode};


#[no_mangle]
pub extern "C" fn _start() -> ! {
    // test_main();
    should_fail();
    loop{};
}


#[panic_handler]
fn panic(_info: &PanicInfo) ->! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Failed);
    loop {
    }
}


// #[test_case]
fn should_fail() {
    serial_println!("should_fail... ");
    assert_eq!(1, 1);
}


// pub fn test_runner(tests: &[&dyn Fn()]) {
//     serial_println!("Running {} tests", tests.len());
//     for test in tests {
//         test();
//         serial_println!("[test did not panic]");
//         exit_qemu(QemuExitCode::Failed);
//     }
//     exit_qemu(QemuExitCode::Success);
// }