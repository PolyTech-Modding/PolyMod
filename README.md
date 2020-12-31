# Poly Mod

This is the Distribution API and Front end for Poly Bridge 2 Mods.
The API is designed in a way thay allows it to be used for mods other than Poly Bridge 2 if someone desires.

## Installation instructions

First steps:

Since one of the dependencies for this only works on Linux, only that OS is supported, so be aware.

You will require PostgreSQL 12 or newer, along with Redis.
Both of these can easily be ran with docker.

```bash
docker run --name polymod-redis -p 6379:6379 redis
docker run --name polymod-pgsql -e POSTGRES_PASSWORD=password1 -d -p 5432:5432 postgres
```

You will also need The Rust Programming Language installed, for that, just follow the instructions [here](https://www.rust-lang.org/tools/install).\
tl;dr `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

You will also need the sqlx cli extension for cargo, which can be obtained running the following command:

```bash
cargo install sqlx-cli
```

Next we will setup the database, for this you will need to run the following commands:

```bash
export DATABASE_URL="postgres://postgres:password1@localhost:5432/polymod"
cargo sqlx database setup
```

Next we will need to prepare some files/folders:

```bash
touch Config.toml
mkdir tmp
echo DATABASE_URL="postgres://postgres:password1@localhost:5432/polymod" > .env
```

And write on the Config.toml file the following base config:

```toml
[debug]
address = "127.0.0.1"
port = 8000
workers = 1
keep_alive = 30
log = "info,sqlx=warn"

# You can generate these with this:
# https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=b5d238039658071a83b21ce6b5e4d99a
secret_key = "" # 32 bytes hex string
iv_key = "" # 16 bytes hex string

# https://discord.com/developers/applications
oauth2_url = "" # must have scopes "identify email guilds"
client_id = 0
client_secret = ""
redirect_uri = "" # must be the same as the oauth2 url redirect uri

redis_uri = "127.0.0.1:6379"
mods_path = "./tmp"

[release]
# same fields as debug
```

and fill in all the required values.

## Updating

When updating, you only ever need to run `git pull` and `cargo sqlx migrate run`

## Running

To run the project, it's just as simple as `cargo run`
