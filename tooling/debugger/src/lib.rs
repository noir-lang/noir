mod context;
mod repl;

use std::io::{BufReader, BufWriter};

use acvm::BlackBoxFunctionSolver;
use acvm::{acir::circuit::Circuit, acir::native_types::WitnessMap};

use dap::errors::ServerError;
use dap::prelude::Event;
use dap::requests::Command;
use dap::responses;
use dap::server::Server;
use dap::types;
use nargo::artifacts::debug::DebugArtifact;

use nargo::NargoError;

pub fn debug_circuit<B: BlackBoxFunctionSolver>(
    blackbox_solver: &B,
    circuit: &Circuit,
    debug_artifact: DebugArtifact,
    initial_witness: WitnessMap,
) -> Result<Option<WitnessMap>, NargoError> {
    repl::run(blackbox_solver, circuit, &debug_artifact, initial_witness)
}

pub fn start_dap_server() -> Result<(), ServerError> {
    let output = BufWriter::new(std::io::stdout());
    let input = BufReader::new(std::io::stdin());
    let mut server = Server::new(input, output);

    loop {
        let req = match server.poll_request()? {
            Some(req) => req,
            None => break,
        };
        if let Command::Initialize(_) = req.command {
            let rsp = req.success(responses::ResponseBody::Initialize(types::Capabilities {
                ..Default::default()
            }));

            // When you call respond, send_event etc. the message will be wrapped
            // in a base message with a appropriate seq number, so you don't have to keep track of that yourself
            server.respond(rsp)?;

            server.send_event(Event::Initialized)?;
        } else {
            eprintln!("ERROR: unhandled command");
        }
    }
    Ok(())
}
