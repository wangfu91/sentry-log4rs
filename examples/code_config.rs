use std::{thread, time::Duration};

use log::LevelFilter;
use log4rs::{
    append::console::ConsoleAppender,
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
};
use sentry_log4rs::SentryAppender;

use log::{error, info};

fn main() {
    let stdout = ConsoleAppender::builder().build();
    let sentry = SentryAppender::builder()
        .dsn("https://key@sentry.io/42")
        .threshold(LevelFilter::Error)
        .encoder(Box::new(PatternEncoder::new("{m}")))
        .build();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("sentry", Box::new(sentry)))
        .build(
            Root::builder()
                .appender("stdout")
                .appender("sentry")
                .build(LevelFilter::Info),
        )
        .unwrap();

    log4rs::init_config(config).unwrap();

    info!("booting up");
    error!("[code-config] Something went wrong!");

    // Wait some time for SentryAppender to send the message to server.
    thread::sleep(Duration::from_secs(1));
}
