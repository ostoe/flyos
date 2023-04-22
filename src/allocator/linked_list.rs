



struct ListNode {
    size: usize,
    next: Option<&'static mut ListNode>
}


impl ListNode {
    const fn  new(size: usiz) -> Slef {
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

    unsafe fn add_free_region(&mut self, addr: usize, size: usize) {
        todo!();
    }

}