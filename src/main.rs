use async_std::task;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use chrono::{DateTime, Utc, Local};
extern crate firebase_rs;
use firebase_rs::*;

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

fn store_future_time(givenTime: Option<i64>, how_long: i64) {
    let start_time = match givenTime {
        Some(time) => time,
        None => Utc::now().timestamp(),
    };

    println!("startTime: {:?}", start_time);
    
    let end_time = start_time + how_long * 60 * 1000;
    //let utc_time = DateTime::<Utc>::from_utc(local_time.naive_utc(), Utc);

    println!("end time: {}", end_time);

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
    fn writes_to_firebase() {
        let result = Firebase::new("https://rust-timer-default-rtdb.firebaseio.com");
        let firebase = result.unwrap();

        let users = firebase.at("users").unwrap();
        users.set("{\"username\":\"test\"}").unwrap();

        let res = users.get().unwrap();
        assert_eq!(res.body, "{\"username\":\"test\"}");

    }

    #[test]
    fn save_future_time_from_duration() {
        let duration: u32 = 5;

        store_future_time(None, 5);

        // assert_eq!(... a string holding the UTC 5 minutes out..., if weask for what was sent to Firebase)
//        println!("{:?}", utc_time);
    }

    
}
