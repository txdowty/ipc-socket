use std::{
    io::{self, BufReader, ErrorKind, prelude::*},
    mem,
    net::{TcpListener, TcpStream}
};
    use std::net::SocketAddrV4;

pub struct Connection {
    sock_addr: SocketAddrV4,
    listener: Option<TcpListener>,
}

impl Connection {
    pub fn new(sock_addr: SocketAddrV4) -> Self {
        Connection {
            sock_addr: sock_addr,
            listener: None,
        }
    }

    pub fn connect(&mut self) -> io::Result<()> {
        let listener = TcpListener::bind(self.sock_addr)?;
        // set non-blocking
        listener.set_nonblocking(true)?;
        self.listener = Some(listener);
        Ok(())
    }

    pub fn get_data(&self, max_size: usize) -> io::Result<Vec<u8>> {
        // if self.listener.is_none() {
        //     return None;
        // }

        let incoming = self.listener.as_ref().unwrap().incoming();
        for stream in incoming {
            match stream {
                Ok(stream) => {
                    return Self::handle_connection(stream, max_size);
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No incoming connection, return empty vec
                    break;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Ok(Vec::new())
    }

    fn get_message_size(stream: &mut BufReader<&TcpStream>) -> io::Result<usize> {
        let mut size_buffer = [0u8; 4];
        stream.read_exact(&mut size_buffer)?;
        let size = u32::from_be_bytes(size_buffer) as usize;
        Ok(size)
    }

    fn handle_connection(stream: TcpStream, max_size: usize) -> io::Result<Vec<u8>> {
        let mut buf_reader = BufReader::new(&stream);

        // get message size from message header
        let total_size = Self::get_message_size(&mut buf_reader).unwrap();

        // is max_size adequate?
        if total_size + mem::size_of::<u32>() > max_size {
            return Err(io::Error::new(ErrorKind::FileTooLarge, "max size exceeded"));
        }

        // allocate data buffer
        let mut buffer: Vec<u8> = vec![0; total_size];
        // let buffer_slice: &mut [u8] = &mut buffer[..];
        let buffer_slice: &mut [u8] = buffer.as_mut_slice();

        buf_reader.read_exact(buffer_slice)?;

        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // use serde_cbor::from_slice;
    use serde::{Deserialize, Serialize};
    use std::path::{Path, PathBuf};
    use std::process::{Child, Command, Stdio};
    use std::str::FromStr;
    use std::{env};
    use std::net::SocketAddrV4;

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

    #[test]
    fn basic() {

        let my_struct = MyStruct {
            big: 128,
            small: 64
        };

        let endpoint = "127.0.0.1:12345";
        let socket_addr = SocketAddrV4::from_str(endpoint);

        let mut connection: Connection = Connection::new(socket_addr.unwrap());
        match connection.connect() {
            Err(e) => assert!(false, "{}", e),
            Ok(()) => {}
        }

        // Take the stdout handle from the child and wrap it in a BufReader
        let mut child_handle = spawn_python_script(Path::new("python/client.py")).unwrap();
        let child_stdout = child_handle.stdout.take().expect("Could not capture stdout");
        let reader = BufReader::new(child_stdout);

        loop {
            std::thread::sleep(std::time::Duration::from_millis(100));
            // let x = connection.get_data(&mut buffer);
            match connection.get_data(512) {
                Err(e) => {
                    assert!(false, "{}", e);
                }
                Ok(buffer) => {
                    if buffer.len() > 0 {
                        println!("Received data size: {:?}", buffer.len());
                        let json_str = String::from_utf8(buffer)
                                                    .unwrap();
                        // deserialize from JSON format
                        println!("Received data: {:?}", json_str);
                        let deserialized_obj: MyStruct = serde_json::from_str(json_str.as_str()).unwrap();
                        assert_eq!(deserialized_obj.big, my_struct.big);
                        assert_eq!(deserialized_obj.small, my_struct.small);
                        break;
                    }
                }
            }
        }
        // Read and print the output line by line
        println!("Reading output from child process:");
        for line in reader.lines() {
            match line {
                Ok(line) => println!("Child Output: {}", line),
                Err(e) => eprintln!("Error reading line: {}", e),
            }
        }
    }

}
