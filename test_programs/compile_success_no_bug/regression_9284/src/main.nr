fn main() {
    let msg: [_; 3] = std::mem::zeroed();
    // At this point `msg` still has unbounded type variables.
    assert(msg.len() == 3, msg);
    let _ = msg[0] + 1;
}
