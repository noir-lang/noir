// Capture the "attempt to subtract with overflow" from https://github.com/AztecProtocol/aztec-packages/pull/11294

pub global MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX: u32 = 8;

unconstrained fn main(
    previous_kernel_public_inputs: PrivateKernelCircuitPublicInputs,
) -> pub PrivateKernelCircuitPublicInputs {
    let private_inputs = PrivateKernelInnerCircuitPrivateInputs::new(previous_kernel_public_inputs);
    private_inputs.execute()
}

pub struct PrivateKernelCircuitPublicInputs {
    pub end: PrivateAccumulatedData,
}

pub struct PrivateKernelData {
    pub public_inputs: PrivateKernelCircuitPublicInputs,
}

pub struct PrivateAccumulatedData {
    pub private_call_stack: [PrivateCallRequest; MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX],
}

pub struct PrivateCallRequest {
    pub args_hash: Field,
    pub returns_hash: Field,
    pub start_side_effect_counter: u32,
    pub end_side_effect_counter: u32,
}

pub struct PrivateKernelCircuitPublicInputsComposer {
    pub public_inputs: PrivateKernelCircuitPublicInputsBuilder,
}

impl PrivateKernelCircuitPublicInputsComposer {
    pub unconstrained fn new_from_previous_kernel(
        previous_kernel_public_inputs: PrivateKernelCircuitPublicInputs,
    ) -> Self {
        let mut public_inputs = PrivateKernelCircuitPublicInputsBuilder {
            end: PrivateAccumulatedDataBuilder { private_call_stack: BoundedVec::new() },
        };

        let start = previous_kernel_public_inputs.end;
        public_inputs.end.private_call_stack = array_to_bounded_vec(start.private_call_stack);

        PrivateKernelCircuitPublicInputsComposer { public_inputs }
    }

    pub fn pop_top_call_request(&mut self) -> Self {
        // Pop the top item in the call stack, which is the caller of the current call, and shouldn't be propagated to the output.
        let _call_request = self.public_inputs.end.private_call_stack.pop();
        *self
    }

    pub fn finish(self) -> PrivateKernelCircuitPublicInputs {
        self.public_inputs.finish()
    }
}

pub struct PrivateKernelCircuitPublicInputsBuilder {
    pub end: PrivateAccumulatedDataBuilder,
}

impl PrivateKernelCircuitPublicInputsBuilder {
    pub fn finish(self) -> PrivateKernelCircuitPublicInputs {
        PrivateKernelCircuitPublicInputs { end: self.end.finish() }
    }
}

pub struct PrivateAccumulatedDataBuilder {
    pub private_call_stack: BoundedVec<PrivateCallRequest, MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX>,
}

impl PrivateAccumulatedDataBuilder {
    pub fn finish(self) -> PrivateAccumulatedData {
        PrivateAccumulatedData { private_call_stack: self.private_call_stack.storage() }
    }
}

pub struct PrivateKernelInnerCircuitPrivateInputs {
    previous_kernel: PrivateKernelData,
}

impl PrivateKernelInnerCircuitPrivateInputs {
    pub fn new(public_inputs: PrivateKernelCircuitPublicInputs) -> Self {
        Self { previous_kernel: PrivateKernelData { public_inputs } }
    }

    unconstrained fn generate_output(self) -> PrivateKernelCircuitPublicInputs {
        // XXX: Declaring `let mut composer = ` would make the circuit pass.
        PrivateKernelCircuitPublicInputsComposer::new_from_previous_kernel(
            self.previous_kernel.public_inputs,
        )
            .pop_top_call_request()
            .finish()
    }

    pub fn execute(self) -> PrivateKernelCircuitPublicInputs {
        // XXX: Running both this and the bottom assertion would make the circuit pass.
        // assert(!is_empty(self.previous_kernel.public_inputs.end.private_call_stack[0]), "not empty before");

        // Safety: This is where the program treated the input as mutable.
        let output = unsafe { self.generate_output() };

        assert(
            !is_empty(self.previous_kernel.public_inputs.end.private_call_stack[0]),
            "not empty after",
        );

        output
    }
}

pub trait Empty {
    fn empty() -> Self;
}

pub fn is_empty<T>(item: T) -> bool
where
    T: Empty + Eq,
{
    item.eq(T::empty())
}

impl Eq for PrivateCallRequest {
    fn eq(self, other: PrivateCallRequest) -> bool {
        (self.args_hash == other.args_hash)
            & (self.returns_hash == other.returns_hash)
            & (self.start_side_effect_counter == other.start_side_effect_counter)
            & (self.end_side_effect_counter == other.end_side_effect_counter)
    }
}

impl Empty for PrivateCallRequest {
    fn empty() -> Self {
        PrivateCallRequest {
            args_hash: 0,
            returns_hash: 0,
            start_side_effect_counter: 0,
            end_side_effect_counter: 0,
        }
    }
}

// Copy of https://github.com/AztecProtocol/aztec-packages/blob/f1fd2d104d01a4582d8a48a6ab003d8791010967/noir-projects/noir-protocol-circuits/crates/types/src/utils/arrays.nr#L110
pub fn array_length<T, let N: u32>(array: [T; N]) -> u32
where
    T: Empty + Eq,
{
    // We get the length by checking the index of the first empty element.

    // Safety: This is safe because we have validated the array (see function doc above) and the emptiness
    // of the element and non-emptiness of the previous element is checked below.
    let length = unsafe { find_index_hint(array, |elem: T| is_empty(elem)) };
    // if length != 0 {
    //     assert(!is_empty(array[length - 1]));
    // }
    // if length != N {
    //     assert(is_empty(array[length]));
    // }
    length
}

// Helper function to find the index of the first element in an array that satisfies a given predicate. If the element
// is not found, the function returns N as the index.
pub unconstrained fn find_index_hint<T, let N: u32, Env>(
    array: [T; N],
    find: fn[Env](T) -> bool,
) -> u32 {
    let mut index = N;
    for i in 0..N {
        // We check `index == N` to ensure that we only update the index if we haven't found a match yet.
        if (index == N) & find(array[i]) {
            index = i;
        }
    }
    index
}

pub unconstrained fn array_to_bounded_vec<T, let N: u32>(array: [T; N]) -> BoundedVec<T, N>
where
    T: Empty + Eq,
{
    let len = array_length(array);
    BoundedVec::from_parts_unchecked(array, len)
}
