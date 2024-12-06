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

    pub fn append_public_call_requests_inner(&mut self) {
        self.public_call_requests.push(self.next_counter());
    }

    pub fn append_public_call_requests(&mut self) {
        for _ in 0..4 {
            // Note that here we push via a method call
            self.append_public_call_requests_inner();
        }
    }

    fn next_counter(&mut self) -> Field {
        let counter = self.counter;
        self.counter += 1;
        counter
    }
}

pub struct PrivateKernelCircuitPublicInputsComposer {
    pub l2_to_l1_msgs: [Field; 4],
    pub public_call_requests: [Field; 4],
}

pub unconstrained fn sort_by(array: [Field; 4]) -> [Field; 4] {
    let result = array;
    get_sorting_index(array);
    result
}

unconstrained fn get_sorting_index(array: [Field; 4]) {
    let _ = [0; 4];
    let mut a = array;
    for i in 1..4 {
        for j in 0..i {
            a[i] = a[j];
        }
    }
}

unconstrained fn main() {
    let mut previous_kernel = FixtureBuilder::new();
    previous_kernel.append_public_call_requests();

    let mut output_composer = PrivateKernelCircuitPublicInputsComposer {
        l2_to_l1_msgs: [0; 4],
        public_call_requests: previous_kernel.public_call_requests.storage,
    };
    output_composer.l2_to_l1_msgs = sort_by(output_composer.l2_to_l1_msgs);
    output_composer.public_call_requests = sort_by(output_composer.public_call_requests);

    assert_eq(previous_kernel.public_call_requests.storage[1], 1, "equality");
}
