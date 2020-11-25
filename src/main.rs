fn timer(duration: u64, callback: &dyn Fn(&mut String),  status: &mut String ) {
    // sleep(duration);
    callback(status);
}


fn done(message: &mut String) {
    *message = "New String".to_string();
    println!("I'm done");
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calls_back_on_timer_completion() {
        let mut status: String = String::new();
        timer(5, &done, &mut status);

        // let dir_option = readdir("assets");
        // assert_eq!(dir_option.is_ok(), true);
    }
}
