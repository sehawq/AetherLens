#[cfg(feature = "packet-capture")]
use std::collections::HashSet;
#[cfg(feature = "packet-capture")]
use std::net::IpAddr;
#[cfg(feature = "packet-capture")]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(feature = "packet-capture")]
use compact_str::CompactString;
#[cfg(feature = "packet-capture")]
use pnet::datalink::{self, Channel::Ethernet, Config};
#[cfg(feature = "packet-capture")]
use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
#[cfg(feature = "packet-capture")]
use pnet::packet::ip::IpNextHeaderProtocols;
#[cfg(feature = "packet-capture")]
use pnet::packet::ipv4::Ipv4Packet;
#[cfg(feature = "packet-capture")]
use pnet::packet::ipv6::Ipv6Packet;
#[cfg(feature = "packet-capture")]
use pnet::packet::tcp::TcpPacket;
#[cfg(feature = "packet-capture")]
use pnet::packet::udp::UdpPacket;
#[cfg(feature = "packet-capture")]
use pnet::packet::Packet;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
#[cfg(not(feature = "packet-capture"))]
use tokio::time::{sleep, Duration};

use crate::models::common::{CapturedPacket, SuspiciousConnection};
#[cfg(feature = "packet-capture")]
use crate::network::process_map::ProcessCache;

pub fn spawn_packet_sniffer(
    packet_tx: mpsc::Sender<CapturedPacket>,
    suspicious_tx: mpsc::Sender<SuspiciousConnection>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        #[cfg(all(feature = "packet-capture", feature = "libpcap"))]
        {
            let _ = pcap::Device::list();
        }
        #[cfg(not(feature = "packet-capture"))]
        {
            let _ = (&packet_tx, &suspicious_tx);
            run_sniffer_loop(packet_tx, suspicious_tx).await;
        }
        #[cfg(feature = "packet-capture")]
        {
            let process_cache = ProcessCache::new();
            process_cache.start_background_refresh();
            run_sniffer_loop(packet_tx, suspicious_tx, process_cache).await;
        }
    })
}

#[cfg(feature = "packet-capture")]
async fn run_sniffer_loop(
    packet_tx: mpsc::Sender<CapturedPacket>,
    suspicious_tx: mpsc::Sender<SuspiciousConnection>,
    process_cache: ProcessCache,
) {
    let interfaces = datalink::interfaces();
    
    eprintln!("[core-engine] --- Network Interface Discovery ---");
    for iface in &interfaces {
         eprintln!("[core-engine] Found Interface: Name='{}', Up={}, Loopback={}, IPs={:?}", 
             iface.name, iface.is_up(), iface.is_loopback(), iface.ips);
    }
    eprintln!("[core-engine] -----------------------------------");

    let local_ips = interfaces
        .iter()
        .flat_map(|iface| iface.ips.iter().map(|ip| ip.ip()))
        .collect::<HashSet<IpAddr>>();

    let selected_interface = if let Ok(target_ip) = std::env::var("CAPTURE_INTERFACE_IP") {
        eprintln!("[core-engine] looking for interface with IP: {}", target_ip);
        interfaces
            .into_iter()
            .find(|iface| iface.ips.iter().any(|ip| ip.ip().to_string() == target_ip))
    } else {
        // Default behavior: prefer Wi-Fi or Ethernet
        let mut candidates: Vec<_> = interfaces.into_iter().filter(|iface| !iface.is_loopback() && !iface.ips.is_empty()).collect();
        
        // Sort to prefer "Wi-Fi" or "Ethernet" in name if possible (Windows specific heuristic)
        candidates.sort_by(|a, b| {
            let a_score = if a.name.to_lowercase().contains("wi-fi") { 2 } else if a.name.to_lowercase().contains("ethernet") { 1 } else { 0 };
            let b_score = if b.name.to_lowercase().contains("wi-fi") { 2 } else if b.name.to_lowercase().contains("ethernet") { 1 } else { 0 };
            b_score.cmp(&a_score)
        });

        candidates.into_iter().next()
    };

    let Some(interface) = selected_interface else {
        eprintln!("[core-engine] no active interface for packet capture");
        return;
    };

    let cfg = Config {
        read_timeout: Some(std::time::Duration::from_millis(500)),
        promiscuous: false,
        ..Default::default()
    };

    let channel = match datalink::channel(&interface, cfg) {
        Ok(Ethernet(_, rx)) => rx,
        Ok(_) => {
            eprintln!("[apex_core] unsupported datalink channel type");
            return;
        }
        Err(err) => {
            eprintln!("[apex_core] failed to open datalink channel: {err}");
            return;
        }
    };

    eprintln!("[apex_core] packet capture running on {}", interface.name);

    tokio::task::spawn_blocking(move || {
        let mut rx = channel;
        eprintln!("[apex_core] Capture loop started. Waiting for packets...");
        
        while let Ok(frame) = rx.next() {
            let Some(packet) = parse_packet(frame, &local_ips, &process_cache) else {
                continue;
            };

            if packet_tx.blocking_send(packet.clone()).is_err() {
                break;
            }

            if let Some(suspicious) = score_suspicious(&packet) {
                if suspicious_tx.blocking_send(suspicious).is_err() {
                    break;
                }
            }
        }
    })
    .await
    .ok();
}

