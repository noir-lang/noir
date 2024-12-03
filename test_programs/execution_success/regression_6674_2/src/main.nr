use std::mem::zeroed;

pub struct BoundedVec4 {
    storage: [Field; 4],
    len: u32,
}

impl BoundedVec4 {
    pub fn new() -> Self {
        BoundedVec4 { storage: [0; 4], len: 0 }
    }

    pub fn push(&mut self, elem: Field) {
        self.storage[self.len] = elem;
        self.len += 1;
    }
}

pub struct PrivateKernelCircuitPublicInputs {
    pub l2_to_l1_msgs: [Field; 4],
    pub public_call_requests: [Field; 4],
}

pub struct FixtureBuilder {
    pub public_call_requests: BoundedVec4,
    pub counter: Field,
}

impl FixtureBuilder {
    pub fn new() -> Self {
        FixtureBuilder { public_call_requests: zeroed(), counter: 0 }
    }

    pub fn append_public_call_requests(&mut self, num: u32) {
        for _ in 0..num {
            // Note that here we push directly, not through a method call
            self.public_call_requests.push(self.next_counter());
        }
    }

    fn next_counter(&mut self) -> Field {
        let counter = self.counter;
        self.counter += 1;
        counter
    }
}

pub struct PrivateAccumulatedDataBuilder {
    pub l2_to_l1_msgs: BoundedVec4,
    pub public_call_requests: BoundedVec4,
}

pub struct PrivateKernelCircuitPublicInputsBuilder {
    end: PrivateAccumulatedDataBuilder,
}

pub struct PrivateKernelCircuitPublicInputsComposer {
    public_inputs: PrivateKernelCircuitPublicInputsBuilder,
}

impl PrivateKernelCircuitPublicInputsComposer {
    pub fn new_from_previous_kernel(
        previous_kernel_public_inputs: PrivateKernelCircuitPublicInputs,
    ) -> Self {
        let mut public_inputs = PrivateKernelCircuitPublicInputsBuilder {
            end: PrivateAccumulatedDataBuilder {
                l2_to_l1_msgs: BoundedVec4::new(),
                public_call_requests: BoundedVec4::new(),
            },
        };

        let start = previous_kernel_public_inputs;
        public_inputs.end.public_call_requests = BoundedVec4 {
            storage: start.public_call_requests,
            len: start.public_call_requests.len(),
        };

        PrivateKernelCircuitPublicInputsComposer { public_inputs }
    }

    pub unconstrained fn sort_ordered_values(&mut self) {
        self.public_inputs.end.l2_to_l1_msgs.storage = sort_by(
            self.public_inputs.end.l2_to_l1_msgs.storage,
            |a: Field, _b: Field| a != 0,
        );

        self.public_inputs.end.public_call_requests.storage = sort_by(
            self.public_inputs.end.public_call_requests.storage,
            |a: Field, _b: Field| a != 0,
        );
    }
}

pub unconstrained fn sort_by<Env>(
    array: [Field; 4],
    ordering: fn[Env](Field, Field) -> bool,
) -> [Field; 4] {
    let mut result = array;
    let sorted_index = unsafe { get_sorting_index(array, ordering) };
    for i in 0..4 {
        result[i] = array[sorted_index[i]];
    }
    result
}

unconstrained fn get_sorting_index<Env>(
    array: [Field; 4],
    ordering: fn[Env](Field, Field) -> bool,
) -> [u32; 4] {
    let mut result = [0; 4];
    let mut a = array;
    for i in 0..4 {
        result[i] = i;
    }
    for i in 1..4 {
        for j in 0..i {
            if ordering(a[i], a[j]) {
                let old_a_j = a[j];
                a[j] = a[i];
                a[i] = old_a_j;
                let old_j = result[j];
                result[j] = result[i];
                result[i] = old_j;
            }
        }
    }
    result
}

unconstrained fn main() {
    let mut previous_kernel = FixtureBuilder::new();
    previous_kernel.append_public_call_requests(4);

    let public_inputs = PrivateKernelCircuitPublicInputs {
        l2_to_l1_msgs: [0; 4],
        public_call_requests: previous_kernel.public_call_requests.storage,
    };

    let mut output_composer =
        PrivateKernelCircuitPublicInputsComposer::new_from_previous_kernel(public_inputs);
    output_composer.sort_ordered_values();

    assert_eq(public_inputs.public_call_requests[3], 3, "equality");
}
