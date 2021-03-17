use async_std::task;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use chrono::Utc;
extern crate firebase_rs;
use firebase_rs::*;
extern crate machine_uid;
extern crate clap;
use clap::{Arg, App, SubCommand};
use serde_json::{Value, Deserializer};
use anyhow::Result;

//static FIREBASE_URL: &str = "https://rust-timer-default-rtdb.firebaseio.com";
static FIREBASE_URL: &str = "rust-timer-default-rtdb.firebaseio.com";


fn main() {
    // let matches = App::new("My Super Program")
    //      .subcommand(SubCommand::with_name("new"))
    //      .subcommand(SubCommand::with_name("join").arg(Arg::with_name("id").required(true)))
    //      .get_matches();
    // if let Some(matches) = matches.subcommand_matches("new") {
    //     let id: String = machine_uid::get().unwrap();
    //     let end_time = store_future_time(None, 1, id.as_str());
    //     println!("Timer start, id: {}", id);
    //     // TODO: remove unwrap?
    //     task::block_on(task::spawn(notify_at(end_time.unwrap(), notification, Arc::new(AtomicBool::new(false)))));
    // }
    // if let Some(matches) = matches.subcommand_matches("join") {
    //     println!("join");
    //     let id = matches.value_of("id").unwrap();
    //     let current_time = Utc::now().timestamp();
    //     let end_time = retrieve_future_time(id).unwrap();
    //     match (current_time < end_time) {
    //         true =>
    //                 task::block_on(task::spawn(notify_at(end_time, notification, Arc::new(AtomicBool::new(false))))),
    //         false => println!("YOLO"),
    //     }
    //
    //
    //     // 1: timer not expired yet. Start on machine X, now on machine Y
    //     // 2: (timer expired)
    //     // 3: (invalid id)
    //     println!("id: {}", id);
    // }
}

fn notification(ran: Arc<AtomicBool>) {
    println!("Yeah!");
}

async fn notify_at(
    wakeup_time_epoch: i64,
    callback: fn(Arc<AtomicBool>),
    ran: Arc<AtomicBool>,
) {
    let sleep_seconds = wakeup_time_epoch - Utc::now().timestamp();
    task::block_on(async move {
    task::sleep(Duration::from_secs(sleep_seconds as u64)).await });
    callback(ran);
}

fn done(ran: Arc<AtomicBool>) {
    ran.store(true, Ordering::SeqCst);
}

#[derive(Debug)]
struct CustomError(String);

fn firebase() -> Result<Firebase,UrlParseError> {
    Firebase::new(FIREBASE_URL) //.map_err(|x| CustomError(format!("unable to create Firebase")))?;
}

fn store_future_time(given_time: Option<i64>, wait_minutes: i64, uid: &str) -> Result<i64, CustomError> {
//fn store_future_time(given_time: Option<i64>, wait_minutes: i64, uid: &str) -> Result<i64, UrlParseError> {
    let start_time_epoch = match given_time {
        Some(time) => time,
        None => Utc::now().timestamp(),
    };

    let end_time = start_time_epoch + wait_minutes * 60;
    let end_time_text = format!("{{\"endTime\":{}}}", end_time);
    //let timer = firebase().at(uid).map_err(|x| std::fmt::Error)?;
    // let timer = firebase().at(uid).map_err(|x| {println!("==========> {:?}", x); return std::fmt::Error} )?;
    let firebase = firebase().map_err(|_| {
        println!("firebase problem");
        CustomError(format!("unable to connect to firebase"))
    })?;
    println!("should not see this");
    let timer = firebase.at(uid)
        .map_err(|x| {
            println!("map err on x");
            return CustomError(format!("Hey toes"));
        }) ? ;
    println!("Hi mum {:?}", timer);

    // timer.set(&end_time_text).map_err(|_| std::fmt::Error)?;

    return Ok(end_time);
}

// fn retrieve_future_time(uid: &str) -> Result<i64> {
//     let timer = firebase().at(uid).map_err(|_| std::fmt::Error)?;
//     let json_payload = timer.get().map_err(|_| std::fmt::Error)?;
//     let node: Value = serde_json::from_str(&json_payload.body)?;
//
//     // TODO: What if endtime is garbage and can't convert to i64?
//     // let end_time_i64 = node["endTime"].as_i64().unwrap();
//     let end_time_i64 = node["endTime"].as_i64();
//     match end_time_i64 {
//         Some(x) => {println!("====> Go some: {}", x) ; return Ok(x)},
//         None => {println!("===> Got none"); return Ok(0);},
//     }
//     //return Ok(end_time_i64);
// }

#[cfg(test)]
mod tests {
    use super::*;

    // // Fix random failures
    // #[test]
    // fn notify_at_correct_time() {
    //     println!("notify at _correct_time");
    //     let start_time = Utc::now().timestamp();
    //     let wakeup_time = start_time + 2;
    //
    //     task::block_on(task::spawn(notify_at(wakeup_time, done, Arc::new(AtomicBool::new(false)))));
    //
    //     assert!((Utc::now().timestamp() - wakeup_time).abs() < 1);
    // }
    //
    // #[test]
    // fn calls_back_on_timer_completion() {
    //     println!("calls_back_on_timer_blah");
    //     let ran = Arc::new(AtomicBool::new(false));
    //     let read_ran = ran.clone();
    //
    //     task::block_on(task::spawn(notify_at(Utc::now().timestamp() + 1, done, ran)));
    //
    //     assert!(read_ran.load(Ordering::SeqCst));
    // }

    #[test]
    /*
    fn store_future_time_from_duration() {
        println!("store_future_time_from_duration");
        let wait_minutes: i64 = 5;
        let start_time_epoch = 0;
        let uid = "TEST123456ABC";

        store_future_time(Some(start_time_epoch), wait_minutes, uid);

        let end_time = retrieve_future_time(uid).unwrap();
        assert_eq!(end_time, 300)
    }
    */
    fn store_future_time_from_duration() {
        println!("store_future_time_from_duration");
        let wait_minutes: i64 = 5;
        let start_time_epoch = 0;
        let uid = "TEST123456ABC";

        let result = store_future_time(Some(start_time_epoch), wait_minutes, uid);

        match result {
            Ok(x) => { println!("====> Got some: {}", x); },
            Err(e) => { println!("===> Error {:?}", e); },
        }
    }

    // #[test]
    // fn store_future_time_returns_end_time() {
    //     println!("store_future_time_returns_end_time");
    //     let wait_minutes: i64 = 5;
    //     let start_time_epoch = 0;
    //     let uid = "TEST123456ABC";
    //
    //     let end_time_result = store_future_time(Some(start_time_epoch), wait_minutes, uid);
    //
    //     assert_eq!(end_time_result.unwrap(), 300);
    // }
}
