
use std::collections::BTreeMap; // for sorted orders storing
use std::collections::HashMap;  // for quick search by ID
use ordered_float::OrderedFloat;
use log::debug;
use crate::market_data::parser::Tick;

// order side (buy or sell)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OrderSide {
    Bid,
    Ask,
}

// structure representing an order
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Order {
    pub id: u64,
    pub price: f64,
    pub quantity: u64,
    pub side: OrderSide,
}

// structure representing an order book
pub struct OrderBook {
    // tree of buy orders sorted by price (high to low)
    bids: BTreeMap<OrderedFloat<f64>, Vec<Order>>,
    // tree of sell orders sorted by price (low to high)
    asks: BTreeMap<OrderedFloat<f64>, Vec<Order>>,
    // hash table for quick search of orders by their ID
    orders_by_id: HashMap<u64, Order>,
}

impl OrderBook {
    pub fn new() -> Self {
        OrderBook {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            orders_by_id: HashMap::new(),
        }
    }

    pub fn add_order(&mut self, order: Order) {
        let price = OrderedFloat(order.price);
        let side = order.side;

        match side {
            OrderSide::Bid => {
                self.bids.entry(price).or_insert_with(Vec::new).push(order);
            }
            OrderSide::Ask => {
                self.asks.entry(price).or_insert_with(Vec::new).push(order);
            }
        }
        self.orders_by_id.insert(order.id, order);
        debug!("Added order {} to order book", order.id);
    }

    pub fn remove_order(&mut self, order_id: u64) -> Option<Order> {
        if let Some(order) = self.orders_by_id.remove(&order_id) {
            let price = OrderedFloat(order.price);
            let side = order.side;

            match side {
                OrderSide::Bid => {
                    if let Some(orders) = self.bids.get_mut(&price) {
                        orders.retain(|o| o.id != order_id);
                        if orders.is_empty() {
                            self.bids.remove(&price);
                        }
                    }
                }
                OrderSide::Ask => {
                    if let Some(orders) = self.asks.get_mut(&price) {
                        orders.retain(|o| o.id != order_id);
                        if orders.is_empty() {
                            self.asks.remove(&price);
                        }
                    }
                }
            }
            debug!("Removed order {} from order book", order_id);
            Some(order)
        } else {
            None
        }
    }

    pub fn update_order(&mut self, order_id: u64, new_quantity: u64) -> Option<()> {
        if let Some(order) = self.orders_by_id.get_mut(&order_id) {
            let price = OrderedFloat(order.price);
            let side = order.side;

            match side {
                OrderSide::Bid => {
                    if let Some(orders) = self.bids.get_mut(&price) {
                        for o in orders.iter_mut() {
                            if o.id == order_id {
                                o.quantity = new_quantity;
                                debug!("Updated order {} quantity to {}", order_id, new_quantity);
                                return Some(());
                            }
                        }
                    }
                }
                OrderSide::Ask => {
                    if let Some(orders) = self.asks.get_mut(&price) {
                        for o in orders.iter_mut() {
                            if o.id == order_id {
                                o.quantity = new_quantity;
                                debug!("Updated order {} quantity to {}", order_id, new_quantity);
                                return Some(());
                            }
                        }
                    }
                }
            }
        }
        None
    }

    pub fn process_market_data(&mut self, tick: &Tick) {
        // example of simple implementation:
        // create an order to buy and sell at the current price
        let bid_order = Order {
            id: rand::random(),
            price: tick.price * 0.999,  // slightly below market price
            quantity: tick.volume / 10,
            side: OrderSide::Bid,
        };

        let ask_order = Order {
            id: rand::random(),
            price: tick.price * 1.001,  // slightly higher than market price
            quantity: tick.volume / 10,
            side: OrderSide::Ask,
        };

        self.add_order(bid_order);
        self.add_order(ask_order);

        debug!("Processed market data, order book depth: bids={}, asks={}",
            self.bids.len(), self.asks.len());
    }

    pub fn get_best_bid(&self) -> Option<f64> {
        self.bids.keys().next_back().map(|of| of.0) // Last key (highest price)
    }

    pub fn get_best_ask(&self) -> Option<f64> {
        self.asks.keys().next().map(|of| of.0) // First key (lowest price)
    }

    pub fn get_order(&self, order_id: u64) -> Option<Order> {
        self.orders_by_id.get(&order_id).copied()
    }

    // returns a copy of all buy orders for the given price
    pub fn get_bids_at_price(&self, price: f64) -> Option<Vec<Order>> {
        self.bids.get(&OrderedFloat(price)).map(|orders| orders.clone())
    }

    // returns a copy of all sell orders for the given price
    pub fn get_asks_at_price(&self, price: f64) -> Option<Vec<Order>> {
        self.asks.get(&OrderedFloat(price)).map(|orders| orders.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_book() {
        let mut order_book = OrderBook::new();

        let order1 = Order { id: 1, price: 10.0, quantity: 10, side: OrderSide::Bid };
        let order2 = Order { id: 2, price: 10.5, quantity: 5, side: OrderSide::Bid };
        let order3 = Order { id: 3, price: 11.0, quantity: 8, side: OrderSide::Ask };
        let order4 = Order { id: 4, price: 11.5, quantity: 12, side: OrderSide::Ask };

        order_book.add_order(order1);
        order_book.add_order(order2);
        order_book.add_order(order3);
        order_book.add_order(order4);

        assert_eq!(order_book.get_best_bid(), Some(10.5));
        assert_eq!(order_book.get_best_ask(), Some(11.0));
        assert_eq!(order_book.get_order(1), Some(order1));
        assert_eq!(order_book.get_order(5), None);

        // test update order
        order_book.update_order(1, 20);
        let updated_order1 = order_book.get_order(1).unwrap();
        assert_eq!(updated_order1.quantity, 20);

        // test remove order
        order_book.remove_order(2);
        assert_eq!(order_book.get_order(2), None);
        assert_eq!(order_book.get_best_bid(), Some(10.0));
    }
}