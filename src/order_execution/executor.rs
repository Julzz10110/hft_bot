
use crate::strategy::Order;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use log::{info, error};
use tokio::time::{timeout, Duration}; // import for timeout

pub struct OrderExecutor {
    address: String,
    protocol: String, // consider an enum for well-defined protocols
    connection_timeout: Duration,
    read_timeout: Duration,
}

impl OrderExecutor {
    pub async fn new(address: &str, protocol: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            address: address.to_string(),
            protocol: protocol.to_string(),
            connection_timeout: Duration::from_secs(5), // set a reasonable default
            read_timeout: Duration::from_secs(5),
        })
    }

    // add a setter for connection and read timeouts for flexibility
    pub fn set_timeouts(&mut self, connection_timeout: Duration, read_timeout: Duration) {
        self.connection_timeout = connection_timeout;
        self.read_timeout = read_timeout;
    }

    pub async fn execute(&mut self, order: Order) -> Result<(), Box<dyn std::error::Error>> {
        info!("Executing order: {:?}", order);

        // connect to exchange with a timeout
        let connect_result = timeout(self.connection_timeout, TcpStream::connect(&self.address)).await;

        let mut stream = match connect_result {
            Ok(Ok(stream)) => stream, // connect succeeded
            Ok(Err(e)) => {
                error!("Failed to connect to exchange: {}", e);
                return Err(e.into());
            }
            Err(_elapsed) => {
                error!("Connection to exchange timed out");
                return Err("Connection timed out".into());
            }
        };



        // format the order into a string (replace with actual FIX/Binary formatting)
        // crucially, implement proper serialization here based on the `protocol`.
        let order_string = self.format_order(&order)?;

        // send the order to the exchange with a timeout
        let write_result = timeout(self.read_timeout, stream.write_all(order_string.as_bytes())).await;

        match write_result {
            Ok(Ok(_)) => {}, // write succeeded
            Ok(Err(e)) => {
                error!("Failed to send order: {}", e);
                return Err(e.into());
            }
            Err(_elapsed) => {
                error!("Timeout sending order");
                return Err("Timeout sending order".into());
            }
        }


        // receive confirmation (dummy response) with a timeout
        let mut buffer = [0; 1024];
        let read_result = timeout(self.read_timeout, stream.read(&mut buffer)).await;

        match read_result {
            Ok(Ok(n)) => {
                let confirmation = String::from_utf8_lossy(&buffer[..n]);
                info!("Order confirmation: {}", confirmation);
            }
            Ok(Err(e)) => {
                error!("Failed to receive confirmation: {}", e);
                return Err(e.into());
            }
            Err(_elapsed) => {
                error!("Timeout receiving confirmation");
                return Err("Timeout receiving confirmation".into());
            }
        }



        Ok(())
    }

    // add order formatting based on protocol
    fn format_order(&self, order: &Order) -> Result<String, Box<dyn std::error::Error>> {
        match self.protocol.as_str() {
            "binary" => {
                // @todo: implement binary serialization here
                Ok(format!("{:?}", order))
            }
            "fix" => {
                // @todo: implement FIX formatting here
                Ok(format!("{:?}", order))
            }
            "json" => {
                match serde_json::to_string(order) {
                    Ok(json_string) => Ok(json_string),
                    Err(e) => Err(e.into()), // convert serde_json::Error to Box<dyn Error>
                }
            }
            _ => {
                Err(format!("Unsupported protocol: {}", self.protocol).into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::{Order, OrderSide};
    use tokio::net::TcpListener;

    async fn start_mock_server() -> Result<u16, Box<dyn std::error::Error>> {
        let listener = TcpListener::bind("127.0.0.1:0").await?; // bind to any available port
        let addr = listener.local_addr()?;
        let port = addr.port();

        tokio::spawn(async move {
            if let Ok((mut socket, _)) = listener.accept().await {
                let mut buf = [0; 1024];
                if let Ok(n) = socket.read(&mut buf).await {
                    println!("Received order: {}", String::from_utf8_lossy(&buf[..n]));
                    socket.write_all(b"Order Accepted").await.unwrap();
                }
            }
        });

        Ok(port)
    }

    #[tokio::test]
    async fn test_order_execution() -> Result<(), Box<dyn std::error::Error>> {
        let port = start_mock_server().await?;
        let address = format!("127.0.0.1:{}", port);
        let mut executor = OrderExecutor::new(&address, "binary").await?;

        let order = Order {
            symbol: "TEST".to_string(),
            price: 100.0,
            quantity: 10,
            side: OrderSide::Buy,
        };

        executor.execute(order).await?; // should send and receive "Order Accepted"
        Ok(())
    }

    #[tokio::test]
    async fn test_order_execution_json() -> Result<(), Box<dyn std::error::Error>> {
        let port = start_mock_server().await?;
        let address = format!("127.0.0.1:{}", port);
        let mut executor = OrderExecutor::new(&address, "json").await?; // use JSON protocol

        let order = Order {
            symbol: "TEST".to_string(),
            price: 100.0,
            quantity: 10,
            side: OrderSide::Buy,
        };

        executor.execute(order).await?; // should send and receive "Order Accepted"
        Ok(())
    }

    #[tokio::test]
    async fn test_connection_timeout() -> Result<(), Box<dyn std::error::Error>> {
        // this test assumes that there is no server listening on port 65535
        let address = "127.0.0.1:65535".to_string(); // an unlikely port
        let mut executor = OrderExecutor::new(&address, "binary").await?;
        executor.set_timeouts(Duration::from_millis(100), Duration::from_secs(5)); // very short timeout

        let order = Order {
            symbol: "TEST".to_string(),
            price: 100.0,
            quantity: 10,
            side: OrderSide::Buy,
        };

        let result = executor.execute(order).await;
        assert!(result.is_err(), "Expected a timeout error");
        Ok(())
    }
}
