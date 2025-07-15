use super::{Order, OrderSide, Strategy};
use crate::market_data::parser::Tick;
use log::{debug, info};

pub struct SimpleMovingAverageStrategy {
    window_size: usize,
    prices: Vec<f64>,
    symbol: String,
}

impl SimpleMovingAverageStrategy {
    pub fn new() -> Self {
        SimpleMovingAverageStrategy {
            window_size: 10,
            prices: Vec::new(),
            symbol: "SYMBOL".to_string(),
        }
    }
}

impl Strategy for SimpleMovingAverageStrategy {
    fn evaluate(&mut self, market_data: &Tick) -> Option<Order> {
        self.prices.push(market_data.price);
        if self.prices.len() > self.window_size {
            self.prices.remove(0);
        }

        if self.prices.len() < self.window_size {
            debug!("Not enough data yet to calculate SMA. Need {} ticks.", self.window_size);
            return None;
        }

        let sma: f64 = self.prices.iter().sum::<f64>() / self.prices.len() as f64;

        debug!("Tick Price: {}, SMA: {}", market_data.price, sma);

        if market_data.price > sma {
            info!("Generating Buy order: Price={}, SMA={}", market_data.price, sma);
            Some(Order {
                symbol: self.symbol.clone(),
                price: market_data.price,
                quantity: 1,
                side: OrderSide::Buy,
            })
        } else if market_data.price < sma {
            info!("Generating Sell order: Price={}, SMA={}", market_data.price, sma);
            Some(Order {
                symbol: self.symbol.clone(),
                price: market_data.price,
                quantity: 1,
                side: OrderSide::Sell,
            })
        } else {
            debug!("No order generated: Price equals SMA");
            None
        }
    }
}