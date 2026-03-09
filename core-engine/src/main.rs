use std::env;
use dotenv::dotenv;
use tokio::sync::{broadcast, mpsc};
use tonic::transport::Server;

mod engine;
mod models;
mod network;

use engine::processor::{self, proto::{PacketMessage, SuspiciousConnectionMessage}};
use network::stream::spawn_packet_sniffer;
use network::demo_stream::spawn_demo_generator;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let (packet_tx, packet_rx) = mpsc::channel(4096);
    let (packet_broadcast_tx, _) = broadcast::channel::<PacketMessage>(8192);

    let (suspicious_tx, suspicious_rx) = mpsc::channel(1024);
    let (suspicious_broadcast_tx, _) = broadcast::channel::<SuspiciousConnectionMessage>(4096);

    let packet_svc = processor::grpc_packet_server(packet_broadcast_tx.clone());
    let suspicious_svc = processor::grpc_suspicious_server(suspicious_broadcast_tx.clone());

    let grpc = tokio::spawn(async move {
        let port = env::var("CORE_GRPC_PORT").unwrap_or_else(|_| "50051".to_string());
        let host = env::var("CORE_GRPC_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let addr_str = format!("{}:{}", host, port);
        let addr = addr_str.parse().expect("invalid grpc addr");
        
        eprintln!("[core-engine] gRPC server listening on {addr}");
        Server::builder()
            .tcp_nodelay(true)
            .initial_stream_window_size(2 * 1024 * 1024)
            .initial_connection_window_size(4 * 1024 * 1024)
            .add_service(packet_svc)
            .add_service(suspicious_svc)
            .serve(addr)
            .await
            .expect("grpc server failed");
    });

    let mut tasks = vec![grpc];

    let demo_env = env::var("DEMO_MODE").unwrap_or_else(|_| "unset".to_string());
    let demo_env_clean = demo_env.trim();
    let demo_mode = demo_env_clean == "true" || demo_env_clean == "1";
    
    eprintln!("[core-engine] Env: DEMO_MODE='{}'", demo_mode);

    if demo_mode {
        eprintln!("[core-engine] Starting in DEMO MODE (Synthetic Traffic)");
        tasks.push(spawn_demo_generator(packet_tx, suspicious_tx));
    } else {
        eprintln!("[core-engine] Starting in LIVE CAPTURE MODE (Real Interface)");
        tasks.push(spawn_packet_sniffer(packet_tx, suspicious_tx));
    }

    tasks.push(tokio::spawn(async move {
        processor::process_packets(packet_rx, packet_broadcast_tx).await;
    }));

    tasks.push(tokio::spawn(async move {
        processor::process_suspicious(suspicious_rx, suspicious_broadcast_tx).await;
    }));

    futures_util::future::join_all(tasks).await;
}
