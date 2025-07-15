pub mod strategy_1;
pub mod strategy_2;
pub mod risk_management;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub symbol: String,
    pub price: f64,
    pub quantity: u64,
    pub side: OrderSide,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum OrderSide {
    Buy,
    Sell,
}

impl std::fmt::Display for OrderSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderSide::Buy => write!(f, "BUY"),
            OrderSide::Sell => write!(f, "SELL"),
        }
    }
}

pub trait Strategy {
    fn evaluate(&mut self, market_data: &super::market_data::parser::Tick) -> Option<Order>;
}

// re-export strategies for easy access
pub use strategy_1::SimpleMovingAverageStrategy;
pub use strategy_2::TrendFollowingStrategy;