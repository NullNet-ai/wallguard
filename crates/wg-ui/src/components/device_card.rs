use leptos::prelude::*;
use leptos_router::components::A;
use wg_shared::types::{Device, FirewallKind};

use super::status_badge::StatusBadge;

fn firewall_label(kind: FirewallKind) -> &'static str {
    match kind {
        FirewallKind::PfSense  => "pfSense",
        FirewallKind::OPNSense => "OPNsense",
        FirewallKind::NFTables => "nftables",
        FirewallKind::None     => "Unknown",
    }
}

#[component]
pub fn DeviceCard(device: Device, #[prop(default = false)] connected: bool) -> impl IntoView {
    let id            = device.id.to_string();
    let href          = format!("/devices/{}", id);
    let name          = device.display_name.clone();
    let fw_label      = firewall_label(device.firewall_kind);
    let agent_version = device.agent_version.clone().unwrap_or_else(|| "—".to_string());

    view! {
        <div class="device-card">
            <A href=href>
                <div class="device-card__header">
                    <span class="device-card__name">{name}</span>
                    <StatusBadge connected=connected />
                </div>
                <div class="device-card__meta">
                    <span class="device-card__fw">{fw_label}</span>
                    <span class="device-card__version">"Agent " {agent_version}</span>
                </div>
            </A>
        </div>
    }
}
