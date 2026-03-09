use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::models::common::{CapturedPacket, SuspiciousConnection};
use compact_str::CompactString;

pub fn spawn_demo_generator(
    packet_tx: mpsc::Sender<CapturedPacket>,
    suspicious_tx: mpsc::Sender<SuspiciousConnection>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        eprintln!("[core-engine] DEMO MODE ENABLED: Generating synthetic traffic...");
        
        let processes = ["chrome.exe", "code.exe", "spotify.exe", "svchost.exe", "powershell.exe", "nmap.exe", "curl.exe"];
        let protocols = ["TCP", "UDP", "TLSv1.3"];
        let ips = ["192.168.1.15", "192.168.1.22", "10.0.0.5"];
        let remote_ips = ["142.250.185.78", "104.21.55.2", "40.114.177.156", "185.199.108.153", "1.1.1.1"];

        let mut counter: u64 = 0;
        eprintln!("[core-engine] ⏳ Generating initial packets...");
        let mut packets_generated = 0;

        loop {
            // Variable delay to feel more natural
            let delay = 50 + (counter % 150);
            sleep(Duration::from_millis(delay)).await; 
            counter = counter.wrapping_add(1);

            let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64;
            
            // Simple pseudo-random selection
            let p_idx = (ts as usize + counter as usize) % processes.len();
            let proc_name = processes[p_idx];
            
            let proto_idx = (ts as usize) % protocols.len();
            let proto = protocols[proto_idx];

            let local_ip = ips[(counter as usize) % ips.len()];
            let remote_ip = remote_ips[(ts as usize / 10) % remote_ips.len()]; // Change remote IP less frequently
            let remote_port = 80 + (ts % 5000) as u16;

            let packet = CapturedPacket {
                process: CompactString::from(proc_name),
                local_addr: CompactString::from(format!("{}:{}", local_ip, 40000 + (counter % 10000))),
                remote_addr: CompactString::from(format!("{}:{}", remote_ip, remote_port)),
                protocol: CompactString::from(proto),
                bytes: 64 + (ts % 1400),
                timestamp: ts,
                outbound: true,
            };

            packets_generated += 1;
            if packets_generated % 10 == 0 {
                eprintln!("[core-engine] Generated demo packet #{} ({} -> {})", packets_generated, packet.local_addr, packet.remote_addr);
            }

            if packet_tx.send(packet.clone()).await.is_err() {
                eprintln!("[core-engine] ERROR: Failed to send demo packet to channel!");
                break;
            }

            // Generate suspicious event occasionally (approx every 10th packet if dangerous process, or random)
            let is_dangerous = proc_name == "nmap.exe" || proc_name == "powershell.exe";
            if is_dangerous && counter.is_multiple_of(10) {
                let reason = if proc_name == "nmap.exe" { "Port Scan Detected" } else { "Suspicious Payload" };
                let suspicious = SuspiciousConnection {
                    process: CompactString::from(proc_name),
                    remote_addr: packet.remote_addr.clone(),
                    reason: CompactString::from(reason),
                    score: 0.95,
                    timestamp: ts,
                };
                let _ = suspicious_tx.send(suspicious).await;
            } else if counter.is_multiple_of(200) {
                 // Random anomaly
                 let suspicious = SuspiciousConnection {
                    process: CompactString::from("unknown_process.exe"),
                    remote_addr: packet.remote_addr.clone(),
                    reason: CompactString::from("Unsigned Binary Connection"),
                    score: 0.75,
                    timestamp: ts,
                };
                let _ = suspicious_tx.send(suspicious).await;
            }
        }
    })
}
