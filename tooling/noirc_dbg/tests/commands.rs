mod mock_server;
use mock_server::MockDap;
use noirc_dbg::app::{App, State};

#[test]
fn check_registers() {
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

#[test]
fn check_registers_and_memory() {
    let dap = MockDap::new("value_in_registers_and_memory");
    let mut app = App::initialize(dap);

    // registers on init step
    app.server.command = "launch".to_string();
    app.run().unwrap();
    app.server.command = "registers".to_string();
    app.run().unwrap();
    let resp = String::from("2");
    assert_eq!(resp, app.server.response.borrow().1);
    app.server.command = "memory".to_string();
    app.run().unwrap();
    let resp = String::from("ReadMemory(ReadMemoryResponse { address: \"Memory\", unreadable_bytes: None, data: Some(\"0, 0, 0, 0, 1, 2\") })");
    assert_eq!(resp, app.server.response.borrow().1);
}

#[test]
fn exit_from_uninitialized() {
    let dap = MockDap::new("simple");
    let mut app = App::initialize(dap);

    app.server.command = "quit".to_string();
    let r = app.run();
    let resp = String::from("Disconnect");
    assert_eq!(resp, app.server.response.borrow().1);
    assert!(r.is_ok());
    match app.state {
        State::Exit => assert_eq!(true, true),
        _ => assert_eq!(false, true),
    }
}

#[test]
fn exit_from_running() {
    let dap = MockDap::new("simple");
    let mut app = App::initialize(dap);

    app.server.command = "launch".to_string();
    let r = app.run();
    let resp = String::from("Stopped(StoppedEventBody { reason: Entry, description: Some(\"Entry\"), thread_id: Some(0), preserve_focus_hint: Some(false), text: None, all_threads_stopped: Some(false), hit_breakpoint_ids: None })");
    assert_eq!(resp, app.server.response.borrow().1);
    assert!(r.is_ok());
    match app.state {
        State::Running(ref _s) => assert_eq!(true, true),
        _ => assert_eq!(false, true),
    }

    //
    app.server.command = "quit".to_string();
    let r = app.run();
    let resp = String::from("Disconnect");
    assert_eq!(resp, app.server.response.borrow().1);
    assert!(r.is_ok());
    match app.state {
        State::Exit => assert_eq!(true, true),
        _ => assert_eq!(false, true),
    }
}

#[test]
fn check_dissaseble() {
    let dap = MockDap::new("simple");
    let mut app = App::initialize(dap);

    // registers on init step
    app.server.command = "launch".to_string();
    app.run().unwrap();
    app.server.command = "disassemble".to_string();
    app.run().unwrap();
    let resp = String::from(
        "0  ==>
<<<<<
Mov {
    destination: RegisterIndex(
        2,
    ),
    source: RegisterIndex(
        0,
    ),
}
>>>>>
1  Const {
    destination: RegisterIndex(
        0,
    ),
    value: Value {
        inner: 0,
    },
}
2  Const {
    destination: RegisterIndex(
        1,
    ),
    value: Value {
        inner: 0,
    },
}
3  Call {
    location: 5,
}
4  Stop
5  Const {
    destination: RegisterIndex(
        4,
    ),
    value: Value {
        inner: 5,
    },
}
6  BinaryIntOp {
    destination: RegisterIndex(
        3,
    ),
    op: LessThan,
    bit_size: 64,
    lhs: RegisterIndex(
        2,
    ),
    rhs: RegisterIndex(
        4,
    ),
}
7  Const {
    destination: RegisterIndex(
        6,
    ),
    value: Value {
        inner: 1,
    },
}
8  BinaryIntOp {
    destination: RegisterIndex(
        5,
    ),
    op: Equals,
    bit_size: 1,
    lhs: RegisterIndex(
        3,
    ),
    rhs: RegisterIndex(
        6,
    ),
}
9  JumpIf {
    condition: RegisterIndex(
        5,
    ),
    location: 11,
}
10  Trap
11  Return",
    );
    assert_eq!(resp, app.server.response.borrow().1);
    match app.state {
        State::Running(ref _s) => assert_eq!(true, true),
        _ => assert_eq!(false, true),
    }
}
