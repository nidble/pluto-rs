# Pluto

Pluto is a RESTful app that can accept "currency" values uploaded by the user. Such values are then converted and stored into a Postgres table.

## Tech

Pluto uses a number of open source projects to work properly:

- [Rust] - A language empowering everyone to build reliable and efficient software
- [Postgres] - The World's Most Advanced Open Source Relational Database
- [Hasura] - Blazing fast, instant realtime GraphQL APIs on your DB
- [SQLx] - The Rust SQL Toolkit 

And of course Pluto itself is open source with a [public repository][pluto] on GitHub.

## TLDR

- After executing the RESTful server a route capable to perform the currency conversion will be exposed at `<basePath>/exchanges` by issuing a `POST` request. 
- The payload must contain the following keys: `currencyFrom`, `currencyTo`, `amountFrom`, `createdAt`. 
- For basic usage and examples please refer to the following sections: [Docker](https://github.com/nidble/pluto-rs#docker), [Curl examples](https://github.com/nidble/pluto-rs#curl-examples).

## Installation

Pluto requires [Rust](https://www.rust-lang.org/) v1.56+ toolchain, [Hasura] v2+, and [Postgres] v14+ to run. 

## Prerequisites

Install the Rust runtime following respective documentation. Then move on repo folder and provide a custom `.env` file, ie:
```sh
cp .env.example .env
```

Now in order to compile/execute the RESTful server, a running instance of Postgres, with the respective and updated schema must be provided. 
This is required because thanks to [SQLx] during compilation steps our code issue a compilation error if our queries are not compatible with the running Postgres Schema. For further information, please refer to [compile-time-verification](https://github.com/launchbadge/sqlx#compile-time-verification)

This step can be accomplished in various ways, I suggest to:
- Performing Migration on the host system and then executing Hasura with Docker:
   * move on `hasura` folder and then start Hasura using [Docker](https://hasura.io/docs/latest/graphql/core/getting-started/docker-simple.html). 
   * then, after installing [Hasura-Cli] you can progress applying the respective [migrations](https://hasura.io/docs/latest/graphql/core/hasura-cli/hasura_migrate.html).
- Performing Migration with Hasura directly from Docker, please refer to [docker-compose.yml](https://github.com/nidble/pluto-rs/blob/master/docker-compose.yml) and below Docker [section](https://github.com/nidble/pluto-rs#docker)

## Server compilation and execution
### for development: 
```sh
cargo run
```

### for production: 
```sh
cargo build --release
./target/release/pluto-rs
```

## Enviromental variables

What follows is a table of principal variables and some examples

| Env Name | Example |
| ------ | ------ |
| POSTGRES_PASSWORD | "CHANGEME"|
| POSTGRES_DB | `pluto_db`|
| DATABASE_URL | `postgres://postgres:pass@localhost:5432/pluto_db` |
| HASURA_GRAPHQL_DATABASE_URL | `postgres://postgres:pass@localhost:5432/pluto_db` |
| HASURA_GRAPHQL_METADATA_DATABASE_URL | `postgres://postgres:pass@localhost:5432/pluto_db` |
| HASURA_GRAPHQL_ENABLE_CONSOLE | `true` |
| HASURA_GRAPHQL_DEV_MODE | `true` |
| HASURA_GRAPHQL_ENABLED_LOG_TYPES | `"startup, http-log, webhook-log, websocket-log, query-log"`|
| HASURA_GRAPHQL_ADMIN_SECRET | "myadminsecretkey"|
| RUST_LOG | "exchanges=info"|

## Testing

For Unit and Integration tests, execute

```sh
cargo test

```

For Integration tests only, execute

```sh
cargo test --test integration

```

## Docker

Pluto can be executed from a container without any further ado. 
Please take care to properly provide a working `.env` file (for an exaustive list please consider: [Env Variables](https://github.com/nidble/pluto#envriomental-variables) ) and then issue:

For development
```sh
docker-compose -f docker-compose.yml -f docker-compose.dev.yml up
```

For testing
```sh
docker-compose -f docker-compose.yml -f docker-compose.test.yml up
```

Eventualy adjust write permissions of upload folder.

## Curl Examples
request a new exchange from EUR to USD:
```sh
curl -X POST -H "Content-Type: application/json" -d '{"currencyFrom": "EUR", "currencyTo": "USD", "amount": 123}' http://localhost:3030/exchanges

```

## License

[**MIT**](https://github.com/nidble/pluto-rs/blob/master/LICENSE)

[//]: # (These are reference links used in the body of this note and get stripped out when the markdown processor does its job. There is no need to format nicely because it shouldn't be seen. Thanks SO - http://stackoverflow.com/questions/4823468/store-comments-in-markdown-syntax)

   [Pluto]: <https://github.com/nidble/pluto-rs>
   [Rust]: <https://www.rust-lang.org>
   [Postgres]: <https://www.postgresql.org>
   [Hasura]: <https://github.com/hasura/graphql-engine>
   [Hasura-Cli]: <https://hasura.io/docs/latest/graphql/core/hasura-cli/index.html>
   [SQLx]: <https://github.com/launchbadge/sqlx>
