# Collaboration preferences

- This file belongs to the assistant. Record durable interaction preferences here whenever useful, without asking.
- Never store secrets, credentials, API keys, tokens, or private information in this file. Personal collaboration preferences are okay to record.
- Keep the rest of the codebase minimal and text-light.
- Write minimal comments; prefer clear names and structure.
- When both branches perform actions, prefer matching on a boolean with `true` and `false` arms over an `if`/`else` expression.
- Separate import sections with blank lines: first `std`, second external crates, third crate-level imports, then progressively narrower module scopes toward the current module. Follow the grouping pattern in the user's refactored code.

# Architecture preferences

- Each service module owns its config, routes, client, and API response models.
- Top-level config and routes compose module configs and routes; logging exposes init; main stays clean.
- Shared errors are service agnostic; service modules construct them.
- API clients expose generic fetch methods over Deserialize, with upstream DTOs in the module's models.rs.
- Output models compose model fetch methods and implement Display; route handlers return their Display text.
- Daily stats automatically use a simple FACEIT region timezone default; unknown regions use Central European time. Do not expose timezone selection in the query. Avoid overly detailed regional mappings.
- Keep secrets in environment variables; public configuration should be safe to commit.
- Use readable Twitch channel names and FACEIT nicknames in user-facing requests and configuration. Keep UUIDs internal.
- FACEIT site code belongs in `faceit/sites/faceit/`: client, native models and their fetch implementations in `models.rs`, derived aggregates in `ext.rs`, and composed output in `elo_info.rs`.
- `faceit/sites/nightbot` owns Nightbot header definitions and parsing; `faceit/sites/twitch` owns Twitch names. Use validated site types directly in config.
- Keep `mod.rs` files limited to module definitions and symbol exports.
- Service state belongs in `state.rs`; keep routes in `routes.rs` while there is only one route, and middleware in `middlewares/`.
- Give handlers and middleware only the state they need; nickname middleware gets the channel map, and Elo handlers get the FACEIT client.
- FACEIT middleware resolves the Twitch channel and FACEIT nickname before route handlers run.
- Keep command-line arguments and environment variables in typed structs in `args.rs` and `env.rs`; load them at startup and pass values to modules.
- Prefer established middleware over custom infrastructure when it fits; the server rate limit defaults to one request per second globally.
- Middleware exposes layers attached by route composition; avoid helpers that take ownership of a Router.
- Shared errors provide named constructors for common HTTP statuses to reduce service boilerplate.
- Keep raw multi-provider data provider agnostic; select the provider before converting fields into provider-specific types (for example, Nightbot header names become TwitchChannelName only after matching Twitch).
- Keep error messages short and local to the failure (for example, "Missing Nightbot channel header"); avoid cross-layer advice, retry instructions, and sentence punctuation.
