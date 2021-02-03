use async_std::task;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use chrono::Utc;
extern crate firebase_rs;
use firebase_rs::*;

static FIREBASE_URL: &str = "https://rust-timer-default-rtdb.firebaseio.com";

fn firebase() -> Firebase {
    Firebase::new(FIREBASE_URL).unwrap()
}

fn main() {}

#[derive(PartialEq, Eq, Debug)]
enum Status {
    Inactive,
    Running,
    Ended,
}

async fn timer(
    seconds: u64,
    status: Arc<RwLock<Status>>,
    callback: fn(Arc<AtomicBool>),
    ran: Arc<AtomicBool>,
) {
    *status.write().unwrap() = Status::Running;
    task::block_on(async move {
        task::sleep(Duration::from_secs(seconds)).await });
    *status.write().unwrap() = Status::Ended;
    callback(ran);
}

fn done(ran: Arc<AtomicBool>) {
    ran.store(true, Ordering::SeqCst);
}

fn store_future_time(given_time: Option<i64>, wait_minutes: i64) {
    let start_time_epoch = match given_time {
        Some(time) => time,
        None => Utc::now().timestamp(),
    };

    let end_time = start_time_epoch + wait_minutes * 60 * 1000;
    let end_time_text = format!("{{\"endTime\":{}}}", end_time);
    let timer = firebase().at("timer").unwrap();
    timer.set(&end_time_text).unwrap();

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calls_back_on_timer_completion() {
        let status = Arc::new(RwLock::new(Status::Inactive));
        let read_status = status.clone();
        let ran = Arc::new(AtomicBool::new(false));
        let read_ran = ran.clone();

        task::block_on(task::spawn(timer(3, status, done, ran)));
        assert_eq!(*read_status.read().unwrap(), Status::Ended);
        assert!(read_ran.load(Ordering::SeqCst));
    }

    #[test]
    fn save_future_time_from_duration() {
        let wait_minutes: i64 = 5;
        let start_time_epoch = 0;

        store_future_time(Some(start_time_epoch), wait_minutes);

        let timer = firebase().at("timer").unwrap();
        let res = timer.get().unwrap();
        assert_eq!(res.body, "{\"endTime\":300000}")
    }
}
