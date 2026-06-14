use std::thread;

use crate::Task;
use crate::app::mailbox::Message;
use crate::app::{MailboxSender, Sender};

pub fn spawn<T: Send + 'static>(task: Task<T>, sender: Sender<T>) {
    spawn_with(task, sender);
}

pub fn spawn_with<T, S>(task: Task<T>, sender: S)
where
    T: Send + 'static,
    S: MailboxSender<T> + Send + 'static,
{
    thread::spawn(move || {
        let event = task.run();
        let _ = sender.send_message(Message::AppTaskCompleted(event));
    });
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use crate::app::SendError;

    use super::*;

    #[derive(Clone)]
    struct TestSender<T> {
        sender: mpsc::Sender<Message<T>>,
    }

    impl<T: Send + 'static> MailboxSender<T> for TestSender<T> {
        fn send_message(&self, message: Message<T>) -> Result<(), SendError> {
            self.sender.send(message).map_err(|_| SendError)
        }
    }

    #[test]
    fn app_task_output_becomes_app_task_completion_message() {
        let (sender, receiver) = mpsc::channel();

        spawn_with(Task::future(async { 7 }), TestSender { sender });

        assert_eq!(receiver.recv().unwrap(), Message::AppTaskCompleted(7));
    }
}
