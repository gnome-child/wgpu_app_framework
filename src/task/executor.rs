use std::{
    sync::{Arc, Mutex, mpsc},
    thread,
};

type Job = Box<dyn FnOnce() + Send + 'static>;

pub(crate) struct Executor {
    sender: Option<mpsc::Sender<Job>>,
    workers: Vec<thread::JoinHandle<()>>,
}

impl Executor {
    pub(crate) fn new() -> Self {
        let worker_count = thread::available_parallelism()
            .map(usize::from)
            .unwrap_or(1)
            .clamp(1, 8);
        let (sender, receiver) = mpsc::channel::<Job>();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(worker_count);

        for index in 0..worker_count {
            let receiver = Arc::clone(&receiver);
            let worker = thread::Builder::new()
                .name(format!("wgpu_l3-worker-{index}"))
                .spawn(move || worker_loop(&receiver))
                .expect("worker executor thread should start");
            workers.push(worker);
        }

        Self {
            sender: Some(sender),
            workers,
        }
    }

    pub(crate) fn spawn(&self, job: impl FnOnce() + Send + 'static) -> bool {
        self.sender
            .as_ref()
            .is_some_and(|sender| sender.send(Box::new(job)).is_ok())
    }
}

impl Drop for Executor {
    fn drop(&mut self) {
        self.sender.take();
        for worker in self.workers.drain(..) {
            let _ = worker.join();
        }
    }
}

fn worker_loop(receiver: &Mutex<mpsc::Receiver<Job>>) {
    loop {
        let job = {
            let Ok(receiver) = receiver.lock() else {
                return;
            };
            receiver.recv()
        };
        let Ok(job) = job else {
            return;
        };
        job();
    }
}
