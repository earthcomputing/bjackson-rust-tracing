
use std::sync::{Mutex};

use futures::Future;
use lazy_static::lazy_static;
use rdkafka::config::ClientConfig;
use rdkafka::error::KafkaError;
use rdkafka::message::OwnedMessage;
use rdkafka::producer::{ DeliveryFuture, FutureProducer, FutureRecord };

const KAFKA_SERVER: &'static str = "localhost:9092";
const KAFKA_TOPIC: &'static str = "IPC";

lazy_static! {
    static ref PRODUCER_RD: FutureProducer = 
        ClientConfig::new()
            .set("bootstrap.servers", KAFKA_SERVER)
            .set("message.timeout.ms", "5000")
        .create().expect("Producer creation error"); // .unwrap();
}

// fn builder<'a>() -> &'a mut ClientConfig { let ref mut factory = ...; factory }

lazy_static! {
    static ref FUTURES: Mutex<Vec<DeliveryFuture>> = {
        let v = vec![];
        let m = Mutex::new(v);
        m
    };
}

pub fn monitor_future(xmit: DeliveryFuture) {
    FUTURES.lock().unwrap().push(xmit);
}

// ref: https://github.com/fede1024/rust-rdkafka/blob/master/examples/asynchronous_processing.rs
pub fn log(key: String, doclet: String) {
    let item : FutureRecord<String, String> = FutureRecord::to(KAFKA_TOPIC).key(&key).payload(&doclet);
    let xmit : DeliveryFuture = PRODUCER_RD.send(item, 0);
    monitor_future(xmit);
}

// FIXME: lots of things to be handled here
pub fn flush() {
    while let Some(xmit) = FUTURES.lock().unwrap().pop() {
        let f = xmit.then(|result| -> Result<(), KafkaError> {
            match result {
                Ok(Ok(delivery)) => { println!("Sent: {:?}", delivery); Ok(())},
                Ok(Err((e, msg))) => { println!("Error: {:?}", e); Ok(())}, // Err(e, OwnedMessage(msg))
                Err(e) => { println!("Cancelled: {:?}", e); Ok(())}         // Err(e: futures::Canceled)
            }
        });
        let _ = f.wait(); // Result - always Ok(())
    }
}

/*
Ok(Err((KafkaError (Message production error: MessageTimedOut (Local: Message timed out)), OwnedMessage {
    payload: Some([123, 34, ...  125, 125]),
    key: Some([69, 118, ....  32, 125]),
    topic: "IPC",
    timestamp: NotAvailable,
    partition: -1,
    offset: -1001,
    headers: None
})))
*/