#[cfg(not(feature = "packet-capture"))]
async fn run_sniffer_loop(
    _packet_tx: mpsc::Sender<CapturedPacket>,
    _suspicious_tx: mpsc::Sender<SuspiciousConnection>,
) {
    eprintln!("[apex_core] packet capture disabled (enable feature 'packet-capture')");
    loop {
        sleep(Duration::from_secs(30)).await;
    }
}

#[cfg(feature = "packet-capture")]
fn parse_packet(frame: &[u8], local_ips: &HashSet<IpAddr>, process_cache: &ProcessCache) -> Option<CapturedPacket> {
    let ethernet = EthernetPacket::new(frame)?;
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_millis() as u64;

    match ethernet.get_ethertype() {
        EtherTypes::Ipv4 => {
            let ipv4 = Ipv4Packet::new(ethernet.payload())?;
            let local = IpAddr::V4(ipv4.get_source());
            let remote = IpAddr::V4(ipv4.get_destination());
            let outbound = local_ips.contains(&local);
            let (protocol, local_addr, remote_addr, local_port) = transport_details_v4(&ipv4, local, remote);

            Some(CapturedPacket {
                process: resolve_process_label(&protocol, local_port, process_cache),
                local_addr,
                remote_addr,
                protocol,
                bytes: frame.len() as u64,
                timestamp: ts,
                outbound,
            })
        }
        EtherTypes::Ipv6 => {
            let ipv6 = Ipv6Packet::new(ethernet.payload())?;
            let local = IpAddr::V6(ipv6.get_source());
            let remote = IpAddr::V6(ipv6.get_destination());
            let outbound = local_ips.contains(&local);
            let (protocol, local_addr, remote_addr, local_port) = transport_details_v6(&ipv6, local, remote);

            Some(CapturedPacket {
                process: resolve_process_label(&protocol, local_port, process_cache),
                local_addr,
                remote_addr,
                protocol,
                bytes: frame.len() as u64,
                timestamp: ts,
                outbound,
            })
        }
        _ => None,
    }
}

#[cfg(feature = "packet-capture")]
fn transport_details_v4(
    ip: &Ipv4Packet<'_>,
    local: IpAddr,
    remote: IpAddr,
) -> (CompactString, CompactString, CompactString, Option<u16>) {
    match ip.get_next_level_protocol() {
        IpNextHeaderProtocols::Tcp => {
            if let Some(tcp) = TcpPacket::new(ip.payload()) {
                return (
                    CompactString::from("TCP"),
                    CompactString::from(format!("{local}:{}", tcp.get_source())),
                    CompactString::from(format!("{remote}:{}", tcp.get_destination())),
                    Some(tcp.get_source()),
                );
            }
            (
                CompactString::from("TCP"),
                CompactString::from(local.to_string()),
                CompactString::from(remote.to_string()),
                None,
            )
        }
        IpNextHeaderProtocols::Udp => {
            if let Some(udp) = UdpPacket::new(ip.payload()) {
                return (
                    CompactString::from("UDP"),
                    CompactString::from(format!("{local}:{}", udp.get_source())),
                    CompactString::from(format!("{remote}:{}", udp.get_destination())),
                    Some(udp.get_source()),
                );
            }
            (
                CompactString::from("UDP"),
                CompactString::from(local.to_string()),
                CompactString::from(remote.to_string()),
                None,
            )
        }
        _ => (
            CompactString::from("IP"),
            CompactString::from(local.to_string()),
            CompactString::from(remote.to_string()),
            None,
        ),
    }
}

