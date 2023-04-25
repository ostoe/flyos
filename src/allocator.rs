

use alloc::alloc::{GlobalAlloc, Layout};
use core::{ptr::null_mut, iter::Map};

use x86_64:: {
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB
    },
    VirtAddr,
};


/// 几种不同的分配器
///  - 只是简单的增加，效率高，不能重复利用，地址完全释放才回收==runtime不回收！
pub mod bump; 
///  - 有回收，能重复利用，但是也会有碎片，效率低
pub mod linked_list; // 
// 还有一种是固定大小的块，比如16 32 64 128大小的块分别使用不同的链表节点：
pub mod fixed_size_block;

// 可以增加合并策略，但是实现起来效率低，
// 好像go的hashmap也是这种玩的，小的和大的实现的算法不同。



pub struct Dummy;

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize  = 100 * 1024; // 100kib

unsafe impl GlobalAlloc for Dummy {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("dealloc should be never called.")
    }
}



/// 初始化heap大小100kib，由HEAP_SIZE定义
/// 从虚拟地址分出来一块heap，多个Page，映射到物理地址
/// 这里传入的第二个参数为4kib的桢页分配器，想象一下这个heap是套在frame分配器之上的，
pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE -1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };
    for page in page_range  {
        // 之前的frame其实是手动映射到了VGA的缓冲区，这里我们重新分配。
        // 这里的`allocate_frame`函数和分配PageTable用的是同一个！！！在memory.rs里面。
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        // 和之前映射到VGA的映射没区别：Page就是内存地址范围，frame是目标内存地址，frame_allocator是PageTable的分配器。
        unsafe {mapper.map_to(page, frame, flags, frame_allocator)?.flush()};
    }
    unsafe {
        // 不同的实现，初始化是不同的，
        super::ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())

}


// spin 

pub struct Locked<A> {
    inner: spin::Mutex<A>
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked { inner: spin::Mutex::new(inner), }
    }

    pub fn lock(&self, ) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}


///  4kib靠左对齐
pub fn align_up(addr: usize, align: usize) -> usize {
    let remainder = addr % align; 
    if remainder == 0 {
        addr 
    } else {
        addr - remainder + align
    }
}