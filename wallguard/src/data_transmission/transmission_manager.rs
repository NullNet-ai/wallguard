use crate::client_data::Platform;
use crate::constants::SNAPLEN;
use crate::data_transmission::packets::transmitter::transmit_packets;
use crate::data_transmission::resources::transmitter::transmit_system_resources;
use crate::data_transmission::sysconfig;
use crate::netinfo::monitor_services;
use crate::wg_server::WGServer;
use crate::{data_transmission::dump_dir::DumpDir, token_provider::TokenProvider};
use async_channel::Receiver;
use nullnet_libresmon::SystemResources;
use nullnet_traffic_monitor::PacketInfo;
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub(crate) struct TransmissionManager {
    packet_capture: Option<Receiver<PacketInfo>>,
    resource_monitoring: Option<Receiver<SystemResources>>,
    sysconf_monitoring: Option<broadcast::Sender<()>>,
    services_monitoring: Option<broadcast::Sender<()>>,

    interface: WGServer,
    dump_dir: DumpDir,
    token_provider: TokenProvider,

    server_addr: String,
    platform: Platform,
}

impl TransmissionManager {
    pub(crate) fn new(
        interface: WGServer,
        dump_dir: DumpDir,
        token_provider: TokenProvider,
        server_addr: String,
        platform: Platform,
    ) -> Self {
        Self {
            packet_capture: None,
            resource_monitoring: None,
            sysconf_monitoring: None,
            services_monitoring: None,

            interface,
            dump_dir,
            token_provider,

            server_addr,
            platform,
        }
    }

    pub(crate) fn has_services_monitoring(&self) -> bool {
        self.services_monitoring.is_some()
    }

    pub(crate) fn has_packet_capture(&self) -> bool {
        self.packet_capture.is_some()
    }

    pub(crate) fn has_resource_monitoring(&self) -> bool {
        self.resource_monitoring.is_some()
    }

    pub(crate) fn has_sysconf_monitoring(&self) -> bool {
        self.sysconf_monitoring.is_some()
    }

    pub(crate) fn start_packet_capture(&mut self) {
        if self.has_packet_capture() {
            return;
        }

        if !self.platform.can_monitor_traffic() {
            log::error!("Platform does not support traffic monitoring");
            return;
        }

        let monitor_config = nullnet_traffic_monitor::MonitorConfig {
            addr: self.server_addr.clone(),
            snaplen: SNAPLEN as i32,
        };
        log::info!("Starting packet capture");
        let rx = nullnet_traffic_monitor::monitor_devices(&monitor_config);
        self.packet_capture = Some(rx.clone());
        let token = self.token_provider.clone();
        let dump_dir = self.dump_dir.clone();
        let interface = self.interface.clone();
        tokio::spawn(async move {
            transmit_packets(rx, token, dump_dir, interface).await;
        });
    }

    pub(crate) fn start_resource_monitoring(&mut self) {
        if self.has_resource_monitoring() {
            return;
        }

        if !self.platform.can_monitor_telemetry() {
            log::error!("Platform does not support telemetry monitoring");
            return;
        }

        log::info!("Starting resource monitoring");
        let rx = nullnet_libresmon::poll_system_resources(1000);
        self.resource_monitoring = Some(rx.clone());
        let token_provider = self.token_provider.clone();
        let dump_dir = self.dump_dir.clone();
        let interface = self.interface.clone();
        tokio::spawn(async move {
            transmit_system_resources(rx, token_provider, dump_dir, interface).await;
        });
    }

    pub(crate) fn start_sysconf_monitroing(&mut self) {
        if self.has_sysconf_monitoring() {
            return;
        }

        if !self.platform.can_monitor_config() {
            log::error!("Platform does not support sysconfig monitoring");
            return;
        }

        let (terminate, _) = broadcast::channel(1);

        let interface = self.interface.clone();
        let platform = self.platform;
        let token_provider = self.token_provider.clone();
        let receiver = terminate.subscribe();

        self.sysconf_monitoring = Some(terminate);

        tokio::spawn(async move {
            sysconfig::watch_sysconfig(
                interface.clone(),
                platform,
                token_provider.clone(),
                receiver,
            )
            .await
        });
    }

    pub(crate) fn start_services_monitoring(&mut self) {
        if self.has_services_monitoring() {
            return;
        }

        let (terminate, _) = broadcast::channel(1);

        let interface = self.interface.clone();
        let token_provider = self.token_provider.clone();
        let mut receiver = terminate.subscribe();

        self.services_monitoring = Some(terminate);

        tokio::spawn(async move {
            tokio::select! {
                _ = receiver.recv() => {},
                _ = monitor_services(interface, token_provider) => {}
            }
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

    pub(crate) fn terminate_sysconfig_monitoring(&mut self) {
        let Some(terminate) = &self.sysconf_monitoring else {
            return;
        };

        log::info!("Terminating sysconf monitoring");
        let _ = terminate.send(());

        self.sysconf_monitoring = None
    }

    pub(crate) fn terminate_services_monitoring(&mut self) {
        let Some(terminate) = &self.services_monitoring else {
            return;
        };

        log::info!("Terminating sysconf monitoring");
        let _ = terminate.send(());

        self.services_monitoring = None
    }
}
