use axum::{extract::State, response::Html};

use crate::state::SharedState;

pub async fn show_ui(State(state): State<SharedState>) -> Html<String> {
    let snapshot = state.latest_snapshot().await;

    let disks = if snapshot.disks.is_empty() {
        "<li>No disks reported</li>".to_string()
    } else {
        snapshot
            .disks
            .iter()
            .map(|disk| {
                format!(
                    "<li><strong>{}</strong> — {:.1}% used ({:.1} / {:.1} GB) at {}</li>",
                    disk.name, disk.used_pct, disk.used_gb, disk.total_gb, disk.mount_point
                )
            })
            .collect::<Vec<_>>()
            .join("")
    };

    let processes = if snapshot.top_processes.is_empty() {
        "<li>No process data</li>".to_string()
    } else {
        snapshot
            .top_processes
            .iter()
            .map(|process| {
                let thread_display = process
                    .thread_count
                    .map(|count| format!(" · Threads {count}"))
                    .unwrap_or_default();
                format!(
                    "<li><strong>{}</strong> — CPU {:.1}% · MEM {} MB ({:.1}%) · Disk ↓ {:.1} kbps ↑ {:.1} kbps{}</li>",
                    html_escape(&process.name),
                    process.cpu_pct,
                    process.memory_mb,
                    process.memory_pct,
                    process.disk_read_kbps,
                    process.disk_write_kbps,
                    thread_display
                )
            })
            .collect::<Vec<_>>()
            .join("")
    };

    let gpu = if snapshot.gpus.is_empty() {
        "<li>No GPU data</li>".to_string()
    } else {
        snapshot
            .gpus
            .iter()
            .map(|gpu| {
                format!(
                    "<li><strong>{}</strong> — Util {:.1}% · Mem {} / {} MB ({:.1}%){} </li>",
                    html_escape(&gpu.name),
                    gpu.gpu_usage_pct.unwrap_or(0.0),
                    gpu.memory_used_mb.unwrap_or(0),
                    gpu.memory_total_mb.unwrap_or(0),
                    gpu.memory_usage_pct.unwrap_or(0.0),
                    gpu.temperature_celsius
                        .map(|temp| format!(" · Temp {:.0}C", temp))
                        .unwrap_or_default()
                )
            })
            .collect::<Vec<_>>()
            .join("")
    };

    let network = if snapshot.network.is_empty() {
        "<li>No network interfaces</li>".to_string()
    } else {
        snapshot
            .network
            .iter()
            .map(|iface| {
                format!(
                    "<li><strong>{}</strong> — ↓ {:.1} kbps · ↑ {:.1} kbps (total ↓ {:.1} MB / ↑ {:.1} MB)</li>",
                    iface.name,
                    iface.received_kbps,
                    iface.transmitted_kbps,
                    iface.received_total_bytes as f64 / (1024.0 * 1024.0),
                    iface.transmitted_total_bytes as f64 / (1024.0 * 1024.0),
                )
            })
            .collect::<Vec<_>>()
            .join("")
    };

    let title = snapshot
        .hostname
        .clone()
        .unwrap_or_else(|| "Agent Monitor".to_string());

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8" />
    <meta http-equiv="refresh" content="5" />
    <title>{title}</title>
    <style>
        body {{ font-family: Arial, sans-serif; background: #111827; color: #e5e7eb; margin: 0; padding: 2rem; }}
        h1 {{ margin-top: 0; font-size: 2rem; }}
        section {{ margin-bottom: 2rem; }}
        ul {{ padding-left: 1.25rem; }}
        .cards {{ display: flex; gap: 1rem; flex-wrap: wrap; }}
        .card {{ background: #1f2937; padding: 1rem 1.5rem; border-radius: 0.75rem; min-width: 200px; }}
        small {{ color: #9ca3af; }}
    </style>
</head>
<body>
    <h1>{title}</h1>
    <section class="cards">
        <div class="card">
            <h2>CPU</h2>
            <p><strong>{cpu:.1}%</strong></p>
            <small>Load: {load_one} {load_five} {load_fifteen}</small>
        </div>
        <div class="card">
            <h2>Memory</h2>
            <p><strong>{mem_used}/{mem_total} MB</strong></p>
            <small>Available {mem_available} MB</small>
        </div>
        <div class="card">
            <h2>Uptime</h2>
            <p><strong>{uptime}</strong></p>
            <small>Last refreshed {timestamp}</small>
        </div>
    </section>
    <section>
        <h2>Disks</h2>
        <ul>{disks}</ul>
    </section>
    <section>
        <h2>GPU</h2>
        <ul>{gpu}</ul>
    </section>
    <section>
        <h2>Network</h2>
        <ul>{network}</ul>
    </section>
    <section>
        <h2>Top Processes</h2>
        <ul>{processes}</ul>
    </section>
</body>
</html>"#,
        title = html_escape(&title),
        cpu = snapshot.cpu_usage_pct,
        load_one = snapshot
            .load_avg_one
            .map(|value| format!("1m {value:.2}"))
            .unwrap_or_else(|| "1m n/a".into()),
        load_five = snapshot
            .load_avg_five
            .map(|value| format!("5m {value:.2}"))
            .unwrap_or_else(|| "5m n/a".into()),
        load_fifteen = snapshot
            .load_avg_fifteen
            .map(|value| format!("15m {value:.2}"))
            .unwrap_or_else(|| "15m n/a".into()),
        mem_used = snapshot.mem_used_mb,
        mem_total = snapshot.mem_total_mb,
        mem_available = snapshot.mem_available_mb,
        uptime = format_duration(snapshot.uptime_seconds),
        timestamp = chrono::DateTime::from_timestamp_millis(snapshot.timestamp)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| snapshot.timestamp.to_string()),
        disks = disks,
        gpu = gpu,
        network = network,
        processes = processes,
    );

    Html(html)
}

fn format_duration(total_seconds: u64) -> String {
    let days = total_seconds / 86_400;
    let hours = (total_seconds % 86_400) / 3_600;
    let minutes = (total_seconds % 3_600) / 60;
    let seconds = total_seconds % 60;

    if days > 0 {
        format!("{days}d {hours}h {minutes}m")
    } else if hours > 0 {
        format!("{hours}h {minutes}m")
    } else if minutes > 0 {
        format!("{minutes}m {seconds}s")
    } else {
        format!("{seconds}s")
    }
}

fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
