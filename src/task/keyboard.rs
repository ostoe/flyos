use core::{
    iter::Scan,
    task::{Context, Poll},
};

use core::pin::Pin;
use conquer_once::spin::OnceCell;

use crossbeam_queue::ArrayQueue;

use crate::{println, print};
use futures::{task::AtomicWaker, Stream, stream::StreamExt};
// use futures_util::{task::AtomicWaker, Stream};

static WAKER: AtomicWaker = AtomicWaker::new();

static SCAN_AUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();

// 给中断回调的函数

pub(crate) fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCAN_AUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            println!("WARNING: scanqueue_code full; dropping keyboad input")
        } else {
            // 放进去了一个值，就能调用wake了，唤醒之前谁注册过waker的task
            WAKER.wake();
        }
    } else {
        println!("WARNING: uninitialized.")
    }
}
// 加了以后可用next

// 键盘任务：
pub async fn print_keypress() {
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    // PS/2 控制器的数据端口
    let mut scancodes = ScancodeStream::new();
    let mut keybord = Keyboard::new(ScancodeSet1::new(), layouts::Us104Key, HandleControl::Ignore);

    while let Some(byte) = scancodes.next().await  {
        if let Ok(Some(event)) = keybord.add_byte(byte) {
            if let Some(key) = keybord.process_keyevent(event) {
                match key {
                    DecodedKey::RawKey(key) => print!("{:?}", key),
                    DecodedKey::Unicode(character) => print!("{}", character),
                }
            }
        }
    }
}

// future开始


pub struct ScancodeStream {
    _private: (),
}

impl ScancodeStream {
    pub fn new() -> ScancodeStream {
        SCAN_AUEUE
            .try_init_once(|| ArrayQueue::new(100))
            .expect("new should only call once.");
        ScancodeStream { _private: () }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // fn poll_text(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
            let queue = SCAN_AUEUE.try_get().expect("not initialed.");
    
            // 快路径
            if let Some(scancode) = queue.pop() {
                return Poll::Ready(Some(scancode));
            }
    
    
            let waker = cx.waker();
            WAKER.register(&waker);
            match queue.pop() {
                Some(scancode) => {
                    // 为什么这里要加上wake， 正常情况下，executor调用pull的时候，waker会被调用，并且自动失效，正如快路径那样，
                    // 但是如果是刚注册好就发现有值了，这时候直接返回是不会销毁waker的，必须手动销毁、使其失效
                    WAKER.take();
                    Poll::Ready(Some(scancode))
                }
                None => Poll::Pending,
            }
        // }
    }
}
