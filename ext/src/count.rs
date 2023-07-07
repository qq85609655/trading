use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

pub struct CountDownLatch {
    size: Arc<AtomicUsize>,
    waker: Arc<Mutex<Option<Waker>>>,
}

pub struct WaitHold {
    size: Arc<AtomicUsize>,
    waker: Arc<Mutex<Option<Waker>>>,
}

impl CountDownLatch {
    pub fn new(size: usize) -> Self {
        Self { size: Arc::new(AtomicUsize::new(size)), waker: Arc::new(Mutex::new(None)) }
    }
    pub fn count_down(&self) -> usize {
        let mut waker = self.waker.lock().unwrap();
        let size = self.size.fetch_sub(1, Ordering::Relaxed);
        if let Some(waker) = waker.take() {
            waker.wake();
        }
        size
    }

    pub fn wait(&self) -> WaitHold {
        WaitHold { size: self.size.clone(), waker: self.waker.clone() }
    }
}

impl Drop for CountDownLatch {
    fn drop(&mut self) {}
}

impl Future for WaitHold {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut waker = self.waker.lock().unwrap();
        match self.size.load(Ordering::Relaxed) {
            0 => Poll::Ready(()),
            _ => {
                *waker = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}
