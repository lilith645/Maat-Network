use mio::net::{TcpListener, TcpStream, UdpSocket};
use mio::{Interest, Token};

use clap::Parser;

use core::time::Duration;

use modules::{
  read_functions::{accept_connections, print_data, recieve_data},
  ConnectionType, EventHandler, NetworkStream,
};

pub type ReadFunc = Box<dyn Fn(&mut ConnectionType, &[u8]) -> Vec<(ConnectionType, String)>>;

mod modules;

pub const TCP_SERVER_ADDRESS: &str = "0.0.0.0";
pub const TCP_SERVER_PORT: &str = "6767";
pub const UDP_SERVER_ADDRESS: &str = "0.0.0.0";
pub const UDP_SERVER_PORT: &str = "6768";

pub struct NewConnection {
  pub token: usize,
  pub connection: ConnectionType,
  pub addr: String,
  pub read_func: Option<ReadFunc>,
}

impl NewConnection {
  pub fn new(
    token: usize,
    connection: ConnectionType,
    addr: &str,
    read_func: Option<ReadFunc>,
  ) -> NewConnection {
    NewConnection {
      token,
      connection: create_connection(connection, addr),
      addr: addr.into(),
      read_func,
    }
  }
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
  #[clap(short, long)]
  client: bool,
}

pub struct NetworkData {
  token: usize,
  data: Vec<u8>,
}

pub struct MaatNetwork {
  event_handler: EventHandler,
  connections: Vec<NetworkStream>,
  new_connections: Vec<NewConnection>,
  pending_data: Vec<(usize, Vec<u8>)>,
}

impl MaatNetwork {
  pub fn new() -> MaatNetwork {
    MaatNetwork {
      event_handler: EventHandler::new(),
      connections: Vec::new(),
      new_connections: Vec::new(),
      pending_data: Vec::new(),
    }
  }

  pub fn host_tcp_server<S, A>(&mut self, addr: S, port: A, read_func: Option<ReadFunc>) -> usize
  where
    S: Into<String>,
    A: Into<String>,
  {
    let token = self.event_handler.next_token();
    self.new_connections.push(NewConnection::new(
      token,
      ConnectionType::NewTcpListener,
      &format!("{}:{}", addr.into(), port.into()),
      Some(read_func.unwrap_or(Box::new(accept_connections))),
    ));
    token
  }

  pub fn host_udp_server<S, A>(&mut self, addr: S, port: A, read_func: Option<ReadFunc>) -> usize
  where
    S: Into<String>,
    A: Into<String>,
  {
    let token = self.event_handler.next_token();
    self.new_connections.push(NewConnection::new(
      token,
      ConnectionType::NewUdpSocket,
      &format!("{}:{}", addr.into(), port.into()),
      read_func,
    ));
    token
  }

  pub fn connect_to_tcp<S, A>(&mut self, addr: S, port: A, read_func: Option<ReadFunc>) -> usize
  where
    S: Into<String>,
    A: Into<String>,
  {
    let token = self.event_handler.next_token();
    self.new_connections.push(NewConnection::new(
      token,
      ConnectionType::NewTcpStream,
      &format!("{}:{}", addr.into(), port.into()),
      read_func,
    ));

    token
  }

  pub fn add_exisiting_connection(&mut self, connection: NewConnection) {
    self.new_connections.push(connection);
  }

  pub fn add_existing_tcp_listener<S, A>(
    &mut self,
    listener: TcpListener,
    addr: S,
    port: A,
    read_func: Option<ReadFunc>,
  ) -> usize
  where
    S: Into<String>,
    A: Into<String>,
  {
    let token = self.event_handler.next_token();
    self.new_connections.push(NewConnection::new(
      token,
      ConnectionType::from(listener),
      &format!("{}:{}", addr.into(), port.into()),
      read_func,
    ));
    token
  }

  pub fn add_existing_udp_connection<S, A>(
    &mut self,
    udp: UdpSocket,
    addr: S,
    port: A,
    read_func: Option<ReadFunc>,
  ) -> usize
  where
    S: Into<String>,
    A: Into<String>,
  {
    let token = self.event_handler.next_token();
    self.new_connections.push(NewConnection::new(
      token,
      ConnectionType::from(udp),
      &format!("{}:{}", addr.into(), port.into()),
      read_func,
    ));
    token
  }

  pub fn add_existing_tcp_connection<S, A>(
    &mut self,
    tcp_connection: TcpStream,
    addr: S,
    port: A,
    read_func: Option<ReadFunc>,
  ) -> usize
  where
    S: Into<String>,
    A: Into<String>,
  {
    let token = self.event_handler.next_token();
    self.new_connections.push(NewConnection::new(
      token,
      ConnectionType::from(tcp_connection),
      &format!("{}:{}", addr.into(), port.into()),
      read_func,
    ));

    token
  }

  pub fn removed_connection(&mut self, token: usize) {
    self
      .connections
      .iter_mut()
      .filter(|x| {
        if let Some(t) = x.token() {
          t.0 == token
        } else {
          false
        }
      })
      .for_each(|c| c.deregister(self.event_handler.poll.registry()));
  }

  pub fn write_data(&mut self, token: usize, data: &[u8]) {
    let mut found_connection = false;
    self
      .connections
      .iter_mut()
      .filter(|c| {
        if let Some(t) = c.token() {
          t.0 == token
        } else {
          false
        }
      })
      .for_each(|c| {
        found_connection = true;
        c.data_to_write(data);
      });

    if !found_connection {
      self.pending_data.push((token, data.to_vec()));
    }
  }

