use tracing::log::{log, Level};

pub async fn app_log(level: Level, log: &str) {
    log!(level, "{log}");
}
