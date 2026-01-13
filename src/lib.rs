pub mod ipc;

// use serde::{Deserialize, Serialize};
// #[derive(Deserialize, Serialize, PartialEq, Eq, Debug)]
// struct MyStruct {
//     big: u64,
//     small: u32,
// }

// #[cfg(test)]
// mod tests {
//     // use super::*;
//     use crate::ipc::IpcServer;
//     use serde::{Deserialize, Serialize};
//     use std::{env};
//     use std::io;
//     use std::io::{BufRead, BufReader};
//     use std::net::SocketAddrV4;
//     use std::path::{Path, PathBuf};
//     use std::process::{Child, Command, Stdio, ChildStdout};
//     use std::str::FromStr;
//     use serde_json::{Value};

//     #[derive(Deserialize, Serialize, PartialEq, Eq, Debug)]
//     struct MyStruct {
//         big: u64,
//         small: u32,
//     }

//     fn spawn_python_script(rel_path_to_script: &Path) -> io::Result<Child> {
//         // start the client app using the venv python
//         let package_root_dir = env!("CARGO_MANIFEST_DIR");
//         let python_path = PathBuf::from(package_root_dir)
//             .join(".venv").join("bin").join("python");
//         let script_path = PathBuf::from(package_root_dir)
//             .join(rel_path_to_script);

//         println!("spawning: {} {}", python_path.to_str().unwrap(),
//             script_path.to_str().unwrap());

//         let mut script_cmd = Command::new(python_path);
//         script_cmd.arg(&script_path.to_str().unwrap()).stdout(Stdio::piped());
//         script_cmd.spawn()
//     }

// fn print_bufreader(reader: BufReader<ChildStdout>) {
//    for line in reader.lines() {
//         match line {
//             Ok(line) => println!("Child Output: {}", line),
//             Err(e) => eprintln!("Error reading line: {}", e),
//         }
//     }
// }

//     #[test]
//     fn basic() {

//         let my_struct = MyStruct {
//             big: 128,
//             small: 64
//         };

//         let endpoint = "127.0.0.1:12345";
//         let socket_addr = SocketAddrV4::from_str(endpoint);

//         let mut connection: IpcServer = IpcServer::new(socket_addr.unwrap());
//         match connection.connect() {
//             Err(e) => assert!(false, "{}", e),
//             Ok(()) => {}
//         }

//         let mut child_handle = spawn_python_script(Path::new("python/client.py")).unwrap();
//         // Take the stdout handle from the child and wrap it in a BufReader
//         let child_stdout = child_handle.stdout.take().expect("Could not capture stdout");

//         loop {
//             std::thread::sleep(std::time::Duration::from_millis(100));

//             match connection.get_data(512) {
//                 Ok(None) => continue, // no data available
//                 Ok(buffer) => {
//                     let unwrapped_buf = buffer.unwrap();
//                     let len = unwrapped_buf.len();
//                     if len > 0 {
//                         println!("Received data size: {:?}", len);
//                         let json_str = String::from_utf8(unwrapped_buf)
//                         .unwrap();

//                         println!("Received data: {}", json_str);
//                         let _d: Value = serde_json::from_str(json_str.as_str()).unwrap();
//                         println!("{:#?}", &_d);
//                         if let Value::String(s) = &_d["cmd"] {
//                             match &s[..] {
//                                 "do-it" => {
//                                     let deserialized_obj: MyStruct = serde_json::from_str(json_str.as_str()).unwrap();
//                                     assert_eq!(deserialized_obj.big, my_struct.big);
//                                     assert_eq!(deserialized_obj.small, my_struct.small);
//                                     break;

//                                 },
//                                 _ => assert!(false, "Unknown cmd: {}", s)
//                             }
//                         }
//                     }
//                 }
//                 Err(e) => {
//                     assert!(false, "{}", e);
//                 }
//             }
//         }

//         // Read and print the python script stdout
//         println!("Reading output from child process:");
//         let buf_reader = BufReader::new(child_stdout);
//         print_bufreader(buf_reader);
//     }
// }
