

use alloc::alloc::{GlobalAlloc, Layout};


/// Bump heap分配器
pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
    allocations: usize,
}




impl BumpAllocator {

    pub const fn new() -> Self {
        BumpAllocator { heap_start: 0, heap_end: 0, next: 0, allocations: 0 }

    }

    // call only onces
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.next = heap_start;

    }
}

// use spin::Mutex;

// unsafe impl GlobalAlloc for BumpAllocator {
//     // 这里必须改变&self，但是trait又规定了参数不能是&mut self,怎么办？山人自有妙计！！！将BumpAllocator改为spin::Mutex<>
//     unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
//         let alloc_start = self.next;
//         self.next = alloc_start + layout.size();
//         self.allocations += 1;
//         alloc_start as *mut u8
//     }

//     unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
//         todo!()
//     }

// }



// spin : Mutex : lock


use super::{align_up, Locked};
use core::ptr;

unsafe impl GlobalAlloc for Locked<BumpAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut bump = self.lock();
        let alloc_start = align_up(bump.next, layout.size()); // 万一起始地址不是4kib，也能对齐，开始使用的。
        let alloc_end = alloc_start + layout.size();
        if alloc_end > bump.heap_end {
            ptr::null_mut() // oom
        } else {
            bump.next = alloc_end;
            bump.allocations += 1;
            alloc_start as *mut u8
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut bump = self.lock();

        bump.allocations -=  1;
        if bump.allocations == 0 {
            bump.next = bump.heap_start;
        }
    }
}