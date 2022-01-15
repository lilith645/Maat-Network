use std::{
  io::{Error, ErrorKind, Read},
  net::SocketAddr,
};

use mio::{
  net::{TcpListener, TcpStream, UdpSocket},
  Interest, Token,
};

use crate::modules::EventHandler;

pub enum ConnectionType {
  NewTcpListener,
  NewTcpStream,
  NewUdpSocket,
  TcpListener(TcpListener),
  TcpStream(TcpStream),
  UdpSocket(UdpSocket),
}

impl ConnectionType {
  pub fn tcp_listener() -> ConnectionType {
    ConnectionType::NewTcpListener
  }

  pub fn tcp_stream() -> ConnectionType {
    ConnectionType::NewTcpStream
  }

  pub fn udp_socket() -> ConnectionType {
    ConnectionType::NewUdpSocket
  }

  pub fn add_existing_tcp_stream(stream: TcpStream) -> ConnectionType {
    ConnectionType::TcpStream(stream)
  }

  pub fn add_existing_tcp_listener(stream: TcpListener) -> ConnectionType {
    ConnectionType::TcpListener(stream)
  }

  pub fn add_existing_udp_stream(stream: UdpSocket) -> ConnectionType {
    ConnectionType::UdpSocket(stream)
  }

  pub fn is_type(&self, connection_type: ConnectionType) -> bool {
    match (self, connection_type) {
      (ConnectionType::TcpListener(_), ConnectionType::TcpListener(_))
      | (ConnectionType::TcpStream(_), ConnectionType::TcpStream(_))
      | (ConnectionType::UdpSocket(_), ConnectionType::UdpSocket(_)) => true,
      _ => false,
    }
  }

  pub fn reregister(&mut self, handler: &mut EventHandler, token: Token, interest: Interest) {
    match self {
      ConnectionType::TcpStream(stream) => {
        if let Err(e) = handler.poll.registry().reregister(stream, token, interest) {
          println!("{}", e);
        }
      }
      ConnectionType::TcpListener(stream) => {
        if let Err(e) = handler.poll.registry().reregister(stream, token, interest) {
          println!("{}", e);
        }
      }
      ConnectionType::UdpSocket(stream) => {
        if let Err(e) = handler.poll.registry().reregister(stream, token, interest) {
          println!("{}", e);
        }
      }
      _ => {}
    }
  }

  pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
    match self {
      ConnectionType::TcpStream(stream) => stream.read(buf),
      ConnectionType::TcpListener(_stream) => {
        return Ok(0);
      }
      ConnectionType::UdpSocket(stream) => stream.recv(buf),
      _ => Ok(0),
    }
  }

  pub fn accept(&self) -> Result<(TcpStream, SocketAddr), Error> {
    match self {
      ConnectionType::TcpListener(stream) => stream.accept(),
      //ConnectionType::TcpStream(stream) => stream.accept(),
      //ConnectionType::UdpSocket(stream) => {
      //  stream.accept()
      //},
      _ => Err(Error::new(ErrorKind::WouldBlock, "")),
    }
  }
}
