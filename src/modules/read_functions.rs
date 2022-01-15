use std::io::ErrorKind;

use mio::Interest;

use crate::{
  modules::{ConnectionType, EventHandler, NetworkStream},
  register_connection,
};

pub fn print_data(
  handler: &mut EventHandler,
  connection: &ConnectionType,
  data: &[u8],
) -> Vec<NetworkStream> {
  println!("{:?}", data);

  Vec::new()
}

pub fn udp_read(
  handler: &mut EventHandler,
  connection: &ConnectionType,
  data: &[u8],
) -> Vec<NetworkStream> {
  println!("{:?}", data);

  Vec::new()
}

pub fn handle_client_read(
  handler: &mut EventHandler,
  connection: &ConnectionType,
  data: &[u8],
) -> Vec<NetworkStream> {
  println!("Recieved data from client: {:?}", data);

  Vec::new()
}

pub fn accept_connections(
  handler: &mut EventHandler,
  connection: &ConnectionType,
  data: &[u8],
) -> Vec<NetworkStream> {
  let mut streams = Vec::new();

  match connection.accept() {
    Ok((new_connection, address)) => {
      println!("Accepting connection");
      println!("    Address: {}", address);

      let stream = register_connection(
        handler,
        ConnectionType::TcpStream(new_connection),
        address.to_string(),
        Some(Box::new(handle_client_read)),
        None,
        Interest::READABLE | Interest::WRITABLE,
      );

      streams.push(stream);
    }
    Err(e) if e.kind() == ErrorKind::WouldBlock => {
      println!("{}", e);
    }
    Err(e) => {
      panic!("{}", e);
    }
  };

  streams
}
