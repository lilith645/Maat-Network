use std::io::prelude::*;
use std::thread;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write, ErrorKind};
use std::io;
use std::net::SocketAddr;
use std::time::Duration;

use parking_lot::Mutex;
use std::sync::Arc;

use maat_network::{bincode, Network, NetworkMessage, NetworkError, StreamHandler};

#[derive(Debug)]
pub struct Session {
  id: String,
  clients: Vec<SocketAddr>,
  messages: Vec<(SocketAddr, NetworkMessage)>, // From, to all other clients
}

impl Session {
  pub fn new(id: &str, client: SocketAddr) -> Session {
    Session {
      id: id.to_string(),
      clients: vec!(client),
      messages: Vec::new(),
    }
  }
  
  pub fn add_client(&mut self, client: SocketAddr) {
    self.clients.push(client);
  }
  
  pub fn remove_client(&mut self, client: SocketAddr) -> bool {
    for i in 0..self.clients.len() {
      if self.clients[i] == client {
        self.clients.remove(i);
        break;
      }
    }
    
    self.clients.len() == 0
  }
  
  pub fn end_session(&mut self) {
    for i in 0..self.clients.len() {
      self.messages.push((self.clients[i], NetworkMessage::Shutdown));
    }
  }
  
  pub fn notify_host(&mut self) {
    self.messages.push((self.clients[0], NetworkMessage::ClientConnected));
  }
  
  pub fn send_message(&mut self, snd_client: SocketAddr, message: &str) {
    for recv_client in &self.clients {
      if *recv_client != snd_client {
        self.messages.push((*recv_client, NetworkMessage::Data(message.to_string())));
      }
    }
  }
  
  pub fn send_vec_message(&mut self, snd_client: SocketAddr, message: &[u8]) {
    for recv_client in &self.clients {
      if *recv_client != snd_client {
        self.messages.push((*recv_client, NetworkMessage::RawData(message.to_vec())));
      }
    }
  }
  
  pub fn get_message(&mut self, client: &SocketAddr) -> Option<NetworkMessage> {
    let mut msg = None;
    
    let mut offset = 0;
    
    for i in 0..self.messages.len() {
      if i > offset {
        break;
      }
      
      let mut found = false;
      
      match &self.messages[i-offset] {
        (recv_client, message) => {
          if recv_client == client {
            msg = Some(message.clone());
            found = true;
          }
        },
        _ => {},
      }
      
      if found {
        self.messages.remove(i-offset);
        offset += 1;
        break;
      }
    }
    
    msg
  }
  
  pub fn has_client(&self, client: &SocketAddr) -> bool {
    let mut found_client = false;
    for c in &self.clients {
      if c == client {
        found_client = true;
        break;
      }
    }
    found_client
  }
  
  pub fn is_named(&self, name: &str) -> bool {
    self.id == name
  }
  
  pub fn name(&self) -> String {
    self.id.to_string()
  }
}

fn main() {
  let listener = TcpListener::bind("0.0.0.0:8008").unwrap();
  //let listener = TcpListener::bind("127.0.0.1:8008").unwrap();
  listener.set_nonblocking(true).expect("Cannot set non-blocking");
  
  let mut sessions = Arc::new(Mutex::new(Vec::new()));
  
  for stream in listener.incoming() {
    match stream {
      Ok(mut s) => {
        s.set_nonblocking(true).expect("set_nonblocking call failed");
        
        let thread_sessions = Arc::clone(&sessions);
        let mut stream_handler = Network::from_stream(s);
        
        println!("Client {} connected", stream_handler.peer());
        thread::spawn(move || {
          let peer = stream_handler.peer();
          let mut t_sessions = thread_sessions;
          loop {
            if handle_connection(&mut stream_handler, &mut t_sessions, &peer) {
              break;
            }
          }
          
          println!("Client {:?} disconnected", peer);
          let mut mutex_session = t_sessions.lock();
        
          let mut session_exists = false;
          let mut idx = 0;
          
          for i in 0..mutex_session.len() {
            if mutex_session[i].has_client(&peer) {
              session_exists = true;
              idx = i;
              break;
            }
          }
          
          if session_exists {
            println!("Ending session {}", &mutex_session[idx].name());
            mutex_session[idx].end_session();
            if mutex_session[idx].remove_client(peer) {
              println!("Session ended: {}", mutex_session[idx].name());
              mutex_session.remove(idx);
            }
          }
        });
      },
      Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
        // wait until network socket is ready, typically implemented
        // via platform-specific APIs such as epoll or IOCP
        //wait_for_fd();
        
        continue;
      },
      Err(e) => {
        println!("Stream connection error: {}", e);
      }
    }
  }
  
  // close the socket server
  drop(listener);
}

fn handle_connection(mut client: &mut Network, session: &mut Arc<Mutex<Vec<Session>>>, socket: &SocketAddr) -> bool {
  let mut connection_closed = false;
  
  if !client.peer_connected() {
    return true;
  }
  
  if client.read() {
    let data = (*client.message()).clone();
    match data {
      NetworkMessage::NewSession(name) => {
        let mut mutex_session = session.lock();
        
        let mut session_exists = false;
        let mut idx = 0;
        
        for i in 0..mutex_session.len() {
          if mutex_session[i].is_named(&name) {
            session_exists = true;
            idx = i;
            break;
          }
        }
        
        if session_exists {
          mutex_session[idx].add_client(*socket);
          println!("Session {} joined", &name);
          mutex_session[idx].notify_host();
        } else {
          mutex_session.push(Session::new(&name, *socket));
          println!("Session {} created", &name);
        }
        
        client.write_network_message(NetworkMessage::Success);
      },
      NetworkMessage::EndSession => {
        let mut mutex_session = session.lock();
        
        let mut session_exists = false;
        let mut idx = 0;
        
        for i in 0..mutex_session.len() {
          if mutex_session[i].has_client(socket) {
            session_exists = true;
            idx = i;
            break;
          }
        }
        
        if session_exists {
          mutex_session[idx].end_session();
          println!("Ending session {}", &mutex_session[idx].name());
        }
        
        client.write_network_message(NetworkMessage::Success);
      },
      NetworkMessage::Shutdown => {
        connection_closed = true;
        
        client.write_network_message(NetworkMessage::Shutdown);
        println!("Connection termination from {:?}", socket);
        
        let mut mutex_session = session.lock();
        
        for i in 0..mutex_session.len() {
          if mutex_session[i].remove_client(*socket) {
            println!("Session ended: {}", mutex_session[i].name());
            mutex_session.remove(i);
          }
        }
      },
      NetworkMessage::Data(msg_data) => {
        let mut mutex_session = session.lock();
        
        for i in 0..mutex_session.len() {
          if mutex_session[i].has_client(socket) {
            println!("Data sent from {} to session {}", socket, mutex_session[i].name());
            mutex_session[i].send_message(*socket, &msg_data);
          }
        }
      },
      NetworkMessage::RawData(msg_data) => {
        let mut mutex_session = session.lock();
        
        for i in 0..mutex_session.len() {
          if mutex_session[i].has_client(&socket) {
            println!("Data sent from {} to session {}", socket, mutex_session[i].name());
            mutex_session[i].send_vec_message(*socket, &msg_data);
          }
        }
        client.write_network_message(NetworkMessage::Success);
      },
      _ => {},
    }
  }
  
  let mut mutex_session = session.lock();
  for i in 0..mutex_session.len() {
    if mutex_session[i].has_client(&socket) {
      if let Some(msg) = mutex_session[i].get_message(&socket) {
        client.write_network_message(msg);
      }
    }
  }
  
  connection_closed
}

