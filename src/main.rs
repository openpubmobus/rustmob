use anyhow::Result;
use async_std::task;
use chrono::Utc;
use firebase_rs::*;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;

use clap::{App, AppSettings, Arg, SubCommand};
use serde_json::{json, Value};

static FIREBASE_URL: &str = "https://rust-timer-default-rtdb.firebaseio.com";

fn main() {
    let app = extract_command_line_args(String::from("RustMob"));
    let arg_matches = app.get_matches();

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
    } else if arg_matches.subcommand_matches("printid").is_some() {
        option_print_id();
    }
}

fn extract_command_line_args<'a>(app_name: String) -> App<'a, 'a> {
    App::new(app_name)
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("new")
                .about("Create a new timer.")
                .arg(
                    Arg::with_name("duration")
                        .required(true)
                        .help("Duration in minutes"),
                ),
        )
        .subcommand(
            SubCommand::with_name("join")
                .about("Join an existing timer")
                .arg(
                    Arg::with_name("id")
                        .required(true)
                        .help("Connection id provided when timer created."),
                ),
        )
        .subcommand(SubCommand::with_name("printid").about("Print local connection id."))
}

fn firebase() -> Result<Firebase> {
    Firebase::new(FIREBASE_URL).map_err(|e| e.into())
}

fn get_connection_id() -> String {
    let uid: String = machine_uid::get().unwrap();
    let digest = md5::compute(uid);
    format!("{:x}", digest)
}

fn option_print_id() {
    println!("Your connection id is {}", get_connection_id());
}

fn option_new(db: Firebase, duration: u64) {
    let connection_id = get_connection_id();
    let existing_timer = retrieve_future_time(&db, connection_id.as_str());
    if let Ok(Some(current_end_time)) = existing_timer {
        if !is_in_past(current_end_time) {
            eprintln!("Timer already started on this machine.");
            std::process::exit(1);
        }
    }

    let end_time = store_future_time(&db, None, duration, connection_id.as_str());
    println!("Timer start, id: {}", connection_id);
    task::block_on(task::spawn(notify_at(
        end_time.unwrap(),
        notification,
        Arc::new(AtomicBool::new(false)),
    )));
}

fn is_in_past(existing_timer: i64) -> bool {
    Utc::now().timestamp() > existing_timer
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

    Ok(end_time)
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

    const SOME_UID: &str = "TEST123456ABC";
    const START_TIME_EPOCH: i64 = 0;
    const FIVE_MINUTES: u64 = 5;

    fn done(ran: Arc<AtomicBool>) {
        ran.store(true, Ordering::SeqCst);
    }

    #[test]
    fn notify_at_correct_time() {
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
        let firebase = firebase().unwrap();

        let _ = store_future_time(&firebase, Some(START_TIME_EPOCH), FIVE_MINUTES, SOME_UID);

        let end_time = retrieve_future_time(&firebase, SOME_UID).unwrap();
        assert_eq!(end_time, Some(300))
    }

    #[test]
    fn store_future_time_returns_end_time() {
        let firebase = firebase().unwrap();

        let end_time_result =
            store_future_time(&firebase, Some(START_TIME_EPOCH), FIVE_MINUTES, SOME_UID);

        assert_eq!(end_time_result.unwrap(), 300);
    }
}
