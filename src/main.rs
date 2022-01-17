use mio::net::{TcpListener, TcpStream, UdpSocket};
use mio::{Interest, Token};

use clap::Parser;

use core::time::Duration;
use std::io::ErrorKind;
use std::net::SocketAddr;

use modules::{
  read_functions::{accept_connections, read_data_from_client, read_data_from_server, udp_read},
  write_functions::write_ones,
  ConnectionType, EventHandler, NetworkStream,
};

pub type ReadFunc = Box<dyn Fn(&mut ConnectionType) -> (Vec<(ConnectionType, String)>, Vec<u8>)>;
pub type WriteFunc = Box<dyn Fn(&mut ConnectionType, &mut Vec<u8>)>;

mod modules;

pub const TCP_SERVER_ADDRESS: &str = "0.0.0.0";
pub const TCP_SERVER_PORT: &str = "6767";
pub const UDP_SERVER_ADDRESS: &str = "0.0.0.0";
pub const UDP_SERVER_PORT: &str = "6768";

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
  #[clap(short, long)]
  client: bool,
}

use mio::event::Event;
use mio::Registry;
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::str::from_utf8;

const SERVER: Token = Token(0);

const DATA: &[u8] = b"Hello world!\n";

//fn main() -> io::Result<()> {
//  test_main();
//  return Ok(());
//
//  let args = Args::parse();
//
//  if args.client {
//  } else {
//  }
//
//  let mut network = EventHandler::new();
//  let mut connections = HashMap::new();
//
//  connections.insert(
//    SERVER,
//    NetworkStream::new(
//      "",
//      SERVER,
//      network.poll.registry(),
//      Interest::READABLE,
//      ConnectionType::add_existing_tcp_listener(
//        TcpListener::bind(
//          format!("{}:{}", TCP_SERVER_ADDRESS, TCP_SERVER_PORT)
//            .parse()
//            .unwrap(),
//        )
//        .unwrap(),
//      ),
//      Some(Box::new(accept_connections)),
//      None,
//    ),
//  );
//
//  let mut unique_token = Token(SERVER.0 + 1);
//
//  println!("You can connect to the server using `nc`:");
//  println!(" $ nc {} {}", TCP_SERVER_ADDRESS, TCP_SERVER_PORT);
//  println!("You'll see our welcome message and anything you type will be printed here.");
//
//  loop {
//    network
//      .poll
//      .poll(&mut network.events, Some(Duration::ZERO))?;
//
//    let mut new_connections = network
//      .events
//      .iter()
//      .map(|event| match event.token() {
//        SERVER => {
//          let mut new_connections = Vec::new();
//          loop {
//            match connections.get(&SERVER).unwrap().stream.accept() {
//              Ok((connection, address)) => new_connections.push((connection, address)),
//              Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
//                // ignore
//                break;
//              }
//              Err(e) => {
//                println!("Server read error: {}", e);
//                break;
//              }
//            };
//          }
//
//          new_connections
//        }
//        token => {
//          if let Some(c) = connections.get_mut(&token) {
//            let done = {
//              let stream = &mut c.stream;
//              match stream {
//                ConnectionType::TcpStream(stream) => {
//                  handle_connection_event(network.poll.registry(), stream, event).unwrap()
//                }
//                _ => false,
//              }
//            };
//
//            if done {
//              c.deregister(network.poll.registry());
//            }
//          }
//
//          Vec::new()
//        }
//      })
//      .flatten()
//      .collect::<Vec<(TcpStream, SocketAddr)>>();
//
//    new_connections
//      .drain(..)
//      .map(|(c, _addr)| {
//        let token = next(&mut unique_token);
//        (token, c)
//      })
//      .for_each(|(token, connection)| {
//        connections.insert(
//          token,
//          NetworkStream::new(
//            "",
//            token,
//            network.poll.registry(),
//            Interest::READABLE.add(Interest::WRITABLE),
//            ConnectionType::add_existing_tcp_stream(connection),
//            Some(Box::new(accept_connections)),
//            None,
//          ),
//        );
//      });
//  }
//}
//
//fn next(current: &mut Token) -> Token {
//  let next = current.0;
//  current.0 += 1;
//  Token(next)
//}
//
///// Returns `true` if the connection is done.
//fn handle_connection_event(
//  registry: &Registry,
//  connection: &mut TcpStream,
//  event: &Event,
//) -> io::Result<bool> {
//  if event.is_writable() {
//    // We can (maybe) write to the connection.
//    match connection.write(DATA) {
//      // We want to write the entire `DATA` buffer in a single go. If we
//      // write less we'll return a short write error (same as
//      // `io::Write::write_all` does).
//      Ok(n) if n < DATA.len() => return Err(io::ErrorKind::WriteZero.into()),
//      Ok(_) => {
//        // After we've written something we'll reregister the connection
//        // to only respond to readable events.
//        registry.reregister(connection, event.token(), Interest::READABLE)?
//      }
//      // Would block "errors" are the OS's way of saying that the
//      // connection is not actually ready to perform this I/O operation.
//      Err(ref err) if would_block(err) => {}
//      // Got interrupted (how rude!), we'll try again.
//      Err(ref err) if interrupted(err) => {
//        return handle_connection_event(registry, connection, event)
//      }
//      // Other errors we'll consider fatal.
//      Err(err) => return Err(err),
//    }
//  }
//
//  if event.is_readable() {
//    let mut connection_closed = false;
//    let mut received_data = vec![0; 4096];
//    let mut bytes_read = 0;
//    // We can (maybe) read from the connection.
//    loop {
//      match connection.read(&mut received_data[bytes_read..]) {
//        Ok(0) => {
//          // Reading 0 bytes means the other side has closed the
//          // connection or is done writing, then so are we.
//          connection_closed = true;
//          break;
//        }
//        Ok(n) => {
//          bytes_read += n;
//          if bytes_read == received_data.len() {
//            received_data.resize(received_data.len() + 1024, 0);
//          }
//        }
//        // Would block "errors" are the OS's way of saying that the
//        // connection is not actually ready to perform this I/O operation.
//        Err(ref err) if would_block(err) => break,
//        Err(ref err) if interrupted(err) => continue,
//        // Other errors we'll consider fatal.
//        Err(err) => return Err(err),
//      }
//    }
//
//    if bytes_read != 0 {
//      let received_data = &received_data[..bytes_read];
//      if let Ok(str_buf) = from_utf8(received_data) {
//        println!("Received data: {}", str_buf.trim_end());
//      } else {
//        println!("Received (none UTF-8) data: {:?}", received_data);
//      }
//    }
//
//    if connection_closed {
//      println!("Connection closed");
//      return Ok(true);
//    }
//  }
//
//  Ok(false)
//}
//
//fn would_block(err: &io::Error) -> bool {
//  err.kind() == io::ErrorKind::WouldBlock
//}
//
//fn interrupted(err: &io::Error) -> bool {
//  err.kind() == io::ErrorKind::Interrupted
//}

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
    println!(
      "Main: Token {}: \n    Recieved data: {:?}",
      self.token, self.data
    );
  }
}

