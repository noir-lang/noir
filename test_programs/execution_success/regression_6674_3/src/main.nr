use std::mem::zeroed;

pub struct PrivateAccumulatedData {
    pub public_call_requests: [Counted<Field>; 4],
}

pub struct PrivateAccumulatedDataBuilder {
    pub l2_to_l1_msgs: BoundedVec<Field, 4>,
    pub public_call_requests: BoundedVec<Counted<Field>, 4>,
    pub private_call_stack: BoundedVec<Field, 4>,
}

impl PrivateAccumulatedDataBuilder {
    pub fn finish(self) -> PrivateAccumulatedData {
        PrivateAccumulatedData { public_call_requests: self.public_call_requests.storage() }
    }
}

pub struct Counted<T> {
    pub inner: T,
    pub counter: u32,
}

impl<T> Counted<T> {
    pub fn new(inner: T, counter: u32) -> Self {
        Self { inner, counter }
    }
}

pub struct PrivateKernelCircuitPublicInputs {
    pub end: PrivateAccumulatedData,
}

pub struct PrivateKernelData {
    pub public_inputs: PrivateKernelCircuitPublicInputs,
}

pub struct FixtureBuilder2 {
    pub public_teardown_call_request: Field,
    pub private_call_requests: BoundedVec<Field, 4>,
    pub public_call_requests: BoundedVec<Counted<Field>, 4>,
    pub counter: u32,
}

impl FixtureBuilder2 {
    pub fn new() -> Self {
        let mut builder: FixtureBuilder2 = zeroed();
        builder.counter = 1;
        builder
    }

    pub fn to_private_accumulated_data_builder(self) -> PrivateAccumulatedDataBuilder {
        PrivateAccumulatedDataBuilder {
            l2_to_l1_msgs: zeroed(),
            public_call_requests: self.public_call_requests,
            private_call_stack: vec_reverse(self.private_call_requests),
        }
    }

    pub fn to_private_accumulated_data(self) -> PrivateAccumulatedData {
        self.to_private_accumulated_data_builder().finish()
    }

    pub fn to_private_kernel_circuit_public_inputs(self) -> PrivateKernelCircuitPublicInputs {
        PrivateKernelCircuitPublicInputs { end: self.to_private_accumulated_data() }
    }

    pub fn to_private_kernel_data(self) -> PrivateKernelData {
        let public_inputs =
            PrivateKernelCircuitPublicInputs { end: self.to_private_accumulated_data() };
        PrivateKernelData { public_inputs }
    }

    pub fn add_public_call_request(&mut self) {
        self.public_call_requests.push(Counted::new(zeroed(), self.next_counter()));
    }

    pub fn append_public_call_requests(&mut self, num: u32) {
        for _ in 0..num {
            self.add_public_call_request();
        }
    }

    pub fn set_public_teardown_call_request(&mut self) {
        let mut fields = [0; 5];
        for i in 0..5 {
            fields[i] = i as Field;
        }

        self.public_teardown_call_request = zeroed();
    }

    fn next_counter(&mut self) -> u32 {
        let counter = self.counter;
        self.counter += 1;
        counter
    }
}

struct PrivateKernelTailToPublicInputsBuilder {
    previous_kernel: FixtureBuilder2,
}

impl PrivateKernelTailToPublicInputsBuilder {
    pub unconstrained fn execute(&mut self) {
        let kernel = PrivateKernelTailToPublicCircuitPrivateInputs {
            previous_kernel: self.previous_kernel.to_private_kernel_data(),
        };
        let mut output_composer = PrivateKernelCircuitPublicInputsComposer::new_from_previous_kernel(
            kernel.previous_kernel.public_inputs,
        );
        output_composer.sort_ordered_values();
    }
}

pub struct PrivateKernelTailToPublicCircuitPrivateInputs {
    previous_kernel: PrivateKernelData,
}

pub struct PrivateKernelCircuitPublicInputsComposer {
    public_inputs: PrivateKernelCircuitPublicInputsBuilder,
}

impl PrivateKernelCircuitPublicInputsComposer {
    pub unconstrained fn sort_ordered_values(&mut self) {
        // Note hashes, nullifiers, and private logs are sorted in the reset circuit.
        self.public_inputs.end.l2_to_l1_msgs.storage =
            sort_by_counter_desc(self.public_inputs.end.l2_to_l1_msgs.storage);
        self.public_inputs.end.public_call_requests.storage =
            sort_by_counter_desc(self.public_inputs.end.public_call_requests.storage);
    }
}

impl PrivateKernelCircuitPublicInputsComposer {
    pub fn new_from_previous_kernel(
        previous_kernel_public_inputs: PrivateKernelCircuitPublicInputs,
    ) -> Self {
        let mut public_inputs: PrivateKernelCircuitPublicInputsBuilder = zeroed();
        let start = previous_kernel_public_inputs.end;
        public_inputs.end.public_call_requests = BoundedVec {
            storage: start.public_call_requests,
            len: start.public_call_requests.len(),
        };
        PrivateKernelCircuitPublicInputsComposer { public_inputs }
    }
}

pub struct PrivateKernelCircuitPublicInputsBuilder {
    end: PrivateAccumulatedDataBuilder,
}

fn vec_reverse<T, let N: u32>(vec: BoundedVec<T, N>) -> BoundedVec<T, N> {
    let mut reversed = BoundedVec::new();
    let len = vec.len();
    for i in 0..N {
        if i < len {
            reversed.push(vec.get_unchecked(len - i - 1));
        }
    }
    reversed
}

pub unconstrained fn sort_by_counter_desc<T, let N: u32>(array: [T; N]) -> [T; N] {
    sort_by(array)
}

pub unconstrained fn sort_by<T, let N: u32>(array: [T; N]) -> [T; N] {
    let mut result = array;
    unsafe {
        //@safety: testing context
        get_sorting_index(array)
    };
    result
}

unconstrained fn get_sorting_index<T, let N: u32>(array: [T; N]) {
    let _ = [0; 4];
    let mut a = array;
    for i in 1..4 {
        for j in 0..i {
            a[i] = a[j];
        }
    }
}

unconstrained fn main() {
    let mut previous_kernel = FixtureBuilder2::new();
    let mut builder = PrivateKernelTailToPublicInputsBuilder { previous_kernel };
    builder.previous_kernel.append_public_call_requests(4);
    assert_eq(builder.previous_kernel.public_call_requests.storage[3].counter, 4);
    builder.previous_kernel.set_public_teardown_call_request();
    builder.execute();
    assert_eq(builder.previous_kernel.public_call_requests.storage[3].counter, 4);
}
