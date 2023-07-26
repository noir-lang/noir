use std::fs;

use acvm::Backend;
use clap::Args;
use decompress::{ExtractOpts, ExtractOptsBuilder};
use dirs::home_dir;
use nargo::manifest::{Backends, GlobalConfig};
use tokio::runtime::Builder;
use tracing::debug;
use trauma::{
    download::{Download, Status, Summary},
    downloader::DownloaderBuilder,
};

use crate::{cli::fs::global_config, constants, errors::CliError};

use crate::cli::arguments::NargoConfig;

/// Install a backend handling proof creation and verification
#[derive(Debug, Clone, Args)]
pub(crate) struct InstallBackendCommand {
    ///Name of backend that will be installed    
    #[arg(long)]
    pub(crate) name: String,

    ///Url from which backend binary would be installed
    #[arg(long)]
    pub(crate) url: reqwest::Url,

    ///Should backend be overwritten if already exsists
    #[arg(long)]
    pub(crate) overwrite: bool,
}

pub(crate) fn run<B: Backend>(
    _backend: &B,
    args: InstallBackendCommand,
    _config: NargoConfig,
) -> Result<(), CliError<B>> {
    debug!("Supplied arguments: {:?}", args);

    let runtime = Builder::new_current_thread().enable_all().build().unwrap();

    runtime.block_on(async {
        let backend_assumed_path_buf = home_dir()
            .unwrap()
            .join(constants::NARGO_HOME_FOLDER_NAME)
            .join(constants::NARGO_BACKENDS_FOLDER_NAME)
            .join("bin");
        // .join("bb.js");

        let temp_dir = std::env::temp_dir();
        debug!("Will download to temp directory located at: {:?}", temp_dir);

        let download = trauma::download::Download::try_from(&args.url).unwrap();
        let downlod_file_name = download.filename.clone();

        #[allow(unused_must_use)]
        {
            fs::remove_file(temp_dir.join(&downlod_file_name));
        }

        let downloads = vec![download];
        let downloader = DownloaderBuilder::new().directory(temp_dir.clone()).build();
        let summaries = downloader.download(&downloads).await;

        match summaries.iter().last().unwrap().status() {
            Status::Fail(fail_msg) => {
                println!(
                    "{}",
                    format!("❌ Failed to get {} due to {}", downlod_file_name, fail_msg)
                );
            }
            Status::NotStarted => {
                println!("{}", format!("🔜Download of {} not started", downlod_file_name));
            }
            Status::Skipped(msg) => {
                println!("{}", format!("🔜Download skipped for {}", msg));
            }
            Status::Success => {
                println!("{}", format!("✅ {} download ok...", downlod_file_name));
                fs::create_dir_all(&backend_assumed_path_buf).unwrap();
                let decompress_to =
                    String::from(backend_assumed_path_buf.as_os_str().to_str().unwrap());
                debug!("Will decompress to {}", decompress_to);
                match decompress::decompress(
                    temp_dir.join(&downlod_file_name),
                    backend_assumed_path_buf.clone(),
                    &ExtractOptsBuilder::default()
                        .strip(1)
                        // .filter(|path| {
                        //     if let Some(path) = path.to_str() {
                        //     return path.ends_with("abc.sh");
                        //     }
                        //     false
                        // })
                        .build()
                        .unwrap(),
                ) {
                    Ok(decomp) => {
                        println!("decompression ok to {:?}.", backend_assumed_path_buf);
                        global_config::write_global_config_file(GlobalConfig {
                            backends: Some(Backends {
                                default: Some(String::from(
                                    backend_assumed_path_buf
                                        .join(args.name)
                                        .as_os_str()
                                        .to_str()
                                        .unwrap(),
                                )),
                            }),
                        });
                    
                    
                    }
                    Err(err) => println!(
                        "decompression to {:?} failed due to {}.",
                        backend_assumed_path_buf, err
                    ),
                }
            }
        }
        Ok(())
    })
}
//