//pub fn register_connection<S: Into<String>>(
//  handler: &mut EventHandler,
//  stream: ConnectionType,
//  addr: S,
//  read_func: Option<ReadFunc>,
//  write_func: Option<WriteFunc>,
//  interest: Interest,
//) -> NetworkStream {
//  let token = handler.next_token();
//
//  let stream = {
//    match stream {
//      ConnectionType::NewTcpListener => {
//        let listener = TcpListener::bind(addr.into().parse().unwrap()).unwrap();
//        ConnectionType::from(listener)
//      }
//      ConnectionType::NewTcpStream => {
//        let stream = TcpStream::connect(addr.into().parse().unwrap()).unwrap();
//        ConnectionType::from(stream)
//      }
//      ConnectionType::NewUdpSocket => {
//        let udp = UdpSocket::bind(addr.into().parse().unwrap()).unwrap();
//        ConnectionType::from(udp)
//      }
//      _ => stream,
//    }
//  };
//
//  let stream = NetworkStream::new(
//    Token(token),
//    handler.poll.registry(),
//    interest,
//    stream,
//    read_func,
//    write_func,
//  );
//
//  return stream;
//}

fn create_connection<S: Into<String>, T: Into<String>>(
  connection_type: ConnectionType,
  addr: S,
  port: T,
) -> ConnectionType {
  match connection_type {
    ConnectionType::NewTcpListener => ConnectionType::from(
      TcpListener::bind(format!("{}:{}", addr.into(), port.into()).parse().unwrap()).unwrap(),
    ),
    ConnectionType::NewTcpStream => ConnectionType::from(
      TcpStream::connect(format!("{}:{}", addr.into(), port.into()).parse().unwrap()).unwrap(),
    ),
    ConnectionType::NewUdpSocket => ConnectionType::from(
      UdpSocket::bind(format!("{}:{}", addr.into(), port.into()).parse().unwrap()).unwrap(),
    ),
    c => c,
  }
}

