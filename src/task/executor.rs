use std::{
    io,
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
        Self::with_spawner(worker_count, |index, receiver| {
            thread::Builder::new()
                .name(format!("wgpu_l3-worker-{index}"))
                .spawn(move || worker_loop(&receiver))
        })
    }

    fn with_spawner(
        worker_count: usize,
        mut spawn: impl FnMut(
            usize,
            Arc<Mutex<mpsc::Receiver<Job>>>,
        ) -> io::Result<thread::JoinHandle<()>>,
    ) -> Self {
        let (sender, receiver) = mpsc::channel::<Job>();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(worker_count);

        for index in 0..worker_count {
            let receiver = Arc::clone(&receiver);
            match spawn(index, receiver) {
                Ok(worker) => workers.push(worker),
                Err(error) => {
                    log::error!("failed to start task worker {index}: {error}");
                }
            }
        }

        Self {
            sender: (!workers.is_empty()).then_some(sender),
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

#[cfg(test)]
mod tests {
    use std::{io, sync::mpsc, thread, time::Duration};

    use super::{Executor, worker_loop};

    #[test]
    fn executor_rejects_work_when_no_worker_can_start() {
        let executor = Executor::with_spawner(2, |_, _| {
            Err(io::Error::other("simulated worker startup failure"))
        });

        assert!(executor.workers.is_empty());
        assert!(!executor.spawn(|| {}));
    }

    #[test]
    fn executor_keeps_workers_that_started_before_a_later_failure() {
        let executor = Executor::with_spawner(2, |index, receiver| {
            if index == 1 {
                return Err(io::Error::other("simulated worker startup failure"));
            }

            thread::Builder::new()
                .name(format!("wgpu_l3-worker-test-{index}"))
                .spawn(move || worker_loop(&receiver))
        });
        let (sender, receiver) = mpsc::channel();

        assert_eq!(executor.workers.len(), 1);
        assert!(executor.spawn(move || {
            let _ = sender.send(());
        }));
        assert_eq!(
            receiver.recv_timeout(Duration::from_secs(1)),
            Ok(()),
            "the retained worker should execute queued work"
        );
    }
}
