pub struct CRS {
    pub g1_data: Vec<u8>,
    pub g2_data: Vec<u8>,
    pub num_points: usize,
}

const G1_START: usize = 28;
const G2_START: usize = 28 + (5_040_000 * 64);
const G2_END: usize = G2_START + 128 - 1;

impl CRS {
    pub fn new(num_points: usize) -> CRS {
        let g1_end = G1_START + (num_points * 64) - 1;
        let crs = read_crs();

        let g1_data = crs[G1_START..=g1_end].to_vec();
        let g2_data = crs[G2_START..=G2_END].to_vec();

        CRS {
            g1_data,
            g2_data,
            num_points,
        }
    }
}

fn read_crs() -> Vec<u8> {
    match std::fs::read("ignition/transcript00.dat") {
        Ok(bytes) => bytes,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                eprintln!("please run again with appropriate permissions.");
            }
            panic!("Could not find file transcript00.dat at location ignition/transcript00.dat.");
                    }
    }
}

#[test]
fn does_not_panic() {
    use super::Barretenberg;
    use wasmer_runtime::Value;

    let mut barretenberg = Barretenberg::new();

    let num_points = 4 * 1024;

    let crs = CRS::new(num_points);

    let crs_ptr = barretenberg.allocate(&crs.g1_data);

    let pippenger_ptr = barretenberg.call_multiple(
        "new_pippenger",
        vec![&crs_ptr, &Value::I32(num_points as i32)],
    );
    barretenberg.free(crs_ptr);

    let scalars = vec![0; num_points * 32];
    let mem = barretenberg.allocate(&scalars);
    barretenberg.free(mem);
}
