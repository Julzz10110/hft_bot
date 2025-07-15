# High-Frequency Trading (HFT) Bot

A Rust-based high-frequency trading bot designed for efficient market data processing, strategy execution, and order management. This project provides a modular framework for connecting to exchanges, analyzing market data, and executing trades with risk management controls.

## Features

- **Exchange Connectivity**
  - Supports multiple protocols (Binary, FIX, Text)
  - Configurable retry logic and heartbeat mechanism
  - Asynchronous TCP communication using Tokio

- **Market Data Processing**
  - Aggregates ticks into moving averages (SMA/WMA)
  - Order book implementation with bid/ask tracking
  - Supports CSV and JSON data formats

- **Trading Strategies**
  - Simple Moving Average (SMA) crossover strategy
  - Trend-following strategy with dual SMA periods
  - Extensible strategy trait for custom implementations

- **Risk Management**
  - Position size limits
  - Maximum loss per trade controls
  - Stop-loss percentage enforcement
  - Capital tracking

- **Order Execution**
  - Protocol-specific formatting (FIX/Binary/JSON)
  - Timeout handling for connections and responses
  - Execution confirmation handling

## Technical Details

- Built with Rust for performance and safety
- Asynchronous I/O using Tokio runtime
- Modular architecture with clear separation of concerns
- Comprehensive logging system with file and console output
- Unit tests for core functionality
- Configurable via TOML configuration file

## Configuration

The bot is configured via `config.toml` with sections for:
- Exchange connection parameters
- Market data processing settings
- Order execution preferences
- Strategy configuration
- Risk management parameters
- Logging preferences

## Getting Started

1. Clone the repository
2. Configure `config.toml` for your environment
3. Build with `cargo build --release`
4. Run with `cargo run --release`

## Dependencies

- Tokio for async runtime
- Serde for serialization
- Logging with env_logger
- QuickFIX for FIX protocol support
- Byteorder for binary message handling

## Testing

The project includes comprehensive unit tests:
```bash
cargo test