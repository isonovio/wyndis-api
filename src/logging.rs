use tracing_subscriber::EnvFilter;

pub fn init(filter: EnvFilter) {
    tracing_subscriber::fmt().with_env_filter(filter).init();
}
