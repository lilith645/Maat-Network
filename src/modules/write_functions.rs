use std::io::ErrorKind;

use mio::Interest;

use crate::modules::{ConnectionType, EventHandler};

pub fn write_data(connection: &mut ConnectionType, data: &[u8]) -> bool {
  println!("writing data");
  match connection.write(data) {
    Ok(n) if n < data.len() => {
      dbg!(ErrorKind::WriteZero);
      todo!()
    }
    Ok(_) => {
      // reregister to read
      true
    }
    Err(ref err) if err.kind() == ErrorKind::WouldBlock => false,
    Err(ref err) if err.kind() == ErrorKind::Interrupted => write_data(connection, data),
    Err(e) => {
      panic!("Write Functions: Error writting: {}", e);
    }
  }
}

pub fn empty_write(connection: &mut ConnectionType, data: &mut Vec<u8>) {}

pub fn write_ones(connection: &mut ConnectionType, data: &mut Vec<u8>) {
  data.append(&mut [1, 2, 3, 4, 5, 6].to_vec());

  match connection.write(data) {
    Ok(n) if n < data.len() => {
      dbg!(ErrorKind::WriteZero);
    }
    Ok(_) => {
      // reregister to read
    }
    Err(ref err) if err.kind() == ErrorKind::WouldBlock => {}
    Err(ref err) if err.kind() == ErrorKind::Interrupted => {
      write_ones(connection, data);
    }
    Err(e) => {
      panic!("Write Functions: Error writting: {}", e);
    }
  }
}
