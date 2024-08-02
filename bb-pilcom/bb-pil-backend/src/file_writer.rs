use std::fs::File;
use std::io::Write;
use std::path::Path;

pub struct BBFiles {
    pub vm_name: String,
    pub base_dir: String,
    pub relations: String,
}

impl BBFiles {
    pub fn default(vm_name: &str) -> Self {
        Self::new(vm_name, None, None)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(vm_name: &str, base_dir: Option<&str>, relations: Option<&str>) -> Self {
        let base_dir = base_dir
            .map(|x| x.to_owned())
            .unwrap_or(format!("src/barretenberg/vm/{}/generated", vm_name));
        let relations = relations.unwrap_or("relations").to_owned();

        Self {
            vm_name: vm_name.to_owned(),
            base_dir,
            relations,
        }
    }

    pub fn remove_generated_dir(&self) {
        let path = Path::new(&self.base_dir);
        if path.exists() {
            std::fs::remove_dir_all(path).unwrap();
        }
    }

    pub fn write_file(&self, folder: Option<&str>, filename: &str, contents: &str) {
        // attempt to create dir
        let base_path = Path::new(&self.base_dir).join(folder.unwrap_or(""));
        let _ = std::fs::create_dir_all(&base_path);

        let joined = base_path.join(filename);
        let mut file = File::create(joined).unwrap();
        file.write_all(contents.as_bytes()).unwrap();
    }
}
