mod config;
mod exchange;
mod market_data;
mod strategy;
mod order_execution;
mod logging;

use anyhow::{anyhow, Context, Result};
use chrono::Local;
use exchange::connection::ExchangeConnection;
use log::{info, error, warn, debug, LevelFilter};
use market_data::{
    parser::{MarketDataParser, MarketDataFormat, Tick},
    aggregator::MarketDataAggregator,
    order_book::OrderBook,
};
// use order_execution::executor::OrderExecutor;
use config::{Config};
use strategy::{
    Strategy,
    SimpleMovingAverageStrategy,
    TrendFollowingStrategy,
    risk_management::RiskManager,
    Order, OrderSide,
};
use std::{
    fs::{OpenOptions, File},
    io::Write,
};
use tokio::{
    signal,
    sync::mpsc,
};
use tokio::time::{sleep, Duration, timeout};



#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initializing the configuration and logger
    let config = Config::load("config.toml").context("Failed to load config")?;
    logging::init(&config.logging).context("Failed to initialize logging")?;
    info!("Starting HFT bot with config: {:#?}", config);

    // 2. Connecting to the exchange with timeout
    let mut exchange = match timeout(
        Duration::from_secs(5),
        ExchangeConnection::new(&config.exchange.address, &config.exchange.protocol)
    ).await {
        Ok(Ok(conn)) => conn,
        Ok(Err(e)) => return Err(e)
        .map_err(|e| anyhow::anyhow!("Exchange connection failed: {}", e))?,
        Err(_) => return Err(anyhow!("Exchange connection timeout")),
    };

    // 3. Heartbeat mechanism
    let (tx, mut rx) = mpsc::channel(32);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(config.exchange.heartbeat_interval));
        loop {
            interval.tick().await;
            if tx.send(()).await.is_err() {
                break;
            }
        }
    });

    // 4. Main loop with timeout handling
    let mut market_data_parser = MarketDataParser::new(MarketDataFormat::JSON);
    let mut market_data_aggregator = MarketDataAggregator::new(
        config.market_data.aggregation_window,
        config.market_data.use_wma.then(|| vec![1.0; config.market_data.aggregation_window]),
    );
    // init OrderBook
    let mut order_book = OrderBook::new();
    info!("Order book initialized");
    // init strategy
    let mut strategy: Box<dyn Strategy> = match config.strategy.name.to_lowercase().as_str() {
        "sma" => Box::new(SimpleMovingAverageStrategy::new()),
        "trendfollowing" => Box::new(TrendFollowingStrategy::new(
            config.strategy.long_period,
            config.strategy.short_period,
        )),
        _ => {
            warn!("Unknown strategy, defaulting to SMA");
            Box::new(SimpleMovingAverageStrategy::new())
        }
    };
    let mut risk_manager = RiskManager::new(
        config.risk.max_position_size,
        config.risk.max_loss_per_trade,
        config.risk.stop_loss_percentage,
        config.risk.initial_capital,
    );

    info!("Entering main trading loop");
    let mut is_first_message = true;
    loop {
        tokio::select! {
            // market data processing
            result = timeout(Duration::from_secs(10), exchange.receive_message()) => {
                match result {
                    Ok(Ok(data)) => {
                        if is_first_message {
                            debug!("First message received: {}", data.trim());
                            is_first_message = false;
                        }

                        // skip heartbeat responses
                        if data.trim() != "HEARTBEAT_ACK" {
                            if let Err(e) = process_market_data(
                                &data,
                                &market_data_parser,
                                &mut market_data_aggregator,
                                &mut order_book,
                                &mut *strategy,
                                &mut risk_manager,
                                &mut exchange,
                            ).await {
                                error!("Error processing market data: {}", e);
                            }
                        }
                    }
                    Ok(Err(e)) => error!("Error receiving message: {}", e),
                    Err(_) => {
                        warn!("Market data receive timeout, reconnecting...");
                        exchange = ExchangeConnection::new(&config.exchange.address, "text")
                            .await
                            .map_err(|e| anyhow::anyhow!("Reconnect failed: {}", e))?;
                        exchange.send_message("SUBSCRIBE_MARKET_DATA").await?;
                    }
                }
            },

            // heartbeat
            _ = rx.recv() => {
                if let Err(e) = exchange.send_message("HEARTBEAT").await {
                    error!("Failed to send heartbeat: {}", e);
                } else {
                    debug!("Heartbeat sent");
                }
            },

            // graceful shutdown
            _ = signal::ctrl_c() => {
                info!("Shutting down gracefully");
                break;
            }
        }
    }
    Ok(())
}

