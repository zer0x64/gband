use gband::SerialTransport;

use std::io;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::time::Duration;

pub struct SocketSerialTransport {
    address: SocketAddr,

    socket_type: SocketType,
    socket: Option<TcpStream>,
}

enum SocketType {
    Client,
    Server(Option<TcpListener>),
}

impl SocketSerialTransport {
    pub fn new(address: SocketAddr, server: bool) -> Self {
        let socket_type = if server {
            SocketType::Server(None)
        } else {
            SocketType::Client
        };

        Self {
            address,
            socket_type,
            socket: None,
        }
    }
}

impl SerialTransport for SocketSerialTransport {
    fn connect(&mut self) -> bool {
        match self.socket {
            // Already connected
            Some(_) => true,
            None => {
                match &mut self.socket_type {
                    SocketType::Client => {
                        // Connect the client
                        match TcpStream::connect_timeout(&self.address, Duration::from_millis(100))
                        {
                            Ok(socket) => {
                                log::info!("Connected to {}", &self.address);

                                if let Err(e) = socket.set_nonblocking(true) {
                                    log::warn!("Could not set the listener to non-blocking! {e}");
                                };

                                self.socket = Some(socket);

                                true
                            }
                            Err(e) => {
                                log::error!("Failed to connect: {}", e);

                                false
                            }
                        }
                    }
                    SocketType::Server(listener) => {
                        if let None = listener {
                            // Bind the server
                            match TcpListener::bind(&self.address) {
                                Ok(l) => {
                                    log::info!("Started listener on {}", &self.address);

                                    if let Err(e) = l.set_nonblocking(true) {
                                        log::warn!(
                                            "Could not set the listener to non-blocking! {e}"
                                        );
                                    };

                                    *listener = Some(l);
                                }
                                Err(e) => {
                                    log::error!("Unable to create listener: {}", e);
                                }
                            };
                        };

                        if let Some(listener) = listener {
                            match listener.accept() {
                                Ok((socket, addr)) => {
                                    log::info!("Accepted connection from {addr}");

                                    if let Err(e) = socket.set_nonblocking(true) {
                                        log::warn!("Could not set the socket to non-blocking! {e}");
                                    };

                                    self.socket = Some(socket);
                                    true
                                }
                                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                                    // No client connected yet
                                    false
                                }
                                Err(e) => {
                                    log::error!("Socket accept failed: {}", e);
                                    false
                                }
                            }
                        } else {
                            false
                        }
                    }
                }
            }
        }
    }

    fn is_connected(&self) -> bool {
        match &self.socket {
            Some(socket) => {
                let mut dummy = [0u8];
                let connected = match socket.peek(&mut dummy) {
                    Ok(_) => true,
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => true,
                    Err(e) => {
                        log::error!("is_connected peek error: {}", e);
                        false
                    }
                };

                connected
            }
            None => false,
        }
    }

    fn reset(&mut self) {
        self.socket = None
    }

    fn send(&mut self, data: u8) {
        if let Some(socket) = &mut self.socket {
            let send_buf = [data];

            if let Err(e) = socket.write(&send_buf) {
                log::warn!("Couldn't write to the socket: {e}");
                self.reset();
            };
        } else {
            log::warn!("Tried to write to a closed socket!")
        }
    }

    fn recv(&mut self) -> Option<u8> {
        if let Some(socket) = &mut self.socket {
            let mut recv_buf = [0u8];

            match socket.read(&mut recv_buf) {
                Ok(_) => Some(recv_buf[0]),
                Err(e) => match e.kind() {
                    io::ErrorKind::WouldBlock => None,
                    _ => {
                        log::warn!("Failed to receive from the socket! {e}");
                        self.reset();

                        None
                    }
                },
            }
        } else {
            log::warn!("Tried to receive from a closed socket!");

            None
        }
    }
}
