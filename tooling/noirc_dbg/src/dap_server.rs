use crossbeam::channel::unbounded;
use crossbeam::channel::Receiver;
use crossbeam::channel::TryRecvError;
use std::io::{stdin, stdout, BufReader, BufWriter, Stdin, Stdout};
use std::sync::Mutex;
use std::thread::spawn;

use dap::base_message::*;
use dap::requests::*;
use dap::server::*;

pub(crate) struct Dap {
    output: Mutex<Server<Stdin, Stdout>>,
    input: Receiver<Request>,
}

impl Dap {
    pub(crate) fn new() -> Self {
        let (tx, rx) = unbounded::<Request>();
        spawn(move || {
            let mut server = Server::new(BufReader::new(stdin()), BufWriter::new(stdout()));
            loop {
                let req = match server.poll_request() {
                    Ok(Some(req)) => req,
                    Ok(None) => continue,
                    Err(_) => return,
                };
                if tx.send(req).is_err() {
                    return;
                }
            }
        });

        let server = Server::new(BufReader::new(stdin()), BufWriter::new(stdout()));
        Dap { output: Mutex::new(server), input: rx }
    }

    pub(crate) fn read(&self) -> Option<Request> {
        match self.input.try_recv() {
            Ok(req) => Some(req),
            Err(TryRecvError::Disconnected) => None,
            Err(TryRecvError::Empty) => None,
        }
    }

    pub(crate) fn write(&self, message: Sendable) {
        self.output.lock().unwrap().send(message).unwrap();
    }
}
