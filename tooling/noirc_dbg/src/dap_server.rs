#[cfg(feature = "dap")]
use crossbeam::channel::unbounded;
#[cfg(feature = "dap")]
use crossbeam::channel::Receiver;

use crossbeam::channel::TryRecvError;

#[cfg(not(feature = "dap"))]
use std::io::Write;
#[cfg(feature = "dap")]
use std::io::{stdin, stdout, BufReader, BufWriter, Stdin, Stdout};

#[cfg(feature = "dap")]
use std::sync::Mutex;
#[cfg(feature = "dap")]
use std::thread::spawn;

use dap::base_message::Sendable;
use dap::requests::Request;
#[cfg(not(feature = "dap"))]
use dap::requests::{
    Command, ContinueArguments, DisassembleArguments, DisconnectArguments, LaunchRequestArguments,
    NextArguments, ReadMemoryArguments, VariablesArguments,
};
#[cfg(feature = "dap")]
use dap::server::*;
#[cfg(not(feature = "dap"))]
use serde_json::Value;

#[cfg(feature = "dap")]
pub(crate) struct Dap {
    output: Mutex<Server<Stdin, Stdout>>,
    input: Receiver<Request>,
}

#[cfg(not(feature = "dap"))]
#[derive(Debug, Default)]
pub(crate) struct Dap {
    seq: i64,
}

#[cfg(feature = "dap")]
impl Dap {
    pub(crate) fn new() -> Self {
        let (tx, rx) = unbounded::<Request>();
        spawn(move || {
            let mut server = Server::new(BufReader::new(stdin()), BufWriter::new(stdout()));
            loop {
                let req = match server.poll_request() {
                    Ok(Some(req)) => req,
                    Ok(None) => continue,
                    Err(_) => return,
                };
                if tx.send(req).is_err() {
                    return;
                }
            }
        });

        let server = Server::new(BufReader::new(stdin()), BufWriter::new(stdout()));
        Dap { output: Mutex::new(server), input: rx }
    }

    pub(crate) fn read(&self) -> Option<Request> {
        match self.input.try_recv() {
            Ok(req) => Some(req),
            Err(TryRecvError::Disconnected) => None,
            Err(TryRecvError::Empty) => None,
        }
    }

    pub(crate) fn write(&self, message: Sendable) {
        self.output.lock().unwrap().send(message).unwrap();
    }
}

#[cfg(not(feature = "dap"))]
impl Dap {
    pub(crate) fn new() -> Self {
        Dap::default()
    }

    pub(crate) fn read(&mut self) -> Option<Request> {
        self.seq += 1;
        match self.get_request_from_stdin() {
            Ok(req) => Some(req),
            Err(TryRecvError::Disconnected) => None,
            Err(TryRecvError::Empty) => None,
        }
    }

    fn get_request_from_stdin(&self) -> Result<Request, TryRecvError> {
        let command = stdin_get("Enter command> ");
        let req = match command.trim() {
            "c" => {
                Request { seq: self.seq, command: Command::Continue(ContinueArguments::default()) }
            }
            "d" => Request {
                seq: self.seq,
                command: Command::Disassemble(DisassembleArguments::default()),
            },
            "l" => {
                let path = stdin_get("Enter path to file> ");
                let vm = stdin_get("Select vm: 'a' for acvm and 'b' for brillig> ");
                let data = format!(
                    "
                    {{
                        \"src_path\": \"{}\",
                        \"vm\": \"{}\"
                    }}",
                    path, vm
                );
                let additional_data: Option<Value> = serde_json::from_str(&data).ok();

                Request {
                    seq: self.seq,
                    command: Command::Launch(LaunchRequestArguments {
                        additional_data,
                        ..Default::default()
                    }),
                }
            }
            "m" => Request {
                seq: self.seq,
                command: Command::ReadMemory(ReadMemoryArguments::default()),
            },
            "q" => Request {
                seq: self.seq,
                command: Command::Disconnect(DisconnectArguments::default()),
            },
            "r" => Request {
                seq: self.seq,
                command: Command::Variables(VariablesArguments::default()),
            },
            "s" => Request { seq: self.seq, command: Command::Next(NextArguments::default()) },
            _ => unimplemented!(),
        };
        Ok(req)
    }

    pub(crate) fn write(&self, message: Sendable) {
        match message {
            Sendable::Response(r) => println!("{:#?}", r),
            Sendable::Event(e) => println!("{:#?}", e),
            Sendable::ReverseRequest(r) => println!("{:#?}", r),
        }
    }
}

fn stdin_get(msg: &str) -> String {
    let mut response = String::new();
    print!("{msg}");
    std::io::stdout().flush().ok();
    std::io::stdin().read_line(&mut response).ok();
    response.trim().to_string()
}
