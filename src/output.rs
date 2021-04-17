use std::net::TcpStream;

use crate::connection::Connection;

use reactive_net::{ResultMessage, CommandCode, CommandMessage, Error, EntrypointID};

use log::debug;


pub fn handle_local_connection(payload : Vec<u8>, conn : Connection)
        -> Result<Option<ResultMessage>, Error>{
    debug!("Handling local connection");

    let to_sm = conn.get_sm();
    debug!("To SM: {}", to_sm);

    match connect_to_sm(to_sm, &payload) {
        Ok(res)     => Ok(Some(res)),
        Err(e)      => Err(e)
    }
}


pub fn handle_remote_connection(mut payload : Vec<u8>, conn_id : u16, entry_id : u16,
            conn : Connection) -> Result<Option<ResultMessage>, Error> {
    debug!("Handling remote connection");
    debug!("Connection ID: {}", conn_id);

    let sm_id = conn.get_sm().to_be_bytes();
    // replace entry ID with sm ID
    payload[0] = sm_id[0];
    payload[1] = sm_id[1];


    match EntrypointID::from_u16(entry_id) {
        EntrypointID::HandleInput   => {
            let cmd = CommandMessage::new(CommandCode::RemoteOutput, Some(payload));
            connect_to_em(conn, cmd, false)
        }
        EntrypointID::HandleHandler => {
            let cmd = CommandMessage::new(CommandCode::RemoteRequest, Some(payload));
            connect_to_em(conn, cmd, true)
        }
        _                           => Err(Error::InvalidPayload)
    }
}


pub fn connect_to_sm(sm_id : u16, data : &[u8]) -> Result<ResultMessage, Error> {
    let addr = format!("127.0.0.1:{}", *crate::PORT + sm_id);

    let mut stream = match TcpStream::connect(addr) {
        Ok(s) => s,
        Err(_) => return Err(Error::NetworkError)
    };

    reactive_net::write_message(&mut stream, data)?;
    let result = reactive_net::read_result(&mut stream)?;

    debug!("Response from SM: {:?}", result);
    Ok(result)
}


pub fn connect_to_em(conn : Connection, cmd : CommandMessage, has_resp : bool)
    -> Result<Option<ResultMessage>, Error> {
    let mut stream = match TcpStream::connect(conn.get_address()) {
        Ok(s) => s,
        Err(_) => return Err(Error::NetworkError)
    };

    reactive_net::write_command(&mut stream, &cmd)?;

    match has_resp {
        true    => {
            let result = reactive_net::read_result(&mut stream)?;
            Ok(Some(result))
        }
        false   => Ok(None)
    }
}
