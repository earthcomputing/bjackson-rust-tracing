
use std::thread;
use std::sync::mpsc;

extern crate failure;
extern crate lazy_static;
extern crate serde;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde_json;
extern crate time;

mod emitter;

fn event_loop(rx: mpsc::Receiver<String>) {
    let worker = thread::current();
    let tid = format!("{:?}", worker.id()); // looks like : "ThreadId(8)"
    let name = worker.name().unwrap(); // name is guaranteed

    for received in rx {
        let ref body = json!({ "tid": tid, "name": name, "recv": received });
        println!("{}", body);
        if received == "exit" { return; }
    }
}

fn main() {
    let bodies = [ "msg1", "msg2", "msg3", "msg4", "msg5", "exit" ];
    let mut channels: Vec<mpsc::Sender<String>> = Vec::new();
    let mut handles: Vec<thread::JoinHandle<String>> = Vec::new();
    for i in 1..10 {
        let (tx, rx) = mpsc::channel();
        channels.push(tx);
        let thread_name = format!("event_loop #{}", i);
        let h = thread::Builder::new().name(thread_name.into()).spawn(move || {
            event_loop(rx);
            let worker = thread::current();
            format!("{:?} {}", worker.id(), worker.name().unwrap())
        }).unwrap();
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
