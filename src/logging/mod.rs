use log::LevelFilter;
use chrono::Local;
use std::fs::{OpenOptions, File};
use std::io::Write;
use std::path::Path;
use crate::config::LoggingConfig;

pub fn init(config: &LoggingConfig) -> anyhow::Result<()> {
    let console_level = match config.console_level.to_lowercase().as_str() {
        "trace" => LevelFilter::Trace,
        "debug" => LevelFilter::Debug,
        "info" => LevelFilter::Info,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Info,
    };

    let file_level = match config.file_level.to_lowercase().as_str() {
        "trace" => LevelFilter::Trace,
        "debug" => LevelFilter::Debug,
        "info" => LevelFilter::Info,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Debug,
    };

    let mut builder = env_logger::Builder::from_default_env();
    
    // Исправленный форматтер
    builder.format(|buf, record| {
        writeln!(
            buf,
            "{} [{}] [{}] - {}",
            Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
            record.level(),
            record.module_path().unwrap_or_default(),
            record.args()
        )
    });

    builder.filter(None, console_level);

    if let Some(ref path) = config.file_path {
        if !path.as_os_str().is_empty() {
            if Path::new(path).exists() || File::create(path).is_ok() {
                let file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)?;

                builder.target(env_logger::Target::Pipe(Box::new(file)))
                    .filter(None, file_level);
            } else {
                return Err(anyhow::anyhow!(
                    "Could not create or open log file at {}",
                    path.display()
                ));
            }
        }
    }

    builder.init();
    Ok(())
}