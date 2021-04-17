use std::net::TcpStream;
use std::{thread, time};
use log::{warn};
use reactive_net::{CommandMessage, CommandCode};

use crate::{PERIODIC_TASKS, PORT};

const BASE_FREQUENCY : u32 = 50;

#[derive(Clone, Copy)]
pub struct PeriodicTask {
    module : u16,
    entry : u16,
    frequency : u32,
    counter : u32
}

impl PeriodicTask {
    pub fn new(module : u16, entry : u16, frequency : u32) -> PeriodicTask {
        PeriodicTask {
            module,
            entry,
            frequency : set_frequency(frequency),
            counter : 0u32
        }
    }

    pub fn increment_counter(&mut self) -> bool {
        self.counter += BASE_FREQUENCY;

        if self.counter >= self.frequency {
            self.counter = 0;
            true //entry has to be called
        }
        else {
            false
        }
    }

    pub fn get_module(&self) -> u16 {
        self.module
    }

    pub fn get_entry(&self) -> u16 {
        self.entry
    }
}

fn set_frequency(freq : u32) -> u32 {
    if freq <= BASE_FREQUENCY {
        BASE_FREQUENCY
    }
    else {
        freq - freq % BASE_FREQUENCY
    }
}

pub fn run_periodic_tasks() {
    loop {
        // Phase 1: scan vector to update counters and check which are the entry to call now
        let mut local_tasks : Vec<PeriodicTask> = Vec::new();

        let mut tasks = PERIODIC_TASKS.lock().unwrap();

        for task in &mut *tasks {
            let to_call = task.increment_counter();

            if to_call {
                local_tasks.push(task.clone());
            }
        }

        drop(tasks); // done with it

        // Phase 2: for each element in local_tasks, call entry point
        for task in local_tasks {
            let module = task.get_module();
            let entry = task.get_entry();

            let mut payload = Vec::with_capacity(4);
            payload.extend_from_slice(&module.to_be_bytes());
            payload.extend_from_slice(&entry.to_be_bytes());

            let addr = format!("127.0.0.1:{}", *PORT);
            let mut stream = match TcpStream::connect(addr) {
                Ok(s) => s,
                Err(_) => {
                    warn!("Cannot connect to EM");
                    return;
                }
            };

            let cmd = CommandMessage::new(CommandCode::CallEntrypoint, Some(payload));

            if let Err(e) = reactive_net::write_command(&mut stream, &cmd) {
                warn!("{}", e);
            }

            // i don't care about the response (TODO?)
        }

        // Phase 3: go to sleep and repeat
        let sleep_time = time::Duration::from_millis(BASE_FREQUENCY as u64);
        thread::sleep(sleep_time);
    }
}
