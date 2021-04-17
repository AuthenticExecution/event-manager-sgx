use std::net::TcpStream;

use reactive_net::{ResultCode, ResultMessage, EntrypointID};

use crate::connection::Connection;
use crate::periodic::PeriodicTask;
use crate::helpers::*;
use crate::output::*;
use crate::sm_loaders::*;

use crate::{CONNECTIONS, PERIODIC_TASKS};
use log::{debug, error};


pub fn handle_add_connection(stream : &mut TcpStream) -> Option<ResultMessage> {
    debug!("add_connection payload received");

    // read packet
    let payload = match reactive_net::read_message(stream) {
        Ok(p) => p,
        Err(e) => {
            error!("{}", e);
            return Some(ResultMessage::new(ResultCode::InternalError, None));
        }
    };

    if payload.len() != 10 {
        error!("Payload length is not correct: {}", payload.len());
        return Some(ResultMessage::new(ResultCode::IllegalPayload, None));
    }

    let conn_id = bytes_to_u16(&payload[..2]);
    let to_sm = bytes_to_u16(&payload[2..4]);
    let em_port = bytes_to_u16(&payload[4..6]);
    let addr = match data_to_ipv4(&payload[6..10]) {
        Ok(a) => a,
        Err(e) => {
            error!("{}", e);
            return Some(ResultMessage::new(ResultCode::BadRequest, None));
        }
    };

    debug!("Connection id {} to {}:{} module {}", conn_id, addr, em_port, to_sm);

    let mut connections = CONNECTIONS.lock().unwrap();

    connections.insert(conn_id, Connection::new(to_sm, addr, em_port));

    Some(ResultMessage::new(ResultCode::Ok, None))
}


pub fn handle_call_entrypoint(stream : &mut TcpStream) -> Option<ResultMessage> {
    debug!("call_entrypoint payload received");

    // read packet
    let payload = match reactive_net::read_message(stream) {
        Ok(p) => p,
        Err(e) => {
            error!("{}", e);
            return Some(ResultMessage::new(ResultCode::InternalError, None));
        }
    };

    if payload.len() <= 2 {
        error!("Payload length is not correct");
        return Some(ResultMessage::new(ResultCode::IllegalPayload, None));
    }

    let sm_id = bytes_to_u16(&payload[..2]);

    match connect_to_sm(sm_id, &payload[2..]) {
        Ok(r) => Some(r),
        Err(e) => {
            error!("{}", e);
            Some(ResultMessage::new(ResultCode::InternalError, None))
        }
    }
}


pub fn handle_module_output(stream : &mut TcpStream) -> Option<ResultMessage> {
    debug!("handle_module_output payload received");

    // read packet
    let payload = match reactive_net::read_message(stream) {
        Ok(p) => p,
        Err(e) => {
            error!("{}", e);
            return None;
        }
    };

    if payload.len() <= 4 {
        error!("Payload length is not correct");
        return None;
    }

    let entry_id = bytes_to_u16(&payload[..2]);
    let conn_id = bytes_to_u16(&payload[2..4]);
    let connections = CONNECTIONS.lock().unwrap();
    let conn = match connections.get(&conn_id) {
        Some(c) => (*c).clone(), //copy in order to drop the map and release the lock for other threads
        None => {
            error!("No connection associated to {}", conn_id);
            return None;
        }
    };
    drop(connections); //release lock

    let res = match conn.is_local_connection() {
        true    => handle_local_connection(payload, conn),
        false   => handle_remote_connection(payload, conn_id, entry_id, conn)
    };

    match res {
        Ok(res) => res,
        Err(e)  => {
            error!("handle_module_output: {}", e);
            Some(ResultMessage::new(ResultCode::GenericError, None))
        }
    }
}


pub fn handle_load_sm(stream: &mut TcpStream) -> Option<ResultMessage> {
    debug!("handle_load_sm received");

    match *crate::USE_SGX_LOADER {
        true => load_sm_sgx(stream),
        false => load_sm_native(stream)
    }
}


pub fn handle_ping(_stream : &mut TcpStream) -> Option<ResultMessage> {
    debug!("handle_ping received");

    Some(ResultMessage::new(ResultCode::Ok, None))
}


pub fn handle_register_entrypoint(stream : &mut TcpStream) -> Option<ResultMessage> {
    debug!("register_entrypoint payload received");

    // read packet
    let payload = match reactive_net::read_message(stream) {
        Ok(p) => p,
        Err(e) => {
            error!("{}", e);
            return Some(ResultMessage::new(ResultCode::InternalError, None));
        }
    };

    if payload.len() != 8 {
        error!("Payload length is not correct: {}", payload.len());
        return Some(ResultMessage::new(ResultCode::IllegalPayload, None));
    }

    let module = bytes_to_u16(&payload[..2]);
    let entry = bytes_to_u16(&payload[2..4]);
    let frequency = bytes_to_u32(&payload[4..8]);

    let mut tasks = PERIODIC_TASKS.lock().unwrap();

    tasks.push(PeriodicTask::new(module, entry, frequency));

    Some(ResultMessage::new(ResultCode::Ok, None))
}


pub fn handle_remote_output(stream : &mut TcpStream) -> Option<ResultMessage> {
    // received from another SM
    debug!("handle_remote_output received");

    // read packet
    let mut payload = match reactive_net::read_message(stream) {
        Ok(p) => p,
        Err(e) => {
            error!("{}", e);
            return None;
        }
    };

    if payload.len() <= 6 {
        error!("Payload length is not correct");
        return None;
    }

    let sm_id = bytes_to_u16(&payload[..2]);
    debug!("SM ID: {}", sm_id);

    // HandleInput entrypoint
    let entry_id = (EntrypointID::HandleInput as u16).to_be_bytes();
    payload[0] = entry_id[0];
    payload[1] = entry_id[1];

    if let Err(e) = connect_to_sm(sm_id, &payload) {
        debug!("{}", e);
    }

    None
}

pub fn handle_remote_request(stream : &mut TcpStream) -> Option<ResultMessage> {
    // received from another SM
    debug!("handle_remote_request received");

    // read packet
    let mut payload = match reactive_net::read_message(stream) {
        Ok(p) => p,
        Err(e) => {
            error!("{}", e);
            return None;
        }
    };

    if payload.len() <= 6 {
        error!("Payload length is not correct");
        return None;
    }

    let sm_id = bytes_to_u16(&payload[..2]);
    debug!("SM ID: {}", sm_id);

    // HandleHandler entrypoint
    let entry_id = (EntrypointID::HandleHandler as u16).to_be_bytes();
    payload[0] = entry_id[0];
    payload[1] = entry_id[1];

    match connect_to_sm(sm_id, &payload) {
        Ok(res)     => Some(res),
        Err(e)      => {
            error!("{}", e);
            Some(ResultMessage::new(ResultCode::InternalError, None))
        }
    }
}
