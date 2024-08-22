# Configuration

<!-- toc -->

## Babyrite Configuration

The babyrite configuration is written in `/config/config.yaml`. 

The default settings are as follows:

```yaml
bypass_guilds: false
```

- `bypass_guilds`: Whether to Bypass the guild check. If enabled, allows quoting between guilds where babyrite is installed. (default: `false`)

> [!CAUTION]
> 
> Note that if you enable `bypass_guilds`, bots may be used for malicious purposes. This setting is only intended for use when babyrite is used among a limited group.
>
> It is recommended that this feature be disabled.

## Change the displayed logs

Since babyrite uses [tracing, traching-subscriber](https://github.com/tokio-rs/tracing) for logging, you can change the displayed logs by setting the environment variable `RUST_LOG` at startup.

```shell
RUST_LOG=babyrite=info cargo run
RUST_LOG=babyrite=debug cargo run
RUST_LOG=babyrite=info ./target/release/babyrite
```

When `RUST_LOG` is not set, running cargo run or the binary directly will display babyrite logs including debug events. (default)

If you want to change the type of log displayed, pass `RUST_LOG` as an environment variable.


> [!NOTE]
>
> `babyrite=` specifies the logs for the babyrite. Therefore, if you specify a crate other than babyrite in RUST_LOG (e.g. `RUST_LOG=serenity=info`), the logs for that crate will be displayed.
