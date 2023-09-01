use build_target::{Arch, Os};

// Useful for printing debugging messages during the build
// macro_rules! p {
//     ($($tokens: tt)*) => {
//         println!("cargo:warning={}", format!($($tokens)*))
//     }
// }

fn main() -> Result<(), String> {
    // We need to inject which OS we're building for so that we can download the correct barretenberg binary.
    let os = match build_target::target_os().unwrap() {
        os @ (Os::Linux | Os::MacOs) => os,
        Os::Windows => todo!("Windows is not currently supported"),
        os_name => panic!("Unsupported OS {}", os_name),
    };

    let arch = match build_target::target_arch().unwrap() {
        arch @ (Arch::X86_64 | Arch::AARCH64) => arch,
        arch_name => panic!("Unsupported Architecture {}", arch_name),
    };

    // Arm builds of linux are not supported
    if let (Os::Linux, Arch::AARCH64) = (&os, &arch) {
        panic!("ARM64 builds of linux are not supported")
    };

    println!("cargo:rustc-env=TARGET_OS={os}");
    println!("cargo:rustc-env=TARGET_ARCH={arch}");

    Ok(())
}
