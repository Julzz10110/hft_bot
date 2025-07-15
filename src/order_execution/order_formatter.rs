use crate::strategy::{Order, OrderSide};
use std::collections::HashMap;

pub struct OrderFormatter {
    // may contain configuration specific to the exchange protocol
}

impl OrderFormatter {
    pub fn new() -> Self {
        OrderFormatter {}
    }

    pub fn format_order(&self, order: &Order, protocol: &str) -> Result<String, Box<dyn std::error::Error>> {
        match protocol {
            "fix" => self.format_fix_order(order),
            "binary" => self.format_binary_order(order),
            _ => Err("Unsupported protocol".into()),
        }
    }

    fn format_fix_order(&self, order: &Order) -> Result<String, Box<dyn std::error::Error>> {
        // Example of FIX message formatting (very simplified).
        // In a real application, need to use a library to work with FIX.
        let mut fields: HashMap<String, String> = HashMap::new();
        fields.insert("8".to_string(), "FIX.4.2".to_string()); // BeginString
        fields.insert("35".to_string(), "D".to_string()); // MsgType (New Order - Single)
        fields.insert("11".to_string(), "ORDER123".to_string()); // ClOrdID
        fields.insert("54".to_string(), match order.side {
            OrderSide::Buy => "1".to_string(),
            OrderSide::Sell => "2".to_string(),
        }); // Side
        fields.insert("55".to_string(), order.symbol.clone()); // Symbol
        fields.insert("40".to_string(), "2".to_string()); // OrdType (Limit)
        fields.insert("44".to_string(), order.price.to_string()); // Price
        fields.insert("38".to_string(), order.quantity.to_string()); // OrderQty
        fields.insert("59".to_string(), "0".to_string()); // TimeInForce (Good Till Cancel)
    
        // adding mandatory fields (SenderCompID, TargetCompID, MsgSeqNum)
        fields.insert("49".to_string(), "YOUR_SENDER_COMP_ID".to_string());
        fields.insert("56".to_string(), "YOUR_TARGET_COMP_ID".to_string());
        fields.insert("34".to_string(), "1".to_string()); // MsgSeqNum example (must maintain sequence)
    
        let mut message = String::new();
        for (tag, value) in &fields { // borrow fields here
            message.push_str(&format!("{}={}|", tag, value));
        }
    
        // Calculate checksum (example: simple sum of bytes)
        let checksum: u32 = message.bytes().map(|b| b as u32).sum();
        let checksum = (checksum % 256) as u8;
        message.push_str(&format!("10={}|", checksum).to_string());
    
        Ok(message)
    }

    fn format_binary_order(&self, order: &Order) -> Result<String, Box<dyn std::error::Error>> {
        // Example of binary message formatting (very simplified).
        // In a real application, you need to serialize the structure into binary format.
        let message = format!("BINARY_ORDER: symbol={}, price={}, quantity={}, side={:?}",
                              order.symbol, order.price, order.quantity, order.side);
        Ok(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_fix_order() {
        let formatter = OrderFormatter::new();
        let order = Order {
            symbol: "AAPL".to_string(),
            price: 150.0,
            quantity: 10,
            side: OrderSide::Buy,
        };
        let fix_message = formatter.format_order(&order, "fix").unwrap();
        println!("{}", fix_message);
        // In a real test, you need to check that the message complies with the FIX specification.
        assert!(fix_message.contains("AAPL"));
        assert!(fix_message.contains("150"));
    }

    #[test]
    fn test_format_binary_order() {
        let formatter = OrderFormatter::new();
        let order = Order {
            symbol: "GOOG".to_string(),
            price: 2700.0,
            quantity: 5,
            side: OrderSide::Sell,
        };
        let binary_message = formatter.format_order(&order, "binary").unwrap();
        println!("{}", binary_message);
        // In a real test, you need to check that the message complies with the binary protocol specification.
        assert!(binary_message.contains("GOOG"));
        assert!(binary_message.contains(&order.price.to_string())); // clarified statement
    }

    #[test]
    fn test_format_order_unsupported_protocol() {
        let formatter = OrderFormatter::new();
        let order = Order {
            symbol: "MSFT".to_string(),
            price: 300.0,
            quantity: 20,
            side: OrderSide::Buy,
        };
        let result = formatter.format_order(&order, "unsupported");
        assert!(result.is_err());
    }
}
