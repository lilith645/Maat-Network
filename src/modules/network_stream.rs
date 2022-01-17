use mio::Registry;
use mio::{Interest, Token};

use crate::{
  modules::{
    read_functions::print_data, write_functions::empty_write, ConnectionType, EventHandler,
  },
  ReadFunc, WriteFunc,
};

pub struct NetworkStream {
  pub addr: String,
  pub token: Option<Token>,
  pub stream: ConnectionType,
  pub is_readable: ReadFunc,
  pub is_writeable: WriteFunc,
}

impl NetworkStream {
  pub fn new<S: Into<String>>(
    addr: S,
    token: Token,
    registry: &Registry,
    interest: Interest,
    mut stream: ConnectionType,
    read_func: Option<ReadFunc>,
    write_func: Option<WriteFunc>,
  ) -> NetworkStream {
    stream.register(registry, token, interest);
    NetworkStream {
      addr: addr.into(),
      token: Some(token),
      stream,
      is_readable: read_func.unwrap_or(Box::new(print_data)),
      is_writeable: write_func.unwrap_or(Box::new(empty_write)),
    }
  }

  pub fn from_connection<S: Into<String>>(
    connection: ConnectionType,
    addr: S,
    read_func: Option<ReadFunc>,
    write_func: Option<WriteFunc>,
  ) -> NetworkStream {
    NetworkStream {
      addr: addr.into(),
      token: None,
      stream: connection,
      is_readable: read_func.unwrap_or(Box::new(print_data)),
      is_writeable: write_func.unwrap_or(Box::new(empty_write)),
    }
  }

  pub fn unregistered(&self) -> bool {
    self.token.is_none()
  }

  pub fn token(&self) -> Option<Token> {
    self.token
  }

  pub fn register(&mut self, register: &Registry, token: Token, interest: Interest) {
    debug_assert!(self.unregistered());
    println!("Registering Address: {}", self.addr);
    self.stream.register(register, token, interest);
    self.token = Some(token);
  }

  pub fn deregister(&mut self, register: &Registry) {
    debug_assert!(!self.unregistered());
    println!("Deregistering Address: {}", self.addr);
    self.stream.deregister(register);
  }

  pub fn reregister(&mut self, handler: &mut EventHandler, interest: Interest) {
    debug_assert!(!self.unregistered());
    if let Some(token) = self.token {
      println!("Reregistering Address: {}", self.addr);
      self.stream.reregister(handler, token, interest);
    }
  }

  pub fn is_readable(&mut self) -> (Vec<(ConnectionType, String)>, Vec<u8>) {
    debug_assert!(!self.unregistered());
    let connection = &mut self.stream;
    (self.is_readable)(connection)
  }

  pub fn is_writeable(&mut self, data: &mut Vec<u8>) -> Vec<NetworkStream> {
    debug_assert!(!self.unregistered());
    println!("{}: Writing data", self.addr);
    let connection = &mut self.stream;
    (self.is_writeable)(connection, data);

    Vec::new()
  }
}
