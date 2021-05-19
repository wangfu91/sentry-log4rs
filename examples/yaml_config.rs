use std::{thread, time::Duration};

use log::{error, info};
use log4rs;
use sentry_log4rs::SentryAppender;

fn main() {
    log4rs::init_file("./examples/log4rs.yaml", SentryAppender::deserializers()).unwrap();

    info!("booting up");
    error!("[yaml-config] Something went wrong!");

    // Wait some time for SentryAppender to send the message to server.
    thread::sleep(Duration::from_secs(1));
}
