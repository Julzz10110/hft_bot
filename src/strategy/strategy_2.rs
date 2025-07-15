use super::{Order, OrderSide, Strategy};
use crate::market_data::parser::Tick;
use log::info;

pub struct TrendFollowingStrategy {
    long_period: usize,
    short_period: usize,
    long_sma: Vec<f64>,
    short_sma: Vec<f64>,
    long_prices: Vec<f64>,
    short_prices: Vec<f64>,
    position: i32,
}

impl TrendFollowingStrategy {
    pub fn new(long_period: usize, short_period: usize) -> Self {
        TrendFollowingStrategy {
            long_period,
            short_period,
            long_sma: Vec::new(),
            short_sma: Vec::new(),
            long_prices: Vec::new(),
            short_prices: Vec::new(),
            position: 0,
        }
    }

    fn update_sma(&mut self, price: f64) {
        self.long_prices.push(price);
        if self.long_prices.len() > self.long_period {
            self.long_prices.remove(0);
        }
        self.long_sma.push(self.calculate_sma(&self.long_prices));
        if self.long_sma.len() > self.long_period {
            self.long_sma.remove(0);
        }

        self.short_prices.push(price);
        if self.short_prices.len() > self.short_period {
            self.short_prices.remove(0);
        }
        self.short_sma.push(self.calculate_sma(&self.short_prices));
        if self.short_sma.len() > self.short_period {
            self.short_sma.remove(0);
        }
    }

    fn calculate_sma(&self, prices: &[f64]) -> f64 {
        let sum: f64 = prices.iter().sum();
        sum / prices.len() as f64
    }
}

impl Strategy for TrendFollowingStrategy {
    fn evaluate(&mut self, market_data: &Tick) -> Option<Order> {
        self.update_sma(market_data.price);

        if self.long_sma.len() < self.long_period || self.short_sma.len() < self.short_period {
            return None;
        }

        let long_trend = self.long_sma.last().unwrap();
        let short_trend = self.short_sma.last().unwrap();

        if short_trend > long_trend && self.position <= 0 {
            self.position = 1;
            info!("TrendFollowing: Buy signal");
            Some(Order {
                symbol: "AAPL".to_string(),
                price: market_data.price,
                quantity: 10,
                side: OrderSide::Buy,
            })
        } else if short_trend < long_trend && self.position >= 0 {
            self.position = -1;
            info!("TrendFollowing: Sell signal");
            Some(Order {
                symbol: "AAPL".to_string(),
                price: market_data.price,
                quantity: 10,
                side: OrderSide::Sell,
            })
        } else {
            None
        }
    }
}