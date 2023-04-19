
use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor};
use x86_64::structures::gdt::SegmentSelector;



pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static::lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(unsafe{ &STACK });
            let stack_end = stack_start + STACK_SIZE; // 然后向第0个条目写入了双重异常栈的顶部地址
            // （因为 x86 机器的栈地址向下扩展，也就是从高地址到低地址）。
            stack_end
        };
        tss
    };
}

struct Selector {
    code_selector: SegmentSelector,
    tss_selector : SegmentSelector,
}

lazy_static::lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selector) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (gdt, Selector {code_selector, tss_selector})
    };
}

pub fn init() {
    use x86_64::instructions::{segmentation::set_cs, tables::load_tss};
    GDT.0.load();
    unsafe {
        set_cs(GDT.1.code_selector); // 重装代码段寄存器
        load_tss(GDT.1.tss_selector); // 加载TSS
    }
}

