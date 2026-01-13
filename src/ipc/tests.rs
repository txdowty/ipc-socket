#[cfg(test)] // Tells Rust to compile this module only during tests

mod tests {
    use crate::ipc::IpcClient;

    use std::io::{self, Error, ErrorKind};
    use std::io::{BufRead, BufReader};
    use std::net::SocketAddr;
    use std::process::{Child, Command, Stdio};
    use std::str::FromStr;
    use std::time::Duration;

    // #[derive(Deserialize, Serialize, PartialEq, Eq, Debug)]
    // struct MyStruct {
    //     big: u64,
    //     small: u32,
    // }

    fn cargo_run_program(program_name: &str) -> io::Result<Child> {
        println!("spawning: {}", program_name);

        let mut script_cmd = Command::new("cargo");
        script_cmd
            .arg("run")
            .arg(program_name)
            .stdout(Stdio::piped());
        script_cmd.spawn()
    }

    #[test]
    fn write_and_read_response() -> io::Result<()> {
        // start the echo server via cargo
        let mut child_handle = cargo_run_program("echo_server").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1000));

        // create client object
        // let the os choose the port
        let client_addr = SocketAddr::from_str("127.0.0.1:0").unwrap();
        let mut client = IpcClient::new(client_addr);

        client.bind()?;

        let endpoint = "127.0.0.1:12345";
        let server_addr = SocketAddr::from_str(endpoint).unwrap();

        // send data from client to server
        let str = String::from("hello");

        match client.send_wait_response(&str.as_bytes(), &server_addr, 1024) {
            Ok(Some((rec_addr, rec_data))) => {
                // compare data received with data sent
                assert_eq!(rec_data.as_slice(), str.as_bytes());
                // compare origin address with echo server address
                assert_eq!(rec_addr, server_addr);
            }
            Ok(None) => assert!(false, "No data retured from server"),
            Err(e) => return Err(e),
        }

        let str = String::from("exit");
        let _bytes_sent = client.send(&str.as_bytes(), &server_addr)?;

        // Wait for the child to exit
        let status = child_handle.wait()?;
        println!("Child process exited with status: {}", status);

        // Read and print the child process stdout
        println!("Reading output from child process:");
        let child_stdout = child_handle.stdout.take().unwrap();
        let buf_reader = BufReader::new(child_stdout);
        for line in buf_reader.lines() {
            match line {
                Ok(line) => println!("Child Output: {}", line),
                Err(e) => eprintln!("Error reading line: {}", e),
            }
        }

        Ok(())
    }

    #[test]
    fn write_and_read_response_timeout() -> io::Result<()> {
        // start the echo server via cargo
        // let mut server_handle = cargo_run_program("echo_server").unwrap();
        // std::thread::sleep(std::time::Duration::from_millis(1000));

        let endpoint = "127.0.0.1:12345";
        let server_addr = SocketAddr::from_str(endpoint).unwrap();

        // let the os choose the port
        let client_addr = SocketAddr::from_str("127.0.0.1:0").unwrap();
        let mut client = IpcClient::new(client_addr);

        client.bind()?;
        // set read timeout
        client.set_read_timeout(Some(Duration::from_secs(1)))?;

        // send data from client to server
        let str = String::from("2");
        let result = client.send_wait_response(&str.as_bytes(), &server_addr, 1024);
        match result {
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {/* expected error */},
            Err(ref e) => {
                panic!("wrong error returned: {:#?}", e)
            },
            Ok(_) => panic!("error"),
        }

        let str = String::from("exit");
        let _bytes_sent = client.send(&str.as_bytes(), &server_addr)?;

        // // Wait for the child to exit
        // let status = server_handle.wait()?;
        // println!("Child process exited with status: {}", status);

        // // Read and print the child process stdout
        // println!("Reading output from child process:");
        // let child_stdout = server_handle.stdout.take().unwrap();
        // let buf_reader = BufReader::new(child_stdout);
        // for line in buf_reader.lines() {
        //     match line {
        //         Ok(line) => println!("Child Output: {}", line),
        //         Err(e) => eprintln!("Error reading line: {}", e),
        //     }
        // }

        Ok(())
    }
}
