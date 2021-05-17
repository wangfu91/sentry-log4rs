# log4s integration for Sentry

[![Crates.io version](https://img.shields.io/crates/v/sentry-log4rs.svg)](https://crates.io/crates/sentry-log4rs) [![Documentation](https://docs.rs/sentry-log4rs/badge.svg)](https://docs.rs/sentry-log4rs/)

This crate provides support for integrating sentry with log4rs.

## Quick Start

log4rs.yaml:
```yaml
refresh_rate: 30 seconds
appenders:
  stdout:
    kind: console
  
  sentry:
    kind: sentry
    encoder:
      pattern: "{m}"
    dsn: "https://key@sentry.io/42"
    threshold: error

root:
  level: info
  appenders:
    - stdout
    - sentry

```

main.rs:
```rust
use log::{error, info};
use log4rs::{self, config::Deserializers};
use sentry_log4rs::SentryAppenderDeserializer;

fn main() {
    let mut deserializers = Deserializers::new();
    deserializers.insert("sentry", SentryAppenderDeserializer);
    log4rs::init_file("log4rs.yaml", deserializers).unwrap();

    info!("booting up");
    error!("Something went wrong!");

    // ...
}
```

## Testing

The functionality can be tested with  `examples/yaml_config.rs` and `example/code_config.rs` examples, just update the `dsn` value and run it with:

```shell script
    cargo run --example code_config
    cargo run --example yaml_config
```
## License

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
