use std::io::ErrorKind;

use mio::Interest;

use crate::modules::{write_functions::write_ones, ConnectionType, EventHandler, NetworkStream};

fn recieve_data(stream: &mut ConnectionType) -> Vec<u8> {
  let mut bytes_read = 0;
  let mut recieved_data = vec![0; 4096];

  loop {
    match stream.read(&mut recieved_data[bytes_read..]) {
      Ok(0) => {
        // close connection
        panic!("Should close connection");
      }
      Ok(n) => {
        bytes_read += n;
        if bytes_read == recieved_data.len() {
          recieved_data.resize(recieved_data.len() + 1024, 0);
        }
      }
      Err(ref e) if ErrorKind::WouldBlock == e.kind() => {
        break;
      }
      Err(ref e) if ErrorKind::Interrupted == e.kind() => {
        continue;
      }
      Err(e) => {
        panic!("An error occured: {}", e);
      }
    }
  }

  recieved_data[..bytes_read].to_vec()
}

pub fn print_data(connection: &mut ConnectionType) -> (Vec<(ConnectionType, String)>, Vec<u8>) {
  (Vec::new(), Vec::new())
}

pub fn udp_read(connection: &mut ConnectionType) -> (Vec<(ConnectionType, String)>, Vec<u8>) {
  (Vec::new(), Vec::new())
}

pub fn read_data_from_client(
  connection: &mut ConnectionType,
) -> (Vec<(ConnectionType, String)>, Vec<u8>) {
  let data = recieve_data(connection);
  (Vec::new(), data)
}

pub fn read_data_from_server(
  connection: &mut ConnectionType,
) -> (Vec<(ConnectionType, String)>, Vec<u8>) {
  (Vec::new(), Vec::new())
}

pub fn accept_connections(
  connection: &mut ConnectionType,
) -> (Vec<(ConnectionType, String)>, Vec<u8>) {
  let mut streams = Vec::new();
  let mut data = Vec::new();

  loop {
    match connection.accept() {
      Ok((new_connection, address)) => {
        streams.push((ConnectionType::from(new_connection), address.to_string()));
      }
      Err(e) if e.kind() == ErrorKind::WouldBlock => {
        break;
      }
      Err(e) => {
        panic!("read_functions: error accepting connection: {}", e);
      }
    };
  }

  (streams, data)
}
