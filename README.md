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

### Normal

- Rate limit: 1,000/s, burst 100

#### FACEIT ELO `GET /api/faceit/elo?id=donk666`

- Rate limit: 1/10s on cache misses
- Elo computation timeout: 20s total

```sh
curl -i 'http://127.0.0.1:3000/api/faceit/elo?id=Qiyarah'
```

### Nightbot

- Rate limit: 1,000/s, burst 100
- require a valid header for a configured Twitch channel
- return HTTP 200 with a single space on errors to suppress chat output.

#### FACEIT ELO `GET /api/nightbot/elo`

- Nightbot Command
    - Usage: `!elo` / `!elo donk666`
    - Setup: `$(urlfetch https://api.wynd.is/api/nightbot/elo?id=$(querystring))`
    - Result: `LEVEL 10 | 2345 elo | no. 1234 in EU | Today +25 elo 2W-1L | Last Match 13-8 W 92.50adr 1.50kd`

```sh
curl -i 'http://127.0.0.1:3000/api/nightbot/elo' -H 'Nightbot-Channel: provider=twitch&name=qiyarah'
```

#### Age `GET /api/nightbot/age`

- Nightbot Command
    - Usage: `!age`
    - Nightbot setup: `$(urlfetch https://api.wynd.is/api/nightbot/age)`
    - Result:
        - Normally: `I am 26 years 2 months 3 days 5 hours 6 minutes 7 seconds old.`
        - Within one calendar month of the next birthday: `I will be 27 years old in 1 day 2 hours! 🥳`
        - Birthday: ` 🥳 I am 27 🎂 Today is my 🎈BIRTHDAY🎉`

```sh
curl -i 'http://127.0.0.1:3000/api/nightbot/age' -H 'Nightbot-Channel: provider=twitch&name=qiyarah'
```
