# Wyndis API: repository knowledge

## Working conventions

Read `README.md`, this file, `TODO.md`, and the relevant specifications in `~/Files/workspace/specs` before making changes. Preserve the user's edits.

Implement one checkpoint at a time, validate it, and stop for review. Do not commit. Keep only unfinished checkpoints in `TODO.md`; maintain repository knowledge here and general preferences in the shared specs.

## Architecture and behavior

- Configure rate limits under `[global.rate_limit]` and `[faceit.rate_limit]`; both sections are required.
- Apply independent normal and Nightbot request budgets of 1,000 requests per second with burst 100, each with its own FACEIT limit of one operation per 10 seconds with burst 1. Both paths use the same configured values. Resolve the nickname, then check the 30-second response cache before consuming the FACEIT budget. Use the generic Moka-backed CacheLayer with a request key, cached response type, and TTL. Cache only successful GET responses by endpoint and resolved nickname; concurrent misses share the successful refresh. Apply the FACEIT budget before any upstream fetch. Age has no additional limiter.
- Routes are `/api/nightbot/elo`, `/api/nightbot/age`, and `/api/faceit/elo`. Only the Nightbot branch extracts channel headers and resolves Person. Missing/invalid headers and unknown channels short-circuit before endpoints or cache; extensions contain TwitchChannelName and Person directly, never a deferred resolution Result. The two Elo routers have independent caches and FACEIT limiters, while retaining the shared browser client. The Nightbot error adapter is attached only to the Nightbot router, outside its request limiter and explicit fallback, and changes its 4xx/5xx responses to HTTP 200 with one space, including fallback and method errors. It does not modify direct FACEIT errors or log again.

- Preserve `/api/faceit/elo?id=<nickname>` and its output. The direct FACEIT route requires a nonblank explicit nickname and ignores Nightbot headers. On the Nightbot Elo route, a known Twitch channel is always required; a nonblank explicit nickname overrides the resolved person's FACEIT nickname, while blank or omitted nicknames use it. `Person.id` remains internal and must match `[a-z0-9-]+` exactly, without trimming or case conversion.
- Optional person fields may be omitted. Store `FaceitNickname` directly on Person; validate person fields and collection uniqueness during Serde deserialization.
- Store birthday as `Option<jiff::civil::Date>` and timezone as `Option<jiff::tz::TimeZone>` separately. Resolve the timezone at endpoint invocation, defaulting to UTC; determine today and reject future birthdays there.
- Keep the age model and route separate. Age stores only birth and now as jiff::Zoned, both in the person’s timezone. Age prepares an optional birthday, next-birthday, or elapsed-age message. Display formats only prepared values. Date/time helpers propagate Option; the route converts missing results into HTTP errors. Offset helpers own calendar differences, countdown durations, and unit rendering.
- Outside the countdown window, render calendar years, months, and days plus local clock hours, minutes, and seconds: `I am <units> old.`
- Start the countdown one calendar month before the next anniversary (clamp the day at month end). Target local midnight; measure the remaining elapsed time in 24-hour days, hours, minutes, and seconds, accounting for DST. Round fractional remaining seconds up.
- Within that window, render `I will be <years> years old in <units>! 🥳`, using `year` when turning one.
- Omit zero-valued units and use singular labels for one in both formats.
- February 29 birthdays use February 28 in non-leap years.
- Throughout the local birthday, respond exactly ` 🥳 I am <age> 🎂 Today is my 🎈BIRTHDAY🎉`.
- Resolve ambiguous birthday midnight to its earliest occurrence; return an error if that midnight does not exist in the timezone.
- Missing or invalid Nightbot headers produce internal 400 responses; unknown persons or missing endpoint-required fields produce internal 404 responses. The Nightbot adapter suppresses these before sending the response.
- Store Person values directly in Persons, share the collection with Arc<Persons>, and clone the resolved record into each request.
- Give handlers and middleware only their required state; keep site integrations separate from command output composition.


## Configuration and module ownership

