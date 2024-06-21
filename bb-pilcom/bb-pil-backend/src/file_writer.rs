use std::fs::File;
use std::io::Write;

pub struct BBFiles {
    // Relative paths
    pub file_name: String,
    pub base: String,
    pub rel: String,
    pub circuit: String,
    pub flavor: String,
    pub composer: String,
    pub prover: String, // path for both prover and verifier files
}

impl BBFiles {
    pub fn default(file_name: String) -> Self {
        Self::new(file_name, None, None, None, None, None, None)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        file_name: String,
        base: Option<String>,
        rel: Option<String>,
        circuit: Option<String>,
        flavor: Option<String>,
        composer: Option<String>,
        prover: Option<String>,
    ) -> Self {
        let base = base.unwrap_or("src/barretenberg".to_owned());
        let rel = rel.unwrap_or("relations/generated".to_owned());
        let circuit = circuit.unwrap_or("vm/generated".to_owned());
        let flavor = flavor.unwrap_or("vm/generated".to_owned());
        let composer = composer.unwrap_or("vm/generated".to_owned());
        let prover = prover.unwrap_or("vm/generated".to_owned());

        Self {
            file_name,

            base,
            rel,
            circuit,
            flavor,
            composer,
            prover,
        }
    }

    pub fn write_file(&self, folder: &str, filename: &str, contents: &String) {
        // attempt to create dir
        let base_path = format!("{}/{}", self.base, folder);
        let _ = std::fs::create_dir_all(&base_path);

        let joined = format!("{}/{}", base_path, filename);
        println!("Writing to: {}", joined);
        let mut file = File::create(joined).unwrap();
        file.write_all(contents.as_bytes()).unwrap();
    }
}
