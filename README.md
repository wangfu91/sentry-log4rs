# log4s integration for Sentry

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

lib.rs:
```rust
use log::{error, info, warn};
use log4rs;

fn main() {
    let mut deserializers = Deserializers::new();
    deserializers.insert("sentry", SentryAppenderDeserializer);
    log4rs::init_file("log4rs.yaml", deserializers).unwrap();

    info!("booting up");
    error!("Something went wrong!");

    // ...
}
```

## License

Licensed under either of
 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be dual licensed as above, without any
additional terms or conditions.