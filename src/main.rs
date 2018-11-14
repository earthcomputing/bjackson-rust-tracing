
use std::thread;
use std::sync::mpsc;

extern crate failure;
extern crate lazy_static;
extern crate serde;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde_json;
extern crate time;

mod emitter;

// primitive override flag(s):
pub struct TraceOptions {
    pub all: bool,
    pub el: bool
}

pub const TRACE_OPTIONS: TraceOptions = TraceOptions {
    all: false,
    el:  true,
};

fn event_loop(rx: mpsc::Receiver<String>) {
    let _f = "event_loop"; 
    let worker = thread::current();
    let tid = format!("{:?}", worker.id()); // looks like : "ThreadId(8)"
    let name = worker.name().unwrap(); // name is guaranteed

    for received in rx {
        let ref body = json!({ "tid": tid, "name": name, "recv": received });
        if TRACE_OPTIONS.all || TRACE_OPTIONS.el {
            let ref code_attr = emitter::CodeAttributes { module: file!(), function: _f, line_no: line!(), format: "recv" };
            let (key, entry) = emitter::unhide().trace(code_attr, body).unwrap();
            // let _ = dal::add_trace(entry);
            println!("{} {}", key, entry);
        }
        println!("{}", body);
        if received == "exit" { return; }
    }
}

fn worker(i : usize, rx: mpsc::Receiver<String>) -> thread::JoinHandle<String> {
    let _f = "worker";
    let thread_name = format!("event_loop #{}", i);
    // let mut o = self.clone();
    // let child_emitter = emitter::unhide().fork_trace();
    let h = thread::Builder::new().name(thread_name.into()).spawn(move || {
        // let ref mut working_emitter = child_emitter.clone();
        // emitter::stash(working_emitter);
        event_loop(rx);
        let worker = thread::current();
        format!("{:?} {}", worker.id(), worker.name().unwrap())
    }).unwrap();
    h
}

fn main() {
    let bodies = [ "msg1", "msg2", "msg3", "msg4", "msg5", "exit" ];
    let mut channels: Vec<mpsc::Sender<String>> = Vec::new();
    let mut handles: Vec<thread::JoinHandle<String>> = Vec::new();
    for i in 1..10 {
        let (tx, rx) = mpsc::channel();
        channels.push(tx);
        let h = worker(i, rx);
        handles.push(h);
    }

    for m in bodies.iter() {
        for tx in channels.iter() { tx.send(m.to_string()).unwrap(); }
    }

    // non-optimal latency for set-join
    let mut hist: Vec<Result<String, _>> = Vec::new();
    while let Some(h) = handles.pop() {
        let rc = h.join();
        hist.push(rc);
    }

    for rc in hist.iter() {
        println!("completed {:?}", rc);
    }
}
