use color_eyre::{Result, eyre::eyre};
use tracing::{Level, level_filters::LevelFilter};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use crate::dirs::get_data_dir;

const LOG_ENV: &str = "TARS_LOG_LEVL";

pub fn init(logfile_name: &str, term_out: bool) -> Result<()> {
    let directory = get_data_dir();
    std::fs::create_dir_all(directory.clone())?;
    let log_path = directory.join(logfile_name);
    let log_file = std::fs::File::create(log_path)?;
    let env_filter = EnvFilter::builder().with_default_directive(tracing::Level::INFO.into());
    // If the `RUST_LOG` environment variable is set, use that as the default, otherwise use the
    // value of the `LOG_ENV` environment variable. If the `LOG_ENV` environment variable contains
    // errors, then this will return an error.
    let env_filter = env_filter
        .try_from_env()
        .or_else(|_| env_filter.with_env_var(LOG_ENV.to_owned()).from_env())?;
    let file_subscriber = fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_writer(log_file)
        .with_target(false)
        .with_ansi(false)
        .with_filter(env_filter);

    let term_layer = fmt::layer()
        .compact()
        .with_ansi(true)
        .with_file(true)
        .with_line_number(true)
        .with_filter(LevelFilter::from_level(Level::INFO));

    let res = if term_out {
        tracing_subscriber::registry()
            .with(file_subscriber)
            .with(term_layer)
            .try_init()
    } else {
        tracing_subscriber::registry()
            .with(file_subscriber)
            .try_init()
    };

    res.map_err(|e| eyre!(e))
}
