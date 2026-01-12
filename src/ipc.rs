//! IpcServer
//! 
//! Provides a server on a dgram internet socket
//!
 
#![allow(dead_code)]

#[cfg(test)] // Tells Rust to compile this module only during tests
#[path = "ipc/tests.rs"] // Specifies the file location
mod tests;

use std::{
    io::{self, ErrorKind},
    net::{SocketAddrV4, UdpSocket}
};

pub struct IpcServer {
    addr: SocketAddrV4,
    socket: Option<UdpSocket>,
}

impl IpcServer {

    /// Creates an IpcServer instance.
    /// 
    /// # Arguments
    /// * `sock_addr` - The server address as a SocketAddrV4.
    /// 
    /// # Returns
    /// The new object instance.
    /// 
    pub fn new(sock_addr: SocketAddrV4) -> Self {
        IpcServer {
            addr: sock_addr,
            socket: None,
        }
    }

    /// Binds to the address used to construct the object.
    /// 
    /// # Arguments
    /// * none
    /// 
    /// # Returns
    pub fn connect(&mut self) -> io::Result<()> {
        let socket = UdpSocket::bind(self.addr)?;
        socket.set_nonblocking(true).unwrap();
        self.socket = Some(socket);
        Ok(())
    }

    /// Receives data from the socket. Non-blocking.
    pub fn get_data(&self, max_size: usize) -> io::Result<Option<Vec<u8>>> {
        let mut buffer: Vec<u8> = vec![0; max_size];
        let socket = self.get_socket().unwrap();
        match socket.recv_from(buffer.as_mut_slice()) {
            Ok((0, _)) => return Ok(None),
            Ok((len, _)) => {
                buffer.truncate(len);
                return Ok(Some(buffer));
            },
            Err(e) => {
                if !matches!(e.kind(), ErrorKind::WouldBlock) {
                    return  Err(e);
                }
            },
        }
        Ok(None)
    }

    // ---- private -----
    fn get_socket(&self) -> Option<&UdpSocket> {
        self.socket.as_ref()
    }


 }

 pub struct IpcClient {
    addr: SocketAddrV4,
    socket: Option<UdpSocket>,
 }

impl IpcClient {
    pub fn new(sock_addr: SocketAddrV4) -> Self {
        IpcClient {
            addr: sock_addr,
            socket: None,
        }
    }

    /// Binds to the address used to construct the object.
    /// 
    /// # Arguments
    /// * none
    /// 
    /// # Returns
    pub fn connect(&mut self) -> io::Result<()> {
        let socket = UdpSocket::bind(self.addr)?;
        // socket.set_nonblocking(true).unwrap();
        self.socket = Some(socket);

        Ok(())
    }

    pub fn send_data(&self, data: &[u8], remote_addr: &SocketAddrV4) -> io::Result<usize> {
        let byte_count = self.get_socket().send_to(data, remote_addr)?;
        Ok(byte_count)
    }

    pub fn send_data_with_response(&self, data: &[u8], remote_addr: &SocketAddrV4, max_response_size: usize) -> io::Result<Option<Vec<u8>>> {
        let byte_count = self.get_socket().send_to(data, remote_addr)?;
        println!("Sent {} bytes to {}", byte_count, remote_addr);
        let mut buffer: Vec<u8> = vec![0; max_response_size];
        match self.get_socket().recv_from(buffer.as_mut_slice()) {
            Ok((0, _)) => return Ok(None),
            Ok((len, _)) => {
                buffer.truncate(len);
                return Ok(Some(buffer));
            },
            Err(e) => {
                if !matches!(e.kind(), ErrorKind::WouldBlock) {
                    return  Err(e);
                }
            },
        }
        Ok(None)
    }

        // ---- private -----
    fn get_socket(&self) -> &UdpSocket {
        self.socket.as_ref().unwrap()
    }
// Bind to an OS-assigned port on any interface. Port 0 means the OS chooses an available port.
    // let socket = UdpSocket::bind("0.0.0.0:0")?;
    
    // The address of the remote server you want to communicate with.
    // Ensure a corresponding UDP server is running on this address and port.
 
    // // Create a buffer to receive the response.
    // let mut buf = [0; 1024];
    // // Receive data and the source address of the response using `recv_from`.
    // let (amt, src) = socket.recv_from(&mut buf)?;
    
    // println!("Received {} bytes from {}: {:?}", amt, src, str::from_utf8(&buf[..amt]).unwrap());

    // Ok(())
}
