use std::{thread, time::Duration};

use log::{error, info};
use log4rs::{self, config::Deserializers};
use sentry_log4rs::SentryAppenderDeserializer;

fn main() {
    let mut deserializers = Deserializers::new();
    deserializers.insert("sentry", SentryAppenderDeserializer);
    log4rs::init_file("./examples/log4rs.yaml", deserializers).unwrap();

    info!("booting up");
    error!("[yaml-config] Something went wrong!");

    // Wait some time for SentryAppender to send the message to server.
    thread::sleep(Duration::from_secs(1));
}
