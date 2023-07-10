use acvm::acir::circuit::Circuit;
use noirc_abi::Abi;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{fs::File, io::Write, path::Path};

use flate2::write::GzEncoder;
use flate2::Compression;

mod barretenberg_structures;
use barretenberg_structures::ConstraintSystem;

pub fn main() {
    let path_string = std::env::args()
        .nth(1)
        .unwrap_or("./target/main.json".to_owned());
    let circuit_path = Path::new(&path_string);

    let circuit_bytes = std::fs::read(&circuit_path).unwrap();

    let program: PreprocessedProgram =
        serde_json::from_slice(&circuit_bytes).expect("could not deserialize program");

    write_to_file(&serde_json::to_vec(&program).unwrap(), &circuit_path);
}

fn write_to_file(bytes: &[u8], path: &Path) -> String {
    let display = path.display();

    let mut file = match File::create(path) {
        Err(why) => panic!("couldn't create {display}: {why}"),
        Ok(file) => file,
    };

    match file.write_all(bytes) {
        Err(why) => panic!("couldn't write to {display}: {why}"),
        Ok(_) => display.to_string(),
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PreprocessedProgram {
    pub backend: String,
    pub abi: Abi,

    #[serde(
        serialize_with = "serialize_circuit",
        deserialize_with = "deserialize_circuit"
    )]
    pub bytecode: Circuit
}

fn serialize_circuit<S>(circuit: &Circuit, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let cs: ConstraintSystem =
        ConstraintSystem::try_from(circuit).expect("should have no malformed bb funcs");
    let circuit_bytes = cs.to_bytes();

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&circuit_bytes).unwrap();
    let compressed_bytes = encoder.finish().unwrap();

    let b64_string = base64::encode(compressed_bytes);
    s.serialize_str(&b64_string)
}

fn deserialize_circuit<'de, D>(deserializer: D) -> Result<Circuit, D::Error>
where
    D: Deserializer<'de>,
{
    let circuit_bytes = Vec::<u8>::deserialize(deserializer)?;
    let circuit = Circuit::read(&*circuit_bytes).unwrap();
    Ok(circuit)
}
