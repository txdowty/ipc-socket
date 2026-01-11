
    use std::net::SocketAddrV4;
    use std::str::FromStr;
    use ipc_socket::ipc::{IpcClient, IpcServer};
    use std::io;

   fn main() -> io::Result<()> {
    let endpoint = "127.0.0.1:12345";
    let server_addr = SocketAddrV4::from_str(endpoint).unwrap();

    let mut connection: IpcServer = IpcServer::new(server_addr);
    match connection.connect() {
        Err(e) => assert!(false, "{}", e),
        Ok(()) => {}
    }
    loop {
        std::thread::sleep(std::time::Duration::from_millis(100));

        match connection.get_data(512) {
            Ok(None) => continue, // no data available
            Ok(buffer) => {
                let unwrapped_buf = buffer.unwrap();
                let len = unwrapped_buf.len();
                if len > 0 {
                    println!("Received data size: {:?}", len);
                    let str = String::from_utf8(unwrapped_buf)
                    .unwrap();

                    println!("Received data: {}", str);

                    // send the same data back (echo)
                    let mut client = IpcClient::new(server_addr);
                    client.connect()?;
                    let bytes_sent = client.send_data(
                        &str.as_bytes(), &server_addr).unwrap();
                    println!("Sent back {} bytes", bytes_sent);
                }
            }
            Err(e) => {
                assert!(false, "{}", e);
            }
        }
    }
}