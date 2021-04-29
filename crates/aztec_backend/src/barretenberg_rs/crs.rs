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

// XXX: Below is the logic to download the CRS if it is not already present (taken from the Rust cookbook)
// This has not been optimised.

use error_chain::error_chain;
use reqwest::header::{HeaderValue, CONTENT_LENGTH, RANGE};
use reqwest::StatusCode;
use std::fs::File;
use std::str::FromStr;

error_chain! {
    foreign_links {
        Io(std::io::Error);
        Reqwest(reqwest::Error);
        Header(reqwest::header::ToStrError);
    }
}

struct PartialRangeIter {
    start: u64,
    end: u64,
    buffer_size: u32,
}

impl PartialRangeIter {
    pub fn new(start: u64, end: u64, buffer_size: u32) -> Result<Self> {
        if buffer_size == 0 {
            return Err("invalid buffer_size, give a value greater than zero.".into());
        }
        Ok(PartialRangeIter {
            start,
            end,
            buffer_size,
        })
    }
}

impl Iterator for PartialRangeIter {
    type Item = HeaderValue;
    fn next(&mut self) -> Option<Self::Item> {
        if self.start > self.end {
            None
        } else {
            let prev_start = self.start;
            self.start += std::cmp::min(self.buffer_size as u64, self.end - self.start + 1);
            Some(
                HeaderValue::from_str(&format!("bytes={}-{}", prev_start, self.start - 1))
                    .expect("string provided by format!"),
            )
        }
    }
}

// XXX: Clean up to handle Errors better and remove the partial file in case we cannot
// download the whole file
pub fn download_crs(mut path: std::path::PathBuf) {
    if path.exists() {
        println!("File already exists");
        return;
    }

    // If the path is the path to the 'transcript00.dat' file, pop it off and download the file into the Directory
    if path.ends_with(std::path::Path::new("transcript00.dat")) {
        path.pop();
    }

    let url = "http://aztec-ignition.s3.amazonaws.com/MAIN%20IGNITION/sealed/transcript00.dat";
    const CHUNK_SIZE: u32 = 10240;

    let client = reqwest::blocking::Client::new();
    let response = client.head(url).send().expect("Expected a response");
    let length = response
        .headers()
        .get(CONTENT_LENGTH)
        .ok_or("response doesn't include the content length")
        .expect("Expected the content length");
    let length = u64::from_str(length.to_str().unwrap())
        .map_err(|_| "invalid Content-Length header")
        .unwrap();

    std::fs::create_dir_all(&path)
        .expect("Failed to create the directory named 'ignition'. Please check your permissions");

    let mut output_file = {
        let fname = "transcript00.dat";

        println!("Downloading the Default SRS (340MB) : Ignite!");
        let fname = path.join(fname);
        println!("\nSRS will be saved at location: '{:?}'", fname);

        File::create(fname).unwrap()
    };

    // XXX: This progress bar redraws on macos after a minute. It's not
    // a problem functionally, however it would be nice to fix.
    let bar = indicatif::ProgressBar::new(length / (CHUNK_SIZE as u64));
    bar.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .progress_chars("##-"),
    );

    let started = std::time::Instant::now();
    for range in PartialRangeIter::new(0, length - 1, CHUNK_SIZE).unwrap() {
        bar.inc(1);

        let mut response = client.get(url).header(RANGE, range).send().unwrap();

        let status = response.status();
        if !(status == StatusCode::OK || status == StatusCode::PARTIAL_CONTENT) {
            panic!("Unexpected server response: {}", status);
        }
        std::io::copy(&mut response, &mut output_file).unwrap();
    }
    bar.finish();

    println!(
        "Downloading the SRS took {}",
        indicatif::HumanDuration(started.elapsed())
    );

    let content = response.text().unwrap();
    std::io::copy(&mut content.as_bytes(), &mut output_file).unwrap();

    println!("Finished with success!");
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

    let file_path = dir.path().join("transcript00.dat");
    download_crs(file_path);
}
