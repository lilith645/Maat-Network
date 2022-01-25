use mio::Registry;
use mio::{Interest, Token};

use crate::{
  modules::{
    read_functions::print_data, write_functions::write_data, ConnectionType, EventHandler,
  },
  NewConnection, ReadFunc,
};

pub struct NetworkStream {
  pub addr: String,
  pub token: Option<Token>,
  pub stream: ConnectionType,
  pub is_readable: ReadFunc,
  data_to_write: Vec<Vec<u8>>,
  registered: bool,
  did_write: bool,
}

impl NetworkStream {
  pub fn from_connection<S: Into<String>>(
    connection: ConnectionType,
    addr: S,
    read_func: Option<ReadFunc>,
  ) -> NetworkStream {
    NetworkStream {
      addr: addr.into(),
      token: None,
      stream: connection,
      is_readable: read_func.unwrap_or(Box::new(print_data)),
      data_to_write: Vec::new(),
      registered: false,
      did_write: false,
    }
  }

  pub fn unregistered(&self) -> bool {
    !self.registered
  }

  pub fn token(&self) -> Option<Token> {
    self.token
  }

  pub fn set_token(&mut self, t: usize) {
    self.token = Some(Token(t));
  }

  pub fn did_write(&self) -> bool {
    self.did_write
  }

  pub fn register(&mut self, register: &Registry, token: Token, interest: Interest) {
    debug_assert!(self.unregistered());
    println!("Registering Address: {}", self.addr);
    self.registered = true;
    self.stream.register(register, token, interest);
    self.token = Some(token);
  }

  pub fn deregister(&mut self, register: &Registry) {
    debug_assert!(!self.unregistered());
    self.registered = false;
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

  pub fn data_to_write(&mut self, data: &[u8]) {
    println!("data passed to write :{:?}", data);
    self.data_to_write.push(data.to_vec());
  }

  pub fn data_pending(&self) -> bool {
    self.data_to_write.len() > 0
  }

  pub fn is_readable(&mut self, data: &[u8]) -> Vec<(ConnectionType, String)> {
    debug_assert!(!self.unregistered());
    let connection = &mut self.stream;
    (self.is_readable)(connection, data)
  }

  pub fn is_writeable(&mut self) -> Vec<NetworkStream> {
    debug_assert!(!self.unregistered());
    let connection = &mut self.stream;

    println!("connection is writable");
    self.data_to_write = self
      .data_to_write
      .drain(..)
      .map(|d| {
        if write_data(connection, &d) {
          self.did_write = true;
          None
        } else {
          Some(d)
        }
      })
      .filter(|d| d.is_some())
      .map(|d| d.unwrap())
      .collect::<Vec<Vec<u8>>>();

    Vec::new()
  }
}

impl From<NewConnection> for NetworkStream {
  fn from(connection: NewConnection) -> Self {
    let mut n =
      NetworkStream::from_connection(connection.connection, connection.addr, connection.read_func);
    n.set_token(connection.token);
    n
  }
}
