
use futures::Future;
use lazy_static::lazy_static;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};

const KAFKA_SERVER: &'static str = "localhost:9092";
const KAFKA_TOPIC: &'static str = "IPC";

lazy_static! {
    static ref PRODUCER_RD: FutureProducer = 
        ClientConfig::new()
            .set("bootstrap.servers", KAFKA_SERVER)
            .set("message.timeout.ms", "5000")
        .create().unwrap(); // .expect("");
}

// fn builder<'a>() -> &'a mut ClientConfig { let ref mut factory = ...; factory }

pub fn log(key: String, doclet: String) {
    let item = FutureRecord::to(KAFKA_TOPIC).payload(&doclet).key(&key);
    let xmit = PRODUCER_RD.send(item, 0);
    // FIXME: lots of things to be handled here
    let _ = xmit.then(|result| -> Result<(), rdkafka::error::KafkaError> {
        match result {
            Ok(Ok(_)) => Ok(()),
            Ok(Err((e, msg))) => Err(e), // Err(e, rdkafka::message::OwnedMessage(msg))
            Err(e) => Ok(()) // Err(e: futures::Canceled)
        }
    });
}
