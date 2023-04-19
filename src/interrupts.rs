
use x86_64::structures::idt::PageFaultErrorCode;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::println;
use crate::gdt;
use lazy_static::lazy_static;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        unsafe {
            idt.double_fault
            .set_handler_fn(double_fault_handler)
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt
        
    };
}

extern "x86-interrupt" fn breakpoint_handler(
    stack_frame: InterruptStackFrame)
{
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler (
    stack_frame: InterruptStackFrame,
    error_code: u64) -> ! {
        panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);

}

extern "x86-interrupt" fn page_fault_handler (
    stack_frame: InterruptStackFrame, 
    _error_code: PageFaultErrorCode){
    use crate::hlt_loop;
    use x86_64::registers::control::Cr2;

    println!("EXCETION: PAGE FAULT");
    println!("Access Address {:?}", Cr2::read());
    println!("{:#?}", stack_frame);
    hlt_loop();
}



pub fn init_idt() {
    IDT.load();
}