use serde::Deserialize;
use std::{path::PathBuf};
use anyhow::{Context, Result};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub exchange: ExchangeConfig,
    pub market_data: MarketDataConfig,
    pub strategy: StrategyConfig,
    pub risk: RiskConfig,
    pub order_execution: OrderExecutionConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize)]
pub struct ExchangeConfig {
    pub address: String,
    pub protocol: String,
    pub heartbeat_interval: u64,
    pub max_retries: u32,
    pub retry_delay_secs: u64,
}

#[derive(Debug, Deserialize)]
pub struct MarketDataConfig {
    pub format: String,
    pub aggregation_window: usize,
    pub use_wma: bool,
}

#[derive(Debug, Deserialize)]
pub struct StrategyConfig {
    pub name: String,
    pub long_period: usize,    // for TrendFollowingStrategy
    pub short_period: usize,   // for TrendFollowingStrategy
    pub window_size: usize,    // for SimpleMovingAverageStrategy
}

#[derive(Debug, Deserialize)]
pub struct RiskConfig {
    pub max_position_size: u32,
    pub max_loss_per_trade: f64,
    pub stop_loss_percentage: f64,
    pub initial_capital: f64,
}

#[derive(Debug, Deserialize)]
pub struct OrderExecutionConfig {
    pub address: String,
    pub protocol: String,
    pub connection_timeout_secs: u64,
    pub response_timeout_secs: u64,
}

#[derive(Debug, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_console_level")]
    pub console_level: String,
    
    #[serde(default = "default_file_level")]
    pub file_level: String,
    
    #[serde(default)]
    pub file_path: Option<PathBuf>,
}

fn default_console_level() -> String {
    "info".to_string()
}

fn default_file_level() -> String {
    "debug".to_string()
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let config_content = std::fs::read_to_string(path)
            .context(format!("Failed to read config file at {}", path))?;
        
        toml::from_str(&config_content)
            .context(format!("Failed to parse config file at {}", path))
    }
}