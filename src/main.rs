use mio::net::{TcpListener, TcpStream, UdpSocket};
use mio::{Interest, Token};

use std::io::ErrorKind;

use modules::{
  read_functions::{accept_connections, udp_read},
  ConnectionType, EventHandler, NetworkStream,
};

pub type WriteFunc = Box<dyn Fn(&mut EventHandler, &ConnectionType, &[u8])>;
pub type ReadFunc = Box<dyn Fn(&mut EventHandler, &ConnectionType, &[u8]) -> Vec<NetworkStream>>;

mod modules;

pub struct NetworkData {
  token: usize,
  data: Vec<u8>,
}

pub struct MaatNetwork {}

impl NetworkData {
  pub fn new(token: usize, data: &[u8]) -> NetworkData {
    NetworkData {
      token,
      data: data.to_vec(),
    }
  }

  pub fn debug(&self) {
    println!("Token {}: {:?}", self.token, self.data);
  }
}

pub fn register_connection<S: Into<String>>(
  handler: &mut EventHandler,
  stream: ConnectionType,
  addr: S,
  read_func: Option<ReadFunc>,
  write_func: Option<WriteFunc>,
  interest: Interest,
) -> NetworkStream {
  let token = handler.next_token();

  let stream = {
    match stream {
      ConnectionType::NewTcpListener => {
        let mut listener = TcpListener::bind(addr.into().parse().unwrap()).unwrap();
        if let Err(e) = handler
          .poll
          .registry()
          .register(&mut listener, Token(token), interest)
        {
          panic!("{}", e);
        }

        ConnectionType::TcpListener(listener)
      }
      ConnectionType::NewTcpStream => {
        let mut stream = TcpStream::connect(addr.into().parse().unwrap()).unwrap();
        if let Err(e) = handler
          .poll
          .registry()
          .register(&mut stream, Token(token), interest)
        {
          panic!("{}", e);
        }

        ConnectionType::TcpStream(stream)
      }
      ConnectionType::NewUdpSocket => {
        let mut udp = UdpSocket::bind(addr.into().parse().unwrap()).unwrap();
        if let Err(e) = handler
          .poll
          .registry()
          .register(&mut udp, Token(token), interest)
        {
          panic!("{}", e);
        }

        ConnectionType::UdpSocket(udp)
      }
      _ => stream,
    }
  };

  let stream = NetworkStream::new(token, stream, read_func, write_func);

  return stream;
}

pub fn poll(handler: &mut EventHandler, connections: &mut Vec<NetworkStream>) -> Vec<NetworkData> {
  handler.poll.poll(&mut handler.events, None).unwrap();

  let mut data = Vec::new();

  let mut should_read = Vec::new();
  let mut should_write = Vec::new();

  for event in handler.events.iter() {
    match event.token() {
      token => {
        for i in 0..connections.len() {
          if connections[i].token.eq(&token.0) {
            if event.is_writable() {
              should_write.push(i);
            }
            if event.is_readable() {
              should_read.push(i);
            }
          }
        }
      }
    }
  }

  for i in 0..connections.len() {
    if should_read.contains(&i) {
      let mut bytes_read = 0;
      let mut recieved_data = vec![0; 4096];

      if !connections[i]
        .stream
        .is_type(ConnectionType::tcp_listener())
      {
        match connections[i].stream.read(&mut recieved_data[bytes_read..]) {
          Ok(0) => {
            // close connection
          }
          Ok(n) => {
            bytes_read += n;
            if bytes_read == recieved_data.len() {
              recieved_data.resize(recieved_data.len() + 1024, 0);
            }
          }
          Err(ref e) if ErrorKind::WouldBlock == e.kind() => {}
          Err(ref e) if ErrorKind::Interrupted == e.kind() => {}
          Err(e) => {
            panic!("{}", e);
          }
        }
      }

      let mut new_connections = connections[i].is_readable(handler, &recieved_data[..bytes_read]);
      connections.append(&mut new_connections);
      connections[i].reregister(handler);
    }
    if should_write.contains(&i) {
      let bytes_read = 0;
      let recieved_data = vec![0; 4096];

      connections[i].is_writeable(handler, &recieved_data[..bytes_read]);
      connections[i].reregister(handler);
    }
  }

  data
}

fn main() {
  let mut connections = Vec::new();
  let mut network = EventHandler::new();

  let stream = register_connection(
    &mut network,
    ConnectionType::tcp_listener(),
    "0.0.0.0:6767",
    Some(Box::new(accept_connections)),
    None,
    Interest::READABLE | Interest::WRITABLE,
  );
  let udp = register_connection(
    &mut network,
    ConnectionType::udp_socket(),
    "0.0.0.0:6768",
    Some(Box::new(udp_read)),
    None,
    Interest::READABLE | Interest::WRITABLE,
  );

  connections.push(stream);
  connections.push(udp);

  loop {
    let data = poll(&mut network, &mut connections);
    data.iter().for_each(|d| d.debug());
  }
}
