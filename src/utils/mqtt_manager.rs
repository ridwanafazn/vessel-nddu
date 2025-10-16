use std::net::TcpStream;
use std::time::Duration;
use std::io;

pub struct MqttManager;

impl MqttManager {
    /// Mengetes koneksi TCP langsung ke IP dan Port yang diberikan.
    /// Ini adalah tes jaringan yang sesungguhnya, bukan hanya validasi opsi.
    /// Fungsi ini sengaja dibuat sinkron (blocking) karena tes harus selesai sebelum API merespon.
    pub fn test_connection(ip: &str, port: u16) -> Result<(), io::Error> {
        // Gabungkan IP dan port menjadi satu alamat string.
        let address = format!("{}:{}", ip, port);
        // Set timeout 3 detik agar API tidak menunggu terlalu lama jika host tidak merespon.
        let timeout = Duration::from_secs(3);

        // TcpStream::connect_timeout akan mencoba membuat koneksi TCP.
        // Jika gagal (karena port ditutup, host tidak ada, firewall, dll.), ia akan mengembalikan Err.
        // Jika berhasil, koneksi akan langsung ditutup saat `_stream` di-drop (keluar dari scope).
        match TcpStream::connect_timeout(&address.parse().expect("Invalid socket address"), timeout) {
            Ok(_stream) => Ok(()), // Koneksi berhasil, kembalikan Ok.
            Err(e) => Err(e),      // Koneksi gagal, teruskan error IO-nya.
        }
    }
}

