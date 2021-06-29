#[allow(clippy::upper_case_acronyms)]
pub struct CRS {
    pub g1_data: Vec<u8>,
    pub g2_data: Vec<u8>,
    pub num_points: usize,
}

const G1_START: usize = 28;
const G2_START: usize = 28 + (5_040_000 * 64);
const G2_END: usize = G2_START + 128 - 1;

fn transcript_location() -> std::path::PathBuf {
    let mut transcript_dir = dirs::home_dir().unwrap();
    transcript_dir.push(std::path::Path::new("noir_cache"));
    transcript_dir.push(std::path::Path::new("ignition"));
    transcript_dir.push(std::path::Path::new("transcript00.dat"));
    transcript_dir
}

impl CRS {
    pub fn new(num_points: usize) -> CRS {
        let g1_end = G1_START + (num_points * 64) - 1;

        // If the CRS does not exist, then download it from S3
        if !transcript_location().exists() {
            download_crs(transcript_location());
        }

        let crs = read_crs(transcript_location());

        let g1_data = crs[G1_START..=g1_end].to_vec();
        let g2_data = crs[G2_START..=G2_END].to_vec();

        CRS {
            g1_data,
            g2_data,
            num_points,
        }
    }
}

fn read_crs(path: std::path::PathBuf) -> Vec<u8> {
    match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                panic!("please run again with appropriate permissions.");
            }
            panic!(
                "Could not find file transcript00.dat at location {}.\n Starting Download",
                path.display()
            );
        }
    }
}

// XXX: Below is the logic to download the CRS if it is not already present

pub fn download_crs(mut path_to_transcript: std::path::PathBuf) {
    if path_to_transcript.exists() {
        println!("File already exists {:?}", path_to_transcript);
        return;
    }
    // Pop off the transcript component to get just the directory
    path_to_transcript.pop();

    if !path_to_transcript.exists() {
        std::fs::create_dir_all(&path_to_transcript).unwrap();
    }

    let url = "http://aztec-ignition.s3.amazonaws.com/MAIN%20IGNITION/sealed/transcript00.dat";
    use downloader::Downloader;
    let mut downloader = Downloader::builder()
        .download_folder(path_to_transcript.as_path())
        .build()
        .unwrap();

    let dl = downloader::Download::new(url);
    let dl = dl.progress(SimpleReporter::create());
    let result = downloader.download(&[dl]).unwrap();

    for r in result {
        match r {
            Err(e) => println!("Error: {}", e.to_string()),
            Ok(s) => println!("\nSRS is located at : {:?}", &s.file_name),
        };
    }
}
// Taken from https://github.com/hunger/downloader/blob/main/examples/download.rs
struct SimpleReporterPrivate {
    started: std::time::Instant,
    progress_bar: indicatif::ProgressBar,
}
struct SimpleReporter {
    private: std::sync::Mutex<Option<SimpleReporterPrivate>>,
}

impl SimpleReporter {
    fn create() -> std::sync::Arc<Self> {
        std::sync::Arc::new(Self {
            private: std::sync::Mutex::new(None),
        })
    }
}

impl downloader::progress::Reporter for SimpleReporter {
    fn setup(&self, max_progress: Option<u64>, _message: &str) {
        let bar = indicatif::ProgressBar::new(max_progress.unwrap());
        bar.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
                .progress_chars("##-"),
        );

        let private = SimpleReporterPrivate {
            started: std::time::Instant::now(),
            progress_bar: bar,
        };
        println!("\nDownloading the Ignite SRS (340MB)\n");

        let mut guard = self.private.lock().unwrap();
        *guard = Some(private);
    }

    fn progress(&self, current: u64) {
        if let Some(p) = self.private.lock().unwrap().as_mut() {
            p.progress_bar.set_position(current);
        }
    }

    fn set_message(&self, _message: &str) {}

    fn done(&self) {
        let mut guard = self.private.lock().unwrap();
        let p = guard.as_mut().unwrap();
        p.progress_bar.finish();
        println!("Downloaded the SRS successfully!");
        println!(
            "Time Elapsed: {}",
            indicatif::HumanDuration(p.started.elapsed())
        );
    }
}

#[test]
fn does_not_panic() {
    use super::Barretenberg;
    use wasmer::Value;

    let mut barretenberg = Barretenberg::new();

    let num_points = 4 * 1024;

    let crs = CRS::new(num_points);

    let crs_ptr = barretenberg.allocate(&crs.g1_data);

    let _ = barretenberg.call_multiple(
        "new_pippenger",
        vec![&crs_ptr, &Value::I32(num_points as i32)],
    );
    barretenberg.free(crs_ptr);

    let scalars = vec![0; num_points * 32];
    let mem = barretenberg.allocate(&scalars);
    barretenberg.free(mem);
}
#[test]
#[ignore]
fn downloading() {
    use tempfile::tempdir;
    let dir = tempdir().unwrap();

    let file_path = dir.path().to_path_buf().join("transcript00.dat");
    download_crs(file_path);
}
