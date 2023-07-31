fn main() {
    let _a = ();
    let _b: () = _a;
    let _c: () = ();
    let _d = f1();
    let _e: () = f2();
    let _f: () = f3();
    let _g = f4();
}

fn f1() {}
fn f2() { () }
fn f3() -> () {}
fn f4() -> () { () }
