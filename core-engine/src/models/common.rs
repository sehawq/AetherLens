use compact_str::CompactString;

#[derive(Debug, Clone)]
pub struct CapturedPacket {
    pub process: CompactString,
    pub local_addr: CompactString,
    pub remote_addr: CompactString,
    pub protocol: CompactString,
    pub bytes: u64,
    pub timestamp: u64,
    pub outbound: bool,
}

#[derive(Debug, Clone)]
pub struct SuspiciousConnection {
    pub process: CompactString,
    pub remote_addr: CompactString,
    pub reason: CompactString,
    pub score: f64,
    pub timestamp: u64,
}
