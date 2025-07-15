use log::{debug, error};

// represents a tick of market data containing price and volume
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Tick {
    pub price: f64,
    pub volume: u64,
}

/// responsible for parsing market data from various formats
pub struct MarketDataParser {
    format: MarketDataFormat,
}

// supported market data formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketDataFormat {
    CSV,
    #[cfg(feature = "json")]
    JSON,
}

impl MarketDataParser {
    pub fn new(format: MarketDataFormat) -> Self {
        MarketDataParser { format }
    }

    // Parses market data from a string into a Tick structure
    pub fn parse(&self, data: &str) -> Result<Tick, Box<dyn std::error::Error>> {
        debug!("Parsing market data: {}", data);
        match self.format {
            MarketDataFormat::CSV => self.parse_csv(data),
            #[cfg(feature = "json")]
            MarketDataFormat::JSON => self.parse_json(data),
        }
    }

    /// parses market data from CSV format
    fn parse_csv(&self, data: &str) -> Result<Tick, Box<dyn std::error::Error>> {
        let parts: Vec<&str> = data.split(',').collect();
        if parts.len() != 2 {
            error!("Invalid CSV market data format: {}", data);
            return Err("Invalid CSV market data format".into());
        }

        let price: f64 = parts[0].parse()?;
        let volume: u64 = parts[1].parse()?;

        Ok(Tick { price, volume })
    }

    /// parses market data from JSON format
    #[cfg(feature = "json")]
    fn parse_json(&self, data: &str) -> Result<Tick, Box<dyn std::error::Error>> {
        #[derive(serde::Deserialize)]
        struct JsonTick {
            price: f64,
            volume: u64,
        }

        match serde_json::from_str::<JsonTick>(data) {
            Ok(json_tick) => Ok(Tick { price: json_tick.price, volume: json_tick.volume }),
            Err(e) => {
                error!("Invalid JSON market data format: {}", data);
                Err(e.into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_csv_tick() {
        let parser = MarketDataParser::new(MarketDataFormat::CSV);
        let data = "100.50,100";
        let tick = parser.parse(data).unwrap();
        assert_eq!(tick.price, 100.50);
        assert_eq!(tick.volume, 100);
    }

    #[test]
    fn test_parse_invalid_csv_tick() {
        let parser = MarketDataParser::new(MarketDataFormat::CSV);
        let data = "100.50"; // missing volume
        let result = parser.parse(data);
        assert!(result.is_err());
    }

    #[cfg(feature = "json")]
    #[test]
    fn test_parse_valid_json_tick() {
        let parser = MarketDataParser::new(MarketDataFormat::JSON);
        let data = r#"{"price":100.50,"volume":100}"#;
        let tick = parser.parse(data).unwrap();
        assert_eq!(tick.price, 100.50);
        assert_eq!(tick.volume, 100);
    }

    #[cfg(feature = "json")]
    #[test]
    fn test_parse_invalid_json_tick() {
        let parser = MarketDataParser::new(MarketDataFormat::JSON);
        let data = r#"{"price":"abc","volume":100}"#; // invalid price
        let result = parser.parse(data);
        assert!(result.is_err());
    }
}
