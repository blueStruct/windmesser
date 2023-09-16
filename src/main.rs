use std::fs::{create_dir, OpenOptions};
use std::io::{Error, Write};

use chrono::prelude::*;
use std::thread::sleep;
use std::time::{Duration, Instant};

use signal_hook::flag::register;
use signal_hook::{SIGINT, SIGTERM};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use sysfs_gpio::{Direction, Pin};

fn main() -> Result<(), Error> {
    // signal handling
    let exiting = Arc::new(AtomicBool::new(false));
    register(SIGINT, Arc::clone(&exiting))?;
    register(SIGTERM, Arc::clone(&exiting))?;

    // setup
    let pin = Pin::new(2);
    pin.export().expect("could not export Pin");
    pin.set_direction(Direction::In)
        .expect("could not set pin direction");

    let _ = create_dir("/root/wind_daten");
    let minute = Duration::from_secs(60);

    let mut half_rots = 0;
    let mut already_polled = false;
    let mut start = Instant::now();

    // polling loop
    while !exiting.load(Ordering::Relaxed) {
        if start.elapsed() > minute {
            let now = Local::now();
            let file = OpenOptions::new()
                .append(true)
                .create(true)
                .open(format!("/root/wind_daten/{}", now.format("%F"))) // YY-MM-DD
                .expect("could not open file");

            write!(
                &file,
                "{},{}\n",
                now.format("%s"), // secs since epoch
                half_rots / 2
            )
            .expect("could not write to file");

            half_rots = 0;
            start = Instant::now();
        }

        let pin_state = pin.get_value().unwrap();
        match (pin_state, already_polled) {
            (1, false) => {
                already_polled = true;
                half_rots += 1;
                dbg!(half_rots);
            }
            (0, true) => already_polled = false,
            _ => (),
        }

        sleep(Duration::from_millis(1));
    }

    // cleanup and exiting
    pin.unexport().expect("could not unexport Pin");
    println!("\nexiting...");
    Ok(())
}
