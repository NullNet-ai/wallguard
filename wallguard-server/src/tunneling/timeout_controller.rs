use crate::{app_context::AppContext, tunneling::tunnel_common::WallguardTunnel};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::Mutex;

pub struct TimeoutController {
    idle_timeout: u64,
    awake_interval: u64,
    tunnels: Arc<Mutex<HashMap<String, WallguardTunnel>>>,
}

impl TimeoutController {
    pub fn new(tunnels: Arc<Mutex<HashMap<String, WallguardTunnel>>>) -> Self {
        const DEFAULT_IDLE_TIMEOUT: u64 = 300; // 5 minutes
        const DEFAULT_AWAKE_INTERVAL: u64 = 30; // 30 seconds

        let idle_timeout = std::env::var("TUNNEL_CONTROLLER_IDLE_TIMEOUT")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(DEFAULT_IDLE_TIMEOUT);

        let awake_interval = std::env::var("TUNNEL_CONTROLLER_AWAKE_INTERVAL")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(DEFAULT_AWAKE_INTERVAL);

        Self {
            idle_timeout,
            awake_interval,
            tunnels,
        }
    }

    pub fn idle_timeout_duration(&self) -> Duration {
        Duration::from_secs(self.idle_timeout)
    }

    pub fn awake_interval_duration(&self) -> Duration {
        Duration::from_secs(self.awake_interval)
    }

    pub fn spawn(self, context: AppContext) {
        tokio::spawn(async move {
            loop {
                let mut expired_ids = Vec::new();

                let lock = self.tunnels.lock().await;

                for (_, tunnel) in lock.iter() {
                    match tunnel {
                        WallguardTunnel::Http(http_tunnel) => {
                            let tun = http_tunnel.lock().await;

                            if tun.data.tunnel_data.last_accessed
                                < Self::cutoff_timestamp(self.idle_timeout_duration())
                            {
                                expired_ids.push(tun.data.tunnel_data.id.clone());
                            }
                        }
                        WallguardTunnel::Ssh(ssh_tunnel) => {
                            let tun = ssh_tunnel.lock().await;

                            if tun.data.tunnel_data.last_accessed
                                < Self::cutoff_timestamp(self.idle_timeout_duration())
                                && !tun.has_active_terminals()
                            {
                                expired_ids.push(tun.data.tunnel_data.id.clone());
                            }
                        }
                        WallguardTunnel::Tty(tty_tunnel) => {
                            let tun = tty_tunnel.lock().await;

                            if tun.data.tunnel_data.last_accessed
                                < Self::cutoff_timestamp(self.idle_timeout_duration())
                                && !tun.has_active_terminals()
                            {
                                expired_ids.push(tun.data.tunnel_data.id.clone());
                            }
                        }
                    };
                }

                drop(lock);

                for tunnel_id in expired_ids {
                    context
                        .tunnels_manager
                        .on_tunnel_terminated(&tunnel_id)
                        .await;
                }

                tokio::time::sleep(self.awake_interval_duration()).await;
            }
        });
    }

    fn cutoff_timestamp(idle_timeout: Duration) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

        let cutoff = now
            .checked_sub(idle_timeout)
            .unwrap_or(Duration::from_secs(0));

        cutoff.as_secs()
    }
}
