use const_format::formatcp;

const USERNAME: &str = "AztecProtocol";
const REPO: &str = "barretenberg";
const VERSION: &str = "0.5.1";
const TAG: &str = formatcp!("barretenberg-v{}", VERSION);

const API_URL: &str =
    formatcp!("https://github.com/{}/{}/releases/download/{}", USERNAME, REPO, TAG);

pub(crate) fn get_bb_download_url() -> String {
    let target_os = env!("TARGET_OS");
    let target_arch = env!("TARGET_ARCH");

    let archive_name = match target_os {
        "linux" => "barretenberg-x86_64-linux-gnu.tar.gz",
        "macos" => match target_arch {
            "aarch64" => "barretenberg-aarch64-apple-darwin.tar.gz",
            "x86_64" => "barretenberg-x86_64-apple-darwin.tar.gz",
            arch => panic!("unsupported arch {arch}"),
        },
        os => panic!("Unsupported OS {os}"),
    };

    format!("{API_URL}/{archive_name}")
}
