use crate::time_of_day;
use crate::windows::generic_windows::{GenericWindow, Loglet};
use core::panic;
use std::task::Context;
use std::task::Poll;
use tokio::sync::mpsc::{channel, Receiver, Sender};

pub type ErrorSender = Sender<Loglet>;
pub type ErrorReceiver = Receiver<Loglet>;

pub struct ErrorMessage {
    pub display: GenericWindow,
    receiver: ErrorReceiver,
    sender: ErrorSender,
}

impl ErrorMessage {
    pub fn new() -> Self {
        let (sender, receiver) = channel(32);
        ErrorMessage {
            display: GenericWindow::default(),
            receiver,
            sender,
        }
    }

    pub async fn try_update_log(&mut self) {
        //! Receive any error messages from differnt Threads, Tokios, Async fns
        //!
        //! Async Context only
        self.receiver.recv().await.into_iter().for_each(|loglet| {
            GenericWindow::push_loglet(&mut self.display, loglet);
        });
    }

    pub fn block_update_log(&mut self) {
        let waker = futures::task::noop_waker();
        let mut cx = Context::from_waker(&waker);

        while let Poll::Ready(op) = self.receiver.poll_recv(&mut cx) {
            match op {
                Some(loglet) => GenericWindow::push_loglet(&mut self.display, loglet),
                None => panic!("Error Sender broke somehow...?"),
            }
        }
    }

    pub fn sender_clone(&self) -> Sender<Loglet> {
        //! Provide a sender for async functions or new Threads/Tokios...
        self.sender.clone()
    }

    pub fn push_err(&mut self, msg: &str) {
        // Sync err appending, for sending errors on the main thread
        let loglet = Loglet::new("Error", msg, &time_of_day());
        GenericWindow::push_loglet(&mut self.display, loglet);
    }
}
