# RoboStats
A discord bot for retrieving information regarding robootics competitions and teams.

[![server badge][]][server link] [![invite badge][]][invite link] [![build badge][]][build link] 

## Features

The following features are currently supported:
- The ability to look up information about teams on RobotEvents for all program types (basic info, awards, event attendance).
- Integration with [VRC Data Analysis](https://vrc-data-analysis.com/) to pull a VRC team's [TrueSkill](https://www.microsoft.com/en-us/research/project/trueskill-ranking-system/) value and rank.
- The ability to look up articles on the [Purdue Sigbots wiki](wiki.purduesigbots.com/).
- The ability to predict VRC match results, again using [VRC Data Analysis](https://vrc-data-analysis.com/).

Additionally, the following features are in-development or planned:
- The ability to view information about specific events.
- The ability to perform lookups in documentation for other LemLib projects.
- The ability to look up specific VRC game rules (and potentially Q&A responses).

## Development

This bot is written in [Rust](https://www.rust-lang.org/) using the [serenity framework](https://docs.rs/serenity/) for interacting with the Discord API. It is deployed through [Shuttle](https://shuttle.rs/).

### Prerequisites
- Git
- `rustc` and a valid Rust toolchain for your platform's target.
- Command line of your choice.
- A [RobotEvents v2 API bearer token](https://www.robotevents.com/api/v2/).
- A discord application token.

Tokens should be placed in `Secrets.toml` and `Secrets.dev.toml` files in the repository's root directory. Example file:

```toml
DISCORD_TOKEN = ''
ROBOTEVENTS_TOKEN = ''
```

> `Secrets.dev.toml` will be used for local deployments while `Secrets.toml` will be used for deployments to the actual shuttle service.

### Testing the bot on a local instance
This will temporarily deploy the bot using your local machine as a server for testing purposes:
```sh
cargo shuttle run
```

### Deploying the bot to Shuttle
This will deploy the bot to a shuttle project. Doing this will require a shuttle account login and a valid project name in [Shuttle.toml](./Shuttle.toml)
```sh
cargo shuttle deploy
```

## License
This project is licensed under the MIT license. Check [LICENSE](https://github.com/LemLib/V5-Sim-//blob/main/LICENSE) for more details.

## Documentation

## Contributing

## Code of Conduct
See the [Code of Conduct](https://github.com/LemLib/robostats/blob/main/.github/CODE_OF_CONDUCT.md) on how to behave like an adult.

[server badge]: https://img.shields.io/badge/discord-join-green?serverId=1094397185141002340&label=Discord%20Server&logo=discord&logoColor=e6edf3&labelColor=30363d&color=2f81f7&style=for-the-badge
[server link]: https://discord.gg/yXPytbW9TC
[invite badge]: https://img.shields.io/badge/Invite-238636?style=for-the-badge&label=Invite%20Bot&labelColor=30363d&color=238636&logo=discord&logoColor=e6edf3
[invite link]: https://discord.com/api/oauth2/authorize?client_id=1181453354677850172&permissions=414464657472&scope=bot+applications.commands
[build badge]: https://img.shields.io/github/actions/workflow/status/LemLib/robostats/rust.yml?style=for-the-badge&labelColor=30363d&logo=rust&logoColor=e6edf3
[build link]: https://github.com/LemLib/robostats/actions
