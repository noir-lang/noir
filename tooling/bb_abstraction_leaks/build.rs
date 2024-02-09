use build_target::{Arch, Os};
use const_format::formatcp;

// Useful for printing debugging messages during the build
// macro_rules! p {
//     ($($tokens: tt)*) => {
//         println!("cargo:warning={}", format!($($tokens)*))
//     }
// }

const USERNAME: &str = "AztecProtocol";
const REPO: &str = "aztec-packages";
const VERSION: &str = "0.23.0";
const TAG: &str = formatcp!("aztec-packages-v{}", VERSION);

const API_URL: &str =
    formatcp!("https://github.com/{}/{}/releases/download/{}", USERNAME, REPO, TAG);

fn main() -> Result<(), String> {
    // We need to inject which OS we're building for so that we can download the correct barretenberg binary.
    let os = match build_target::target_os().unwrap() {
        os @ (Os::Linux | Os::MacOs) => os,
        Os::Windows => todo!("Windows is not currently supported"),
        os_name => panic!("Unsupported OS {os_name}"),
    };

    let arch = match build_target::target_arch().unwrap() {
        arch @ (Arch::X86_64 | Arch::AARCH64) => arch,
        arch_name => panic!("Unsupported Architecture {arch_name}"),
    };

    // Arm builds of linux are not supported
    // We do not panic because we allow users to run nargo without a backend.
    if let (Os::Linux, Arch::AARCH64) = (&os, &arch) {
        println!(
            "cargo:warning=ARM64 builds of linux are not supported for the barretenberg binary"
        );
    };

    println!("cargo:rustc-env=BB_BINARY_URL={}", get_bb_download_url(arch, os));
    println!("cargo:rustc-env=BB_VERSION={}", VERSION);

    Ok(())
}

fn get_bb_download_url(target_arch: Arch, target_os: Os) -> String {
    let archive_name = match target_os {
        Os::Linux => "barretenberg-x86_64-linux-gnu.tar.gz",
        Os::MacOs => match target_arch {
            Arch::AARCH64 => "barretenberg-aarch64-apple-darwin.tar.gz",
            Arch::X86_64 => "barretenberg-x86_64-apple-darwin.tar.gz",
            arch => panic!("unsupported arch {arch}"),
        },
        os => panic!("Unsupported OS {os}"),
    };

    format!("{API_URL}/{archive_name}")
}
