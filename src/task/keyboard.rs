
use conquer_once::spin::OnceCell;

use crossbeam_queue::ArrayQueue;

static SCAN_AUEUE: OnceCell::<ArrayQueue<u8>> = OnceCell::uninit();



