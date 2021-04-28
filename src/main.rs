use anyhow::Result;
use async_std::task;
use chrono::Utc;
use clap;
use firebase_rs::*;
use machine_uid;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;

use clap::{App, Arg, SubCommand};
use serde_json::{json, Value};

static FIREBASE_URL: &str = "https://rust-timer-default-rtdb.firebaseio.com";

fn main() {
    let arg_matches = App::new("RustMob")
        .subcommand(SubCommand::with_name("new").arg(Arg::with_name("duration").required(true)))
        .subcommand(SubCommand::with_name("join").arg(Arg::with_name("id").required(true)))
        .get_matches();

    let db: Firebase;
    match firebase() {
        Ok(f) => db = f,
        Err(e) => {
            eprintln!("Firebase connection error:");
            eprintln!("{}", e);
            std::process::exit(1)
        }
    }

    if let Some(matches) = arg_matches.subcommand_matches("new") {
        let duration = clap::value_t!(matches.value_of("duration"), u64).unwrap_or_else(|e| {
            eprintln!("{}", e);
            std::process::exit(1)
        });
        option_new(db, duration);
    } else if let Some(matches) = arg_matches.subcommand_matches("join") {
        let id = matches.value_of("id").unwrap();
        option_join(db, &id);
    }
}

fn firebase() -> Result<Firebase> {
    Firebase::new(FIREBASE_URL).map_err(|e| e.into())
}

fn get_connection_id() -> String {
    let uid: String = machine_uid::get().unwrap();
    let digest = md5::compute(uid);
    format!("{:x}", digest)
}

fn option_new(db: Firebase, duration: u64) {
    let connection_id = get_connection_id();
    let existing_timer = retrieve_future_time(&db, connection_id.as_str());
    println!("XXXX {:?}", existing_timer);
    // We need to check if the time retrieve is in the past <<<<<<== TODO
    if let Ok(Some(_)) = existing_timer {
        eprintln!("Timer already started on this machine."); std::process::exit(1);
    }


    let end_time = store_future_time(&db, None, duration, connection_id.as_str());
    println!("Timer start, id: {}", connection_id);
    task::block_on(task::spawn(notify_at(
        end_time.unwrap(),
        notification,
        Arc::new(AtomicBool::new(false)),
    )));
}

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
            false => println!("timer already expired"),
        }
    } else {
        println!("Could not retrieve id: {}", id);
    }
    // 1: timer not expired yet. Start on machine X, now on machine Y
    // 2: (timer expired)
    // 3: (invalid id)
}

fn notification(_ran: Arc<AtomicBool>) {
    println!("--done--");
}

async fn notify_at(wakeup_time_epoch: i64, callback: fn(Arc<AtomicBool>), ran: Arc<AtomicBool>) {
    let sleep_seconds = wakeup_time_epoch - Utc::now().timestamp();
    task::block_on(async move { task::sleep(Duration::from_secs(sleep_seconds as u64)).await });
    callback(ran);
}

fn store_future_time(
    firebase: &Firebase,
    given_time: Option<i64>,
    wait_minutes: u64,
    uid: &str,
) -> Result<i64> {
    let start_time_epoch = match given_time {
        Some(time) => time,
        None => Utc::now().timestamp(),
    };

    let end_time = start_time_epoch + (wait_minutes as i64) * 60;
    let timer = firebase.at(uid)?;
    timer.set(&format!("{{\"endTime\":{}}}", end_time))?;

    return Ok(end_time);
}

fn retrieve_future_time(firebase: &Firebase, uid: &str) -> Result<Option<i64>> {
    let timer = firebase.at(uid)?;
    let json_payload = timer.get()?;
    let node: Value = serde_json::from_str(&json_payload.body)?;
    if node == json!(null) {
        return Ok(None);
    }
    if let Some(node) = node["endTime"].as_i64() {
        Ok(Some(node))
    } else {
        Ok(None)
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
