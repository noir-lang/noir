mod mock_server;
use mock_server::MockDap;
use noirc_dbg::{
    app::{App, State},
    error::DebuggingError,
};

#[test]
fn provide_wrong_command() {
    let dap = MockDap::new("simple");
    let mut app = App::initialize(dap);

    //

    match app.state {
        State::Uninitialized(ref _s) => assert_eq!(true, true),
        _ => assert_eq!(false, true),
    }

    // check wrong command on initialized state
    app.server.command = "step".to_string();
    let r = app.run();
    let resp = String::from("");
    assert_eq!(resp, app.server.response.borrow().1);
    assert!(r.is_err());
    match r {
        Err(DebuggingError::CustomError(msg)) => {
            assert_eq!(
                msg,
                "Invalid request. You need to initialize and launch program.".to_string()
            )
        }
        _ => assert_eq!(true, false),
    }
    match app.state {
        State::Uninitialized(ref _s) => assert_eq!(true, true),
        _ => assert_eq!(false, true),
    }

    // launch
    app.server.command = "launch".to_string();
    let r = app.run();
    let resp = String::from("Stopped(StoppedEventBody { reason: Entry, description: Some(\"Entry\"), thread_id: Some(0), preserve_focus_hint: Some(false), text: None, all_threads_stopped: Some(false), hit_breakpoint_ids: None })");
    assert_eq!(resp, app.server.response.borrow().1);
    assert!(r.is_ok());
    match app.state {
        State::Running(ref _s) => assert_eq!(true, true),
        _ => assert_eq!(false, true),
    }

    // perform unsupported command in running state
    let r = app.run();
    let resp = String::from("Stopped(StoppedEventBody { reason: Entry, description: Some(\"Entry\"), thread_id: Some(0), preserve_focus_hint: Some(false), text: None, all_threads_stopped: Some(false), hit_breakpoint_ids: None })");
    assert_eq!(resp, app.server.response.borrow().1);
    assert!(r.is_err());
    match r {
        Err(DebuggingError::CustomError(msg)) => {
            assert_eq!(
                msg,
                "Unsupported command. Please, use the other commands to continue process."
                    .to_string()
            )
        }
        _ => assert_eq!(true, false),
    }
    match app.state {
        State::Running(ref _s) => assert_eq!(true, true),
        _ => assert_eq!(false, true),
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
fn failing() {
    let dap = MockDap::new("failing");
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
    let resp = String::from("Continue(ContinueResponse { all_threads_continued: Some(true) })");
    assert_eq!(resp, app.server.response.borrow().1);
    assert!(r.is_err());
    match r {
        Err(DebuggingError::CustomError(msg)) => {
            assert_eq!(msg, "explicit trap hit in brillig".to_string())
        }
        _ => assert_eq!(true, false),
    }
    match app.state {
        State::Running(ref _s) => assert_eq!(true, true),
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
fn value_in_registers_and_memory_with_success() {
    let dap = MockDap::new("value_in_registers_and_memory");
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
