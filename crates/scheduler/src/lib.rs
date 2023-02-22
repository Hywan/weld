use std::{
    future::Future,
    num::NonZeroUsize,
    pin::Pin,
    thread::{self, JoinHandle},
};

use smol::{
    block_on,
    channel::{unbounded, Receiver, SendError, Sender},
    Executor,
};

pub struct ThreadPool<'e> {
    _workers: Vec<Worker>,
    executor: Executor<'e>,
    sender: Sender<Job>,
}

impl<'e> ThreadPool<'e> {
    pub fn new(pool_size: NonZeroUsize) -> Self {
        let pool_size: usize = pool_size.into();
        let mut workers = Vec::with_capacity(pool_size);

        let (sender, receiver) = unbounded::<Job>();

        for nth in 0..pool_size {
            workers.push(Worker::new(nth, receiver.clone()));
        }

        Self { _workers: workers, executor: Executor::new(), sender }
    }

    pub fn execute<F>(&self, work: F) -> Result<(), SendError<Job>>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        block_on(self.executor.run(self.sender.send(Box::pin(work))))
    }
}

struct Worker {
    _id: usize,
    _thread_handle: JoinHandle<()>,
}

type Job = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

impl Worker {
    fn new(worker_id: usize, receiver: Receiver<Job>) -> Self {
        let thread_handle = thread::Builder::new()
            .name(format!("{prefix}-worker-{worker_id}", prefix = env!("CARGO_PKG_NAME")))
            .spawn(move || {
                let executor = Executor::new();

                block_on(executor.run(async {
                    while let Ok(received_future) = receiver.recv().await {
                        executor.spawn(received_future).detach();
                    }
                }))
            })
            .unwrap();

        Self { _id: worker_id, _thread_handle: thread_handle }
    }
}
