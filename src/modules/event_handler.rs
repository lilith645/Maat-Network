use mio::{
  net::{TcpListener, TcpStream, UdpSocket},
  {Events, Poll},
};

use crate::modules::ConnectionType;

pub struct EventHandler {
  pub poll: Poll,
  pub events: Events,
  pub next_token: usize,
}

impl EventHandler {
  pub fn new() -> EventHandler {
    let poll = Poll::new().unwrap();
    EventHandler {
      poll,
      events: Events::with_capacity(128),
      next_token: 0,
    }
  }

  pub fn next_token(&mut self) -> usize {
    let token = self.next_token;
    self.next_token += 1;
    return token;
  }
}