// async fn process_market_data(
//     data: &str,
//     parser: &MarketDataParser,
//     strategy: &mut impl Strategy,
//     risk_manager: &mut RiskManager,
//     exchange: &mut ExchangeConnection,
// ) -> anyhow::Result<()> {
//     let tick = parser.parse(data)
//     .map_err(|e| anyhow::anyhow!("Failed to parse market data: {}", e))?;
//     
//     if let Some(mut order) = strategy.evaluate(&tick) {
//         if let Some(approved_order) = risk_manager.evaluate_order(&mut order, tick.price) {
//             match timeout(
//                 Duration::from_secs(5),
//                 exchange.send_order(&approved_order)
//             ).await {
//                 Ok(Ok(_)) => {
//                     risk_manager.update_position(&approved_order);
//                     info!("Order executed successfully: {:?}", approved_order);
//                 },
//                 Ok(Err(e)) => error!("Order execution failed: {}", e),
//                 Err(_) => error!("Order execution timeout"),
//             }
//         }
//     }
//     
//     Ok(())
// }
/*
async fn process_market_data(
    data: &str,
    parser: &MarketDataParser,
    strategy: &mut impl Strategy,
    risk_manager: &mut RiskManager,
    exchange: &mut ExchangeConnection,
) -> anyhow::Result<()> {
    let tick = parser.parse(data)
        .map_err(|e| anyhow::anyhow!("Failed to parse market data: {}", e))?;
    
    if let Some(mut order) = strategy.evaluate(&tick) {
        if let Some(approved_order) = risk_manager.evaluate_order(&mut order, tick.price) {
            let order_msg = format!(
                "PLACE_ORDER {}",
                serde_json::to_string(&approved_order)?
            );
            
            exchange.send_message(&order_msg).await?;
            
            // Получаем ответ от эмулятора
            let response = exchange.receive_message().await?;
            info!("Order response: {}", response);
        }
    }
    Ok(())
}
*/

async fn process_market_data(
    data: &str,
    parser: &MarketDataParser,
    aggregator: &mut MarketDataAggregator,
    order_book: &mut OrderBook,
    strategy: &mut dyn Strategy,
    risk_manager: &mut RiskManager,
    exchange: &mut ExchangeConnection,
) -> Result<()> {
    if data.trim() == "HEARTBEAT_ACK" {
        debug!("Received heartbeat ack");
        return Ok(());
    }

    let tick = parser.parse(data)
    .map_err(|e| anyhow::anyhow!("Failed to parse market data: {}", e))?;
    debug!("Received tick: price={}, volume={}", tick.price, tick.volume);
    
    aggregator.update(&tick);
    
    if let Some(mut order) = strategy.evaluate(&tick) {
        if let Some(approved_order) = risk_manager.evaluate_order(&mut order, tick.price) {
            let order_msg = format!(
                "PLACE_ORDER {}",
                serde_json::to_string(&approved_order)?
            );
            
            debug!("Sending order: {}", order_msg.trim());
            exchange.send_message(&order_msg).await?;
            
            let response = timeout(Duration::from_secs(5), exchange.receive_message()).await??;
            info!("Order response: {}", response.trim());
            
            if response.contains("EXECUTED") {
                risk_manager.update_position(&approved_order);
            }
        }
    }
    
    Ok(())
}

fn init_logging(config: &config::LoggingConfig) -> Result<()> {
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

    let env = env_logger::Env::default()
        .filter_or("HFT_LOG_LEVEL", console_level.to_string());

    let mut builder = env_logger::Builder::from_env(env);
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

    if let Some(ref path) = config.file_path {
        if !path.as_os_str().is_empty() {
            if path.exists() || File::create(path).is_ok() {
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

async fn initialize_exchange_connection(
    address: &str,
    protocol: &str,
    max_retries: u32,
    retry_delay_secs: u64,
) -> Result<ExchangeConnection> {
    let mut attempts = 0;
    let mut last_error = None;

    while attempts < max_retries {
        attempts += 1;
        match ExchangeConnection::new(address, protocol).await {
            Ok(conn) => return Ok(conn),
            Err(e) => {
                last_error = Some(e);
                warn!(
                    "Connection attempt {}/{} failed, retrying in {}s...",
                    attempts, max_retries, retry_delay_secs
                );
                tokio::time::sleep(tokio::time::Duration::from_secs(retry_delay_secs)).await;
            }
        }
    }

    Err(anyhow::anyhow!(
        "Failed to connect after {} attempts. Last error: {}",
        max_retries,
        last_error.unwrap_or_else(|| Box::<dyn std::error::Error>::from("unknown error"))
    ))
}

fn calculate_pnl(order: &Order, tick: &Tick) -> f64 {
    match order.side {
        OrderSide::Buy => (tick.price - order.price) * order.quantity as f64,
        OrderSide::Sell => (order.price - tick.price) * order.quantity as f64,
    }
}