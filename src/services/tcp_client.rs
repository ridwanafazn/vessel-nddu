use crate::data::gps_data::{SharedGpsConfig, SharedGpsState};
use crate::data::gyro_data::SharedGyroState;
use crate::data::navigation_data::{SharedDepthState, SharedSpeedState};
use crate::data::weather_data::{SharedTemperatureState, SharedWindState};
use crate::utils::nmea;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::time::{sleep, Duration};
use tokio_util::sync::CancellationToken;

/// Helper function to check if a sentence type is enabled in the configuration
fn is_sentence_enabled(config: &crate::data::gps_data::GpsConfig, sentence_type: &str) -> bool {
    match &config.enabled_sentences {
        None => true, // If not configured, send all (backward compatible)
        Some(sentences) if sentences.is_empty() => true, // Empty list means send all
        Some(sentences) => sentences
            .iter()
            .any(|s| s.eq_ignore_ascii_case(sentence_type)),
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn start_tcp_client(
    gps_state: SharedGpsState,
    gyro_state: SharedGyroState,
    wind_state: SharedWindState,
    temperature_state: SharedTemperatureState,
    speed_state: SharedSpeedState,
    depth_state: SharedDepthState,
    config: SharedGpsConfig,
    shutdown: CancellationToken,
) {
    const RECONNECT_DELAY_SECS: u64 = 5;
    const DEFAULT_UPDATE_RATE_MS: u64 = 1000;

    println!(" TCP client service started");

    loop {
        if shutdown.is_cancelled() {
            break;
        }
        // Read configuration
        let (tcp_enabled, ip, port, update_rate) = {
            let cfg = config.read().unwrap();
            (
                cfg.tcp_enabled.unwrap_or(false),
                cfg.ip.clone(),
                cfg.port,
                cfg.update_rate.unwrap_or(DEFAULT_UPDATE_RATE_MS),
            )
        };

        // Check if TCP streaming is enabled
        if !tcp_enabled {
            println!("TCP streaming is disabled. Waiting...");
            sleep(Duration::from_secs(10)).await;
            continue;
        }

        // Check if IP and port are configured
        let (ip, port) = match (ip, port) {
            (Some(ip), Some(port)) => (ip, port),
            _ => {
                println!("TCP IP or port not configured. Waiting...");
                sleep(Duration::from_secs(10)).await;
                continue;
            }
        };

        let target = format!("{}:{}", ip, port);
        println!("Attempting to connect to TCP target: {}", target);

        // Attempt to connect
        let stream = match TcpStream::connect(&target).await {
            Ok(s) => {
                println!("Successfully connected to {}", target);
                s
            }
            Err(e) => {
                eprintln!(
                    "Failed to connect to {}: {}. Retrying in {} seconds...",
                    target, e, RECONNECT_DELAY_SECS
                );
                sleep(Duration::from_secs(RECONNECT_DELAY_SECS)).await;
                continue;
            }
        };

        // Enter sending loop
        if let Err(e) = send_loop(
            stream,
            gps_state.clone(),
            gyro_state.clone(),
            wind_state.clone(),
            temperature_state.clone(),
            speed_state.clone(),
            depth_state.clone(),
            config.clone(),
            update_rate,
            shutdown.clone(),
        )
        .await
        {
            eprintln!(
                "Connection to {} lost: {}. Reconnecting in {} seconds...",
                target, e, RECONNECT_DELAY_SECS
            );
            sleep(Duration::from_secs(RECONNECT_DELAY_SECS)).await;
        }
    }
}

/// Internal function that handles the sending loop once connected
#[allow(clippy::too_many_arguments)]
async fn send_loop(
    mut stream: TcpStream,
    gps_state: SharedGpsState,
    gyro_state: SharedGyroState,
    wind_state: SharedWindState,
    temperature_state: SharedTemperatureState,
    speed_state: SharedSpeedState,
    depth_state: SharedDepthState,
    config: SharedGpsConfig,
    update_rate_ms: u64,
    shutdown: CancellationToken,
) -> Result<(), String> {
    let mut interval = tokio::time::interval(Duration::from_millis(update_rate_ms));

    loop {
        tokio::select! {
            _ = shutdown.cancelled() => return Ok(()),
            _ = interval.tick() => {}
        }

        // Read all sensor states (skip if not initialized or not running)
        let gps_state = {
            let state_lock = gps_state.read().unwrap();
            match state_lock.as_ref() {
                Some(s) if s.is_running => Some(s.clone()),
                _ => None,
            }
        };
        let gyro_state = {
            let state_lock = gyro_state.read().unwrap();
            match state_lock.as_ref() {
                Some(s) if s.is_running => Some(s.clone()),
                _ => None,
            }
        };
        let wind_state = {
            let state_lock = wind_state.read().unwrap();
            match state_lock.as_ref() {
                Some(s) if s.is_running => Some(s.clone()),
                _ => None,
            }
        };
        let temperature_state = {
            let state_lock = temperature_state.read().unwrap();
            match state_lock.as_ref() {
                Some(s) if s.is_running => Some(s.clone()),
                _ => None,
            }
        };
        let speed_state = {
            let state_lock = speed_state.read().unwrap();
            match state_lock.as_ref() {
                Some(s) if s.is_running => Some(s.clone()),
                _ => None,
            }
        };
        let depth_state = {
            let state_lock = depth_state.read().unwrap();
            match state_lock.as_ref() {
                Some(s) if s.is_running => Some(s.clone()),
                _ => None,
            }
        };

        // Read configuration for enabled sentences and prepare sentences
        let sentences = {
            let cfg = config.read().unwrap();
            let mut sentences = Vec::new();
            if let Some(ref gps) = gps_state {
                if is_sentence_enabled(&cfg, "GPRMC") {
                    sentences.push(gps.to_gprmc());
                }
                if is_sentence_enabled(&cfg, "GPGLL") {
                    sentences.push(gps.to_gpgll());
                }
                if is_sentence_enabled(&cfg, "GPVTG") {
                    sentences.push(gps.to_gpvtg());
                }
                if is_sentence_enabled(&cfg, "GPZDA") {
                    sentences.push(gps.to_gpzda());
                }
            }
            if let Some(ref gyro) = gyro_state {
                if is_sentence_enabled(&cfg, "HEROT") {
                    sentences.push(gyro.to_rot());
                }
                if is_sentence_enabled(&cfg, "HDT") {
                    sentences.push(gyro.to_hdt());
                }
                if is_sentence_enabled(&cfg, "HDG") {
                    sentences.push(gyro.to_hdg());
                }
                if is_sentence_enabled(&cfg, "XDR") {
                    sentences.push(gyro.to_xdr());
                }
            }
            if let Some(ref wind) = wind_state {
                if is_sentence_enabled(&cfg, "WIMWV") {
                    sentences.push(wind.to_wimwv());
                }
                if is_sentence_enabled(&cfg, "WIMWD") {
                    sentences.push(wind.to_wimwd());
                }
            }
            if let Some(ref temp) = temperature_state {
                if is_sentence_enabled(&cfg, "WIMTW") {
                    sentences.push(temp.to_wimtw());
                }
            }
            if let Some(ref speed) = speed_state {
                if is_sentence_enabled(&cfg, "VHW") {
                    sentences.push(speed.to_vhw());
                }
            }
            if let Some(ref depth) = depth_state {
                if is_sentence_enabled(&cfg, "DBT") {
                    sentences.push(depth.to_dbt());
                }
                if is_sentence_enabled(&cfg, "DPT") {
                    sentences.push(depth.to_dpt());
                }
                if is_sentence_enabled(&cfg, "DBK") {
                    sentences.push(depth.to_dbk());
                }
            }
            sentences
        }; // cfg is dropped here before any await

        // Send all enabled sentences
        for sentence in sentences {
            // Jangan pernah mengirim NMEA yang tidak valid ke integrator.
            if !nmea::validate(&sentence) {
                tracing::warn!(sentence = %sentence.trim_end(), "Skipping invalid NMEA sentence");
                continue;
            }
            stream
                .write_all(sentence.as_bytes())
                .await
                .map_err(|e| e.to_string())?;
            stream.flush().await.map_err(|e| e.to_string())?;
        }
    }
}
