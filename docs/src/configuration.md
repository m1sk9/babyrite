# Configuration

<!-- toc -->

## Change the displayed logs

Since babyrite uses [tracing, traching-subscriber](https://github.com/tokio-rs/tracing) for logging, you can change the displayed logs by setting the environment variable `RUST_LOG` at startup.

```shell
RUST_LOG=babyrite=info cargo run
RUST_LOG=babyrite=debug cargo run
RUST_LOG=babyrite=info ./target/release/babyrite
```

The default behavior is as follows:

- When `RUST_LOG` is not set, running cargo run or the binary directly will display babyrite logs including debug events.
- Running the Docker image will display babyrite logs with `info` events only.

> [!NOTE]
>
> `babyrite=` specifies the logs for the babyrite. Therefore, if you specify a crate other than babyrite in RUST_LOG (e.g. `RUST_LOG=serenity=info`), the logs for that crate will be displayed.

## Set Sentry DSN (deprecated)

> [!WARNING]
>
> Sentry support was removed in [v0.8.0](https://github.com/m1sk9/babyrite/releases/tag/v0.8.0).

By setting the environment variable SENTRY_DSN to a Sentry DSN, babyrite will send errors to Sentry.

For more information about Sentry, refer to [the official site](https://sentry.io/).
