use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

pub(super) mod program;

pub(super) fn create_named_dir(named_dir: &Path, name: &str) -> PathBuf {
    std::fs::create_dir_all(named_dir)
        .unwrap_or_else(|_| panic!("could not create the `{name}` directory"));

    PathBuf::from(named_dir)
}

pub(super) fn write_to_file(bytes: &[u8], path: &Path) -> String {
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
