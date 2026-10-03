# Wyndis API

## Startup

```sh
cargo run -- config.toml
```

## Pre-Commit

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
```

## Endpoints

### FACEIT

- Endpoint: `GET /api/faceit/elo?id=donk666`
- Nightbot Command
    - Usage: `!elo` / `!elo donk666`
    - Setup: `$(urlfetch https://your-domain.example/api/faceit/elo?id=$(querystring))`
    - Result: `LEVEL 10 | 2345 elo | no. 1234 in EU | Today N/A elo 2W 1L | Last Match 13-8 W 92.50adr 1.50kd`
