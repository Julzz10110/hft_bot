
use crate::market_data::parser::Tick;
use std::collections::VecDeque;

// struct for aggregating market data such as 
// moving average, total volume, high and low prices.
pub struct MarketDataAggregator {
    // price window for calculating the moving average
    price_window: VecDeque<f64>,
    window_size: usize,
    // simple moving average (SMA).
    sma: Option<f64>,

    total_volume: u64,
    high_price: Option<f64>,
    low_price: Option<f64>,

    // weights for the weighted moving average (WMA). If `None`, a simple SMA is used.
    weights: Option<Vec<f64>>,
}

impl MarketDataAggregator {
    pub fn new(window_size: usize, weights: Option<Vec<f64>>) -> Self {
        // validate weights
        if let Some(ref w) = weights {
            if w.len() != window_size {
                panic!("The number of weights must equal the window size.");
            }
        }

        MarketDataAggregator {
            price_window: VecDeque::new(),
            window_size,
            sma: None,
            total_volume: 0,
            high_price: None,
            low_price: None,
            weights,
        }
    }

    // update aggregated data based on a new tick
    pub fn update(&mut self, tick: &Tick) {
        self.price_window.push_back(tick.price);
        if self.price_window.len() > self.window_size {
            self.price_window.pop_front();
        }
        self.sma = self.calculate_sma();

        self.total_volume += tick.volume;
        self.high_price = match self.high_price {
            Some(high) => Some(high.max(tick.price)),
            None => Some(tick.price),
        };
        self.low_price = match self.low_price {
            Some(low) => Some(low.min(tick.price)),
            None => Some(tick.price),
        };
    }

    fn calculate_sma(&self) -> Option<f64> {
        if self.price_window.is_empty() {
            return None;
        }

        if let Some(ref weights) = self.weights {
            // weighted moving average (WMA)
            let mut weighted_sum = 0.0;
            for (i, &price) in self.price_window.iter().enumerate() {
                weighted_sum += price * weights[i];
            }
            Some(weighted_sum)
        } else {
            // simple moving average (SMA)
            let sum: f64 = self.price_window.iter().sum();
            Some(sum / self.price_window.len() as f64)
        }
    }

    pub fn get_sma(&self) -> Option<f64> {
        self.sma
    }

    pub fn get_total_volume(&self) -> u64 {
        self.total_volume
    }

    pub fn get_high_price(&self) -> Option<f64> {
        self.high_price
    }

    pub fn get_low_price(&self) -> Option<f64> {
        self.low_price
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_data_aggregator_sma() {
        let mut aggregator = MarketDataAggregator::new(3, None); // Simple Moving Average

        let tick1 = Tick { price: 10.0, volume: 100 };
        let tick2 = Tick { price: 11.0, volume: 150 };
        let tick3 = Tick { price: 12.0, volume: 200 };
        let tick4 = Tick { price: 13.0, volume: 250 };

        aggregator.update(&tick1);
        assert_eq!(aggregator.get_sma(), Some(10.0));
        assert_eq!(aggregator.get_total_volume(), 100);
        assert_eq!(aggregator.get_high_price(), Some(10.0));
        assert_eq!(aggregator.get_low_price(), Some(10.0));

        aggregator.update(&tick2);
        assert_eq!(aggregator.get_sma(), Some(10.5));
        assert_eq!(aggregator.get_total_volume(), 250);
        assert_eq!(aggregator.get_high_price(), Some(11.0));
        assert_eq!(aggregator.get_low_price(), Some(10.0));

        aggregator.update(&tick3);
        assert_eq!(aggregator.get_sma(), Some(11.0));
        assert_eq!(aggregator.get_total_volume(), 450);
        assert_eq!(aggregator.get_high_price(), Some(12.0));
        assert_eq!(aggregator.get_low_price(), Some(10.0));

        aggregator.update(&tick4);
        assert_eq!(aggregator.get_sma(), Some(12.0)); // (11 + 12 + 13) / 3
        assert_eq!(aggregator.get_total_volume(), 700);
        assert_eq!(aggregator.get_high_price(), Some(13.0));
        assert_eq!(aggregator.get_low_price(), Some(10.0));
    }

    #[test]
    fn test_market_data_aggregator_wma() {
        // weights: [0.1, 0.3, 0.6]
        let weights = vec![0.1, 0.3, 0.6];
        let mut aggregator = MarketDataAggregator::new(3, Some(weights));

        let tick1 = Tick { price: 10.0, volume: 100 };
        let tick2 = Tick { price: 11.0, volume: 150 };
        let tick3 = Tick { price: 12.0, volume: 200 };
        let tick4 = Tick { price: 13.0, volume: 250 };

        aggregator.update(&tick1);
        assert_eq!(aggregator.get_sma(), Some(10.0)); // only one value

        aggregator.update(&tick2);
        assert_eq!(aggregator.get_sma(), Some(10.7)); // (10 * 0.1 + 11 * 0.3)

        aggregator.update(&tick3);
        assert_eq!(aggregator.get_sma(), Some(11.5)); // (10 * 0.1 + 11 * 0.3 + 12 * 0.6)

        aggregator.update(&tick4);
        //(11 * 0.1 + 12 * 0.3 + 13 * 0.6) = 1.1 + 3.6 + 7.8 = 12.5
        assert_eq!(aggregator.get_sma(), Some(12.5));
    }

    #[test]
    #[should_panic]
    fn test_market_data_aggregator_invalid_weights() {
        let weights = vec![0.1, 0.3]; // incorrect size
        let _aggregator = MarketDataAggregator::new(3, Some(weights));
    }
}
