mod mock_server;
use mock_server::MockDap;
use noirc_dbg::app::App;

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

// #[test]
// fn check_dissaseble() {
//     let dap = MockDap::new("simple");
//     let mut app = App::initialize(dap);
//
//     // registers on init step
//     app.server.command = "launch".to_string();
//     app.run().unwrap();
//     app.server.command = "disassemble".to_string();
//     app.run().unwrap();
//     let resp = String::from(
//         "Current position: 0
// Program:
// 0  Mov {
//     destination: RegisterIndex(
//         2,
//     ),
//     source: RegisterIndex(
//         0,
//     ),
// }
// 1  Const {
//     destination: RegisterIndex(
//         0,
//     ),
//     value: Value {
//         inner: 0,
//     },
// }
// 2  Const {
//     destination: RegisterIndex(
//         1,
//     ),
//     value: Value {
//         inner: 0,
//     },
// }
// 3  Call {
//     location: 5,
// }
// 4  Stop
// 5  Const {
//     destination: RegisterIndex(
//         4,
//     ),
//     value: Value {
//         inner: 5,
//     },
// }
// 6  BinaryIntOp {
//     destination: RegisterIndex(
//         3,
//     ),
//     op: LessThan,
//     bit_size: 64,
//     lhs: RegisterIndex(
//         2,
//     ),
//     rhs: RegisterIndex(
//         4,
//     ),
// }
// 7  Const {
//     destination: RegisterIndex(
//         6,
//     ),
//     value: Value {
//         inner: 1,
//     },
// }
// 8  BinaryIntOp {
//     destination: RegisterIndex(
//         5,
//     ),
//     op: Equals,
//     bit_size: 1,
//     lhs: RegisterIndex(
//         3,
//     ),
//     rhs: RegisterIndex(
//         6,
//     ),
// }
// 9  JumpIf {
//     condition: RegisterIndex(
//         5,
//     ),
//     location: 11,
// }
// 10  Trap
// 11  Return",
//     );
//     let mut r_l = resp.lines();
//     let rstr = app.server.response.borrow().1.clone();
//     let mut r_l2 = rstr.lines();
//
//     let mut l = r_l.next();
//     while l.is_some() {
//         assert_eq!(l, r_l2.next());
//         l = r_l.next();
//     }
// }
