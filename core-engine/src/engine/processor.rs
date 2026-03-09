use std::pin::Pin;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use tokio::sync::{broadcast, mpsc};
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;
use tonic::{Request, Response, Status};

use crate::models::common::{CapturedPacket, SuspiciousConnection};

pub mod proto {
    tonic::include_proto!("aether");
}

use proto::packet_service_server::{PacketService, PacketServiceServer};
use proto::suspicious_service_server::{SuspiciousService, SuspiciousServiceServer};
use proto::{PacketMessage, SubscribeRequest, SuspiciousConnectionMessage};

pub fn grpc_packet_server(
    broadcast_tx: broadcast::Sender<PacketMessage>,
) -> PacketServiceServer<PacketServiceImpl> {
    PacketServiceServer::new(PacketServiceImpl {
        tx: Arc::new(broadcast_tx),
    })
}

pub fn grpc_suspicious_server(
    broadcast_tx: broadcast::Sender<SuspiciousConnectionMessage>,
) -> SuspiciousServiceServer<SuspiciousServiceImpl> {
    SuspiciousServiceServer::new(SuspiciousServiceImpl {
        tx: Arc::new(broadcast_tx),
    })
}

pub struct PacketServiceImpl {
    tx: Arc<broadcast::Sender<PacketMessage>>,
}

#[tonic::async_trait]
impl PacketService for PacketServiceImpl {
    type SubscribePacketsStream =
        Pin<Box<dyn tokio_stream::Stream<Item = Result<PacketMessage, Status>> + Send>>;

    async fn subscribe_packets(
        &self,
        _req: Request<SubscribeRequest>,
    ) -> Result<Response<Self::SubscribePacketsStream>, Status> {
        let rx = self.tx.subscribe();
        let stream = BroadcastStream::new(rx)
            .filter_map(|res: Result<PacketMessage, _>| res.ok())
            .map(Ok);
        Ok(Response::new(Box::pin(stream)))
    }
}

pub async fn process_packets(
    mut rx: mpsc::Receiver<CapturedPacket>,
    broadcast_tx: broadcast::Sender<PacketMessage>,
) {
    let mut packet_count = 0_u64;
    let mut total_bytes = 0_u64;
    let mut latency_sum = 0_u64;
    let mut latency_max = 0_u64;

    while let Some(p) = rx.recv().await {
        if packet_count.is_multiple_of(500) {
            eprintln!("[core-engine] Processing packet: {} -> {}", p.local_addr, p.remote_addr);
        }
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let latency = now_ms.saturating_sub(p.timestamp);
        latency_sum += latency;
        latency_max = latency_max.max(latency);

        packet_count += 1;
        total_bytes += p.bytes;

        if packet_count.is_multiple_of(500) {
            let avg = latency_sum / 500;
            eprintln!(
                "[aether_core] packets={packet_count} bytes={total_bytes} capture_to_ui_avg={avg}ms max={latency_max}ms"
            );
            latency_sum = 0;
            latency_max = 0;
        }

        let msg = PacketMessage {
            process: p.process.to_string(),
            local_addr: p.local_addr.to_string(),
            remote_addr: p.remote_addr.to_string(),
            protocol: p.protocol.to_string(),
            bytes: p.bytes,
            timestamp: p.timestamp,
            outbound: p.outbound,
        };
        let _ = broadcast_tx.send(msg);
    }
}

pub struct SuspiciousServiceImpl {
    tx: Arc<broadcast::Sender<SuspiciousConnectionMessage>>,
}

#[tonic::async_trait]
impl SuspiciousService for SuspiciousServiceImpl {
    type SubscribeSuspiciousStream =
        Pin<Box<dyn tokio_stream::Stream<Item = Result<SuspiciousConnectionMessage, Status>> + Send>>;

    async fn subscribe_suspicious(
        &self,
        _req: Request<SubscribeRequest>,
    ) -> Result<Response<Self::SubscribeSuspiciousStream>, Status> {
        let rx = self.tx.subscribe();
        let stream = BroadcastStream::new(rx)
            .filter_map(|res: Result<SuspiciousConnectionMessage, _>| res.ok())
            .map(Ok);
        Ok(Response::new(Box::pin(stream)))
    }
}

pub async fn process_suspicious(
    mut rx: mpsc::Receiver<SuspiciousConnection>,
    broadcast_tx: broadcast::Sender<SuspiciousConnectionMessage>,
) {
    while let Some(item) = rx.recv().await {
        let msg = SuspiciousConnectionMessage {
            process: item.process.to_string(),
            remote_addr: item.remote_addr.to_string(),
            reason: item.reason.to_string(),
            score: item.score,
            timestamp: item.timestamp,
        };
        let _ = broadcast_tx.send(msg);
    }
}
