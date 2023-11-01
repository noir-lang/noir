use crossbeam::channel::TryRecvError;
use dap::requests::{
    Command,
    ContinueArguments,
    // DisassembleArguments, DisconnectArguments,
    LaunchRequestArguments,
    NextArguments,
    // ReadMemoryArguments,
    VariablesArguments,
};
use dap::responses::ResponseBody;
use dap::{base_message::Sendable, requests::Request};
use noirc_dbg::app::{App, State};
use noirc_dbg::dap_server::Server;
use serde_json::Value;

use std::cell::RefCell;

//

#[derive(Debug, Default)]
struct MockDap {
    seq: i64,
    command: String,
    response: RefCell<(i64, String)>,
    asset_dir: String,
}

impl MockDap {
    pub(crate) fn new(asset_dir: impl Into<String>) -> Self {
        Self { asset_dir: asset_dir.into(), ..Default::default() }
    }

    pub(crate) fn get_request(&self) -> Result<Request, TryRecvError> {
        match self.command.as_str() {
            "continue" => Ok(Request {
                seq: self.seq,
                command: Command::Continue(ContinueArguments::default()),
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
                    Some(any) => {
                        *self.response.borrow_mut() = (seq, format!("{:?}", any));
                    }
                    None => {
                        *self.response.borrow_mut() = (seq, "".to_string());
                    }
                };
            }
            Sendable::Event(e) => {
                *self.response.borrow_mut() = (0, format!("{:?}", e));
            }
            Sendable::ReverseRequest(r) => {
                *self.response.borrow_mut() = (r.seq, format!("{:?}", r));
            }
        }
    }
}

#[test]
fn simple_with_success() {
    let dap = MockDap::new("simple");
    let mut app = App::initialize(dap);

    //

    match app.state {
        State::Uninitialized(ref _s) => assert_eq!(true, true),
        _ => assert_eq!(false, true),
    }
    app.server.command = "launch".to_string();
    let r = app.run();
    let resp = String::from("Stopped(StoppedEventBody { reason: Entry, description: Some(\"Entry\"), thread_id: Some(0), preserve_focus_hint: Some(false), text: None, all_threads_stopped: Some(false), hit_breakpoint_ids: None })");
    assert_eq!(resp, app.server.response.borrow().1);
    assert!(r.is_ok());
    match app.state {
        State::Running(ref _s) => assert_eq!(true, true),
        _ => assert_eq!(false, true),
    }

    app.server.command = "step".to_string();
    let r = app.run();
    let resp = String::from("Stopped(StoppedEventBody { reason: Step, description: Some(\"Step\"), thread_id: Some(0), preserve_focus_hint: Some(false), text: None, all_threads_stopped: Some(false), hit_breakpoint_ids: None })");
    assert_eq!(resp, app.server.response.borrow().1);
    assert!(r.is_ok());
    match app.state {
        State::Running(ref _s) => assert_eq!(true, true),
        _ => assert_eq!(false, true),
    }

    app.server.command = "continue".to_string();
    let r = app.run();
    let resp = String::from("Terminate");
    assert_eq!(resp, app.server.response.borrow().1);
    assert!(r.is_ok());
    match app.state {
        State::Exit => assert_eq!(true, true),
        _ => assert_eq!(false, true),
    }
}

#[test]
fn value_in_memory_with_success() {
    let dap = MockDap::new("value_in_memory");
    let mut app = App::initialize(dap);

    //

    match app.state {
        State::Uninitialized(ref _s) => assert_eq!(true, true),
        _ => assert_eq!(false, true),
    }
    app.server.command = "launch".to_string();
    let r = app.run();
    let resp = String::from("Stopped(StoppedEventBody { reason: Entry, description: Some(\"Entry\"), thread_id: Some(0), preserve_focus_hint: Some(false), text: None, all_threads_stopped: Some(false), hit_breakpoint_ids: None })");
    assert_eq!(resp, app.server.response.borrow().1);
    assert!(r.is_ok());
    match app.state {
        State::Running(ref _s) => assert_eq!(true, true),
        _ => assert_eq!(false, true),
    }

    app.server.command = "continue".to_string();
    let r = app.run();
    let resp = String::from("Terminate");
    assert_eq!(resp, app.server.response.borrow().1);
    assert!(r.is_ok());
    match app.state {
        State::Exit => assert_eq!(true, true),
        _ => assert_eq!(false, true),
    }
}

#[test]
fn check_registers_success() {
    let dap = MockDap::new("simple");
    let mut app = App::initialize(dap);

    // registers on init step
    app.server.command = "launch".to_string();
    app.run().unwrap();
    app.server.command = "registers".to_string();
    app.run().unwrap();
    let resp = String::from("0");
    assert_eq!(resp, app.server.response.borrow().1);

    // steps + checks
    app.server.command = "step".to_string();
    app.run().unwrap();
    app.server.command = "registers".to_string();
    app.run().unwrap();
    let resp = String::from("0,0,0");
    assert_eq!(resp, app.server.response.borrow().1);

    app.server.command = "step".to_string();
    app.run().unwrap();
    app.run().unwrap();
    app.run().unwrap();
    app.run().unwrap();
    app.server.command = "registers".to_string();
    app.run().unwrap();
    let resp = String::from("0,0,0,0,5");
    assert_eq!(resp, app.server.response.borrow().1);

    app.server.command = "step".to_string();
    app.run().unwrap();
    app.server.command = "registers".to_string();
    app.run().unwrap();
    let resp = String::from("0,0,0,1,5");
    assert_eq!(resp, app.server.response.borrow().1);

    app.server.command = "step".to_string();
    app.run().unwrap();
    app.server.command = "registers".to_string();
    app.run().unwrap();
    let resp = String::from("0,0,0,1,5,0,1");
    assert_eq!(resp, app.server.response.borrow().1);

    app.server.command = "step".to_string();
    app.run().unwrap();
    app.server.command = "registers".to_string();
    app.run().unwrap();
    let resp = String::from("0,0,0,1,5,1,1");
    assert_eq!(resp, app.server.response.borrow().1);

    app.server.command = "step".to_string();
    app.run().unwrap();
    app.server.command = "registers".to_string();
    app.run().unwrap();
    let resp = String::from("0,0,0,1,5,1,1");
    assert_eq!(resp, app.server.response.borrow().1);

    app.server.command = "step".to_string();
    app.run().unwrap();
    app.run().unwrap();
    app.server.command = "registers".to_string();
    app.run().unwrap();
    let resp = String::from("Terminate");
    assert_eq!(resp, app.server.response.borrow().1);
}
