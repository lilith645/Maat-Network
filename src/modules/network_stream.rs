use mio::{Interest, Token};

use crate::{
  modules::{
    read_functions::print_data, write_functions::empty_write, ConnectionType, EventHandler,
  },
  ReadFunc, WriteFunc,
};

pub struct NetworkStream {
  pub token: usize,
  pub stream: ConnectionType,
  pub is_readable: ReadFunc,
  pub is_writeable: WriteFunc,
}

impl NetworkStream {
  pub fn new(
    token: usize,
    stream: ConnectionType,
    read_func: Option<ReadFunc>,
    write_func: Option<WriteFunc>,
  ) -> NetworkStream {
    NetworkStream {
      token,
      stream,
      is_readable: read_func.unwrap_or(Box::new(print_data)),
      is_writeable: write_func.unwrap_or(Box::new(empty_write)),
    }
  }

  pub fn reregister(&mut self, handler: &mut EventHandler) {
    let token = self.token;
    let interest = Interest::READABLE | Interest::WRITABLE;

    self.stream.reregister(handler, Token(token), interest);
  }

  pub fn is_readable(&mut self, handler: &mut EventHandler, data: &[u8]) -> Vec<NetworkStream> {
    let connection = &self.stream;
    (self.is_readable)(handler, connection, data)
  }

  pub fn is_writeable(&mut self, handler: &mut EventHandler, data: &[u8]) -> Vec<NetworkStream> {
    let connection = &self.stream;
    (self.is_writeable)(handler, connection, data);

    Vec::new()
  }
}
