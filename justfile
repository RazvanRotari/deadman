# vim: set ft=make
# code: language=makefile

export RUST_LOG := env_var_or_default("RUST_LOG", "error")
export DATABASE_URL := "sqlite://deadman.sqlite"

default:
    just --list

# bootstrap tools like sqlx-cli, cargo-watch
bootstrap-tools:
    # Install sqlx-cli to manage migrations and database from local environment
    cargo install sqlx-cli --no-default-features --features native-tls,postgres
    # Setup cargo watch to get automatic rebuild on changes behaviour
    cargo install cargo-watch
    # Setup rust-script
    cargo install rust-script
    # Setup cargo-release
    cargo install cargo-release
    # Setup cargo-deny
    cargo install cargo-deny --locked
    # Setup tokio-console
    cargo install --locked tokio-console
    # Setup git-cliff
    cargo install git-cliff
    # Setup cargo-tarpaulin
    cargo install cargo-tarpaulin

# run `cargo sqlx migrate` subcommand (`run` by default)
migrate subcommand="run":
    cargo sqlx migrate {{ subcommand }}  --source=./migrations

# add new up & down migration with the provided description
add-migration description:
    cargo sqlx migrate add -r --source=./migrations {{ description }}

# generate sqlx data for offline mode
for-offline: migrate
    cargo sqlx prepare --workspace -- --tests 

db-fresh:
	touch deadman.sqlite; rm deadman.sqlite ;sqlite3 deadman.sqlite "VACUUM;" ; just migrate

# run development server with Tokio settings
runserver-tokio:
    RUST_LOG="tokio=trace,runtime=trace" RUSTFLAGS="--cfg tokio_unstable" cargo run 

# run development server without Tokio settings
runserver:
    RUST_LOG=debug cargo watch -x "run" -w src --why

check:
    cargo check
    cargo deny check

alias t := test

# run all package tests (broker by default)
test test_name="" :
    RUST_LOG=debug cargo test --color always {{ test_name }} -- --nocapture

test-coverage-report:
    cargo tarpaulin

build-release:
    cargo build --release

release-docker:
    just db-fresh; rm -rf target; docker build -t deadman:v1.0  .

run-docker:
    docker run -e DATABASE_URL="sqlite:///data/deadman.sqlite" -e TELOXIDE_TOKEN="6717277222:AAFA_8FeBkcP5DaZReY4_F-dnPAHmlShK4I" --volume "./data:/data" deadman:v1.0
