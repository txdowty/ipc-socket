

mod tests {
    use crate::ipc::IpcClient;
  
    use std::io::{self};
    use std::io::{BufReader, BufRead};
    use std::net::SocketAddrV4;
    use std::process::{Child, Command, Stdio};
    use std::str::FromStr;
    use nix::unistd::Pid;
    use nix::sys::signal::{self, Signal};

    // #[derive(Deserialize, Serialize, PartialEq, Eq, Debug)]
    // struct MyStruct {
    //     big: u64,
    //     small: u32,
    // }

    fn cargo_run_program(program_name: &str) -> io::Result<Child> {
        println!("spawning: {}", program_name);

        let mut script_cmd = Command::new("cargo");
        script_cmd.arg("run").arg(program_name).stdout(Stdio::piped());
        script_cmd.spawn()
    }

    #[test]
    fn basic() -> io::Result<()> {

        // let my_struct = MyStruct {
        //     big: 128,
        //     small: 64
        // };

        let endpoint = "127.0.0.1:12345";
        let server_addr = SocketAddrV4::from_str(endpoint).unwrap();

        let mut server_handle = cargo_run_program("echo_server").unwrap();

        let client_addr = SocketAddrV4::from_str("127.0.0.1:12346").unwrap();
        let mut client_socket = IpcClient::new(client_addr);
        client_socket.connect().unwrap();
        let str = String::from("hello");
        let bytes_sent = client_socket.send_data(
            &str.as_bytes(), &server_addr).unwrap();
        assert_eq!(bytes_sent, str.len());
    
        let id = server_handle.id() as i32;

        let child_pid = Pid::from_raw(id);

        println!("Sending SIGTERM to child process with PID: {}", id);
        signal::kill(child_pid, Signal::SIGTERM)?;

        // 4. Wait for the child to exit
        let status = server_handle.wait()?;
        println!("Child process exited with status: {}", status);
        
        // Read and print the child process stdout
        println!("Reading output from child process:");
        let child_stdout = server_handle.stdout.take().unwrap();
        // print_stdout(&server_handle);
        let buf_reader = BufReader::new(child_stdout);
        for line in buf_reader.lines() {
            match line {
                Ok(line) => println!("Child Output: {}", line),
                Err(e) => eprintln!("Error reading line: {}", e),            
            }
        }

        Ok(())
    }
}

