use async_std::task;
use chrono::Utc;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;

extern crate firebase_rs;

use firebase_rs::*;

extern crate clap;
extern crate machine_uid;

use anyhow::Result;
use clap::{App, Arg, SubCommand};
use serde_json::{json, Value};

#[derive(Debug)]
struct CustomError(String);

static FIREBASE_URL: &str = "https://rust-timer-default-rtdb.firebaseio.com";
// static BAD_FIREBASE_URL: &str = "rust-timer-default-rtdb.firebaseio.com";

fn main() {
    let arg_matches = App::new("My Super Program")
        .subcommand(SubCommand::with_name("new"))
        .subcommand(SubCommand::with_name("join").arg(Arg::with_name("id").required(true)))
        .get_matches();

    let db: Firebase;
    match firebase() {
        Ok(f) => db = f,
        Err(_) => {
            println!("ERROR TODO FIX ME");
            std::process::exit(1)
        }
    }

    if arg_matches.subcommand_matches("new").is_some() {
        option_new(db);
    } else if let Some(matches) = arg_matches.subcommand_matches("join") {
        let id = matches.value_of("id").unwrap();
        option_join(db, &id);
    }
}

fn get_connection_id() -> String {
    let uid: String = machine_uid::get().unwrap();
    let digest = md5::compute(uid);
    format!("{:x}", digest)
}

fn option_new(db: Firebase) {
    let connection_id = get_connection_id();
    let end_time = store_future_time(&db, None, 1, connection_id.as_str());
    println!("Timer start, id: {}", connection_id);
    // TODO: remove unwrap?
    task::block_on(task::spawn(notify_at(
        end_time.unwrap(),
        notification,
        Arc::new(AtomicBool::new(false)),
    )));
}

/*
fn already_set_option(db: Firebase, connection_id: &str) -> Option() {
    let timer = firebase
        .at(uid);
        .map_err(|_| CustomError(String::from("Could not set timer")))?;
    let myresult: Option<

}
*/

fn option_join(db: Firebase, id: &str) {
    let current_time = Utc::now().timestamp();
    if let Some(end_time) = retrieve_future_time(&db, id).unwrap() {
        println!("join");
        match current_time < end_time {
            true => task::block_on(task::spawn(notify_at(
                end_time,
                notification,
                Arc::new(AtomicBool::new(false)),
            ))),
            false => println!("YOLO"),
        }
    } else {
        println!("Could not retrieve id: {}", id);
    }
    // 1: timer not expired yet. Start on machine X, now on machine Y
    // 2: (timer expired)
    // 3: (invalid id)
}

fn notification(_ran: Arc<AtomicBool>) {
    println!("Yeah!");
}

async fn notify_at(wakeup_time_epoch: i64, callback: fn(Arc<AtomicBool>), ran: Arc<AtomicBool>) {
    let sleep_seconds = wakeup_time_epoch - Utc::now().timestamp();
    task::block_on(async move { task::sleep(Duration::from_secs(sleep_seconds as u64)).await });
    callback(ran);
}

fn firebase() -> Result<Firebase, UrlParseError> {
    Firebase::new(FIREBASE_URL) //.map_err(|x| CustomError(format!("unable to create Firebase")))?;
}

fn store_future_time(
    firebase: &Firebase,
    given_time: Option<i64>,
    wait_minutes: i64,
    uid: &str,
) -> Result<i64, CustomError> {
    let start_time_epoch = match given_time {
        Some(time) => time,
        None => Utc::now().timestamp(),
    };

    let end_time = start_time_epoch + wait_minutes * 60;
    let end_time_text = format!("{{\"endTime\":{}}}", end_time);
    let timer = firebase
        .at(uid)
        .map_err(|_| CustomError(String::from("Hey toes")))?;

    timer
        .set(&end_time_text)
        .map_err(|_| CustomError(String::from("Unable to set timer")))?;

    return Ok(end_time);
}

fn retrieve_future_time(firebase: &Firebase, uid: &str) -> Result<Option<i64>, CustomError> {
    let timer = firebase
        .at(uid)
        .map_err(|_| CustomError(String::from("Could not set timer")))?;
    let json_payload = timer
        .get()
        .map_err(|_| CustomError(String::from("Could not get timer payload")))?;
    let node: Value = serde_json::from_str(&json_payload.body)
        .map_err(|_| CustomError(String::from("unable parse json")))?;
    if node == json!(null) {
        return Ok(None);
    }
    // TODO: What if endtime is garbage and can't convert to i64?
    // let end_time_i64 = node["endTime"].as_i64().unwrap();
    /*
    node["endTime"]
        .as_i64()
        .ok_or(CustomError(String::from("Could not convert to i64")))
        */
    if let Some(node) = node["endTime"].as_i64() {
        return Ok(Some(node));
    } else {
        return Err(CustomError(String::from("Could not convert to i64")));
        //return Err("Some error happened here")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    fn done(ran: Arc<AtomicBool>) {
        ran.store(true, Ordering::SeqCst);
    }

    // // Fix random failures
    #[test]
    fn notify_at_correct_time() {
        println!("notify at _correct_time");
        let start_time = Utc::now().timestamp();
        let wakeup_time = start_time + 2;

        task::block_on(task::spawn(notify_at(
            wakeup_time,
            done,
            Arc::new(AtomicBool::new(false)),
        )));

        assert!((Utc::now().timestamp() - wakeup_time).abs() < 1);
    }

    #[test]
    fn calls_back_on_timer_completion() {
        println!("calls_back_on_timer_blah");
        let ran = Arc::new(AtomicBool::new(false));
        let read_ran = ran.clone();

        task::block_on(task::spawn(notify_at(
            Utc::now().timestamp() + 1,
            done,
            ran,
        )));

        assert!(read_ran.load(Ordering::SeqCst));
    }

    #[test]
    fn store_future_time_from_duration() {
        let wait_minutes: i64 = 5;
        let start_time_epoch = 0;
        let uid = "TEST123456ABC";
        let firebase = firebase().unwrap();

        let _ = store_future_time(&firebase, Some(start_time_epoch), wait_minutes, uid);

        let end_time = retrieve_future_time(&firebase, uid).unwrap();
        assert_eq!(end_time, Some(300))
    }

    #[test]
    fn store_future_time_returns_end_time() {
        let wait_minutes: i64 = 5;
        let start_time_epoch = 0;
        let uid = "TEST123456ABC";
        let firebase = firebase().unwrap();

        let end_time_result =
            store_future_time(&firebase, Some(start_time_epoch), wait_minutes, uid);

        assert_eq!(end_time_result.unwrap(), 300);
    }
}
