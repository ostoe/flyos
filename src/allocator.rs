

use alloc::alloc::{GlobalAlloc, Layout};
use core::{ptr::null_mut, iter::Map};

use x86_64:: {
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB
    },
    VirtAddr,
};

pub mod bump;
pub mod linked_list;


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

pub fn align_up(addr: usize, align: usize) -> usize {
    let remainder = addr % align; 
    if remainder == 0 {
        addr 
    } else {
        addr - remainder + align
    }
}