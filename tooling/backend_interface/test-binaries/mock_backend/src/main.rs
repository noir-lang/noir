#![forbid(unsafe_code)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

use clap::{Parser, Subcommand};

mod contract_cmd;
mod gates_cmd;
mod prove_cmd;
mod verify_cmd;
mod write_vk_cmd;

#[derive(Parser, Debug)]
#[command(name = "mock_backend")]
struct BackendCli {
    #[command(subcommand)]
    command: BackendCommand,
}

#[derive(Subcommand, Clone, Debug)]
enum BackendCommand {
    Contract(contract_cmd::ContractCommand),
    Gates(gates_cmd::GatesCommand),
    Prove(prove_cmd::ProveCommand),
    Verify(verify_cmd::VerifyCommand),
    #[command(name = "write_vk")]
    WriteVk(write_vk_cmd::WriteVkCommand),
}

fn main() {
    let BackendCli { command } = BackendCli::parse();

    match command {
        BackendCommand::Contract(args) => contract_cmd::run(args),
        BackendCommand::Gates(args) => gates_cmd::run(args),
        BackendCommand::Prove(args) => prove_cmd::run(args),
        BackendCommand::Verify(args) => verify_cmd::run(args),
        BackendCommand::WriteVk(args) => write_vk_cmd::run(args),
    };
}
