use crate::strategy::{Order};
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use log::{info, error};
use std::time::Duration;
use tokio::time::sleep;

use crate::exchange::{binary};

pub struct ExchangeConnection {
    stream: TcpStream,
    address: String,
    protocol: String,
}

impl ExchangeConnection {
    pub async fn new(address: &str, protocol: &str) -> Result<Self, Box<dyn std::error::Error>> {
        info!("Connecting to exchange at: {} using {} protocol", address, protocol);
        let stream = Self::connect(address).await?;
        info!("Connected to exchange.");
        Ok(Self { stream, address: address.to_string(), protocol: protocol.to_string() })
    }

    async fn connect(address: &str) -> Result<TcpStream, Box<dyn std::error::Error>> {
        let mut attempts = 0;
        let max_attempts = 5;
        let mut stream_result: Result<TcpStream, std::io::Error> = TcpStream::connect(address).await;

        while stream_result.is_err() && attempts < max_attempts {
            attempts += 1;
            error!("Failed to connect (attempt {}/{}), retrying...", attempts, max_attempts);
            sleep(Duration::from_secs(2)).await;
            stream_result = TcpStream::connect(address).await;
        }

        match stream_result {
            Ok(stream) => Ok(stream),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn send(&mut self, message: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        self.stream.write_all(message).await?;
        Ok(())
    }

    pub async fn receive(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let mut buffer = [0; 1024];
        let n = self.stream.read(&mut buffer).await?;
        if n == 0 {
            return Err("Connection closed by server".into());
        }
        Ok(String::from_utf8_lossy(&buffer[..n]).to_string())
    }

    // prev
    /*
    pub async fn send_message(&mut self, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        match self.protocol.as_str() {
            "binary" => {
                // Assuming message type 1 for simplicity
                let binary_data = binary::format_binary_message(1, message.as_bytes())?;
                self.send(&binary_data).await?;
            }
            "fix" => {
                // Assuming message is already formatted FIX
                self.send(message.as_bytes()).await?;
            }
            _ => {
                return Err("Invalid protocol specified".into());
            }
        }
        Ok(())
    }
    */

    pub async fn send_order(&mut self, order: &Order) -> anyhow::Result<()> {
        let order_json = serde_json::to_string(order)?;
        let message = format!("PLACE_ORDER {}\n", order_json);
        self.stream.write_all(message.as_bytes()).await?;
        Ok(())
    }
    // prev
    /* 
    pub async fn receive_message(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let data = self.receive().await?;
        match self.protocol.as_str() {
            "binary" => {
                let binary_message = binary::parse_binary_message(data.as_bytes())?;
                Ok(String::from_utf8_lossy(&binary_message.data).to_string())
            }
            "fix" => {
                Ok(data)
            }
            _ => {
                return Err("Invalid protocol specified".into());
            }
        }
    }
    */
    pub async fn send_message(&mut self, message: &str) -> anyhow::Result<()> {
        let msg = format!("{}\n", message);
        self.stream.write_all(msg.as_bytes()).await?;
        Ok(())
    }

    pub async fn receive_message(&mut self) -> anyhow::Result<String> {
        let mut buffer = [0; 1024];
        let n = self.stream.read(&mut buffer).await?;
        Ok(String::from_utf8_lossy(&buffer[..n]).to_string())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    async fn start_mock_server() -> Result<u16, Box<dyn std::error::Error>> {
        let listener = TcpListener::bind("127.0.0.1:0").await?; // bind to any available port
        let addr = listener.local_addr()?;
        let port = addr.port();

        tokio::spawn(async move {
            if let Ok((mut socket, _)) = listener.accept().await {
                let mut buf = [0; 1024];
                if let Ok(n) = socket.read(&mut buf).await {
                    println!("Received: {}", String::from_utf8_lossy(&buf[..n]));
                    socket.write_all(&buf[..n]).await.unwrap(); // echo back
                }
            }
        });

        Ok(port)
    }

    #[tokio::test]
    async fn test_exchange_connection() -> Result<(), Box<dyn std::error::Error>> {
        let port = start_mock_server().await?;
        let address = format!("127.0.0.1:{}", port);
        let mut connection = ExchangeConnection::new(&address, "binary").await?;

        let message = "test message";
        connection.send_message(message).await?;

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let received = connection.receive_message().await?;
        assert_eq!(received, message);

        Ok(())
    }
}