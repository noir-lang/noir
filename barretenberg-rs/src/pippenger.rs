use super::Barretenberg;
use wasmer_runtime::Value;

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

    pub fn destroy(&mut self, barretenberg: &mut Barretenberg) {
        barretenberg.call("delete_pippenger", &self.pippenger_ptr);
    }

    pub fn pippenger_unsafe(
        &mut self,
        barretenberg: &mut Barretenberg,
        scalars: &[u8],
        from: usize,
        range: usize,
    ) -> Vec<u8> {
        let mem = barretenberg.allocate(scalars);
        barretenberg.call_multiple(
            "pippenger_unsafe",
            vec![
                &self.pippenger_ptr,
                &mem,
                &Value::I32(from as i32),
                &Value::I32(range as i32),
                &Value::I32(0),
            ],
        );
        barretenberg.free(mem);
        barretenberg.slice_memory(0, 96)
    }

    pub fn sum_elements(&mut self, barretenberg: &mut Barretenberg, buffer: &[u8]) -> Vec<u8> {
        let buffer_len = buffer.len();
        let mem = barretenberg.allocate(buffer);

        barretenberg.call_multiple(
            "g1_sum",
            vec![&mem, &Value::I32((buffer_len / 96) as i32), &Value::I32(0)],
        );
        barretenberg.free(mem);
        return barretenberg.slice_memory(0, 96);
    }
}
