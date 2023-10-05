use crossbeam::channel::unbounded;
use crossbeam::channel::Receiver;
use crossbeam::channel::TryRecvError;
use std::io::{stdin, stdout, BufReader, BufWriter, Stdin, Stdout};
use std::sync::Mutex;
use std::thread::spawn;

use dap::base_message::*;
use dap::requests::*;
use dap::server::*;

use lazy_static::lazy_static;

type StdServer = Server<Stdin, Stdout>;

lazy_static! {
    static ref SERVER: DapServer = DapServer::new();
}

struct DapServer {
    outgoing: Mutex<StdServer>,
    incoming: Receiver<Request>,
}

impl DapServer {
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
        DapServer { outgoing: Mutex::new(server), incoming: rx }
    }
}

pub(crate) fn read() -> Option<Request> {
    match SERVER.incoming.try_recv() {
        Ok(req) => Some(req),
        Err(TryRecvError::Disconnected) => None,
        Err(TryRecvError::Empty) => None,
    }
}

pub(crate) fn write(message: Sendable) {
    SERVER.outgoing.lock().unwrap().send(message).unwrap();
}
