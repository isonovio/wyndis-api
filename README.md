# Wyndis API

## Startup

```env
FACEIT_API_KEY=
RUST_LOG=wyndis_api=info
```

```sh
cargo run -- config.toml
```

`[rate_limit]` uses `tower_governor` and allows one request per second by default (`requests_per_minute = 60`, `burst = 1`), shared across all routes and callers in one server process. Excess requests receive HTTP 429 with `Retry-After`. Each instance has its own budget. A typical Elo lookup makes five FACEIT calls, with more needed for paginated history.

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
