use async_std::task;
use std::sync::{Arc, RwLock};
use std::time::Duration;

fn main() {}

#[derive(PartialEq, Eq, Debug)]
enum Status {
    Inactive,
    Running,
    Ended,
}

async fn timer(seconds: u64 /*, callback: &dyn Fn()*/, status: std::sync::Arc<RwLock<Status>>) {
    {
        let mut write_status = status.write().unwrap();
        *write_status = Status::Running;
    }
    let duration = Duration::from_secs(seconds);
    task::block_on(async move { task::sleep(duration).await });
    {
        let mut write_status = status.write().unwrap();
        *write_status = Status::Ended;
    }
    // callback();
}

fn done() {
    println!("I'm done");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calls_back_on_timer_completion() {
        let status = Arc::new(RwLock::new(Status::Inactive));
        let read_status = status.clone();

        task::block_on(task::spawn(timer(3, status)));
        // println!("{:?}", read_status.read().unwrap());

        assert_eq!(*read_status.read().unwrap(), Status::Ended);
    }
}
