//! IpcServer
//! 
//! Provides a server on a dgram internet socket
//!
 
// #![allow(dead_code)]

// #[cfg(test)] // Tells Rust to compile this module only during tests
#[path = "ipc/tests.rs"] // Specifies the file location of the tests
mod tests;

use std::{
    io::{self, ErrorKind},
    net::{SocketAddr, UdpSocket},
    time::Duration,
};

/// Implements an IPC server that uses a DGRAM socket
pub struct IpcServer {
    addr: SocketAddr,
    socket: Option<UdpSocket>,
}

impl IpcServer {

    /// Creates an IpcServer instance.
    /// 
    /// # Arguments
    /// * `sock_addr` - The server address as a SocketAddrV4.
    pub fn new(sock_addr: SocketAddr) -> Self {
        IpcServer {
            addr: sock_addr,
            socket: None,
        }
    }

    /// Binds to the address used to construct the object.
    /// 
    /// This must be called once before any calls to get() or send().
    /// 
    /// # Arguments
    /// * none
    /// 
    pub fn bind(&mut self) -> io::Result<()> {
        let socket = UdpSocket::bind(self.addr)?;
        socket.set_nonblocking(true).unwrap();
        self.socket = Some(socket);
        Ok(())
    }

    /// Receives data from the socket.
    /// 
    /// Performs non-blocking reads, so this should be called
    /// in a polling loop.
    /// 
    /// # Arguments
    /// * `max_size` - the maximum size of the expected message
    /// 
    pub fn get(&self, max_size: usize) -> io::Result<Option<(SocketAddr, Vec<u8>)>> {
        let socket = self.get_socket();
        let mut buffer: Vec<u8> = vec![0; max_size];
        match socket.recv_from(buffer.as_mut_slice()) {
            Ok((0, _)) => return Ok(None),
            Ok((len, source_addr)) => {
                println!("rec from {}", source_addr);
                buffer.truncate(len);
                return Ok(Some((source_addr, buffer)));
            },
            Err(e) => {
                if !matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) {
                    return  Err(e);
                }
            },
        }
        Ok(None)
    }

    /// Sends data to another endpoint.
    ///
    /// # Arguments
    /// * `send_data` - the data to be sent.
    /// * `remote_addr` - the destination address.
    /// 
    pub fn send(&self, send_data: &[u8], remote_addr: &SocketAddr) -> io::Result<usize> {
        match self.get_socket().send_to(send_data, remote_addr) {
            Ok(size) => Ok(size),
            Err(e) => Err(e),
        }
    }

    // ---- private -----
    fn get_socket(&self) -> &UdpSocket {
        self.socket.as_ref().unwrap()
    }
 }

 /// Implements an IPC client that uses a DGRAM socket
 pub struct IpcClient {
    addr: SocketAddr,
    socket: Option<UdpSocket>,
 }

 impl IpcClient {

    /// Creates an IpcClient instance.
    /// 
    /// # Arguments
    /// * `sock_address` - the address to be used for the client
    /// 
    pub fn new(sock_addr: SocketAddr) -> Self {
        IpcClient {
            addr: sock_addr,
            socket: None,
        }
    }

    /// Binds to the address used to construct the object.
    /// 
    /// This must be called once before any calls to send() or send_wait_response().
    /// 
    /// # Arguments
    /// * none
    /// 
    pub fn bind(&mut self) -> io::Result<()> {
        let socket = UdpSocket::bind(self.addr)?;
        self.socket = Some(socket);
        self.set_read_timeout(Some(Duration::from_millis(500)))?;
        Ok(())
    }

    /// Sends data to another endpoint.
    ///
    /// # Arguments
    /// * `send_data` - the data to be sent.
    /// * `remote_addr` - the destination address.
    /// 
    pub fn send(&self, data: &[u8], remote_addr: &SocketAddr) -> io::Result<usize> {
        let byte_count = self.get_socket().send_to(data, remote_addr)?;
        Ok(byte_count)
    }

    /// Sends data to another endpoint and waits for a response.
    /// 
    /// The response data is returned.
    ///
    /// # Arguments
    /// * `send_data` - the data to be sent.
    /// * `remote_addr` - the destination address.
    /// * `max_response_size` - maximum size of the response payload
    /// 
    pub fn send_wait_response(&self, send_data: &[u8], remote_addr: &SocketAddr, max_response_size: usize) -> io::Result<Option<(SocketAddr, Vec<u8>)>> {
        let byte_count = self.send(send_data, remote_addr)?;
        println!("Sent {} bytes to {}", byte_count, remote_addr);
        let mut buffer: Vec<u8> = vec![0; max_response_size];
        // this should block
        match self.get_socket().recv_from(buffer.as_mut_slice()) {
            Ok((0, _)) => Ok(None),
            Ok((len, src_addr)) => {
                buffer.truncate(len);
                Ok(Some((src_addr, buffer)))
            },
            Err(e) => {
                Err(e)
            },
        }
    }

    pub fn set_read_timeout(&self, duration: Option<Duration>) -> io::Result<()> {
        self.get_socket().set_read_timeout(duration)?;
        Ok(())
    }

    pub fn read_timeout(&self) -> io::Result<Option<Duration>> {
        let duration = self.get_socket().read_timeout()?;
        Ok(duration)
    }

    // ---- private -----
    fn get_socket(&self) -> &UdpSocket {
        self.socket.as_ref().unwrap()
    }
}
