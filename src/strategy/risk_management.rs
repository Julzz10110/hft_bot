use crate::strategy::{Order, OrderSide};
use log::{debug, info, warn};

pub struct RiskManager {
    max_position_size: u32, // maximum total position size
    max_loss_per_trade: f64, // maximum loss allowed for a single trade
    stop_loss_percentage: f64, // percentage below entry price to set stop loss
    initial_capital: f64, // starting capital
    current_capital: f64, // current capital after trades
    current_position: i32, // current position (+ for long, - for short)
}

impl RiskManager {
    pub fn new(
        max_position_size: u32,
        max_loss_per_trade: f64,
        stop_loss_percentage: f64,
        initial_capital: f64,
    ) -> Self {
        RiskManager {
            max_position_size,
            max_loss_per_trade,
            stop_loss_percentage,
            initial_capital,
            current_capital: initial_capital,
            current_position: 0,
        }
    }

    /// evaluates an order and adjusts the quantity if necessary based on risk parameters
    pub fn evaluate_order(&self, order: &mut Order, current_price: f64) -> Option<Order> {
        info!("Evaluating order: {:?}", order);

        let mut adjusted_quantity = order.quantity;

        // check max position size
        let potential_position = match order.side {
            OrderSide::Buy => self.current_position + order.quantity as i32,
            OrderSide::Sell => self.current_position - order.quantity as i32,
        };

        if potential_position.abs() as u32 > self.max_position_size {
            adjusted_quantity = if order.side == OrderSide::Buy {
                (self.max_position_size as i32 - self.current_position).max(0) as u64 // how many can we buy?
            } else {
                (self.max_position_size as i32 + self.current_position).max(0) as u64 // how many can we sell?
            };

            warn!("Order quantity reduced to {} due to max position size", adjusted_quantity);
        }

        // check max loss per trade (simplified)
        let potential_loss = (current_price * adjusted_quantity as f64) * self.stop_loss_percentage;
        if potential_loss > self.max_loss_per_trade {
            warn!("Order rejected due to max loss per trade");
            return None;
        }

        if adjusted_quantity == 0 {
            warn!("Order quantity is zero, order rejected.");
            return None; // reject zero quantity orders.
        }

        let mut approved_order = order.clone();
        approved_order.quantity = adjusted_quantity;
        debug!("Approved order: {:?}", approved_order);
        Some(approved_order)
    }

    /// updates the current position based on the executed order
    pub fn update_position(&mut self, order: &Order) {
        match order.side {
            OrderSide::Buy => self.current_position += order.quantity as i32,
            OrderSide::Sell => self.current_position -= order.quantity as i32,
        }
        info!("Position updated to: {}", self.current_position);
    }

    /// updates the current capital based on the profit/loss of the trade
    pub fn update_capital(&mut self, profit: f64) {
        self.current_capital += profit;
        info!("Capital updated to: {}", self.current_capital);
    }

    pub fn get_current_position(&self) -> i32 {
        self.current_position
    }

    pub fn get_current_capital(&self) -> f64 {
        self.current_capital
    }
}