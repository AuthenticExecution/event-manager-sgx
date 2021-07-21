use std::net::{Ipv4Addr, SocketAddrV4};

#[derive(Clone)]
pub struct Connection {
    to_sm : u16,
    address : SocketAddrV4,
    local : bool
}

impl Connection {
    pub fn new(to_sm : u16, ip : Ipv4Addr, port : u16, local : bool) -> Connection {
        Connection {
            to_sm,
            address : SocketAddrV4::new(ip, port),
            local
        }
    }

    pub fn get_sm(&self) -> u16 {
        self.to_sm
    }

    pub fn get_address(&self) -> SocketAddrV4 {
        self.address.clone()
    }

    pub fn is_local_connection(&self) -> bool {
        self.local
    }
}
