use std::{
  convert::From,
  io::{Error, ErrorKind, Read, Write},
  net::SocketAddr,
};

use mio::{
  net::{TcpListener, TcpStream, UdpSocket},
  Interest, Token,
};

use mio::Registry;

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
      | (ConnectionType::TcpListener(_), ConnectionType::NewTcpListener)
      | (ConnectionType::TcpStream(_), ConnectionType::TcpStream(_))
      | (ConnectionType::TcpStream(_), ConnectionType::NewTcpStream)
      | (ConnectionType::UdpSocket(_), ConnectionType::UdpSocket(_))
      | (ConnectionType::UdpSocket(_), ConnectionType::NewUdpSocket) => true,
      _ => false,
    }
  }

  pub fn register(&mut self, registry: &Registry, token: Token, interest: Interest) {
    match self {
      ConnectionType::TcpStream(stream) => {
        println!("Register");
        if let Err(e) = registry.register(stream, token, interest) {
          panic!("{}", e);
        }
      }
      ConnectionType::TcpListener(stream) => {
        if let Err(e) = registry.register(stream, token, interest) {
          panic!("{}", e);
        }
      }
      ConnectionType::UdpSocket(stream) => {
        if let Err(e) = registry.register(stream, token, interest) {
          panic!("{}", e);
        }
      }
      _ => {}
    }
  }

  pub fn reregister(&mut self, handler: &mut EventHandler, token: Token, interest: Interest) {
    match self {
      ConnectionType::TcpStream(stream) => {
        if let Err(e) = handler.poll.registry().reregister(stream, token, interest) {
          panic!("{}", e);
        }
      }
      ConnectionType::TcpListener(stream) => {
        if let Err(e) = handler.poll.registry().reregister(stream, token, interest) {
          panic!("{}", e);
        }
      }
      ConnectionType::UdpSocket(stream) => {
        if let Err(e) = handler.poll.registry().reregister(stream, token, interest) {
          panic!("{}", e);
        }
      }
      _ => {}
    }
  }

  pub fn deregister(&mut self, registry: &Registry) {
    match self {
      ConnectionType::TcpStream(stream) => {
        println!("Register");
        if let Err(e) = registry.deregister(stream) {
          panic!("{}", e);
        }
      }
      ConnectionType::TcpListener(stream) => {
        if let Err(e) = registry.deregister(stream) {
          panic!("{}", e);
        }
      }
      ConnectionType::UdpSocket(stream) => {
        if let Err(e) = registry.deregister(stream) {
          panic!("{}", e);
        }
      }
      _ => {}
    }
  }

  pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
    match self {
      ConnectionType::TcpStream(stream) => stream.read(buf),
      ConnectionType::TcpListener(stream) => {
        let error = Error::new(ErrorKind::WouldBlock, "tcp listener would block");
        let error2 = Error::new(ErrorKind::WouldBlock, "tcp listener would block");
        Err(stream.take_error().unwrap_or(Some(error)).unwrap_or(error2))
      }
      ConnectionType::UdpSocket(stream) => stream.recv(buf),
      _ => Err(Error::new(ErrorKind::WouldBlock, "udp would block")),
    }
  }

  pub fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
    match self {
      ConnectionType::TcpStream(stream) => stream.write(buf),
      ConnectionType::UdpSocket(stream) => stream.send(buf), // TODO: Check its send and not send to
      _ => Err(Error::new(ErrorKind::WouldBlock, "")),
    }
  }

  pub fn accept(&self) -> Result<(TcpStream, SocketAddr), Error> {
    match self {
      ConnectionType::TcpListener(stream) => stream.accept(),
      _ => Err(Error::new(ErrorKind::WouldBlock, "")),
    }
  }
}

impl From<TcpStream> for ConnectionType {
  fn from(stream: TcpStream) -> Self {
    ConnectionType::add_existing_tcp_stream(stream)
  }
}

impl From<TcpListener> for ConnectionType {
  fn from(stream: TcpListener) -> Self {
    ConnectionType::add_existing_tcp_listener(stream)
  }
}

impl From<UdpSocket> for ConnectionType {
  fn from(stream: UdpSocket) -> Self {
    ConnectionType::add_existing_udp_stream(stream)
  }
}
