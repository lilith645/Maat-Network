use mio::{
  net::{TcpListener, TcpStream, UdpSocket},
  {Events, Poll},
};

use crate::modules::ConnectionType;

pub struct EventHandler {
  pub poll: Poll,
  pub events: Events,
  pub last_token: usize,
}

impl EventHandler {
  pub fn new() -> EventHandler {
    let poll = Poll::new().unwrap();
    EventHandler {
      poll,
      events: Events::with_capacity(128),
      last_token: 0,
    }
  }

  pub fn next_token(&mut self) -> usize {
    self.last_token += 1;
    return self.last_token;
  }
}

pub fn create_connection_stream<T: Into<String>>(
  con_type: ConnectionType,
  addr: T,
) -> ConnectionType {
  match con_type {
    ConnectionType::NewTcpStream => {
      ConnectionType::TcpStream(TcpStream::connect(addr.into().parse().unwrap()).unwrap())
    }
    ConnectionType::NewTcpListener => {
      ConnectionType::TcpListener(TcpListener::bind(addr.into().parse().unwrap()).unwrap())
    }
    ConnectionType::NewUdpSocket => {
      ConnectionType::UdpSocket(UdpSocket::bind(addr.into().parse().unwrap()).unwrap())
    }
    _ => con_type,
  }
}
