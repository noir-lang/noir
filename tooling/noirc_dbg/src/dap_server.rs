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

#[cfg(not(feature = "dap"))]
use dap::requests::{
    Command, ContinueArguments, DisassembleArguments, DisconnectArguments, LaunchRequestArguments,
    NextArguments, ReadMemoryArguments, VariablesArguments,
};
#[cfg(not(feature = "dap"))]
use dap::responses::ResponseBody;
use dap::{
    base_message::Sendable,
    events::{Event, StoppedEventBody},
    requests::Request,
};

#[cfg(feature = "dap")]
use dap::server::*;
#[cfg(not(feature = "dap"))]
use serde_json::Value;

/// To ability change app server.
pub trait Server {
    /// Read request from input.
    fn read(&mut self) -> Option<Request>;
    /// Write response to server output.
    fn write(&self, message: Sendable);
}

#[cfg(feature = "dap")]
/// Dap realization of server. It is realization to communicate with IDEs and provide standard
/// debugging interface.
pub struct Dap {
    /// Write output to server communicated through stdin and stdout
    output: Mutex<Server<Stdin, Stdout>>,
    /// Handled input from receiver thread
    input: Receiver<Request>,
}

#[cfg(not(feature = "dap"))]
/// Repl realization of server. For cli usage.
#[derive(Debug, Default)]
pub struct Dap {
    /// Sequential number of request
    seq: i64,
}

#[cfg(feature = "dap")]
impl Dap {
    /// Create dap server.
    pub fn new() -> Self {
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
}
#[cfg(feature = "dap")]
impl Server for Dap {
    /// Read request from receiver.
    fn read(&mut self) -> Option<Request> {
        match self.input.try_recv() {
            Ok(req) => Some(req),
            Err(TryRecvError::Disconnected) => None,
            Err(TryRecvError::Empty) => None,
        }
    }

    /// Write response to server.
    fn write(&self, message: Sendable) {
        self.output.lock().unwrap().send(message).unwrap();
    }
}

#[cfg(not(feature = "dap"))]
impl Dap {
    /// Create repl server.
    pub fn new() -> Self {
        Dap::default()
    }

    /// Read input from stdin and map it to requests.
    fn get_request_from_stdin(&self) -> Result<Request, TryRecvError> {
        let command = stdin_get("Enter command> ");
        let req = match command.trim() {
            "c" | "continue" => {
                Request { seq: self.seq, command: Command::Continue(ContinueArguments::default()) }
            }
            "d" | "disassemble" => Request {
                seq: self.seq,
                command: Command::Disassemble(DisassembleArguments::default()),
            },
            "l" | "launch" => {
                let path = stdin_get("Enter path to Noir binary module> ");
                // TODO: skipped for now
                // let vm = stdin_get("Select vm: 'a' for acvm and 'b' for brillig> ");
                let data = format!(
                    "
                    {{
                        \"src_path\": \"{}\",
                        \"vm\": \"b\"
                    }}",
                    path /* , vm */
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
            "m" | "memory" => Request {
                seq: self.seq,
                command: Command::ReadMemory(ReadMemoryArguments::default()),
            },
            "q" | "quit" => Request {
                seq: self.seq,
                command: Command::Disconnect(DisconnectArguments::default()),
            },
            "r" | "registers" => Request {
                seq: self.seq,
                command: Command::Variables(VariablesArguments::default()),
            },
            "s" | "step" => {
                Request { seq: self.seq, command: Command::Next(NextArguments::default()) }
            }
            command => {
                println!("Unknown command \"{}\"", command);
                println!("Please type any of valid commands");
                self.get_request_from_stdin()?
            }
        };
        Ok(req)
    }
}
#[cfg(not(feature = "dap"))]
impl Server for Dap {
    fn read(&mut self) -> Option<Request> {
        self.seq += 1;
        match self.get_request_from_stdin() {
            Ok(req) => Some(req),
            Err(TryRecvError::Disconnected) => None,
            Err(TryRecvError::Empty) => None,
        }
    }

    fn write(&self, message: Sendable) {
        match message {
            Sendable::Response(r) => {
                let body = &r.body;
                match body {
                    Some(ResponseBody::Variables(v)) => {
                        println!("Registers:");
                        v.variables.iter().for_each(|v| {
                            println!("    {} => {}", v.name, v.value);
                        });
                    }
                    Some(ResponseBody::Disassemble(v)) => {
                        println!("Program:");
                        v.instructions
                            .iter()
                            .for_each(|v| println!("{}  {}", v.address, v.instruction));
                    }
                    Some(any) => println!("{:#?}", any),
                    None => println!("{:#?}", r),
                };
            }
            Sendable::Event(e) => match e {
                Event::Stopped(StoppedEventBody { reason, description, .. }) => {
                    println!(
                        "Stopped.\n    Reason: {:?}\n    Description: {:?}",
                        reason, description
                    );
                }
                _ => println!("{:#?}", e),
            },
            Sendable::ReverseRequest(r) => println!("{:#?}", r),
        }
    }
}

/// Handler for stdin.
fn stdin_get(msg: &str) -> String {
    let mut response = String::new();
    print!("{msg}");
    std::io::stdout().flush().ok();
    std::io::stdin().read_line(&mut response).ok();
    response.trim().to_string()
}
