use std::net::Ipv4Addr;
use std::net::TcpStream;
use std::io::prelude::*;
use std::fs::OpenOptions;
use std::convert::TryFrom;


pub fn get_sm_index() -> u16 {
    let mut ind = crate::SM_INDEX.lock().unwrap();

    let new = *ind;
    *ind += 1;

    new
}


pub fn data_to_ipv4(data : &[u8]) -> Result<Ipv4Addr, &str> {
    if data.len() != 4 {
        Err("Data len not valid")
    }
    else {
        Ok(Ipv4Addr::new(data[0], data[1], data[2], data[3]))
    }
}


pub fn bytes_to_u16(buf : &[u8]) -> u16 {
    u16::from_be_bytes([buf[0], buf[1]])
}


pub fn bytes_to_u32(buf : &[u8]) -> u32 {
    u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]])
}


pub fn write_to_file(stream : &mut TcpStream, size : u32, filename : &str) -> std::io::Result<()> {
    let mut file = OpenOptions::new().write(true).create(true).open(filename)?;

    // read data
    let mut buf : [u8; 1024] = [0; 1024];
    let mut size_left = usize::try_from(size).unwrap(); // should never panic
    loop {
        let min = std::cmp::min(size_left, 1024);

        stream.read_exact(&mut buf[..min])?;
        file.write(&buf[..min])?;

        size_left -= min;

        if size_left == 0 {
            break;
        }
    }

    Ok(())
}
