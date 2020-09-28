use super::{fft::FFT, pippenger::Pippenger, Barretenberg};
use core::convert::TryInto;
use wasmer_runtime::Value;

pub struct Prover {}

impl Prover {
    pub fn create_proof(
        &mut self,
        barretenberg: &mut Barretenberg,
        pippenger: &mut Pippenger,
        fft: &mut FFT,
        prover_ptr: &Value,
    ) -> Vec<u8> {
        let circuit_size = barretenberg
            .call("unrolled_prover_get_circuit_size", prover_ptr)
            .value();

        barretenberg.call("unrolled_prover_execute_preamble_round", prover_ptr);

        self.process_prover_queue(
            barretenberg,
            pippenger,
            fft,
            prover_ptr,
            circuit_size.to_u128() as usize,
        );

        barretenberg.call("unrolled_prover_execute_first_round", prover_ptr);

        self.process_prover_queue(
            barretenberg,
            pippenger,
            fft,
            prover_ptr,
            circuit_size.to_u128() as usize,
        );

        barretenberg.call("unrolled_prover_execute_second_round", prover_ptr);

        self.process_prover_queue(
            barretenberg,
            pippenger,
            fft,
            prover_ptr,
            circuit_size.to_u128() as usize,
        );

        barretenberg.call("unrolled_prover_execute_third_round", prover_ptr);

        self.process_prover_queue(
            barretenberg,
            pippenger,
            fft,
            prover_ptr,
            circuit_size.to_u128() as usize,
        );

        barretenberg.call("unrolled_prover_execute_fourth_round", prover_ptr);

        self.process_prover_queue(
            barretenberg,
            pippenger,
            fft,
            prover_ptr,
            circuit_size.to_u128() as usize,
        );

        barretenberg.call("unrolled_prover_execute_fifth_round", prover_ptr);

        self.process_prover_queue(
            barretenberg,
            pippenger,
            fft,
            prover_ptr,
            circuit_size.to_u128() as usize,
        );
        barretenberg.call("unrolled_prover_execute_sixth_round", prover_ptr);

        self.process_prover_queue(
            barretenberg,
            pippenger,
            fft,
            prover_ptr,
            circuit_size.to_u128() as usize,
        );

        let proof_size = barretenberg
            .call_multiple("unrolled_prover_export_proof", vec![&prover_ptr, &Value::I32(0)])
            .value();
        let proof_ptr = barretenberg.slice_memory(0, 4);
        let proof_ptr = u32::from_le_bytes(proof_ptr[0..4].try_into().unwrap());

        return barretenberg.slice_memory(
            proof_ptr as usize,
            proof_ptr as usize + proof_size.to_u128() as usize,
        );
    }

    fn process_prover_queue(
        &mut self,
        barretenberg: &mut Barretenberg,
        pippenger: &mut Pippenger,
        fft: &mut FFT,
        prover_ptr: &Value,
        circuit_size: usize,
    ) {
        barretenberg.call_multiple(
            "unrolled_prover_get_work_queue_item_info",
            vec![prover_ptr, &Value::I32(0)],
        );

        let job_info = barretenberg.slice_memory(0, 12);
        let scalar_jobs = u32::from_le_bytes(job_info[0..4].try_into().unwrap());
        let fft_jobs = u32::from_le_bytes(job_info[4..8].try_into().unwrap());
        let ifft_jobs = u32::from_le_bytes(job_info[8..12].try_into().unwrap());
        for i in 0..scalar_jobs {
            let scalars_ptr = barretenberg
                .call_multiple(
                    "unrolled_prover_get_scalar_multiplication_data",
                    vec![prover_ptr, &Value::I32(i as i32)],
                )
                .value();
            let scalars = barretenberg.slice_memory(
                scalars_ptr.to_u128() as usize,
                scalars_ptr.to_u128() as usize + circuit_size * 32,
            );
            let result = pippenger.pippenger_unsafe(barretenberg, &scalars, 0, circuit_size);

            barretenberg.transfer_to_heap(&result, 0);
            barretenberg.call_multiple(
                "unrolled_prover_put_scalar_multiplication_data",
                vec![prover_ptr, &Value::I32(0), &Value::I32(i as i32)],
            );
        }

        struct Job {
            coefficients: Vec<u8>,
            constant: Option<Vec<u8>>,
            inverse: bool,
            i: u32,
        }

        let mut jobs = Vec::new();

        for i in 0..fft_jobs {
            let coeffs_ptr = barretenberg
                .call_multiple(
                    "unrolled_prover_get_fft_data",
                    vec![prover_ptr, &Value::I32(0), &Value::I32(i as i32)],
                )
                .value();

            let coefficients = barretenberg.slice_memory(
                coeffs_ptr.to_u128() as usize,
                coeffs_ptr.to_u128() as usize + (circuit_size * 32),
            );
            let constant = barretenberg.slice_memory(0, 32);
            jobs.push(Job {
                coefficients,
                constant: Some(constant),
                inverse: false,
                i,
            });
        }

        for i in 0..ifft_jobs {
            let coeffs_ptr = &barretenberg
                .call_multiple(
                    "unrolled_prover_get_ifft_data",
                    vec![prover_ptr, &Value::I32(i as i32)],
                )
                .value();
            let coefficients = barretenberg.slice_memory(
                coeffs_ptr.to_u128() as usize,
                coeffs_ptr.to_u128() as usize + circuit_size * 32,
            );
            jobs.push(Job {
                coefficients,
                constant: None,
                inverse: true,
                i,
            });
        }

        for job in jobs.into_iter() {
            if job.inverse {
                self.do_ifft(
                    barretenberg,
                    fft,
                    prover_ptr,
                    job.i as usize,
                    &job.coefficients,
                )
            } else {
                self.do_fft(
                    barretenberg,
                    fft,
                    prover_ptr,
                    job.i as usize,
                    &job.coefficients,
                    &job.constant.unwrap(),
                )
            }
        }
    }

    fn do_fft(
        &mut self,
        barretenberg: &mut Barretenberg,
        fft: &mut FFT,
        prover_ptr: &Value,
        i: usize,
        coefficients: &[u8],
        constant: &[u8],
    ) {
        let result = fft.fft(barretenberg, coefficients, constant);
        let result_ptr = barretenberg.allocate(&result);

        barretenberg.call_multiple(
            "unrolled_prover_put_fft_data",
            vec![prover_ptr, &result_ptr, &Value::I32(i as i32)],
        );
        barretenberg.free(result_ptr);
    }

    fn do_ifft(
        &mut self,
        barretenberg: &mut Barretenberg,
        fft: &mut FFT,
        prover_ptr: &Value,
        i: usize,
        coefficients: &[u8],
    ) {
        let result = fft.ifft(barretenberg, coefficients);
        let result_ptr = barretenberg.allocate(&result);

        barretenberg.call_multiple(
            "unrolled_prover_put_ifft_data",
            vec![prover_ptr, &result_ptr, &Value::I32(i as i32)],
        );
        barretenberg.free(result_ptr);
    }
}
