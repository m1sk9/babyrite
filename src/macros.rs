#[macro_export]
macro_rules! measure_time {
    ($block:expr) => {{
        use tracing::info;

        let start = std::time::Instant::now();
        let result = $block;
        let end = start.elapsed();
        info!(
            "Process is complete: {}.{:2}sec",
            end.as_secs(),
            end.subsec_nanos() / 1_000_000
        );
        result
    }};
}