fn register_stream(
  mut connection: NetworkStream,
  registry: &Registry,
  token: Token,
  interest: Interest,
) -> NetworkStream {
  connection.register(registry, token, interest);
  connection
}

fn add_stream<S: Into<String>>(
  addr: S,
  mut connections: Vec<NetworkStream>,
  connection: ConnectionType,
  read_func: Option<ReadFunc>,
  write_func: Option<WriteFunc>,
) -> Vec<NetworkStream> {
  connections.push(NetworkStream::from_connection(
    connection,
    addr.into(),
    read_func,
    write_func,
  ));

  connections
}

fn main() {
  let mut network = EventHandler::new();

  let mut connections: Vec<NetworkStream> = Vec::new();

  connections.push(NetworkStream::from_connection(
    create_connection(
      ConnectionType::NewTcpListener,
      TCP_SERVER_ADDRESS,
      TCP_SERVER_PORT,
    ),
    format!("{}:{}", TCP_SERVER_ADDRESS, TCP_SERVER_PORT),
    Some(Box::new(accept_connections)),
    None,
  ));

  loop {
    if let Err(e) = network.poll.poll(&mut network.events, Some(Duration::ZERO)) {
      println!("Error polling events: {}", e);
    }

    let mut new_connections = network
      .events
      .iter()
      .map(|e| {
        connections
          .iter_mut()
          .filter(|c| !c.unregistered())
          .filter(|c| c.token().unwrap() == e.token())
          .enumerate()
          .filter_map(|(i, v)| if i == 0 { Some(v) } else { None })
          .map(|connection| {
            let mut new_connections = Vec::new();
            if e.is_readable() {
              let (new_con, data) = &mut NetworkStream::is_readable(connection);
              new_connections.append(new_con);
              if !data.is_empty() {
                println!("{:?}", data);
              }
            }
            if e.is_writable() {
              let mut data = Vec::new();
              NetworkStream::is_writeable(connection, &mut data);
            }

            new_connections
          })
          .flatten()
          .collect::<Vec<(ConnectionType, String)>>()
          .drain(..)
          .map(|(c, addr)| {
            NetworkStream::from_connection(c, addr, Some(Box::new(read_data_from_client)), None)
          })
          .collect::<Vec<NetworkStream>>()
      })
      .flatten()
      .collect::<Vec<NetworkStream>>();

    connections = connections
      .drain(..)
      .chain(new_connections.drain(..))
      .map(|mut x| {
        if x.unregistered() {
          let token = Token(network.next_token());
          x.register(
            network.poll.registry(),
            token,
            Interest::READABLE.add(Interest::WRITABLE),
          );
        }

        x
      })
      .collect::<Vec<NetworkStream>>();
  }
}

