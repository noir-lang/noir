struct ExampleEvent0 {
    value0: Field,
    value1: Field,
}

trait EventInterface {
    fn emit<Env>(self, _emit: fn[Env](Self) -> ());
}

impl EventInterface for ExampleEvent0 {
    fn emit<Env>(self: ExampleEvent0, _emit: fn[Env](Self) -> ()) {
        _emit(self);
    }
}

struct Context {
    a: u32,
    b: [u32; 3],
    log_hashes: BoundedVec<LogHash, 4>,
}

struct LogHash {
    value: Field,
    counter: u32,
    length: Field,
    randomness: Field,
}

impl Context {
    fn emit_raw_log<let M: u32>(&mut self, randomness: Field, _log: [u8; M], log_hash: Field) {
        let log_hash = LogHash { value: log_hash, counter: 0, length: 0, randomness };
        self.log_hashes.push(log_hash);
    }
}

fn compute<Event>(_event: Event) -> ([u8; 5], Field)
where
    Event: EventInterface,
{
    ([0 as u8; 5], 0)
}

fn emit_with_keys<Event, let OB: u32>(
    context: &mut Context,
    randomness: Field,
    event: Event,
    inner_compute: fn(Event) -> ([u8; OB], Field),
)
where
    Event: EventInterface,
{
    let (log, log_hash) = inner_compute(event);
    context.emit_raw_log(randomness, log, log_hash);
}

fn encode_event_with_randomness<Event>(
    context: &mut Context,
    randomness: Field,
) -> fn[(&mut Context, Field)](Event) -> ()
where
    Event: EventInterface,
{
    |e: Event| {
        /// Safety: testing context
        unsafe {
            //@safety: testing context
            func(context.a);
        }
        emit_with_keys(context, randomness, e, compute);
    }
}

unconstrained fn func(input: u32) {
    let mut var = input;
    let ref = &mut &mut var;

    for _ in 0..1 {
        **ref = 2;
    }

    assert(var == 2);
    assert(**ref == 2);
}

// This test aims to allocate a reference which is aliased and only accessed through its alias
// across multiple blocks. We want to guarantee that this allocation is not removed.
fn main(input: [Field; 4], randomness: Field, context_input: u32) {
    let b = [context_input, context_input, context_input];
    let mut context = Context { a: context_input, b, log_hashes: BoundedVec::new() };

    let event0 = ExampleEvent0 { value0: input[0], value1: input[1] };
    event0.emit(encode_event_with_randomness(&mut context, randomness));
}

