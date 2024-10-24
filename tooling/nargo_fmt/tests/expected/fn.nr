fn main(x: pub u8, y: u8) {}

fn main(x: pub u8, y: u8) -> pub Field {}

fn main<A, B>(x: A, y: B) -> pub Field
where
    A: Eq,
    B: Eq,
{}

fn main()
// hello
{}

fn main(
    // hello
) {}

fn main(
    // hello
    unit: (),
) {}

fn main<T>()
where
    T: Eq,
{}

fn main(
    tape: [Field; TAPE_LEN],
    initial_registers: [Field; REGISTER_COUNT],
    initial_memory: [Field; MEM_COUNT],
    initial_program_counter: Field,
    initial_call_stack: [Field; MAX_CALL_STACK],
    initial_call_stack_pointer: u64,
) -> pub ExecutionResult {}

fn apply_binary_field_op<N>(
    lhs: RegisterIndex,
    rhs: RegisterIndex,
    result: RegisterIndex,
    op: u8,
    registers: &mut Registers<N>,
) -> bool {}

fn main() -> pub [Field; 2] {}

fn ret_normal_lambda1() -> ((fn() -> Field)) {}

fn ret_normal_lambda1() -> fn() -> Field {}

fn ret_closure1() -> fn[(Field,)]() -> Field {}

fn ret_closure2() -> fn[(Field, Field)]() -> Field {}

fn ret_closure3() -> fn[(u32, u64)]() -> u64 {}

fn make_counter() -> fn[(&mut Field,)]() -> Field {}

fn get_some<Env>(generator: fn[Env]() -> Field) -> [Field; 5] {}

fn main(
    message: [u8; 10],
    message_field: Field,
    pub_key_x: Field,
    pub_key_y: Field,
    signature: [u8; 64],
) {}

pub fn from_baz(x: [Field; crate::foo::MAGIC_NUMBER]) {}

fn id<T, let I: Field>(x: [Field; I]) -> [Field; I] {}

fn id_two<T, let I: Field>(x: [Field; I]) -> [Field; I] {}

fn whitespace_before_generics<T>(foo: T) {}

fn more_whitespace_before_generics<T>(foo: T) {}

fn with_unconstrained(x: unconstrained fn() -> ()) {}

pub(crate) fn one() {}

pub unconstrained fn two() {}

pub unconstrained comptime fn two() {}

/// Documented
#[test]
fn three() {}

#[test(should_fail)]
// comment
fn four() {}

#[test(should_fail_with = "oops")]
fn five() {}