//pub fn poll(handler: &mut EventHandler, connections: &mut Vec<NetworkStream>) -> Vec<NetworkData> {
//  handler
//    .poll
//    .poll(&mut handler.events, Some(Duration::ZERO))
//    .unwrap();
//
//  let mut data = Vec::new();
//
//  let mut should_read = Vec::new();
//  let mut should_write = Vec::new();
//
//  let mut new_connections = Vec::new();
//
//  for event in handler.events.iter() {
//    match event.token() {
//      token => {
//        for i in 0..connections.len() {
//          if connections[i].token.eq(&token) {
//            if event.is_writable() {
//              should_write.push(i);
//            }
//            if event.is_readable() {
//              should_read.push(i);
//            }
//          }
//        }
//      }
//    }
//  }
//
//  for i in 0..connections.len() {
//    if should_read.contains(&i) {
//      let mut bytes_read = 0;
//      let mut recieved_data = vec![0; 4096];
//
//      loop {
//        match connections[i].stream.read(&mut recieved_data[bytes_read..]) {
//          Ok(0) => {
//            // close connection
//            panic!("Should close connection");
//            break;
//          }
//          Ok(n) => {
//            println!("bytes read");
//            bytes_read += n;
//            if bytes_read == recieved_data.len() {
//              recieved_data.resize(recieved_data.len() + 1024, 0);
//            }
//          }
//          Err(ref e) if ErrorKind::WouldBlock == e.kind() => {
//            break;
//          }
//          Err(ref e) if ErrorKind::Interrupted == e.kind() => {
//            continue;
//          }
//          Err(e) => {
//            panic!("An error occured: {}", e);
//          }
//        }
//      }
//
//      new_connections
//        .append(&mut connections[i].is_readable(handler, &recieved_data[..bytes_read]));
//
//      //connections[i].reregister(handler, Interest::WRITABLE.add(Interest::READABLE));
//      data.push(NetworkData::new(
//        connections[i].token.0,
//        &recieved_data[..bytes_read],
//      ));
//    }
//
//    if should_write.contains(&i) {
//      let mut data = Vec::with_capacity(1024);
//      connections[i].is_writeable(handler, &mut data);
//      //connections[i].reregister(handler, Interest::READABLE);
//    }
//  }
//
//  connections.append(&mut new_connections);
//
//  data
//}

//
//fn main() {
//  let mut connections = Vec::new();
//  let mut network = EventHandler::new();
//
//  let args = Args::parse();
//
//  if args.client {
//    println!("Starting client:");
//    println!("    server: {}", TCP_SERVER_ADDRESS);
//
//    let stream = register_connection(
//      &mut network,
//      ConnectionType::tcp_stream(),
//      TCP_SERVER_ADDRESS,
//      Some(Box::new(read_data_from_server)),
//      Some(Box::new(write_ones)),
//      Interest::READABLE.add(Interest::WRITABLE),
//    );
//
//    connections.push(stream);
//  } else {
//    println!("Starting server on:");
//    println!("    tcp: {}", TCP_SERVER_ADDRESS);
//    //println!("    udp: {}", UDP_SERVER_ADDRESS);
//
//    let stream = register_connection(
//      &mut network,
//      ConnectionType::tcp_listener(),
//      TCP_SERVER_ADDRESS,
//      Some(Box::new(accept_connections)),
//      None,
//      //Some(Box::new(write_ones)),
//      Interest::READABLE,
//    );
//    //let udp = register_connection(
//    //  &mut network,
//    //  ConnectionType::udp_socket(),
//    //  UDP_SERVER_ADDRESS,
//    //  Some(Box::new(udp_read)),
//    //  None,
//    //  Interest::READABLE,
//    //);
//
//    connections.push(stream);
//    //connections.push(udp);
//  }
//
//  loop {
//    let data = poll(&mut network, &mut connections);
//    data.iter().for_each(|d| {
//      if d.data.len() > 0 {
//        d.debug()
//      }
//    });
//
//    //connections.iter().for_each(|c| println!("{}", c.token()));
//  }
//}
