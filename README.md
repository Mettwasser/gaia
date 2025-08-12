This is a rewrite of my existing Discord Bot [Cephalon 6aia](https://github.com/Mettwasser/6aia), which is written in Python.

The current Bot eats a crazy amount of resources for its size, so I thought Rust is the ideal choice.

WIP, more will be added soon..

# Running Locally
## Prerequisites
```sh
cargo install sqlx-cli

sqlx database create
sqlx migration run
```

Additionally, you need to export an environment variable (or use a `.env` file)
which export the following variables:

```ini
BOT_TOKEN=<YOUR TOKEN HERE>
DATABASE_URL=sqlite://dev.db
```

## Starting the app
```sh
cargo run
```

# Contributing
Currently not much to remember, however when you use `rustfmt`, please add the `+nightly` flag.