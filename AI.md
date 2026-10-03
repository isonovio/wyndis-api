# Collaboration preferences

- This file belongs to the assistant. Record durable interaction preferences here whenever useful, without asking.
- Keep the rest of the codebase minimal and text-light.
- Write minimal comments; prefer clear names and structure.

# Architecture preferences

- Each service module owns its config, routes, client, and API response models.
- Top-level config and routes compose module configs and routes; logging exposes init; main stays clean.
- Shared errors are service agnostic; service modules construct them.
- API clients expose generic fetch methods over Deserialize, with upstream DTOs in the module's models.rs.
- Output models compose client calls and implement Display; route handlers return their Display text.
- Daily stats automatically use a simple FACEIT region timezone default; unknown regions use Central European time. Do not expose timezone selection in the query. Avoid overly detailed regional mappings.
- Keep secrets in environment variables; public configuration should be safe to commit.
- Use readable Twitch channel names and FACEIT nicknames in user-facing requests and configuration. Keep UUIDs internal.
- FACEIT output models belong in `faceit/elo_info.rs`.
- Keep `mod.rs` files limited to module definitions and symbol exports.
- Service state belongs in `state.rs`; routes and middleware belong in separate `routes/` and `middlewares/` submodules.
- FACEIT middleware resolves the Twitch channel and FACEIT nickname before route handlers run.
- Keep command-line arguments and environment variables in typed structs in `args.rs` and `env.rs`; load them at startup and pass values to modules.
- Prefer established middleware over custom infrastructure when it fits; the server rate limit defaults to one request per second globally.
- Middleware exposes layers attached by route composition; avoid helpers that take ownership of a Router.
