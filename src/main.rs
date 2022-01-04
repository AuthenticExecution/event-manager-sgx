extern crate ctrlc;
#[macro_use] extern crate lazy_static;
extern crate reactive_net;
extern crate tempfile;

use std::env;
use std::str::FromStr;
use std::sync::Mutex;
use std::thread;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::collections::HashMap;
use log::{info, debug, error, LevelFilter};
use simple_logger::SimpleLogger;
use std::fs;
use threadpool::ThreadPool;

mod handlers;
mod helpers;
mod output;
mod connection;
mod sm_loaders;
mod periodic;
mod time;
use connection::Connection;
use periodic::PeriodicTask;

use reactive_net::{ResultCode, CommandCode, ResultMessage};


lazy_static! {
    static ref PORT : u16 = {
        let port = env::var("EM_PORT");
        port.expect("Missing EM_PORT environment variable").parse::<u16>().expect("Port must be an u16!")
    };

    static ref MEASURE_TIME : bool = {
        env::var("EM_MEASURE_TIME").unwrap_or("false".to_string()).parse::<bool>()
            .expect("EM_MEASURE_TIME must be a bool")
    };

    static ref SM_INDEX : Mutex<u16> = {
        Mutex::new(0)
    };

    static ref TEMP_DIR : tempfile::TempDir = tempfile::tempdir().expect("Failed to create temp dir");

    static ref USE_SGX_LOADER : bool = {
        env::var("EM_SGX").unwrap_or("true".to_string()).parse::<bool>().expect("EM_SGX must be a bool")
    };

    static ref CONNECTIONS: Mutex<HashMap<u16, Connection>> = {
        Mutex::new(HashMap::new())
    };

    static ref PERIODIC_TASKS: Mutex<Vec<PeriodicTask>> = {
        Mutex::new(Vec::new())
    };
}


fn handle_client(mut stream: TcpStream) {
    let mut buf : [u8; 1] = [0; 1];

    // read first byte: message type
    if let Err(_) = stream.read_exact(&mut buf) {
           error!("Error while reading from socket");
           return;
    };

    // check code
    let res = match CommandCode::from_u8(buf[0]) {
        Some(r) => match r {
            CommandCode::AddConnection      => handlers::handle_add_connection(&mut stream),
            CommandCode::CallEntrypoint     => handlers::handle_call_entrypoint(&mut stream),
            CommandCode::RemoteOutput       => handlers::handle_remote_output(&mut stream),
            CommandCode::LoadSM             => handlers::handle_load_sm(&mut stream),
            CommandCode::Ping               => handlers::handle_ping(&mut stream),
            CommandCode::RegisterEntrypoint => handlers::handle_register_entrypoint(&mut stream),
            CommandCode::ModuleOutput       => handlers::handle_module_output(&mut stream),
            CommandCode::RemoteRequest      => handlers::handle_remote_request(&mut stream)
        },
        None    => {
            error!("Invalid code received");
            Some(ResultMessage::new(ResultCode::IllegalCommand, None))
        }
    };

    debug!("Result: {:?}", res);

    if let Some(response) = res {
        if let Err(s) = reactive_net::write_result(&mut stream, &response) {
            error!("{}", s);
        }
    }
}

fn init_loglevel() {
    let level_str = env::var("EM_LOG").unwrap_or("info".to_string());
    let level = LevelFilter::from_str(&level_str).expect("Bad log level EM_LOG");

    SimpleLogger::new()
        .with_level(level)
        .with_utc_timestamps()
        .init()
        .unwrap();

    info!("EM_LOG: {}", level);
}

fn init_periodic_tasks() {
    let is_enabled = env::var("EM_PERIODIC_TASKS").unwrap_or("false".to_string()).parse::<bool>()
        .expect("EM_PERIODIC_TASKS must be a bool");

    if is_enabled {
        thread::spawn(|| {periodic::run_periodic_tasks()});
    }

    debug!("EM_PERIODIC_TASKS: {}", is_enabled);
}

fn init_thread_pool() -> ThreadPool {
    let n_workers = env::var("EM_THREADS").unwrap_or("16".to_string())
        .parse::<usize>().expect("EM_THREADS must be an usize!");

    debug!("EM_THREADS: {}", n_workers);

    ThreadPool::new(n_workers)
}

fn main()  -> std::io::Result<()> {
    let host = format!("0.0.0.0:{}", *PORT);
    init_loglevel();
    info!("EM_SGX: {}", *USE_SGX_LOADER);
    info!("EM_MEASURE_TIME: {}", *MEASURE_TIME);

    // set handler for SIGTERM signal, to delete temp directory
    ctrlc::set_handler(|| {
        let _ = fs::remove_dir_all(&*TEMP_DIR.path());
        std::process::exit(0);
    }).expect("Error setting Ctrl-C handler");

    // init periodic tasks thread (only if env var is defined - default: disabled)
    init_periodic_tasks();

    // init worker threads
    let pool = init_thread_pool();

    info!("Listening on {}", host);
    let listener = TcpListener::bind(host)?;

    for stream in listener.incoming() {
        debug!("Received new connection");

        match stream {
            Ok(s)   => pool.execute(|| { handle_client(s) } ),
            Err(e)  => error!("Connection error: {}", e)
        }

        debug!("Connection ended\n");
    }
    Ok(())
}
