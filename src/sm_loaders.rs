use std::net::TcpStream;
use std::io::prelude::*;
use std::process::Command;

use crate::helpers::*;
use reactive_net::{ResultCode, ResultMessage};

use log::{debug, error};

pub fn load_sm_sgx(stream: &mut TcpStream) -> Option<ResultMessage> {
    let ind = get_sm_index();

    let dir_path =  &*crate::TEMP_DIR.path();
    let sgxs = dir_path.join(&format!("m{}.sgxs", ind));
    let sgxs = sgxs.to_str().unwrap(); // should never panic
    let sig = dir_path.join(&format!("m{}.sig", ind));
    let sig = sig.to_str().unwrap(); // should never panic

    // payload is: [<sgxs_size><sgxs><sig_size><sig>]

    // read data and store files on disk
    let mut buf : [u8; 4] = [0; 4];

    //read sgxs file
    if let Err(_) = stream.read_exact(&mut buf) {
        error!("Wrong payload for handle_load_sm");
        return Some(ResultMessage::new(ResultCode::IllegalPayload, None));
    }

    let sgxs_size = bytes_to_u32(&buf);
    if let Err(msg) = write_to_file(stream, sgxs_size, &sgxs) {
        error!("{}", msg);
        return Some(ResultMessage::new(ResultCode::InternalError, None));
    }

    // read signature
    if let Err(_) = stream.read_exact(&mut buf) {
        error!("Wrong payload for handle_load_sm");
        return Some(ResultMessage::new(ResultCode::IllegalPayload, None));
    }

    let sig_size = bytes_to_u32(&buf);
    if let Err(msg) = write_to_file(stream, sig_size, &sig) {
        error!("{}", msg);
        return Some(ResultMessage::new(ResultCode::InternalError, None));
    }

    // run enclave
    if let Err(_) = Command::new("ftxsgx-runner")
    .args(&["-s", "coresident", &sgxs])
    .spawn() {
        error!("program failed to start");
        Some(ResultMessage::new(ResultCode::InternalError, None))
    }
    else {
        debug!("Module started successfully");
        Some(ResultMessage::new(ResultCode::Ok, None))
    }
}

pub fn load_sm_native(stream: &mut TcpStream) -> Option<ResultMessage> {
    let ind = get_sm_index();

    let dir_path =  &*crate::TEMP_DIR.path();
    let filename = dir_path.join(&format!("sm{}", ind));
    let filename = filename.to_str().unwrap();    // payload is: [<exe_size><exe>]

    // read data and store files on disk
    let mut buf : [u8; 4] = [0; 4];

    //read exec file
    if let Err(_) = stream.read_exact(&mut buf) {
        error!("Wrong payload for handle_load_sm");
        return Some(ResultMessage::new(ResultCode::IllegalPayload, None));
    }

    let exec_size = bytes_to_u32(&buf);
    if let Err(msg) = write_to_file(stream, exec_size, &filename) {
        error!("{}", msg);
        return Some(ResultMessage::new(ResultCode::InternalError, None));
    }

    let out_chmod = match Command::new("chmod")
            .args(&["+x", filename])
            .output() {
                Ok(o) => o,
                Err(_) => {
                    error!("Failed to set permissions");
                    return Some(ResultMessage::new(ResultCode::InternalError, None));
                }
    };

    match out_chmod.status.code() {
        Some(x) if x == 0 => (),
        _ => {
                error!("Chmod failed");
                return Some(ResultMessage::new(ResultCode::InternalError, None));
            }
    };

    if let Err(_) = Command::new(filename).spawn() {
        error!("program failed to start");
        Some(ResultMessage::new(ResultCode::InternalError, None))
    }
    else {
        debug!("Module started successfully");
        Some(ResultMessage::new(ResultCode::Ok, None))
    }
}
