use std::net::{TcpStream, Shutdown};
use std::io::{Read, Write, ErrorKind};
use std::time::Duration;
use std::net::SocketAddr;

#[macro_use]
pub extern crate serde_derive;
pub extern crate bincode;

pub use bincode::{deserialize, serialize};

const BUFFER_SIZE: usize = 2048;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum NetworkError {
  ClientAlreadyConnected,
  Unknown,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum NetworkMessage {
  Null,
  Success,
  Shutdown,
  Err(NetworkError),
  NewSession(String),
  ClientConnected,
  EndSession,
  Data(String),
  RawData(Vec<u8>)
}

impl NetworkMessage {
  pub fn serialise(&self) -> Vec<u8> {
    bincode::serialize(&self).unwrap()
  }
  
  pub fn deserialise(serialised: &[u8]) -> NetworkMessage {
    match bincode::deserialize(&serialised) {
      Ok(msg) => {
        msg
      },
      Err(e) => {
        panic!("{:?}", e);
      }
    }
  }
}

pub struct StreamHandler {
  stream: TcpStream,
}

impl StreamHandler {
  pub fn new(ip: &str) -> StreamHandler {
    let mut stream = TcpStream::connect(ip).expect("Couldn't connect to the server...");
    //stream.set_keepalive(Some(10));
    StreamHandler {
      stream,
    }
  }
  
  pub fn from_stream(stream: TcpStream) -> StreamHandler {
    StreamHandler {
      stream,
    }
  }
  
  pub fn internal(&self) -> &TcpStream {
    &self.stream
  }
  
  pub fn socket(&self) -> SocketAddr {
    self.stream.local_addr().unwrap()
  }
  
  pub fn peer(&self) -> SocketAddr {
    self.stream.peer_addr().unwrap()
  }
  
  pub fn non_blocking(mut self) -> StreamHandler {
    self.stream.set_nonblocking(true).expect("set_nonblocking call failed");
    self
  }
  
  pub fn read_timeout(mut self, millis: u64) -> StreamHandler {
    self.stream.set_read_timeout(Some(Duration::from_millis(millis)));
    self
  }
  
  pub fn write_timeout(mut self, millis: u64) -> StreamHandler {
    self.stream.set_write_timeout(Some(Duration::from_millis(millis)));
    self
  }
  
  pub fn read_buffer(&mut self, buffer: &mut [u8; BUFFER_SIZE]) -> bool {
    let mut read_successful = false;
    
   // while !read_successful {
    match self.stream.read(buffer) {
      Ok(size) => {
        read_successful = true;
      },
      Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
        
      },
      Err(e) => { println!("Error: {}", e); }
    }
   // }
    self.stream.flush().unwrap();
    
    read_successful
  }
  
  pub fn write_buffer(&mut self, buffer: &[u8]) -> bool {
    let mut write_successful = false;
    
    match self.stream.write(&buffer[..]) {
      Ok(size) => { 
        write_successful = true;
      },
      Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
        
      },
      Err(e) => { println!("Error: {}", e); }
    }
    
    self.stream.flush().unwrap();
    
    write_successful
  }
  
  pub fn shutdown(&mut self) {
    self.stream.shutdown(Shutdown::Both).unwrap();
    self.stream.flush().unwrap();
  }
}

pub struct Network {
  session_id: String,
  buffer: [u8; BUFFER_SIZE],
  last_message: NetworkMessage,
  stream: StreamHandler,
}

impl Network {
  pub fn new(server_ip: &str, session_id: &str) -> Network {
    let mut stream = StreamHandler::new(server_ip).non_blocking();//.read_timeout(50).write_timeout(50);
    println!("Successfully connected to server");
    
    Network {
      session_id: format!("{}", session_id),
      buffer: [0; BUFFER_SIZE],
      last_message: NetworkMessage::Null,
      stream,
    }
  }
  
  pub fn from_stream(stream: TcpStream) -> Network {
    Network {
      session_id: "".to_string(),
      buffer: [0; BUFFER_SIZE],
      last_message: NetworkMessage::Null,
      stream: StreamHandler::from_stream(stream),
    }
  }
  
  pub fn message(&self) -> &NetworkMessage {
    &self.last_message
  }
  
  pub fn socket(&self) -> SocketAddr {
    self.stream.socket()
  }
  
  pub fn peer(&self) -> SocketAddr {
    self.stream.peer()
  }
  
  pub fn peer_connected(&self) -> bool {
    match self.stream.internal().peer_addr() {
      Ok(_) => {
        true
      },
      Err(_) => {
        false
      }
    }
  }
  
  pub fn buffer(&self) -> String {
    let chars_to_trim: &[char] = &['\u{0}'];
    
    format!("{}", String::from_utf8_lossy(&self.buffer).trim_matches(chars_to_trim))
  }
  
  pub fn connect_session(&mut self) -> bool {
    let mut response = bincode::serialize(&NetworkMessage::NewSession(self.session_id.to_string())).unwrap();
    
    self.stream.write_buffer(&response[..]);
    
    let mut session_exists = false;
    
    while !session_exists {
      self.read();
      match self.message() {
        NetworkMessage::Success => {
          println!("Session connected");
          break;
        },
        _ => {},
      }
    }
    
    session_exists
  }
  
  pub fn write_network_message(&mut self, msg: NetworkMessage) -> bool {
    let mut response = bincode::serialize(&msg).unwrap();
    
    self.stream.write_buffer(&response[..])
  }
  
  pub fn send_data(&mut self, data: String) -> bool {
    let mut response = bincode::serialize(&NetworkMessage::Data(data)).unwrap();
    
    !self.stream.write_buffer(&response[..])
  }
  
  pub fn send_vec(&mut self, data: Vec<u8>) -> bool {
    let mut response = bincode::serialize(&NetworkMessage::RawData(data)).unwrap();
    
    !self.stream.write_buffer(&response[..])
  }
  
  pub fn read(&mut self) -> bool {
    self.buffer = [0; BUFFER_SIZE];
    
    let did_read = self.stream.read_buffer(&mut self.buffer);
    
    self.last_message = NetworkMessage::deserialise(&self.buffer);
    
    did_read
  }
  
  pub fn close_connection(&mut self) {
    self.stream.shutdown();
  }
}