#[cfg(feature = "packet-capture")]
fn transport_details_v6(
    ip: &Ipv6Packet<'_>,
    local: IpAddr,
    remote: IpAddr,
) -> (CompactString, CompactString, CompactString, Option<u16>) {
    match ip.get_next_header() {
        IpNextHeaderProtocols::Tcp => {
            if let Some(tcp) = TcpPacket::new(ip.payload()) {
                return (
                    CompactString::from("TCP"),
                    CompactString::from(format!("[{local}]:{}", tcp.get_source())),
                    CompactString::from(format!("[{remote}]:{}", tcp.get_destination())),
                    Some(tcp.get_source()),
                );
            }
            (
                CompactString::from("TCP"),
                CompactString::from(local.to_string()),
                CompactString::from(remote.to_string()),
                None,
            )
        }
        IpNextHeaderProtocols::Udp => {
            if let Some(udp) = UdpPacket::new(ip.payload()) {
                return (
                    CompactString::from("UDP"),
                    CompactString::from(format!("[{local}]:{}", udp.get_source())),
                    CompactString::from(format!("[{remote}]:{}", udp.get_destination())),
                    Some(udp.get_source()),
                );
            }
            (
                CompactString::from("UDP"),
                CompactString::from(local.to_string()),
                CompactString::from(remote.to_string()),
                None,
            )
        }
        _ => (
            CompactString::from("IPV6"),
            CompactString::from(local.to_string()),
            CompactString::from(remote.to_string()),
            None,
        ),
    }
}

#[cfg(feature = "packet-capture")]
fn resolve_process_label(protocol: &str, local_port: Option<u16>, process_cache: &ProcessCache) -> CompactString {
    if let Some(port) = local_port {
        let proto_num = match protocol {
            "TCP" => 6,
            "UDP" => 17,
            _ => 0,
        };
        
        if proto_num > 0 {
             if let Some(name) = process_cache.get_process_name(proto_num, port) {
                 return name;
             }
        }
    }

    #[cfg(windows)]
    {
        // If we have a port but couldn't resolve it, try to return pending info
        if local_port.is_some() {
             // Optional: Return "System" or "Unknown" as requested, 
             // but keeping useful debug info might be better. 
             // User requested: "If not found, default to 'System' or 'Unknown'."
             return CompactString::from("Unknown");
        }
        CompactString::from("System")
    }

    #[cfg(not(windows))]
    {
        let _ = protocol;
        let _ = local_port;
        let _ = process_cache;
        CompactString::from("system")
    }
}

#[cfg(feature = "packet-capture")]
fn score_suspicious(packet: &CapturedPacket) -> Option<SuspiciousConnection> {
    if !packet.outbound {
        return None;
    }

    let port = packet
        .remote_addr
        .rsplit(':')
        .next()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or_default();

    let (reason, score) = match port {
        23 => ("Outbound Telnet attempt", 0.96),
        445 => ("Outbound SMB connection", 0.92),
        3389 => ("Outbound RDP session", 0.85),
        6667 => ("Outbound IRC channel", 0.88),
        _ => return None,
    };

    Some(SuspiciousConnection {
        process: packet.process.clone(),
        remote_addr: packet.remote_addr.clone(),
        reason: CompactString::from(reason),
        score,
        timestamp: packet.timestamp,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use compact_str::CompactString;

    #[test]
    #[cfg(feature = "packet-capture")]
    fn test_score_suspicious_telnet() {
        let packet = CapturedPacket {
            process: CompactString::from("test_process.exe"),
            local_addr: CompactString::from("192.168.1.10:12345"),
            remote_addr: CompactString::from("1.2.3.4:23"),
            protocol: CompactString::from("TCP"),
            bytes: 64,
            timestamp: 123456789,
            outbound: true,
        };

        let result = score_suspicious(&packet);
        assert!(result.is_some());
        let sus = result.unwrap();
        assert_eq!(sus.reason, "Outbound Telnet attempt");
        assert_eq!(sus.score, 0.96);
    }

    #[test]
    #[cfg(feature = "packet-capture")]
    fn test_score_suspicious_safe_port() {
        let packet = CapturedPacket {
            process: CompactString::from("chrome.exe"),
            local_addr: CompactString::from("192.168.1.10:12345"),
            remote_addr: CompactString::from("8.8.8.8:443"),
            protocol: CompactString::from("TCP"),
            bytes: 128,
            timestamp: 123456789,
            outbound: true,
        };

        let result = score_suspicious(&packet);
        assert!(result.is_none());
    }
}
