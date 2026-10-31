//! Publisher MQTT yang fail-safe.
//!
//! Prinsip:
//! - Service **tidak pernah** memblokir saat broker mati.
//! - Publish masuk ke queue bounded; jika penuh, pesan di-drop dan dihitung.
//! - Supervisor mengelola koneksi, reconnect, dan perubahan config runtime.
//!
//! Sebelumnya `AsyncClient::publish().await` mengisi channel internal
//! berkapasitas 10; jika broker mati, channel tidak dikuras dan thread
//! publikasi menggantung selamanya.

use crate::config::MqttConfig;
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use std::error::Error;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, watch};
use tracing::{debug, info, warn};

/// Perintah antar service (dipertahankan untuk kompatibilitas).
#[derive(Debug)]
pub enum MqttCommand {
    Reconnect,
    #[allow(dead_code)]
    Stop,
}

/// Statistik publisher yang dibagikan ke semua service.
#[derive(Debug, Clone, Default)]
pub struct MqttStats {
    published: Arc<AtomicU64>,
    dropped: Arc<AtomicU64>,
    connected: Arc<AtomicBool>,
}

impl MqttStats {
    pub fn published(&self) -> u64 {
        self.published.load(Ordering::Relaxed)
    }

    pub fn dropped(&self) -> u64 {
        self.dropped.load(Ordering::Relaxed)
    }

    pub fn connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }
}

#[derive(Debug)]
struct PublishJob {
    topics: Vec<String>,
    payload: Arc<str>,
}

/// MqttManager — fire-and-forget publisher.
#[derive(Clone)]
pub struct MqttManager {
    tx: mpsc::Sender<PublishJob>,
    config_tx: watch::Sender<MqttConfig>,
    stats: MqttStats,
}

impl MqttManager {
    /// Buat manager dan jalankan supervisor di background.
    pub fn spawn(config: MqttConfig) -> Self {
        let (tx, rx) = mpsc::channel(1024);
        let (config_tx, config_rx) = watch::channel(config);
        let stats = MqttStats::default();

        tokio::spawn(supervisor(config_rx, rx, stats.clone()));

        Self {
            tx,
            config_tx,
            stats,
        }
    }

    pub fn config(&self) -> MqttConfig {
        self.config_tx.borrow().clone()
    }

    /// Ubah config koneksi saat runtime; supervisor akan reconnect.
    pub fn set_config(&self, config: MqttConfig) {
        let _ = self.config_tx.send(config);
    }

    pub fn stats(&self) -> MqttStats {
        self.stats.clone()
    }

    /// Enqueue publish. Tidak pernah menunggu broker.
    ///
    /// Jika queue penuh (broker lambat/mati), pesan di-drop dan dihitung
    /// di `stats.dropped`. Pemanggil tetap mendapat `Ok(())` agar pipeline
    /// WebSocket/TCP tidak ikut berhenti.
    pub async fn publish_message(
        &self,
        topics: &[String],
        payload: String,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let job = PublishJob {
            topics: topics.to_vec(),
            payload: Arc::from(payload),
        };

        match self.tx.try_send(job) {
            Ok(()) => Ok(()),
            Err(mpsc::error::TrySendError::Full(_)) => {
                self.stats.dropped.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                Err("MQTT publish queue closed".into())
            }
        }
    }
}

async fn supervisor(
    mut config_rx: watch::Receiver<MqttConfig>,
    mut job_rx: mpsc::Receiver<PublishJob>,
    stats: MqttStats,
) {
    loop {
        let cfg = config_rx.borrow().clone();
        info!(
            host = %cfg.host,
            port = cfg.port,
            client_id = %cfg.client_id,
            "MQTT connecting"
        );

        let mut options =
            MqttOptions::new(cfg.client_id.as_str(), cfg.host.as_str(), cfg.port);
        options.set_keep_alive(Duration::from_secs(cfg.keep_alive_secs));
        if let (Some(username), Some(password)) =
            (cfg.username.as_deref(), cfg.password.as_deref())
        {
            options.set_credentials(username, password);
        }

        // Channel internal kecil: jika eventloop macet, try_publish akan
        // mengembalikan error dan kita menghitung drop, bukan menggantung.
        let (client, mut eventloop) = AsyncClient::new(options, 64);
        let mut config_changed = false;

        loop {
            tokio::select! {
                changed = config_rx.changed() => {
                    if changed.is_err() {
                        return;
                    }
                    info!("MQTT config changed; reconnecting");
                    config_changed = true;
                    break;
                }

                event = eventloop.poll() => {
                    match event {
                        Ok(Event::Incoming(Packet::ConnAck(_))) => {
                            stats.connected.store(true, Ordering::Relaxed);
                            info!("MQTT connected");
                        }
                        Ok(Event::Incoming(Packet::Disconnect)) => {
                            stats.connected.store(false, Ordering::Relaxed);
                            warn!("MQTT disconnected by broker");
                            break;
                        }
                        Ok(_) => {}
                        Err(e) => {
                            stats.connected.store(false, Ordering::Relaxed);
                            warn!(error = %e, "MQTT connection error; retrying");
                            break;
                        }
                    }
                }

                job = job_rx.recv() => {
                    match job {
                        Some(job) => {
                            for topic in &job.topics {
                                match client.try_publish(
                                    topic.as_str(),
                                    QoS::AtLeastOnce,
                                    false,
                                    job.payload.as_bytes(),
                                ) {
                                    Ok(()) => {
                                        stats.published.fetch_add(1, Ordering::Relaxed);
                                    }
                                    Err(e) => {
                                        stats.dropped.fetch_add(1, Ordering::Relaxed);
                                        debug!(
                                            topic = %topic,
                                            error = %e,
                                            "MQTT publish dropped"
                                        );
                                    }
                                }
                            }
                        }
                        None => return,
                    }
                }
            }
        }

        stats.connected.store(false, Ordering::Relaxed);
        if !config_changed {
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }
}
