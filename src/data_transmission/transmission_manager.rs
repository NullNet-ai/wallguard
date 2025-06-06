use crate::cli::Args;
use crate::data_transmission::dump_dir::DumpDir;
use crate::data_transmission::packets::transmitter::transmit_packets;
use crate::data_transmission::resources::transmitter::transmit_system_resources;
use async_channel::Receiver;
use nullnet_libresmon::SystemResources;
use nullnet_libwallguard::WallGuardGrpcInterface;
use nullnet_traffic_monitor::PacketInfo;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

pub(crate) struct TransmissionManager {
    packet_capture: Option<Receiver<PacketInfo>>,
    resource_monitoring: Option<Receiver<SystemResources>>,

    args: Args,
    client: Arc<Mutex<Option<WallGuardGrpcInterface>>>,
    dump_dir: DumpDir,
    token: Arc<RwLock<String>>,
}

impl TransmissionManager {
    pub(crate) fn new(
        args: Args,
        client: Arc<Mutex<Option<WallGuardGrpcInterface>>>,
        dump_dir: DumpDir,
        token: Arc<RwLock<String>>,
    ) -> Self {
        Self {
            packet_capture: None,
            resource_monitoring: None,

            args,
            client,
            dump_dir,
            token,
        }
    }

    pub(crate) fn has_packet_capture(&self) -> bool {
        self.packet_capture.is_some()
    }

    pub(crate) fn has_resource_monitoring(&self) -> bool {
        self.resource_monitoring.is_some()
    }

    pub(crate) fn start_packet_capture(&mut self) {
        if self.packet_capture.is_some() {
            return;
        }
        let monitor_config = nullnet_traffic_monitor::MonitorConfig {
            addr: self.args.addr.clone(),
            snaplen: self.args.snaplen,
        };
        log::info!("Starting packet capture");
        let rx = nullnet_traffic_monitor::monitor_devices(&monitor_config);
        self.packet_capture = Some(rx.clone());
        let args = self.args.clone();
        let token = self.token.clone();
        let dump_dir = self.dump_dir.clone();
        let client = self.client.clone();
        tokio::spawn(async move {
            transmit_packets(rx, args, token, dump_dir, client).await;
        });
    }

    pub(crate) fn start_resource_monitoring(&mut self) {
        if self.resource_monitoring.is_some() {
            return;
        }
        log::info!("Starting resource monitoring");
        let rx = nullnet_libresmon::poll_system_resources(1000);
        self.resource_monitoring = Some(rx.clone());
        let token = self.token.clone();
        let dump_dir = self.dump_dir.clone();
        let client = self.client.clone();
        tokio::spawn(async move {
            transmit_system_resources(rx, token, dump_dir, client).await;
        });
    }

    pub(crate) fn terminate_packet_capture(&mut self) {
        let Some(rx) = &self.packet_capture else {
            return;
        };
        log::info!("Terminating packet capture");
        rx.close();
        self.packet_capture = None;
    }

    pub(crate) fn terminate_resource_monitoring(&mut self) {
        let Some(rx) = &self.resource_monitoring else {
            return;
        };
        log::info!("Terminating resource monitoring");
        rx.close();
        self.resource_monitoring = None;
    }
}
