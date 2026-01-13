
use ipc_socket::ipc::IpcServer;
use std::io;
use std::net::SocketAddr;
use std::str::FromStr;
use std::thread;
use std::time::Duration;

// netstat -tulpn

fn main() -> io::Result<()> {
    let endpoint = "127.0.0.1:12345";
    let server_addr = SocketAddr::from_str(endpoint).unwrap();

    let mut server: IpcServer = IpcServer::new(server_addr);
    server.bind()?;
    println!("Server listening on {}", endpoint);
    loop {
        match server.get(512) {
            Ok(None) => continue, // no data available
            Ok(Some((source_addr, buffer))) => {
                let len = buffer.len();
                if len == 0 {
                    continue;
                }

                let str = String::from_utf8(buffer).unwrap();

                println!("Received data from {} size: {:?}", source_addr, len);
                println!("Received data: {}", str);

                if str == "exit" {
                    println!("Exiting server.");
                    break;
                }

                // if str.parse::<u64>().is_ok() {
                //     let w = str.parse::<u64>().unwrap();
                //         println!("waiting {} secs...", w);
                //         thread::sleep(std::time::Duration::from_secs(w));
                // }

                let wait_time: Result<u64, _> = str.parse();
                match wait_time {
                    Ok(w) => {
                        println!("waiting {} secs...", w);
                        thread::sleep(Duration::from_secs(w));
                    }
                    Err(_) => {}
                }
                // send the same data back (echo)
                server.send(str.as_bytes(), &source_addr)?;

                println!("Sent back {} bytes", len);
            }
            Err(e) => {
                return Err(e);
            }
        }
        thread::sleep(std::time::Duration::from_millis(100));
    }
    Ok(())
}
