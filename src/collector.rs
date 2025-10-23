use std::{collections::HashMap, thread, time::Duration};

use chrono::Utc;
use nvml_wrapper::{enum_wrappers::device::TemperatureSensor, Nvml};
use serde::{Deserialize, Serialize};
use sysinfo::{Disks, Networks, ProcessStatus, System};

const TOP_PROCESS_LIMIT: usize = 25;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
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
    pub gpu_usage_pct: Option<f64>,
    pub gpu_memory_usage_pct: Option<f64>,
    pub gpus: Vec<GpuInfo>,
    pub cpu_per_core_usage_pct: Vec<f64>,
    pub cpu_logical_cores: usize,
    pub cpu_physical_cores: Option<usize>,
    pub swap_total_mb: u64,
    pub swap_used_mb: u64,
    pub swap_free_mb: u64,
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
    pub memory_pct: f64,
    pub virtual_memory_mb: u64,
    pub status: Option<String>,
    pub disk_read_bytes_total: u64,
    pub disk_write_bytes_total: u64,
    pub disk_read_kbps: f64,
    pub disk_write_kbps: f64,
    pub thread_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub index: u32,
    pub name: String,
    pub uuid: Option<String>,
    pub gpu_usage_pct: Option<f64>,
    pub memory_used_mb: Option<u64>,
    pub memory_total_mb: Option<u64>,
    pub memory_usage_pct: Option<f64>,
    pub temperature_celsius: Option<f64>,
}

