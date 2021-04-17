use std::net::{Ipv4Addr, SocketAddrV4};

#[derive(Clone)]
pub struct Connection {
    to_sm : u16,
    address : SocketAddrV4
}

impl Connection {
    pub fn new(to_sm : u16, ip : Ipv4Addr, port : u16) -> Connection {
        Connection {
            to_sm,
            address : SocketAddrV4::new(ip, port)
        }
    }

    pub fn get_sm(&self) -> u16 {
        self.to_sm
    }

    pub fn get_address(&self) -> SocketAddrV4 {
        self.address.clone()
    }

    pub fn is_local_connection(&self) -> bool {
        let ip = self.address.ip();

        if self.address.port() != *crate::PORT {
            false
        }
        else {
            // too hard to get own IP address (e.g. 192.168.1.1), check only if addr is loopback or 0.0.0.0
            // all the other cases are treated as a remote connection
            ip.is_loopback() || ip.is_unspecified()
        }
    }
}
