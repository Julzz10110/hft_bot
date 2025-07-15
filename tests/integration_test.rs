#[cfg(test)]
mod integration {
    use std::process::Command;
    use std::thread;
    use std::time::Duration;
    use std::env;

    #[test]
    fn test_hft_app_with_mock_exchange() -> Result<(), Box<dyn std::error::Error>> {
        // 0. Get the current directory (for debugging)
        let current_dir = env::current_dir()?;
        println!("Current directory: {:?}", current_dir);

        // 1. Run the mock exchange in a separate process
        let mut exchange_process = Command::new("python")
            .arg("mock_exchange.py") // specify the path to the file if it is not in the current directory
            .spawn()?;

        // give the emulator time to start up
        thread::sleep(Duration::from_secs(2));

        // 2. Launch the HFT application
        let mut app_process = Command::new("cargo")
            .arg("run")
            .spawn()?;

        // give the app time to connect and process data
        thread::sleep(Duration::from_secs(5));

        // 3. Check the application and emulator logs
        // (this requires reading log files and analyzing their contents)
        // ... (implement the logic of checking logs) ...
        println!("Integration test completed. Please check the logs for details.");

        // (inside integration::test_hft_app_with_mock_exchange)
        // ...

        // @todo: CORRECT
        // reading the HFT application log file
        // it is assumed that the HFT application writes logs to the file hft_app.log
        let app_log_content = std::fs::read_to_string("hft_app.log")?;

        // checking that the application has sent the order
        assert!(app_log_content.contains("Order sent"));

        // reading the log file of the exchange emulator
        // the emulator is supposed to write logs to the mock_exchange.log file.
        let exchange_log_content = std::fs::read_to_string("mock_exchange.log")?;

        // Checking that the emulator has received and processed the order
        assert!(exchange_log_content.contains("Order received"));
        assert!(exchange_log_content.contains("Trade executed"));

        // ...

        // 4. Terminate the processes
        app_process.kill()?;
        exchange_process.kill()?;

        Ok(())
    }
}

