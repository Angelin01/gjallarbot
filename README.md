# gjallarbot
My personal "stuff" Discord bot

## Features

Right now, the bot only does one thing:
- Wake on Lan: one can register machines and their MAC Addresses in the bot, and authorize users and roles to send Wake
  on Lan magic packets to those machines.

## Data Permanence

The bot writes a `data.json` file to its working directory to persist data. It is a very silly, very simple
implementation, that should probably be replaced with something like SQLite should the bot grow any bigger.

## Configuring

The following environment variables are available for configuration:

| Environment Variable | Description                                                                                   | Required                | Example Value        |
|:--------------------:|-----------------------------------------------------------------------------------------------|-------------------------|----------------------|
|  `GJ_DISCORD_TOKEN`  | The bot token for authenticating with Discord's API.                                          | Yes                     | `ABC123XYZ456...`    |
|    `GJ_LOG_LEVEL`    | Sets the log level for the application. See [env_logger's docs][1] for more information.      | No                      | `gjallarbot=debug`   |
| `GJ_APPLICATION_ID`  | The Discord application ID, used for registering commands in a guild for quick debugging.     | No (only in debug mode) | `123456789012345678` |
|    `GJ_GUILD_ID`     | The guild ID to register commands in for debugging purposes. Instant, unlike global commands. | No (only in debug mode) | `987654321098765432` |

[1]: https://github.com/rust-cli/env_logger

## Development



Standard Rust development cycle, requires nightly builds.

Building:
```shell
cargo build
```

Running:
```shell
 export GJ_DISCORD_TOKEN=123thediscordtoken456
cargo run
```

Tests:
```shell
cargo tests
```
