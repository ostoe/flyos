use pc_keyboard::layouts;

use crate::allocator::align_up;
use core::{mem, alloc::{GlobalAlloc, Layout}};

use super::Locked;





struct ListNode {
    size: usize,
    next: Option<&'static mut ListNode>
}


impl ListNode {
    const fn  new(size: usize) -> Self {
        ListNode { size, next: None}
    }

    fn start_addr(&self, ) -> usize {
        let a = self as *const Self;
        let b = a as usize;
        // 取地址为啥不是&取地址符呢？*才是取地址?
        // as *const ListNode as usize
        self as *const Self as usize
    }

    fn end_addr(&self,) -> usize {
        self.start_addr() + self.size
    }
}

pub struct LinkedListAllocator {
    head: ListNode,
}


impl LinkedListAllocator {
    pub const fn new() -> Self {
        Self {
            head: ListNode::new(0),
        }
    }

    /// 利用给出的heap边界初始化分配器
    /// 不安全，调用者必须保证heap未使用，只能调用一次
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.add_free_region(heap_start, heap_size);
    }
    // 只是做内存块的链表操作
    unsafe fn add_free_region(&mut self, addr: usize, size: usize) {
        assert!(align_up(addr, mem::align_of::<ListNode>()) == addr);
        assert!(size >= mem::size_of::<ListNode>());
        // 在头部插入 新的ListNode
        let mut node = ListNode::new(size);
        node.next = self.head.next.take();
        let node_ptr = addr as *mut ListNode; // 地址作为指针，是* 的类型
        node_ptr.write(node);
        //  *node_ptr 解引用，表示取的变量
        self.head.next = Some(&mut *node_ptr);
    }
    

    fn find_region(&mut self, size: usize, align: usize) -> Option<(&'static mut ListNode, usize)> {
        let mut current = &mut self.head;
        while let Some(ref mut region) = current.next {
            if let Ok(alloc_start) = Self::alloc_from_region(&region, size, align) {
                // 相当于吧中间的节点扣出来！
                let next = region.next.take();
                let ret = Some((current.next.take().unwrap(), alloc_start));
                current.next = next;
                return ret;
            }
            else {
                // 顺藤摸瓜，摸下一个节藤
                current = current.next.as_mut().unwrap();
            }
        }
        // no suitable region found.
        return None
    }

    fn alloc_from_region(region: &ListNode, size: usize, align: usize) -> Result<usize, ()> {
        // 4kib对其校验
        let alloc_start = align_up(region.start_addr(), align);
        let alloc_end = alloc_start + size;

        if alloc_end > region.end_addr() {
            return Err(());
        }

        // rest of resion too small to hold a ListNode;
        let excess_size = region.end_addr() - alloc_end;
        // 这里的size_of和传入参数param区别？
        if excess_size > 0 && excess_size < mem::size_of::<ListNode>() {
            return Err(());
        }

        Ok(alloc_start)
    }


    fn size_align(layout: Layout) -> (usize, usize) {
        let layout = layout.align_to(mem::align_of::<ListNode>())
            .expect("adjusting alignment failed.")
            .pad_to_align();

        let size = layout.size().max(mem::size_of::<ListNode>());
        (size, layout.align())
    }

}

unsafe impl GlobalAlloc for Locked<LinkedListAllocator> {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let (size, align) = LinkedListAllocator::size_align(layout);
        let mut allocator = self.inner.lock();

        if let Some((region, alloc_start)) = allocator.find_region(size, align) {
            let alloc_end = alloc_start + size;
            let excess_size = region.end_addr() - alloc_end;
            if excess_size > 0 {
                allocator.add_free_region(alloc_end, excess_size);
            }
            alloc_start as *mut u8
        } else {
            core::ptr::null_mut()
        }

    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        let (size, _) = LinkedListAllocator::size_align(layout);
        self.inner.lock().add_free_region(ptr as usize, size)
    }
}