use tracing_subscriber::EnvFilter;

pub fn init(filter: EnvFilter) {
    tracing_subscriber::fmt().with_env_filter(filter).init();
}

#[cfg(test)]
#[path = "tests/logging/tests.rs"]
mod tests;
