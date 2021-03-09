use super::Barretenberg;
use wasmer::Value;

pub struct Pippenger {
    pippenger_ptr: Value,
}

impl Pippenger {
    pub fn new(crs_data: &[u8], barretenberg: &mut Barretenberg) -> Pippenger {
        let num_points = Value::I32((crs_data.len() / 64) as i32);

        let crs_ptr = barretenberg.allocate(crs_data);

        let pippenger_ptr = barretenberg
            .call_multiple("new_pippenger", vec![&crs_ptr, &num_points])
            .value();

        barretenberg.free(crs_ptr);

        Pippenger { pippenger_ptr }
    }

    pub fn pointer(&self) -> Value {
        self.pippenger_ptr.clone()
    }
}
