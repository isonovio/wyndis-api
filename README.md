# Wyndis API

## Startup

```env
RUST_LOG=wyndis_api=info
```

```sh
cargo run -- config.toml
```

## Pre-Commit

```sh
cargo fmt --check
OBSCURA_ALLOW_PRIVATE_NETWORK=1 cargo test # allows Obscura to reach the local mock server
cargo clippy --all-targets -- -D warnings
```

## Endpoints

### FACEIT

#### Endpoint `GET /api/faceit/elo?id=donk666`

- Nightbot Command
    - Usage: `!elo` / `!elo donk666`
    - Setup: `$(urlfetch https://api.wynd.is/api/faceit/elo?id=$(querystring))`
    - Result: `LEVEL 10 | 2345 elo | no. 1234 in EU | Today +25 elo 2W-1L | Last Match 13-8 W 92.50adr 1.50kd`

```sh
curl -i 'http://127.0.0.1:3000/api/faceit/elo?id=Qiyarah'
```

```sh
curl -i 'http://127.0.0.1:3000/api/faceit/elo' -H 'Nightbot-Channel: provider=twitch&name=qiyarah'
```
