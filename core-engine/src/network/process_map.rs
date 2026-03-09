#![allow(dead_code)]
use dashmap::DashMap;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use compact_str::CompactString;

#[cfg(windows)]
use std::ffi::c_void;
#[cfg(windows)]
use windows_sys::Win32::Foundation::{CloseHandle, FALSE};
#[cfg(windows)]
use windows_sys::Win32::NetworkManagement::IpHelper::{
    GetExtendedTcpTable, GetExtendedUdpTable, TCP_TABLE_OWNER_PID_ALL, UDP_TABLE_OWNER_PID,
    MIB_TCPTABLE_OWNER_PID, MIB_UDPTABLE_OWNER_PID, MIB_TCPROW_OWNER_PID, MIB_UDPROW_OWNER_PID,
};
#[cfg(windows)]
use windows_sys::Win32::System::ProcessStatus::GetModuleBaseNameA;
#[cfg(windows)]
use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
#[cfg(windows)]
use windows_sys::Win32::Networking::WinSock::AF_INET;

#[derive(Clone)]
pub struct ProcessCache {
    port_map: Arc<DashMap<(u8, u16), u32>>,
    name_cache: Arc<DashMap<u32, CompactString>>,
}

impl ProcessCache {
    pub fn new() -> Self {
        Self {
            port_map: Arc::new(DashMap::new()),
            name_cache: Arc::new(DashMap::new()),
        }
    }

    pub fn start_background_refresh(&self) {
        let cache = self.clone();
        tokio::spawn(async move {
            loop {
                #[cfg(windows)]
                cache.refresh_windows();
                
                sleep(Duration::from_secs(2)).await;
            }
        });
    }

    pub fn get_process_name(&self, protocol: u8, port: u16) -> Option<CompactString> {
        if let Some(pid) = self.port_map.get(&(protocol, port)) {
            let pid = *pid;
            if let Some(name) = self.name_cache.get(&pid) {
                return Some(name.clone());
            }
            #[cfg(windows)]
            if let Some(name) = get_process_name_by_pid(pid) {
                let name = CompactString::from(name);
                self.name_cache.insert(pid, name.clone());
                return Some(name);
            }
        }
        None
    }

    #[cfg(windows)]
    fn refresh_windows(&self) {
        self.refresh_tcp();
        self.refresh_udp();
    }

    #[cfg(windows)]
    fn refresh_tcp(&self) {
        let mut size = 0;
        unsafe {
            GetExtendedTcpTable(
                std::ptr::null_mut(),
                &mut size,
                FALSE,
                AF_INET as u32,
                TCP_TABLE_OWNER_PID_ALL,
                0,
            );
        }

        let mut buffer = vec![0u8; size as usize];
        let ret = unsafe {
            GetExtendedTcpTable(
                buffer.as_mut_ptr() as *mut c_void,
                &mut size,
                FALSE,
                AF_INET as u32,
                TCP_TABLE_OWNER_PID_ALL,
                0,
            )
        };

        if ret == 0 {
            let table = unsafe { &*(buffer.as_ptr() as *const MIB_TCPTABLE_OWNER_PID) };
            let num_entries = table.dwNumEntries as usize;
            let rows_ptr = std::ptr::addr_of!(table.table) as *const MIB_TCPROW_OWNER_PID;
            let rows = unsafe { std::slice::from_raw_parts(rows_ptr, num_entries) };

            for row in rows {
                let port = u16::from_be(row.dwLocalPort as u16);
                let pid = row.dwOwningPid;
                
                if pid > 0 {
                    self.port_map.insert((6, port), pid);
                    if !self.name_cache.contains_key(&pid) {
                        if let Some(name) = get_process_name_by_pid(pid) {
                            self.name_cache.insert(pid, CompactString::from(name));
                        }
                    }
                }
            }
        }
    }

    #[cfg(windows)]
    fn refresh_udp(&self) {
        let mut size = 0;
        unsafe {
            GetExtendedUdpTable(
                std::ptr::null_mut(),
                &mut size,
                FALSE,
                AF_INET as u32,
                UDP_TABLE_OWNER_PID,
                0,
            );
        }

        let mut buffer = vec![0u8; size as usize];
        let ret = unsafe {
            GetExtendedUdpTable(
                buffer.as_mut_ptr() as *mut c_void,
                &mut size,
                FALSE,
                AF_INET as u32,
                UDP_TABLE_OWNER_PID,
                0,
            )
        };

        if ret == 0 {
            let table = unsafe { &*(buffer.as_ptr() as *const MIB_UDPTABLE_OWNER_PID) };
            let num_entries = table.dwNumEntries as usize;
            let rows_ptr = std::ptr::addr_of!(table.table) as *const MIB_UDPROW_OWNER_PID;
            let rows = unsafe { std::slice::from_raw_parts(rows_ptr, num_entries) };

            for row in rows {
                let port = u16::from_be(row.dwLocalPort as u16);
                let pid = row.dwOwningPid;
                
                if pid > 0 {
                    self.port_map.insert((17, port), pid);
                    if !self.name_cache.contains_key(&pid) {
                        if let Some(name) = get_process_name_by_pid(pid) {
                            self.name_cache.insert(pid, CompactString::from(name));
                        }
                    }
                }
            }
        }
    }
}

#[cfg(windows)]
fn get_process_name_by_pid(pid: u32) -> Option<String> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, FALSE, pid);
        if handle.is_null() {
            return None;
        }

        let mut buffer = [0u8; 1024];
        let len = GetModuleBaseNameA(handle, std::ptr::null_mut(), buffer.as_mut_ptr(), buffer.len() as u32);
        CloseHandle(handle);

        if len > 0 {
            let name_slice = &buffer[..len as usize];
            if let Ok(name) = std::str::from_utf8(name_slice) {
                return Some(name.to_string());
            }
        }
    }
    None
}
