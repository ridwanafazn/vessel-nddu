use std::net::TcpStream;
use std::time::Duration;
use std::io;

pub struct MqttManager;

impl MqttManager {
    pub fn test_connection(ip: &str, port: u16) -> Result<(), io::Error> {
        let address = format!("{}:{}", ip, port);
        let timeout = Duration::from_secs(3);

        match TcpStream::connect_timeout(&address.parse().expect("Invalid socket address"), timeout) {
            Ok(_stream) => Ok(()),
            Err(e) => Err(e),
        }
    }
}