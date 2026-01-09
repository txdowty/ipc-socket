//! IpcServer
//! 
//! Provides a server on a dgram internet socket
//!
 
#![allow(dead_code)]

use std::{
    io::{self, ErrorKind},
    net::{SocketAddrV4, UdpSocket}
};

struct IpcServer {
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

#[cfg(test)]
mod tests {
    use super::*;

    use serde::{Deserialize, Serialize};
    use std::{env};
    use std::io::{BufRead, BufReader};
    use std::net::SocketAddrV4;
    use std::path::{Path, PathBuf};
    use std::process::{Child, Command, Stdio, ChildStdout};
    use std::str::FromStr;
    use serde_json::{Value};

    #[derive(Deserialize, Serialize, PartialEq, Eq, Debug)]
    struct MyStruct {
        big: u64,
        small: u32,
    }


    fn spawn_python_script(rel_path_to_script: &Path) -> io::Result<Child> {
        // start the client app using the venv python
        let package_root_dir = env!("CARGO_MANIFEST_DIR");
        let python_path = PathBuf::from(package_root_dir)
            .join(".venv").join("bin").join("python");
        let script_path = PathBuf::from(package_root_dir)
            .join(rel_path_to_script);

        println!("spawning: {} {}", python_path.to_str().unwrap(),
            script_path.to_str().unwrap());

        let mut script_cmd = Command::new(python_path);
        script_cmd.arg(&script_path.to_str().unwrap()).stdout(Stdio::piped());
        script_cmd.spawn()
    }

    fn print_bufreader(reader: BufReader<ChildStdout>) {
       for line in reader.lines() {
            match line {
                Ok(line) => println!("Child Output: {}", line),
                Err(e) => eprintln!("Error reading line: {}", e),
            }
        }
    }

    #[test]
    fn basic() {

        let my_struct = MyStruct {
            big: 128,
            small: 64
        };

        let endpoint = "127.0.0.1:12345";
        let socket_addr = SocketAddrV4::from_str(endpoint);

        let mut connection: IpcServer = IpcServer::new(socket_addr.unwrap());
        match connection.connect() {
            Err(e) => assert!(false, "{}", e),
            Ok(()) => {}
        }

        let mut child_handle = spawn_python_script(Path::new("python/client.py")).unwrap();
        // Take the stdout handle from the child and wrap it in a BufReader
        let child_stdout = child_handle.stdout.take().expect("Could not capture stdout");

        loop {
            std::thread::sleep(std::time::Duration::from_millis(100));
  
            match connection.get_data(512) {
                Ok(None) => continue, // no data available
                Ok(buffer) => {
                    let unwrapped_buf = buffer.unwrap();
                    let len = unwrapped_buf.len();
                    if len > 0 {
                        println!("Received data size: {:?}", len);
                        let json_str = String::from_utf8(unwrapped_buf)
                        .unwrap();
                    
                        println!("Received data: {}", json_str);
                        let _d: Value = serde_json::from_str(json_str.as_str()).unwrap();
                        println!("{:#?}", &_d);
                        if let Value::String(s) = &_d["cmd"] {
                            match &s[..] {
                                "do-it" => {
                                    let deserialized_obj: MyStruct = serde_json::from_str(json_str.as_str()).unwrap();
                                    assert_eq!(deserialized_obj.big, my_struct.big);
                                    assert_eq!(deserialized_obj.small, my_struct.small);
                                    break;
                                    
                                },
                                _ => assert!(false, "Unknown cmd: {}", s)
                            }
                        }
                    }
                }
                Err(e) => {
                    assert!(false, "{}", e);
                }
            }
        }

        // Read and print the python script stdout
        println!("Reading output from child process:");
        let buf_reader = BufReader::new(child_stdout);
        print_bufreader(buf_reader);
    }
}

