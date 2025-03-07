# gjallarbot
My personal "stuff" Discord bot

## Features

Right now, the bot does two things:
- Wake on Lan: one can register machines and their MAC Addresses in the bot, and authorize users and roles to send Wake
  on Lan magic packets to those machines.
- Servitor: support for the [Servitor](https://github.com/Angelin01/servitor/) application and its basic actions,
  allowing one to register Servitor servers and authorizing users and roles to perform actions on them. Requires
  configuring Servitor instances in the bot's configuration.

## Data Permanence

The bot writes a `data.json` file to its working directory to persist data. It is a very silly, very simple
implementation, that should probably be replaced with something like SQLite should the bot grow any bigger.

## Configuring

The bot has two configuration sources:
- TOML file: The default configuration file is `gjallarbot.toml`, in the current working directory, but this can be
  overridden with the `GJ_CONFIG_FILE` environment variable.
- Environment variables: Prefixed with `GJ_`, supporting nested structures using `_` as a separator (case-sensitive).

Supported settings:

| TOML Key                | Environment Variable       | Description                                                                                       |
|-------------------------|----------------------------|---------------------------------------------------------------------------------------------------|
| -                       | `GJ_CONFIG_FILE`           | Overrides the default path for the configuration file.                                            |
| `bot.token`             | `GJ_bot_token`             | Discord bot token (required).                                                                     |
| `log.filter`            | `GJ_log_filter`            | Logging filter (default: `"gjallarbot=info"`, see [env_logger's documentation for more info][1]). |
| `servitor.<name>.url`   | `GJ_servitor_<name>_url`   | Base URL of a servitor instance.                                                                  |
| `servitor.<name>.token` | `GJ_servitor_<name>_token` | Optional authentication token for a servitor.                                                     |


### Example `gjallarbot.toml`

```toml
[bot]
token = "your-secret-bot-token"

[log]
filter = "gjallarbot=debug"

[servitor]
"SomeServer" = { url = "https://example.com", token = "optional-secret-token" }

[servitor.OtherInsecureServer]
url = "http://example.com:8008"
```

[1]: https://github.com/rust-cli/env_logger

## Development

Standard Rust development cycle, requires nightly builds.

Building:
```shell
cargo build
```

Running:
```shell
 export GJ_bot_token=123thediscordtoken456
cargo run
```

Tests:
```shell
cargo test
```
