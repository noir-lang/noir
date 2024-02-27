fn main(x: pub u8, y: u8) {}

fn main(x: pub u8, y: u8) -> pub field {}

fn main<A, B>(x: A, y: B) -> pub field where A: Eq, B: Eq {}

fn main()
// hello
{}

fn main(
    // hello
) {}

fn main(
    // hello
    unit: ()
) {}

fn main<T>() where T: Eq {}

fn main(
    tape: [field; TAPE_LEN],
    initial_registers: [field; REGISTER_COUNT],
    initial_memory: [field; MEM_COUNT],
    initial_program_counter: field,
    initial_call_stack: [field; MAX_CALL_STACK],
    initial_call_stack_pointer: u64
) -> pub ExecutionResult {}

fn apply_binary_field_op<N>(
    lhs: RegisterIndex,
    rhs: RegisterIndex,
    result: RegisterIndex,
    op: u8,
    registers: &mut Registers<N>
) -> bool {}

fn main() -> distinct pub [field; 2] {}

fn ret_normal_lambda1() -> ((fn() -> field)) {}

fn ret_normal_lambda1() -> fn() -> field {}

fn ret_closure1() -> fn[(field,)]() -> field {}

fn ret_closure2() -> fn[(field, field)]() -> field {}

fn ret_closure3() -> fn[(u32, u64)]() -> u64 {}

fn make_counter() -> fn[(&mut field,)]() -> field {}

fn get_some<Env>(generator: fn[Env]() -> field) -> [field; 5] {}

fn main(
    message: [u8; 10],
    message_field: field,
    pub_key_x: field,
    pub_key_y: field,
    signature: [u8; 64]
) {}

pub fn from_baz(x: [field; crate::foo::MAGIC_NUMBER]) {}
