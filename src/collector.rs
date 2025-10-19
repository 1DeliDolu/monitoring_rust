use std::{collections::HashMap, thread, time::Duration};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sysinfo::{ProcessStatus, System, Networks, Disks};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSnapshot {
    pub timestamp: i64,
    pub hostname: Option<String>,
    pub uptime_seconds: u64,
    pub cpu_usage_pct: f64,
    pub load_avg_one: Option<f64>,
    pub load_avg_five: Option<f64>,
    pub load_avg_fifteen: Option<f64>,
    pub mem_used_mb: u64,
    pub mem_total_mb: u64,
    pub mem_available_mb: u64,
    pub disks: Vec<DiskUsage>,
    pub network: Vec<NetworkInterfaceUsage>,
    pub top_processes: Vec<ProcessInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskUsage {
    pub name: String,
    pub mount_point: String,
    pub filesystem: String,
    pub total_gb: f64,
    pub used_gb: f64,
    pub used_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterfaceUsage {
    pub name: String,
    pub received_total_bytes: u64,
    pub transmitted_total_bytes: u64,
    pub received_kbps: f64,
    pub transmitted_kbps: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: i64,
    pub name: String,
    pub cpu_pct: f64,
    pub memory_mb: u64,
    pub status: Option<String>,
}

impl Default for SystemSnapshot {
    fn default() -> Self {
        Self {
            timestamp: Utc::now().timestamp(),
            hostname: None,
            uptime_seconds: 0,
            cpu_usage_pct: 0.0,
            load_avg_one: None,
            load_avg_five: None,
            load_avg_fifteen: None,
            mem_used_mb: 0,
            mem_total_mb: 0,
            mem_available_mb: 0,
            disks: Vec::new(),
            network: Vec::new(),
            top_processes: Vec::new(),
        }
    }
}

pub fn collect_snapshot(previous: Option<SystemSnapshot>, interval: Duration) -> SystemSnapshot {
    let mut system = System::new_all();
    let mut networks = Networks::new_with_refreshed_list();
    let disks = Disks::new_with_refreshed_list();
    
    system.refresh_all();

    // Refresh twice for more accurate CPU/process usage figures.
    thread::sleep(Duration::from_millis(200));
    system.refresh_cpu();
    system.refresh_processes();
    networks.refresh();

    let cpu_usage_pct = system.global_cpu_info().cpu_usage() as f64;

    let load = System::load_average();
    let load_avg_one = to_option(load.one);
    let load_avg_five = to_option(load.five);
    let load_avg_fifteen = to_option(load.fifteen);

    let total_memory_kib = system.total_memory();
    let available_memory_kib = system.available_memory();
    let used_memory_kib = total_memory_kib.saturating_sub(available_memory_kib);

    let disk_usage: Vec<DiskUsage> = disks
        .iter()
        .map(|disk| {
            let total_gb = bytes_to_gb(disk.total_space());
            let available_gb = bytes_to_gb(disk.available_space());
            let used_gb = (total_gb - available_gb).max(0.0);
            let used_pct = if total_gb > f64::EPSILON {
                (used_gb / total_gb) * 100.0
            } else {
                0.0
            };
            DiskUsage {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                filesystem: String::from_utf8_lossy(disk.file_system()).into_owned(),
                total_gb,
                used_gb,
                used_pct,
            }
        })
        .collect();

    let previous_network_totals: HashMap<String, (u64, u64)> = previous
        .as_ref()
        .map(|snap| {
            snap.network
                .iter()
                .map(|iface| {
                    (
                        iface.name.clone(),
                        (iface.received_total_bytes, iface.transmitted_total_bytes),
                    )
                })
                .collect()
        })
        .unwrap_or_default();

    let seconds = interval.as_secs_f64().max(0.1);
    let network_usage: Vec<NetworkInterfaceUsage> = networks
        .iter()
        .map(|(name, data)| {
            let received_total = data.total_received();
            let transmitted_total = data.total_transmitted();

            let (received_kbps, transmitted_kbps) = previous_network_totals
                .get(name)
                .map(|(prev_rx, prev_tx)| {
                    let rx_delta = received_total.saturating_sub(*prev_rx);
                    let tx_delta = transmitted_total.saturating_sub(*prev_tx);
                    (
                        bytes_per_second_to_kbps(rx_delta, seconds),
                        bytes_per_second_to_kbps(tx_delta, seconds),
                    )
                })
                .unwrap_or((0.0, 0.0));

            NetworkInterfaceUsage {
                name: name.clone(),
                received_total_bytes: received_total,
                transmitted_total_bytes: transmitted_total,
                received_kbps,
                transmitted_kbps,
            }
        })
        .collect();

    let mut top_processes: Vec<ProcessInfo> = system
        .processes()
        .iter()
        .map(|(pid, process)| ProcessInfo {
            pid: pid.as_u32() as i64,
            name: process.name().to_string(),
            cpu_pct: process.cpu_usage() as f64,
            memory_mb: kib_to_mb(process.memory()),
            status: process_status(process.status()),
        })
        .collect();

    top_processes.sort_by(|a, b| b.cpu_pct.partial_cmp(&a.cpu_pct).unwrap_or(std::cmp::Ordering::Equal));
    top_processes.truncate(10);

    SystemSnapshot {
        timestamp: Utc::now().timestamp(),
        hostname: System::host_name(),
        uptime_seconds: System::uptime(),
        cpu_usage_pct,
        load_avg_one,
        load_avg_five,
        load_avg_fifteen,
        mem_used_mb: kib_to_mb(used_memory_kib),
        mem_total_mb: kib_to_mb(total_memory_kib),
        mem_available_mb: kib_to_mb(available_memory_kib),
        disks: disk_usage,
        network: network_usage,
        top_processes,
    }
}

fn kib_to_mb(value: u64) -> u64 {
    (value as f64 / 1024.0).round() as u64
}

fn bytes_to_gb(value: u64) -> f64 {
    (value as f64) / (1024.0 * 1024.0 * 1024.0)
}

fn bytes_per_second_to_kbps(bytes: u64, seconds: f64) -> f64 {
    if seconds <= f64::EPSILON {
        return 0.0;
    }
    (bytes as f64 * 8.0) / (seconds * 1024.0)
}

fn process_status(status: ProcessStatus) -> Option<String> {
    match status {
        ProcessStatus::Run => Some("Running".into()),
        ProcessStatus::Sleep => Some("Sleeping".into()),
        ProcessStatus::Stop => Some("Stopped".into()),
        ProcessStatus::Zombie => Some("Zombie".into()),
        ProcessStatus::Idle => Some("Idle".into()),
        ProcessStatus::Dead => Some("Dead".into()),
        ProcessStatus::Tracing => Some("Tracing".into()),
        ProcessStatus::Wakekill => Some("Wakekill".into()),
        ProcessStatus::Waking => Some("Waking".into()),
        ProcessStatus::Parked => Some("Parked".into()),
        ProcessStatus::LockBlocked => Some("LockBlocked".into()),
        ProcessStatus::UninterruptibleDiskSleep => Some("UninterruptibleDiskSleep".into()),
        ProcessStatus::Unknown(_) => None,
    }
}

fn to_option(value: f64) -> Option<f64> {
    if value.is_finite() && value > 0.0 {
        Some(value)
    } else {
        None
    }
}
