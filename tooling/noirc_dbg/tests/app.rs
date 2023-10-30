use crossbeam::channel::TryRecvError;
use dap::requests::{
    Command,
    // ContinueArguments, DisassembleArguments, DisconnectArguments,
    LaunchRequestArguments,
    NextArguments,
    // ReadMemoryArguments, VariablesArguments,
};
use dap::responses::ResponseBody;
use dap::{base_message::Sendable, requests::Request};
use noirc_dbg::app::{App, State};
use noirc_dbg::dap_server::Server;
use serde_json::Value;

use std::sync::Mutex;

//

static COMMAND: Mutex<String> = Mutex::new(String::new());
static RESPONSE: Mutex<(i64, String)> = Mutex::new((0, String::new()));

fn get_command() -> String {
    COMMAND.lock().unwrap().to_string()
}

//

#[derive(Debug, Default)]
struct MockDap {
    seq: i64,
}

impl MockDap {
    pub(crate) fn get_request(&self) -> Result<Request, TryRecvError> {
        match get_command().as_str() {
            "launch" => {
                let module_path = std::env::var("CARGO_MANIFEST_DIR").unwrap();
                let data = format!(
                    "{{
                        \"src_path\": \"{}/tests/simple\",
                        \"vm\": \"b\"
                    }}",
                    module_path
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
                        *RESPONSE.lock().unwrap() = (seq, regs);
                    }
                    Some(any) => {
                        *RESPONSE.lock().unwrap() = (seq, format!("{:?}", any));
                    }
                    None => {
                        *RESPONSE.lock().unwrap() = (seq, "".to_string());
                    }
                };
            }
            Sendable::Event(e) => {
                *RESPONSE.lock().unwrap() = (0, format!("{:?}", e));
            }
            Sendable::ReverseRequest(r) => {
                *RESPONSE.lock().unwrap() = (r.seq, format!("{:?}", r));
            }
        }
    }
}

#[test]
fn success() {
    let dap = MockDap::default();
    let mut app = App::initialize(dap);

    //

    if let State::Uninitialized(ref _s) = app.state {
        assert_eq!(true, true);
    } else {
        assert_eq!(false, true);
    };

    *COMMAND.lock().unwrap() = "launch".to_string();
    let r = app.run();
    let resp = String::from("Stopped(StoppedEventBody { reason: Entry, description: Some(\"Entry\"), thread_id: Some(0), preserve_focus_hint: Some(false), text: None, all_threads_stopped: Some(false), hit_breakpoint_ids: None })");
    assert_eq!(resp, RESPONSE.lock().unwrap().1);
    assert!(r.is_ok());
    if let State::Running(ref _s) = app.state {
        assert_eq!(true, true);
    } else {
        assert_eq!(false, true);
    };

    *COMMAND.lock().unwrap() = "step".to_string();
    let r = app.run();
    let resp = String::from("Stopped(StoppedEventBody { reason: Step, description: Some(\"Step\"), thread_id: Some(0), preserve_focus_hint: Some(false), text: None, all_threads_stopped: Some(false), hit_breakpoint_ids: None })");
    assert_eq!(resp, RESPONSE.lock().unwrap().1);
    assert!(r.is_ok());
    if let State::Running(ref _s) = app.state {
        assert_eq!(true, true);
    } else {
        assert_eq!(false, true);
    };
}
