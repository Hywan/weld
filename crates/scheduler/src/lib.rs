//! `weld-scheduler` provides its own `ThreadPool` implementation.
//!
//! How it works is pretty simple. A `ThreadPool` creates _n_ `Worker`s. Each
//! `Worker` creates and owns a thread. The `ThreadPool` and the `Worker`
//! communicate via an unbounded multi-producer multi-consumer (MPMC)
//! asynchronous channel, where `ThreadPool` owns the sender, and `Worker`s own
//! a clone of the receiver. When `ThreadPool::execute(future)` is called, it
//! sends the `Future` onto the channel. Each `Worker` uses their own
//! asynchronous executor to wait on the receiver to receive a `Future`. When
//! a `Future` is received, the executor spawns it in a detached mode, i.e. the
//! `Future` runs in the background. None of these steps are blocking.
//!
//! Distribution of the work is not based on a work-stealing approach (as it's
//! usually the case), but relies on the fact that the asynchronous MPMC channel
//! sends the `Future` on receiver that are idle. They are idle either because
//! all their `Future`s are pending, or because there is no `Future` at all. In
//! some particular edge cases, it's possible for a `Worker` to receive too much
//! `Future`s because at some points there were all pending, and suddently there
//! is a lot more jobs to do. In practise, this case happens rarely.
//!
//! This `ThreadPool` design does not aim to be general and performant in all
//! case. It's tailored for the needs of this project only. The major constraint
//! was having something simple.
//!
//! Here is a small schema explaining how it works.
//!
//! ```text
//!  ThreadPool                     │
//! ┌───────────────────────────────┴──────────────────────────┐
//! │                               ▼                          │
//! │                    ┌───────────────────────┐             │
//! │                    │  ThreadPool::execute  │             │
//! │                    └──────────┬────────────┘             │
//! │                               │                          │
//! │                Sending future onto the channel           │
//! │                               │                          │
//! │                   ┌───────────┘                          │
//! │    Worker #1      │                       Worker #n      │
//! │  ┌────────────────┴──────────────┐      ┌─────────────┐  │
//! │  │   Executor     ▼              │      │             │  │
//! │  │ ┌───────────────────────────┐ │      │             │  │
//! │  │ │                           │ │      │             │  │
//! │  │ │ Awaiting to receive       │ │      │             │  │
//! │  │ │ a future from the channel │ │      │             │  │
//! │  │ │                           │ │      │      …      │  │
//! │  │ │ │                         │ │      │             │  │
//! │  │ │ └─► Spawning future       │ │      │             │  │
//! │  │ │                           │ │      │             │  │
//! │  │ └───────────────────────────┘ │      │             │  │
//! │  └───────────────────────────────┘      └─────────────┘  │
//! └──────────────────────────────────────────────────────────┘
//! ```

use std::{
    cmp,
    future::Future,
    io,
    num::NonZeroUsize,
    pin::Pin,
    thread::{self, JoinHandle},
};

use smol::{
    block_on,
    channel::{unbounded, Receiver, SendError, Sender},
    Executor,
};

/// A thread pool allows to execute `Future`s on multiple threads automatically.
///
/// The user doesn't have to care about where their `Future`s are going to be
/// executed, they are just sent where there is idleness. In the current design,
/// _idle_ means a thread that has an idle asynchronous executor, either because
/// it has no `Future` running at all, or because a `Future` is pending.
pub struct ThreadPool<'e, T> {
    _workers: Vec<Worker>,
    executor: Executor<'e>,
    sender: Sender<Job<T>>,
}

impl<'e, T> ThreadPool<'e, T>
where
    T: Send + 'static,
{
    /// Create a new pool of threads, of maximum size `desired_pool_size`.
    ///
    /// Threads are creating eargerly. They will be ready when the constructor
    /// returns.
    ///
    /// Why `desired_pool_size` rather than an “exact `pool_size`”? Because
    /// parallism is a resource. A given machine provides a certain capacity
    /// for parallelism, i.e. a bound on the number of computations
    /// it can perform simultaneously. This number often corresponds to the
    /// amount of CPUs a computer has, but it may diverge in various cases.
    ///
    /// Host environments such as VMs or container orchestrators may want to
    /// restrict the amount of parallelism made available to programs in
    /// them. This is often done to limit the potential impact of
    /// (unintentionally) resource-intensive programs on other programs
    /// running on the same machine.
    ///
    /// Thus, `desired_pool_size` is clamped between 1 and
    /// [`std::thread::available_parallelism`].
    pub fn new(desired_pool_size: NonZeroUsize) -> Result<Self, io::Error> {
        let pool_size = cmp::min(desired_pool_size, thread::available_parallelism()?).get();

        let mut workers = Vec::with_capacity(pool_size);

        let (sender, receiver) = unbounded::<Job<T>>();

        for nth in 0..pool_size {
            workers.push(Worker::new(nth, receiver.clone())?);
        }

        Ok(Self { _workers: workers, executor: Executor::new(), sender })
    }

    /// Execute a `Future` onto a thread that can accept it.
    pub fn execute<F>(&self, work: F) -> Result<(), SendError<Job<T>>>
    where
        F: Future<Output = T> + Send + 'static,
    {
        block_on(self.executor.run(self.sender.send(Box::pin(work))))
    }
}

/// A `Worker` executes work, aka `Job`.
struct Worker {
    _thread_handle: JoinHandle<()>,
}

/// Type alias for a job, i.e. what a `Worker` will execute.
type Job<T> = Pin<Box<dyn Future<Output = T> + Send + 'static>>;

impl Worker {
    fn new<T>(worker_id: usize, receiver: Receiver<Job<T>>) -> Result<Self, io::Error>
    where
        T: Send + 'static,
    {
        let thread_handle = thread::Builder::new()
            .name(format!("{prefix}-worker-{worker_id}", prefix = env!("CARGO_PKG_NAME")))
            .spawn(move || {
                let executor = Executor::new();

                block_on(executor.run(async {
                    while let Ok(received_future) = receiver.recv().await {
                        executor.spawn(received_future).detach();
                    }
                }))
            })?;

        Ok(Self { _thread_handle: thread_handle })
    }
}
