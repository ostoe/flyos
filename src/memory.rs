use x86_64::{
    registers::control::Cr3,
    structures::paging::{page_table::FrameError, FrameAllocator, Page, PageTable, Size4KiB, PhysFrame, Mapper},
    PhysAddr, VirtAddr,
};

use x86_64::structures::paging::OffsetPageTable;

pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_page_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset) // 类型假定完整的物理内存以某个偏移量映射到虚拟地址空间
}

/// 读取CR3寄存器，从p4的物理地址，加上偏移，转换成虚拟地址，以此得到PageTable的虚拟地址，从指针转换成实例
/// bootloader 的map_physical_memory 功能将整个物理内存映射到虚拟地址空间中的某个位置。[bootloader crate]
/// 因此，内核可以访问所有物理内存，并且可以遵循 “映射完整物理内存” 方法。
unsafe fn active_level_4_page_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3; //4kib大小

    // 指向l4 table
    let (level_4_page_frame, _) = Cr3::read();
    // 找到 l4 table物理起始地址
    let phys = level_4_page_frame.start_address(); // 条目里存的物理地址
                                                   // +偏移 --> 转换成虚拟地址？？？这里其实就是变量的地址，这里为啥是虚拟地址？
    let virt = physical_memory_offset + phys.as_u64();
    // l4页表指针 返回l4 PageTable 指针
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
    &mut *page_table_ptr // unsafe
}

/// 为在PageTable中添加一条映射
pub fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;
    // 此处使用的是0xb8000 物理地址，因为是恒等映射，虚拟地址同地址。
    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = unsafe {
        mapper.map_to(page, frame, flags, frame_allocator)
    };
    map_to_result.expect("map_to failed").flush();
}

/// A FrameAllocator that always returns `None`;
pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        None
    }
}

/* 不再需要啦～ x86_64的 offset_page_table 模块已实现

pub unsafe fn translate_addr1(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    translate_addr_inner(addr, physical_memory_offset)

}

fn translate_addr_inner(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    let tb_indexes = [
        addr.p4_index(), addr.p3_index(), addr.p2_index(), addr.p1_index()
    ];
    let (l4_tb_frame, _) = Cr3::read(); // 获得P4 表的地址，恒生效！
    let mut frame = l4_tb_frame;

    for &index in &tb_indexes {
        // 为什么左边是虚拟地址？？
        // 虚拟地址就可以理解为64bit的地址，这里就合成了 cpu要读去的地址！
        // 这里叫页表引用
        let virt = physical_memory_offset + frame.start_address().as_u64();
        // 虚拟地址转换类型实例
        let tb_ptr: *const PageTable = virt.as_ptr();
        let table = unsafe {&* tb_ptr};
        // p4 index: (self.0 >> 12 >> 9 >> 9 >> 9) as u16)  取出高位地址,也是一种索引,是cpu的寻址地址，区分cr3
        // 比较形象的理解为，索引到第四级页表中512项条目中的第几个条目，
        let entry = &table[index]; // 拿虚拟地址的高位偏移计算到第一个条目

        frame = match entry.frame() { // 每一次循环都会下钻一层frame
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame ) => return panic!("huge not supported."),
        }

    }
    // 通过添加的页offset 计算物理地址  ｜ 每个虚拟地址都有一个12bit的偏移
    Some(frame.start_address() + u64::from(addr.page_offset()))
}

*/
