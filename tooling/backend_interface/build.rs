use build_target::{Arch, Os};

fn main() -> Result<(), String> {
    // We need to inject which OS we're building so that it can be provided to any installation scripts.
    let os = match build_target::target_os().unwrap() {
        os @ (Os::Linux | Os::MacOs) => os,
        Os::Windows => todo!("Windows is not currently supported"),
        os_name => panic!("Unsupported OS {os_name}"),
    };

    let arch = match build_target::target_arch().unwrap() {
        arch @ (Arch::X86_64 | Arch::AARCH64) => arch,
        arch_name => panic!("Unsupported Architecture {arch_name}"),
    };

    println!("cargo:rustc-env=NARGO_OS={}", os);
    println!("cargo:rustc-env=NARGO_ARCHITECTURE={}", arch);

    Ok(())
}
