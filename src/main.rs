use mio::net::{TcpListener, TcpStream, UdpSocket};
use mio::{Events, Interest, Poll, Token};

use std::io::{Error, ErrorKind, Read};

fn print_data(data: Vec<u8>, poll: &mut Poll) {
  println!("{:?}", data);
}

fn accept_connections(data: Vec<u8>, poll: &mut Poll) {
  println!("Accepting connection: {:?}", data);
}

pub enum ConnectionType {
  TcpListener(Option<TcpListener>),
  TcpStream(Option<TcpStream>),
  UdpSocket(Option<UdpSocket>),
}

pub struct Handler {
  pub poll: Poll,
  pub events: Events,
  pub last_token: usize,
}

pub struct NetworkStream {
  pub token: usize,
  pub stream: ConnectionType,
  pub is_readable: Box<dyn Fn(Vec<u8>, &mut Poll)>,
  pub is_writeable: Box<dyn Fn(Vec<u8>, &mut Poll)>,
}

pub struct NetworkData {
  token: usize,
  data: Vec<u8>,
}

pub struct MaatNetwork {}

impl ConnectionType {
  pub fn tcp_listener() -> ConnectionType {
    ConnectionType::TcpListener(None)
  }

  pub fn tcp_stream() -> ConnectionType {
    ConnectionType::TcpStream(None)
  }

  pub fn udp_socket() -> ConnectionType {
    ConnectionType::UdpSocket(None)
  }

  pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
    match self {
      ConnectionType::TcpStream(stream) => {
        let mut result = Ok(0);
        if let Some(stream) = stream {
          result = stream.read(buf);
        }
        result
      }
      ConnectionType::TcpListener(_stream) => {
        return Ok(0);
      }
      ConnectionType::UdpSocket(stream) => {
        let mut result = Ok(0);
        if let Some(stream) = stream {
          result = stream.recv(buf);
        }
        result
      }
    }
  }
}

impl NetworkStream {
  pub fn new(token: usize, stream: ConnectionType) -> NetworkStream {
    NetworkStream {
      token,
      stream,
      is_readable: Box::new(print_data),
      is_writeable: Box::new(print_data),
    }
  }
}

impl NetworkData {
  pub fn new(token: usize, data: &[u8]) -> NetworkData {
    NetworkData {
      token,
      data: data.to_vec(),
    }
  }
}

impl Handler {
  pub fn new() -> Handler {
    let poll = Poll::new().unwrap();
    Handler {
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
    ConnectionType::TcpStream(stream) => ConnectionType::TcpStream(Some(
      stream.unwrap_or(TcpStream::connect(addr.into().parse().unwrap()).unwrap()),
    )),
    ConnectionType::TcpListener(listener) => ConnectionType::TcpListener(Some(
      listener.unwrap_or(TcpListener::bind(addr.into().parse().unwrap()).unwrap()),
    )),
    ConnectionType::UdpSocket(socket) => ConnectionType::UdpSocket(Some(
      socket.unwrap_or(UdpSocket::bind(addr.into().parse().unwrap()).unwrap()),
    )),
  }
}

pub fn register_connection<S: Into<String>>(
  mut handler: Handler,
  stream: ConnectionType,
  addr: S,
  interest: Interest,
) -> (Handler, NetworkStream) {
  let token = handler.next_token();

  let stream = {
    match stream {
      ConnectionType::TcpListener(_) => {
        let mut listener = TcpListener::bind(addr.into().parse().unwrap()).unwrap();
        if let Err(e) = handler
          .poll
          .registry()
          .register(&mut listener, Token(token), interest)
        {
          panic!("{}", e);
        }

        ConnectionType::TcpListener(Some(listener))
      }
      ConnectionType::TcpStream(_) => {
        let mut stream = TcpStream::connect(addr.into().parse().unwrap()).unwrap();
        if let Err(e) = handler
          .poll
          .registry()
          .register(&mut stream, Token(token), interest)
        {
          panic!("{}", e);
        }

        ConnectionType::TcpStream(Some(stream))
      }
      ConnectionType::UdpSocket(_) => {
        let mut udp = UdpSocket::bind(addr.into().parse().unwrap()).unwrap();
        if let Err(e) = handler
          .poll
          .registry()
          .register(&mut udp, Token(token), interest)
        {
          panic!("{}", e);
        }

        ConnectionType::UdpSocket(Some(udp))
      }
    }
  };

  let stream = NetworkStream::new(token, stream);

  return (handler, stream);
}

pub fn poll(handler: &mut Handler, connections: &mut Vec<NetworkStream>) -> Vec<NetworkData> {
  let events = &mut handler.events;
  handler.poll.poll(events, None).unwrap();

  let mut recieved_data = vec![0; 4096];
  let mut bytes_read = 0;

  let mut network_data = Vec::new();
  for event in events.iter() {
    match event.token() {
      token => {
        connections.iter_mut().for_each(|con| {
          if con.token == token.0 {
            if event.is_writable() {}
            if event.is_readable() {
              match con.stream.read(&mut recieved_data[bytes_read..]) {
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
          }
        });
      }
    }
  }

  return network_data;
}

fn main() {
  let mut connections = Vec::new();

  let (mut network, mut stream) = register_connection(
    Handler::new(),
    ConnectionType::tcp_listener(),
    "0.0.0.0:6767",
    Interest::READABLE | Interest::WRITABLE,
  );

  stream.is_readable = Box::new(accept_connections);
  connections.push(stream);

  loop {
    let data = poll(&mut network, &mut connections);
  }
}