impl Default for SystemSnapshot {
    fn default() -> Self {
        Self {
            timestamp: Utc::now().timestamp_millis(),
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
            gpu_usage_pct: None,
            gpu_memory_usage_pct: None,
            gpus: Vec::new(),
            cpu_per_core_usage_pct: Vec::new(),
            cpu_logical_cores: 0,
            cpu_physical_cores: None,
            swap_total_mb: 0,
            swap_used_mb: 0,
            swap_free_mb: 0,
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
    let cpu_per_core_usage_pct: Vec<f64> = system
        .cpus()
        .iter()
        .map(|cpu| cpu.cpu_usage() as f64)
        .collect();
    let cpu_logical_cores = cpu_per_core_usage_pct.len();
    let cpu_physical_cores = system.physical_core_count();

    let load = System::load_average();
    let load_avg_one = to_option(load.one);
    let load_avg_five = to_option(load.five);
    let load_avg_fifteen = to_option(load.fifteen);

    let total_memory_kib = system.total_memory();
    let available_memory_kib = system.available_memory();
    let used_memory_kib = total_memory_kib.saturating_sub(available_memory_kib);
    let swap_total_kib = system.total_swap();
    let swap_used_kib = system.used_swap();
    let swap_total_mb = kib_to_mb(swap_total_kib);
    let swap_used_mb = kib_to_mb(swap_used_kib);
    let swap_free_mb = swap_total_mb.saturating_sub(swap_used_mb);

    let disk_usage: Vec<DiskUsage> = disks
        .iter()
        .filter(|disk| {
            let mount_point = disk.mount_point().to_string_lossy().to_string();

            // Sadece ana mount point'leri tut (whitelist yaklaşımı)
            mount_point == "/"
                || mount_point == "/home"
                || mount_point == "/mnt/c"
                || mount_point == "/mnt/d"
        })
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
                filesystem: disk.file_system().to_string_lossy().into_owned(),
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

    let previous_process_totals: HashMap<i64, (u64, u64)> = previous
        .as_ref()
        .map(|snap| {
            snap.top_processes
                .iter()
                .map(|proc| {
                    (
                        proc.pid,
                        (proc.disk_read_bytes_total, proc.disk_write_bytes_total),
                    )
                })
                .collect()
        })
        .unwrap_or_default();

    let (gpus, gpu_usage_pct, gpu_memory_usage_pct) = collect_gpu_info();

    let mut top_processes: Vec<ProcessInfo> = system
        .processes()
        .iter()
        .map(|(pid, process)| {
            let pid_i64 = pid.as_u32() as i64;
            let disk_usage = process.disk_usage();
            let (prev_read_total, prev_write_total) = previous_process_totals
                .get(&pid_i64)
                .copied()
                .unwrap_or((disk_usage.total_read_bytes, disk_usage.total_written_bytes));
            let read_delta = disk_usage.total_read_bytes.saturating_sub(prev_read_total);
            let write_delta = disk_usage
                .total_written_bytes
                .saturating_sub(prev_write_total);

            ProcessInfo {
                pid: pid_i64,
                name: process.name().to_string(),
                cpu_pct: process.cpu_usage() as f64,
                memory_mb: kib_to_mb(process.memory()),
                memory_pct: percentage(process.memory(), total_memory_kib),
                virtual_memory_mb: kib_to_mb(process.virtual_memory()),
                status: process_status(process.status()),
                disk_read_bytes_total: disk_usage.total_read_bytes,
                disk_write_bytes_total: disk_usage.total_written_bytes,
                disk_read_kbps: bytes_per_second_to_kbps(read_delta, seconds),
                disk_write_kbps: bytes_per_second_to_kbps(write_delta, seconds),
                thread_count: process.tasks().map(|tasks| tasks.len()),
            }
        })
        .collect();

    top_processes.sort_by(|a, b| {
        b.cpu_pct
            .partial_cmp(&a.cpu_pct)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    top_processes.truncate(TOP_PROCESS_LIMIT);

    SystemSnapshot {
        timestamp: Utc::now().timestamp_millis(),
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
        gpu_usage_pct,
        gpu_memory_usage_pct,
        gpus,
        cpu_per_core_usage_pct,
        cpu_logical_cores,
        cpu_physical_cores,
        swap_total_mb,
        swap_used_mb,
        swap_free_mb,
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

fn percentage(part_kib: u64, total_kib: u64) -> f64 {
    if total_kib == 0 {
        return 0.0;
    }
    (part_kib as f64 / total_kib as f64) * 100.0
}

fn bytes_to_mb(value: u64) -> u64 {
    (value as f64 / (1024.0 * 1024.0)).round() as u64
}

fn collect_gpu_info() -> (Vec<GpuInfo>, Option<f64>, Option<f64>) {
    let nvml = match Nvml::init() {
        Ok(nvml) => nvml,
        Err(_) => return (Vec::new(), None, None),
    };

    let count = match nvml.device_count() {
        Ok(count) => count,
        Err(_) => return (Vec::new(), None, None),
    };

    let mut details = Vec::new();
    let mut usage_total = 0.0;
    let mut usage_count = 0.0;
    let mut mem_usage_total = 0.0;
    let mut mem_usage_count = 0.0;

    for index in 0..count {
        let device = match nvml.device_by_index(index) {
            Ok(device) => device,
            Err(_) => continue,
        };

        let name = device.name().unwrap_or_else(|_| "N/A".to_string());
        let uuid = device.uuid().ok();
        let utilization = device.utilization_rates().ok();
        let memory_info = device.memory_info().ok();
        let temperature = device
            .temperature(TemperatureSensor::Gpu)
            .ok()
            .map(|temp| temp as f64);

        let gpu_usage_pct = utilization.map(|u| u.gpu as f64);
        if let Some(value) = gpu_usage_pct {
            usage_total += value;
            usage_count += 1.0;
        }

        let (memory_used_mb, memory_total_mb, memory_usage_pct) = memory_info
            .map(|info| {
                let total_mb = bytes_to_mb(info.total);
                let used_mb = bytes_to_mb(info.used);
                let usage_pct = if info.total > 0 {
                    (info.used as f64 / info.total as f64) * 100.0
                } else {
                    0.0
                };
                (Some(used_mb), Some(total_mb), Some(usage_pct))
            })
            .unwrap_or((None, None, None));

        if let Some(value) = memory_usage_pct {
            mem_usage_total += value;
            mem_usage_count += 1.0;
        }

        details.push(GpuInfo {
            index,
            name,
            uuid,
            gpu_usage_pct,
            memory_used_mb,
            memory_total_mb,
            memory_usage_pct,
            temperature_celsius: temperature,
        });
    }

    let average_usage_pct = if usage_count > 0.0 {
        Some(usage_total / usage_count)
    } else {
        None
    };

    let average_mem_usage_pct = if mem_usage_count > 0.0 {
        Some(mem_usage_total / mem_usage_count)
    } else {
        None
    };

    (details, average_usage_pct, average_mem_usage_pct)
}