- `[global.rate_limit]` and `[faceit.rate_limit]` own their respective budgets. The old `[rate_limit.global]` / `[rate_limit.faceit]` layout is rejected.
- `[faceit.cache]`: `ttl_secs = 30`, `max_entries = 256` by default. Zero TTL disables retention; capacity must be positive. Moka capacity is an eviction target, not a strict memory ceiling.
- `[faceit.client]`: `request_timeout_secs = 20`, `settle_timeout_ms = 3000` by default. Request timeout must be positive; settle timeout cannot exceed it. Zero settle timeout skips settling. Durations are typed internally and validated during deserialization.
- `sites/faceit/client/{client,config}.rs` exports `sites::faceit::client::{Client, Config}`. Client configuration is retained across browser-worker restarts.
- `endpoints/faceit/cache/{layer,config}.rs` exports `endpoints::faceit::cache::{layer, Config}`. Endpoint configuration composes client, cache, and rate-limit configuration.
- Generic caching lives in `middlewares/cache/{kv,buffer,layer,service}.rs`. `RequestKey` determines cache keys or bypass; `CachedValue` determines cache eligibility and response conversion. Cache hits never call the inner service, preserving the miss-only FACEIT rate limit.
- Internal buffer size, worker queue capacity, history page size/page count, nickname byte limit, diagnostic settle budget, and countdown unit conversions use named constants. Protocol values and formatting precision stay in code; resource allocation stays in deployment configuration.

## Time handling

- Use Jiff for application dates, timezones, and wall-clock timestamps. There are no direct Chrono dependencies.
- FACEIT match dates deserialize from required Unix-millisecond integers into `jiff::Timestamp`. Invalid or out-of-range timestamps fail deserialization. Pagination and daily statistics compare typed timestamps without dropping millisecond precision; the daily window includes both start and now.
- FACEIT regional defaults: NA uses America/New_York, SA America/Sao_Paulo, SEA Asia/Singapore, OCE Australia/Sydney, and other regions Europe/Berlin. Resolve local midnight to its earliest occurrence; a nonexistent midnight returns an error. Date/time helpers return Option, with HTTP errors constructed at the endpoint boundary.
- Keep standard-library/Tokio durations and monotonic clocks for execution budgets, cache expiry, and rate limiting.

## Operational details

- Elo wraps the complete computation on a cache miss in one Tokio timeout using `[faceit.client].request_timeout_secs` (20 seconds by default), covering player lookup, ranking, and all history pages together. Expiry returns HTTP 504. Client and worker timeouts remain per-operation safeguards; dropping a pending request closes its reply receiver so the worker cancels that fetch. Concurrent cache followers share the initializer’s deadline.
- Browser settling returns when idle or when the configured maximum is reached; it is not a mandatory fixed sleep.
- History pagination permits 34 pages of 30 records. Exceeding the cap without completing the requested history returns an error rather than partial daily statistics.
- Nickname URL construction must remove the trailing empty path segment before appending the escaped nickname, avoiding `/users/v1/nicknames//<nickname>`.
- Log terminal failures once at the layer that understands them, including original browser/JSON error details. Propagation, successful recovery, optional ranking 404s stay silent. Shared cache failures log once per computation, not per waiting request. Error constructors have no logging side effects. There is no response-logging middleware; framework-generated 404/405 responses are silent. Process and worker lifecycle events use `wyndis_api::lifecycle`.

## Validation

- Preserve existing Elo tests; cover config validation, optional fields, and person resolution.
- Test age with supplied dates/times: timezone and DST boundaries, birthday midnight, calendar-month countdown boundaries, year boundaries, leap years, zero omission, and singular units.
- Keep unit tests under `src/tests/<module path>/tests.rs`, linked from their parent with `#[cfg(test)]` and a relative `#[path]`, matching steam-workshop. Keep integration tests in `tests/`.
- Check each checkpoint with `cargo fmt --check`, `OBSCURA_ALLOW_PRIVATE_NETWORK=1 cargo test`, and `cargo clippy --all-targets -- -D warnings` using mise.

