use std::time::{SystemTime, UNIX_EPOCH};

use crate::MEASURE_TIME;

pub fn measure_time(msg : &str) {
    if !*MEASURE_TIME {
        return;
    }

    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d)   => println!("{}: {} us", msg, d.as_micros()),
        Err(_)  => println!("{}: ERROR", msg)
    }
}
