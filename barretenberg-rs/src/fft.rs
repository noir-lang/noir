use super::Barretenberg;
use wasmer_runtime::Value;

pub struct FFT {
    domain_ptr: Value,
}

impl FFT {
    pub fn new(barretenberg: &mut Barretenberg, circuit_size: usize) -> FFT {
        let domain_ptr = barretenberg
            .call("new_evaluation_domain", &Value::I32(circuit_size as i32))
            .value();

        FFT { domain_ptr }
    }

    pub fn destroy(&mut self, barretenberg: &mut Barretenberg) {
        barretenberg.call("delete_evaluation_domain", &self.domain_ptr);
    }

    pub fn fft(
        &mut self,
        barretenberg: &mut Barretenberg,
        coefficients: &[u8],
        constant: &[u8],
    ) -> Vec<u8> {
        let circuit_size = coefficients.len() / 32;
        let new_ptr = barretenberg.allocate(coefficients);
        barretenberg.transfer_to_heap(constant, 0);

        barretenberg.call_multiple(
            "coset_fft_with_generator_shift",
            vec![&new_ptr, &Value::I32(0), &self.domain_ptr],
        );
        let result = barretenberg.slice_memory(
            new_ptr.to_u128() as usize,
            (new_ptr.to_u128() as usize) + circuit_size * 32,
        );
        barretenberg.free(new_ptr);
        return result;
    }

    pub fn ifft(&mut self, barretenberg: &mut Barretenberg, coefficients: &[u8]) -> Vec<u8> {
        let circuit_size = coefficients.len() / 32;
        let new_ptr = barretenberg.allocate(coefficients);

        barretenberg.call_multiple("ifft", vec![&new_ptr, &self.domain_ptr]);
        let result = barretenberg.slice_memory(
            new_ptr.to_u128() as usize,
            (new_ptr.to_u128() as usize) + circuit_size * 32,
        );
        barretenberg.free(new_ptr);
        return result;
    }
}
