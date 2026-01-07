use std::{
    io,
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
};

pub struct Connection {
    address: String,
    port: u16,
    listener: Option<TcpListener>,
}

impl Connection {
    pub fn new(address: String, port: u16) -> Self {
        Connection {
            address: address,
            port: port,
            listener: None,
        }
    }

    pub fn connect(&mut self) -> io::Result<()> {
        let full_address = format!("{}:{}", self.address, self.port);
        let listener = TcpListener::bind(full_address)?;
        // non-blocking
        listener.set_nonblocking(true)?;
        self.listener = Some(listener);
        // if self.listener.is_none() {
        //     std::io::Error
        //     return Err("no listener");
        // }
        // assert!(self.listener.is_some());
        Ok(())
    }

    pub fn get_data(&self, buffer: &mut [u8; 512]) -> Option<String> {
        if self.listener.is_none() {
            return None;
        }

        let incoming = self.listener.as_ref().unwrap().incoming();
        for stream in incoming {
            match stream {
                Ok(stream) => {
                    return Some(Self::handle_connection(stream, buffer));
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No incoming connection, return None
                    return None;
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e);
                }
            }
        }
        None
    }


    fn get_message_size(stream: &mut BufReader<&TcpStream>) -> std::io::Result<usize> {
        let mut size_buffer = [0u8; 4];
        stream.read_exact(&mut size_buffer)?;
        let size = u32::from_be_bytes(size_buffer) as usize;
        Ok(size)
    }

    fn handle_connection(stream: TcpStream, buffer: &mut [u8; 512]) -> String {
        let mut buf_reader = BufReader::new(&stream);
        let _total_size = Self::get_message_size(&mut buf_reader);
        let mut_slice: &mut [u8] = &mut buffer[..];
        loop {
            // read data into buffer, returns numer of bytes read
            match buf_reader.read(mut_slice) {
                Ok(0) => break, // connection closed
                Ok(n) => {
                    println!("Read {} bytes", n);
                    // total_bytes_read += n;
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) => {
                    eprintln!("Error reading from stream: {}", e);
                    break;
                }
            }
        }

        String::from("Data received")
        // let http_request: Vec<_> = buf_reader
        //     .lines()
        //     .map(|result| result.unwrap())
        //     .take_while(|line| !line.is_empty())
        //     .collect();

        // let http_request: Vec<String> = raw_data.iter().map(|b| format!("{:02X}", b)).collect();
        // println!("Request: {http_request:#?}");
        // http_request.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // use serde_cbor::from_slice;
    use serde::{Deserialize, Serialize};
    use std::path::PathBuf;
    use std::process::{Command, Stdio};
    use std::{env, fs};

    #[derive(Deserialize, Serialize, PartialEq, Eq, Debug)]
    struct MyStruct {
        big: u64,
        small: u32,
    }

    
    #[test]
    fn basic() {

        
        let ip_address = String::from("127.0.0.1");
        let port = 12345;

        let mut connection: Connection = Connection::new(ip_address, port);
        match connection.connect() {
            Err(e) => assert!(false, "{}", e),
            Ok(()) => {}
        }

        // start the client app
        let package_root_dir = env!("CARGO_MANIFEST_DIR");
        let python_path = PathBuf::from(package_root_dir)
            .join(".venv").join("bin").join("python");
        let file_path = PathBuf::from(package_root_dir)
            .join("python").join("client.py");
        println!("{}", file_path.to_str().unwrap());
        
        assert!(fs::exists(&file_path).unwrap());

        Command::new("ls").arg(&file_path).spawn().expect("ls failed");

        let mut output = Command::new(python_path);

        output.arg(&file_path.to_str().unwrap()).stdout(Stdio::piped());
        let mut child = output.spawn().expect("couldn't start script");

        // Take the stdout handle from the child and wrap it in a BufReader
        let stdout = child.stdout.take().expect("Could not capture stdout");
        let reader = BufReader::new(stdout);

        let mut buffer: [u8; 512] = [0; 512];
        loop {
            std::thread::sleep(std::time::Duration::from_millis(100));
            let x = connection.get_data(&mut buffer);
            if let Some(_data) = x {
                let mut_slice: &mut [u8] = &mut buffer[..33];
                println!("Received data: {:?}", mut_slice);

                // deserialize from CBOR format
                let received: MyStruct = serde_cbor::from_slice(mut_slice).unwrap();
                println!("Deserialized struct: {:?}", received);
                break;
            }
        }
        // Read the output line by line
        println!("Reading output from child process:");
        for line in reader.lines() {
            match line {
            Ok(line) => println!("Child Output: {}", line),
            Err(e) => eprintln!("Error reading line: {}", e),
        }
    }
    }

}