  pub fn poll(
    &mut self,
  ) -> (
    Vec<(usize, Vec<u8>)>,
    Vec<NewConnection>,
    Vec<Option<usize>>,
  ) {
    if let Err(e) = self
      .event_handler
      .poll
      .poll(&mut self.event_handler.events, Some(Duration::ZERO))
    {
      println!("Error polling events: {}", e);
    }

    let mut events = 0;

    let mut recieved_data = Vec::new();

    let mut new_connections = self
      .event_handler
      .events
      .iter()
      .map(|i| {
        events += 1;
        i
      })
      .map(|e| {
        self
          .connections
          .iter_mut()
          .filter(|c| !c.unregistered())
          .filter(|c| c.token().unwrap() == e.token())
          .enumerate()
          .filter_map(|(i, v)| if i == 0 { Some(v) } else { None })
          .map(|connection| {
            let mut new_connections = Vec::new();

            let mut should_close = false;

            if e.is_readable() {
              let (data, close) = recieve_data(&mut connection.stream);
              let new_con = &mut NetworkStream::is_readable(connection, &data);
              new_connections.append(new_con);
              if !data.is_empty() {
                recieved_data.push((connection.token().unwrap().0, data));
              }

              should_close = close;
            }
            if e.is_writable() && connection.data_pending() {
              NetworkStream::is_writeable(connection);
            }

            if should_close {
              connection.deregister(self.event_handler.poll.registry());
            }

            new_connections
          })
          .flatten()
          .collect::<Vec<(ConnectionType, String)>>()
      })
      .flatten()
      .collect::<Vec<(ConnectionType, String)>>();

    let new_connections = new_connections
      .drain(..)
      .map(|(c, addr)| {
        NewConnection::new(
          self.event_handler.next_token(),
          c,
          &addr,
          Some(Box::new(print_data)),
        )
      })
      .collect();

    let removed_connections: Vec<Option<usize>> = self
      .connections
      .iter()
      .filter(|x| x.unregistered())
      .map(|x| {
        if let Some(token) = x.token {
          Some(token.0)
        } else {
          None
        }
      })
      .collect::<Vec<Option<usize>>>();

    self.connections = self
      .connections
      .drain(..)
      .filter(|x| !x.unregistered())
      .map(|mut x| {
        if x.did_write() {
          x.reregister(
            &mut self.event_handler,
            Interest::READABLE.add(Interest::WRITABLE),
          );
        }
        x
      })
      .collect::<Vec<NetworkStream>>();

    self.connections.append(
      &mut (self
        .new_connections
        .drain(..)
        .map(|connection| NetworkStream::from(connection))
        .map(|mut x| {
          if x.unregistered() {
            x.register(
              self.event_handler.poll.registry(),
              x.token().unwrap(),
              Interest::READABLE.add(Interest::WRITABLE),
            );
          }

          x
        })
        .map(|mut x| {
          self.pending_data = self
            .pending_data
            .drain(..)
            .filter_map(|(t, d)| {
              if t == x.token().unwrap().0 {
                x.data_to_write(&d);
                None
              } else {
                Some((t, d))
              }
            })
            .collect::<Vec<(usize, Vec<u8>)>>();

          x
        })
        .collect::<Vec<NetworkStream>>()),
    );

    (recieved_data, new_connections, removed_connections)
  }
}

impl NetworkData {
  pub fn new(token: usize, data: &[u8]) -> NetworkData {
    NetworkData {
      token,
      data: data.to_vec(),
    }
  }

  pub fn debug(&self) {
    println!(
      "Main: Token {}: \n    Recieved data: {:?}",
      self.token, self.data
    );
  }
}

fn create_connection<S>(
  connection_type: ConnectionType,
  addr: S,
  //port: T,
) -> ConnectionType
where
  S: Into<String>,
{
  match connection_type {
    ConnectionType::NewTcpListener => {
      ConnectionType::from(TcpListener::bind(format!("{}", addr.into()).parse().unwrap()).unwrap())
    }
    ConnectionType::NewTcpStream => {
      ConnectionType::from(TcpStream::connect(format!("{}", addr.into()).parse().unwrap()).unwrap())
    }
    ConnectionType::NewUdpSocket => {
      ConnectionType::from(UdpSocket::bind(format!("{}", addr.into()).parse().unwrap()).unwrap())
    }
    c => c,
  }
}

fn main() {
  let args = Args::parse();

  let mut network = MaatNetwork::new();

  let mut tokens: Vec<usize> = Vec::new();

  let mut client_token = None;

  if args.client {
    tokens.push(network.connect_to_tcp("127.0.0.1", TCP_SERVER_PORT, Some(Box::new(print_data))));
    client_token = Some(*tokens.last().unwrap());
    network.write_data(client_token.unwrap() as usize, &[9, 2, 3, 4, 6]);
  } else {
    tokens.push(network.host_tcp_server(
      TCP_SERVER_ADDRESS,
      TCP_SERVER_PORT,
      Some(Box::new(accept_connections)),
    ));
  }

  loop {
    let (mut recieved_data, mut new_connections, mut removed_connections) = network.poll();

    recieved_data.drain(..).for_each(|(t, d)| {
      println!("token: {} data {:?}", t, d);
    });

    if let Some(c_token) = client_token {
      network.write_data(client_token.unwrap() as usize, &[9, 2, 3, 4, 6]);
    }

    new_connections.drain(..).for_each(|c| {
      println!("Token: {}", c.token);
      tokens.push(c.token);
      network.add_exisiting_connection(c);
    });

    removed_connections.drain(..).for_each(|t| {
      println!("Removed token: {:?}", t);
      tokens = tokens.drain(..).filter(|token| t != Some(*token)).collect();
      if let Some(t) = t {
        network.removed_connection(t);
      }
    })
  }
}
