struct Trigger {
    x: u32,
    y: Field,
    z: [Field; 3],
}
struct ResultType {
    a: u32,
    b: Field,
    c: [Field; 3],
}

unconstrained fn convert(trigger: Trigger) -> ResultType {
    let result = ResultType {
        a: trigger.x + 1,
        b: trigger.y - 1 + trigger.z[2],
        c: [trigger.z[0], 0, trigger.z[1]],
    };
    result
}
impl Trigger {
    fn execute(self) -> ResultType {
        /// Safety: testing context
        let result = unsafe { convert(self) };
        assert(result.a == self.x + 1);
        assert(result.b == self.y - 1 + self.z[2]);
        assert(result.c[1] == 0);
        result
    }
}
fn main(x: Trigger) -> pub ResultType {
    x.execute()
}
