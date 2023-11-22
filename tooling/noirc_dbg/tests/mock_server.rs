use crossbeam::channel::TryRecvError;
use dap::requests::{
    Command, ContinueArguments, DisassembleArguments, DisconnectArguments, LaunchRequestArguments,
    NextArguments, ReadMemoryArguments, VariablesArguments,
};
use dap::responses::ResponseBody;
use dap::{base_message::Sendable, requests::Request};
use noirc_dbg::dap_server::Server;
use serde_json::Value;

use std::cell::RefCell;

//

#[derive(Debug, Default)]
pub(crate) struct MockDap {
    pub(crate) seq: i64,
    pub(crate) command: String,
    pub(crate) response: RefCell<(i64, String)>,
    pub(crate) asset_dir: String,
}

impl MockDap {
    // it's a little strange thing. I used the method in the tests but the linter warns about
    // unused method
    #[allow(dead_code)]
    pub(crate) fn new(asset_dir: impl Into<String>) -> Self {
        Self { asset_dir: asset_dir.into(), ..Default::default() }
    }

    pub(crate) fn get_request(&self) -> Result<Request, TryRecvError> {
        match self.command.as_str() {
            "continue" => Ok(Request {
                seq: self.seq,
                command: Command::Continue(ContinueArguments::default()),
            }),
            "disassemble" => Ok(Request {
                seq: self.seq,
                command: Command::Disassemble(DisassembleArguments::default()),
            }),
            "launch" => {
                let module_path = std::env::var("CARGO_MANIFEST_DIR").unwrap();
                let data = format!(
                    "{{
                        \"src_path\": \"{}/tests/{}\",
                        \"vm\": \"b\"
                    }}",
                    module_path, self.asset_dir
                );
                let additional_data: Option<Value> = serde_json::from_str(&data).ok();

                Ok(Request {
                    seq: self.seq,
                    command: Command::Launch(LaunchRequestArguments {
                        additional_data,
                        ..Default::default()
                    }),
                })
            }
            "memory" => Ok(Request {
                seq: self.seq,
                command: Command::ReadMemory(ReadMemoryArguments::default()),
            }),
            "quit" => Ok(Request {
                seq: self.seq,
                command: Command::Disconnect(DisconnectArguments::default()),
            }),
            "registers" => Ok(Request {
                seq: self.seq,
                command: Command::Variables(VariablesArguments::default()),
            }),
            "step" => {
                Ok(Request { seq: self.seq, command: Command::Next(NextArguments::default()) })
            }
            _ => Err(TryRecvError::Empty),
        }
    }
}

impl Server for MockDap {
    fn read(&mut self) -> Option<Request> {
        self.seq += 1;
        match self.get_request() {
            Ok(req) => Some(req),
            Err(TryRecvError::Disconnected) => None,
            Err(TryRecvError::Empty) => None,
        }
    }

    fn write(&self, message: Sendable) {
        match message {
            Sendable::Response(r) => {
                let body = &r.body;
                let seq = r.request_seq;

                match body {
                    Some(ResponseBody::Variables(v)) => {
                        let regs = v
                            .variables
                            .iter()
                            .map(|v| v.value.to_string())
                            .collect::<Vec<_>>()
                            .join(",");
                        *self.response.borrow_mut() = (seq, regs);
                    }
                    Some(ResponseBody::Disassemble(v)) => {
                        let program = v
                            .instructions
                            .iter()
                            .map(|v| format!("{}  {}", v.address, v.instruction))
                            .collect::<Vec<_>>()
                            .join("\n");
                        *self.response.borrow_mut() = (seq, program);
                    }
                    Some(any) => {
                        *self.response.borrow_mut() = (seq, format!("{:?}", any));
                    }
                    None => {
                        *self.response.borrow_mut() = (seq, "".to_string());
                    }
                };
            }
            Sendable::Event(e) => match e {
                dap::prelude::Event::Invalidated(_) => {}
                _ => *self.response.borrow_mut() = (0, format!("{:?}", e)),
            },
            Sendable::ReverseRequest(r) => {
                *self.response.borrow_mut() = (r.seq, format!("{:?}", r));
            }
        }
    }
}
