extern crate mio;

use mio::tcp::*;
use mio::util::Slab;
use mio::{TryRead};
use std::io::Write;
use std::net::SocketAddr;
use sunday::parse_sunday_line;
use sunday_dispatcher::SundayDispatcher;


const SERVER: mio::Token = mio::Token(1);


#[derive(Debug)]
struct Connection {
    socket: TcpStream,
    token: mio::Token,
    buffer: Vec<u8>,
    closed: bool,
    dispatcher: SundayDispatcher,
}


struct SundayServer {
    server: TcpListener,
    connections: Slab<Connection>
}


impl mio::Handler for SundayServer {
    type Timeout = ();
    type Message = ();

    fn ready(&mut self, event_loop: &mut mio::EventLoop<SundayServer>, token: mio::Token, events: mio::EventSet) {
        match token {
            SERVER => {
                assert!(events.is_readable());
                match self.server.accept() {
                    Ok(Some((socket, _))) => {
                        let token = self.connections
                            .insert_with(|token| Connection::new(socket, token))
                            .unwrap();
                        event_loop.register(
                            &self.connections[token].socket,
                            token,
                            mio::EventSet::readable(),
                            mio::PollOpt::edge() | mio::PollOpt::oneshot()).unwrap()
                    }
                    Ok(None) => {
                        println!("wasn't ready");
                    }
                    Err(_) => {
                        event_loop.shutdown();
                    }
                }
            }
            _ => {
                self.connections[token].ready(event_loop, events);
                if self.connections[token].closed {
                    let _ = self.connections.remove(token);
                    println!("Removing socket");
                }
            }
        }
    }
}


impl SundayServer {
    fn new(server: TcpListener) -> SundayServer {
        let slab = Slab::new_starting_at(mio::Token(2), 1024);
        SundayServer {
            server: server,
            connections: slab,
        }
    }
}


impl Connection {
    fn new(socket: TcpStream, token: mio::Token) -> Connection {
        Connection {
            socket: socket,
            token: token,
            buffer: vec![],
            closed: false,
            dispatcher: SundayDispatcher::new(),
        }
    }

    fn ready(&mut self, event_loop: &mut mio::EventLoop<SundayServer>, events: mio::EventSet) {
        if events.is_readable() {
            self.read(event_loop);
        }
    }

    fn read(&mut self, event_loop: &mut mio::EventLoop<SundayServer>) {
        match self.socket.try_read_buf(&mut self.buffer) {
            Ok(Some(0)) => {
                self.closed = true;
            }
            Ok(Some(n)) => {
                if self.buffer[self.buffer.len()-1] == '\n' as u8 {
                    let buf_cpy = &self.buffer.to_owned();
                    let sline = parse_sunday_line(buf_cpy);
                    println!("sline: {:?}", sline);
                    self.socket.write(&self.dispatcher.handle_command(sline));
                    self.buffer.clear();
                }
                self.reregister(event_loop);
            }
            Ok(None) => {
                self.reregister(event_loop);
            }
            Err(e) => {
                // We errored, uhhhhhhhh
                self.closed = true;
            }
        }
    }

    fn reregister(&self, event_loop: &mut mio::EventLoop<SundayServer>) {
        event_loop.reregister(&self.socket,
                              self.token,
                              mio::EventSet::readable(),
                              mio::PollOpt::oneshot()).unwrap();
    }
}


pub fn start(address: SocketAddr) {
    let server: TcpListener = TcpListener::bind(&address).unwrap();
    let mut event_loop = mio::EventLoop::new().unwrap();
    event_loop.register(
        &server,
        SERVER,
        mio::EventSet::readable(),
        mio::PollOpt::edge()).unwrap();
    let mut ss = SundayServer::new(server);
    event_loop.run(&mut ss).unwrap();
}
