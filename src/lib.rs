//! **sentry-log4rs** is a simple add-on for the **log4rs** logging framework to simplify
//! integration with the **Sentry** application monitoring system.
//!
//! Example usage
//! =============
//!
//! ```no_run
//!  log4rs::init_file("log4rs.yaml", SentryAppender::deserializers()).unwrap();
//!  error!("Something went wrong!");
//! ```
//!
//! `log4rs.yaml` file:
//! ```yaml
//! appenders:
//!   sentry:
//!     kind: sentry
//!     encoder:
//!       pattern: "{m}"
//!     dsn: "YOUR_SENTRY_DSN_HERE"
//!     threshold: error
//!
//! root:
//!   appenders:
//!     - sentry
//! ```
//!
//! You can also constructing the configuration programmatically without using a config file:
//!
//! ```no_run
//!  let sentry = SentryAppender::builder()
//!      .dsn("YOUR_SENTRY_DSN_HERE")
//!      .threshold(LevelFilter::Error)
//!      .encoder(Box::new(PatternEncoder::new("{m}")))
//!      .build();
//!
//!  let config = Config::builder()
//!      .appender(Appender::builder().build("sentry", Box::new(sentry)))
//!      .build(
//!          Root::builder()
//!              .appender("sentry")
//!              .build(LevelFilter::Info),
//!      )
//!      .unwrap();
//!
//!  log4rs::init_config(config).unwrap();
//!
//!  error!("Something went wrong!");
//! ```
extern crate log;
extern crate log4rs;
extern crate sentry;

use derivative::Derivative;
use log::{Level, LevelFilter, Record};
use log4rs::{
    append::Append,
    config::{Deserialize, Deserializers},
    encode::{pattern::PatternEncoder, writer::simple::SimpleWriter, Encode, EncoderConfig},
};
use sentry::{
    internals::ClientInitGuard,
    protocol::value::{Number, Value},
    Level as SentryLevel,
};

/// Configuration for the sentry appender.
#[derive(Clone, Eq, PartialEq, Hash, Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SentryAppenderConfig {
    dsn: String,
    encoder: Option<EncoderConfig>,
    threshold: LevelFilter,
}

/// An appender which send log message to sentry.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct SentryAppender {
    #[derivative(Debug = "ignore")]
    _sentry: ClientInitGuard,
    encoder: Box<dyn Encode>,
    threshold: LevelFilter,
}

impl SentryAppender {
    /// Creates a new `SentryAppender` builder.
    pub fn builder() -> SentryAppenderBuilder {
        SentryAppenderBuilder {
            encoder: None,
            dsn: String::default(),
            threshold: None,
        }
    }

    /// Creates a `Deserializers` with sentry appender mapping and the default log4rs mappings.
    ///  * Appenders
    ///     * "sentry" -> `SentryAppenderDeserializer`
    ///  * log4rs default mappings.
    pub fn deserializers() -> Deserializers {
        let mut deserializers = Deserializers::new();
        deserializers.insert("sentry", SentryAppenderDeserializer);
        deserializers
    }
}

impl Append for SentryAppender {
    fn append(&self, record: &Record) -> anyhow::Result<()> {
        if record.level() > self.threshold {
            // Don't send records to sentry if record's level greater than the user defined threshold.
            // e.g. Info > Error
            return Ok(());
        }

        let level = level_mapping(record.level());

        let mut buf: Vec<u8> = Vec::new();
        self.encoder.encode(&mut SimpleWriter(&mut buf), record)?;
        let msg = String::from_utf8(buf)?;

        let mut event = sentry::protocol::Event::new();
        event.level = level;
        event.message = Some(msg);
        event.logger = Some(record.metadata().target().to_owned());

        if let Some(file) = record.file() {
            event
                .extra
                .insert("file".to_owned(), Value::String(file.to_owned()));
        }

        if let Some(line) = record.line() {
            event
                .extra
                .insert("line".to_owned(), Value::Number(Number::from(line)));
        }

        if let Some(module_path) = record.module_path() {
            event
                .tags
                .insert("module_path".to_owned(), module_path.to_owned());
        }

        sentry::capture_event(event);
        Ok(())
    }

    fn flush(&self) {}
}

/// A builder for `SentryAppender`s.
pub struct SentryAppenderBuilder {
    encoder: Option<Box<dyn Encode>>,
    dsn: String,
    threshold: Option<LevelFilter>,
}

impl SentryAppenderBuilder {
    pub fn encoder(mut self, encoder: Box<dyn Encode>) -> SentryAppenderBuilder {
        self.encoder = Some(encoder);
        self
    }

    pub fn dsn(mut self, dsn: &str) -> SentryAppenderBuilder {
        self.dsn = dsn.to_string();
        self
    }

    fn dsn_string(mut self, dsn: String) -> SentryAppenderBuilder {
        self.dsn = dsn;
        self
    }

    pub fn threshold(mut self, threshold: LevelFilter) -> SentryAppenderBuilder {
        self.threshold = Some(threshold);
        self
    }

    pub fn build(self) -> SentryAppender {
        let _sentry: ClientInitGuard = sentry::init(self.dsn);
        SentryAppender {
            _sentry,
            encoder: self
                .encoder
                .unwrap_or_else(|| Box::new(PatternEncoder::new("{m}"))),
            threshold: self.threshold.unwrap_or(LevelFilter::Error),
        }
    }
}

/// A deserializer for the `SentryAppender`.
///
/// # Configuration
///
/// ```yaml
/// kind: sentry
///
/// # The sentry DSN, e.g. "https://key@sentry.io/42"
/// dsn: "YOUR_DSN_HERE"
///
/// # The log level threshold
/// threshold: error  # overriding the logging threshold to the ERROR level
///
/// # The encoder to use to format output. Defaults to `kind: pattern`.
/// encoder:
///   kind: pattern
/// ```
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct SentryAppenderDeserializer;

impl Deserialize for SentryAppenderDeserializer {
    type Trait = dyn Append;

    type Config = SentryAppenderConfig;

    fn deserialize(
        &self,
        config: SentryAppenderConfig,
        deserializers: &Deserializers,
    ) -> anyhow::Result<Box<dyn Append>> {
        let mut appender = SentryAppender::builder();

        if let Some(encoder) = config.encoder {
            appender = appender.encoder(deserializers.deserialize(&encoder.kind, encoder.config)?);
        }

        appender = appender.dsn_string(config.dsn);

        appender = appender.threshold(config.threshold);

        Ok(Box::new(appender.build()))
    }
}

fn level_mapping(level: Level) -> SentryLevel {
    match level {
        Level::Error => SentryLevel::Error,
        Level::Warn => SentryLevel::Warning,
        Level::Info => SentryLevel::Info,
        Level::Debug => SentryLevel::Debug,
        Level::Trace => SentryLevel::Debug,
    }
}
