#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;
use std::time::Instant;
use sysinfo::{System, Networks};

#[derive(Serialize)]
struct NetworkIface {
    name: String,
    received: u64,
    transmitted: u64,
}

#[derive(Serialize)]
struct CpuCore {
    name: String,
    usage: f32,
    frequency: u64,
}

#[derive(Serialize)]
struct GpuInfo {
    name: String,
    vendor: String,
    vram: Option<String>,
}

#[derive(Serialize)]
struct SystemInfo {
    os_name: Option<String>,
    hostname: Option<String>,
    kernel_version: Option<String>,
    os_version: Option<String>,
    cpu_brand: String,
    cpu_physical_cores: Option<usize>,
    cpu_logical_cores: usize,
    cpu_arch: String,
    cpu_usage: f32,
    cpu_cores: Vec<CpuCore>,
    total_memory: u64,
    used_memory: u64,
    total_swap: u64,
    used_swap: u64,
    uptime: u64,
    network_ifaces: Vec<NetworkIface>,
    gpus: Vec<GpuInfo>,
}

#[tauri::command]
fn get_system_info() -> SystemInfo {
    // 初始化带 CPU 信息的 System
    let mut sys = System::new_all();
    
    // 首次刷新
    sys.refresh_all();
    // 等待一小段时间再刷新 CPU 以获得准确使用率
    std::thread::sleep(std::time::Duration::from_millis(200));
    sys.refresh_cpu_all();

    // CPU 信息
    let cpus = sys.cpus();
    let cpu_brand = if !cpus.is_empty() {
        cpus[0].brand().to_string()
    } else {
        "Unknown".to_string()
    };
    
    let cpu_usage: f32 = if !cpus.is_empty() {
        cpus.iter().map(|c| c.cpu_usage()).sum::<f32>() / cpus.len() as f32
    } else {
        0.0
    };
    
    let cpu_cores: Vec<CpuCore> = cpus
        .iter()
        .map(|cpu| CpuCore {
            name: cpu.name().to_string(),
            usage: cpu.cpu_usage(),
            frequency: cpu.frequency(),
        })
        .collect();

    // GPU 信息
    let gpus = get_gpu_info();

    // 网络接口
    let networks = Networks::new_with_refreshed_list();
    let ifaces = networks
        .iter()
        .map(|(name, data)| NetworkIface {
            name: name.clone(),
            received: data.received(),
            transmitted: data.transmitted(),
        })
        .collect::<Vec<_>>();

    SystemInfo {
        os_name: System::name(),
        hostname: System::host_name(),
        kernel_version: System::kernel_version(),
        os_version: System::os_version(),
        cpu_brand,
        cpu_physical_cores: sys.physical_core_count(),
        cpu_logical_cores: cpus.len(),
        cpu_arch: std::env::consts::ARCH.to_string(),
        cpu_usage,
        cpu_cores,
        total_memory: sys.total_memory(),
        used_memory: sys.used_memory(),
        total_swap: sys.total_swap(),
        used_swap: sys.used_swap(),
        uptime: System::uptime(),
        network_ifaces: ifaces,
        gpus,
    }
}

// 获取 GPU 信息
fn get_gpu_info() -> Vec<GpuInfo> {
    #[cfg(target_os = "macos")]
    {
        get_gpu_info_macos()
    }
    #[cfg(target_os = "windows")]
    {
        get_gpu_info_windows()
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        vec![]
    }
}

#[cfg(target_os = "macos")]
fn get_gpu_info_macos() -> Vec<GpuInfo> {
    use std::process::Command;
    
    let output = Command::new("system_profiler")
        .args(["SPDisplaysDataType", "-json"])
        .output();
    
    match output {
        Ok(out) => {
            if let Ok(json_str) = String::from_utf8(out.stdout) {
                parse_macos_gpu_json(&json_str)
            } else {
                vec![]
            }
        }
        Err(_) => vec![],
    }
}

#[cfg(target_os = "macos")]
fn parse_macos_gpu_json(json_str: &str) -> Vec<GpuInfo> {
    // 简单解析 macOS GPU JSON
    let mut gpus = vec![];
    
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(json_str) {
        if let Some(displays) = json.get("SPDisplaysDataType").and_then(|v| v.as_array()) {
            for display in displays {
                let name = display.get("sppci_model")
                    .or_else(|| display.get("_name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown GPU")
                    .to_string();
                
                let vendor = display.get("sppci_vendor")
                    .or_else(|| display.get("spdisplays_vendor"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string();
                
                let vram = display.get("sppci_vram")
                    .or_else(|| display.get("spdisplays_vram"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                
                gpus.push(GpuInfo { name, vendor, vram });
            }
        }
    }
    
    gpus
}

#[cfg(target_os = "windows")]
fn get_gpu_info_windows() -> Vec<GpuInfo> {
    use std::process::Command;
    
    // 使用 WMIC 获取 GPU 信息
    let output = Command::new("wmic")
        .args(["path", "win32_VideoController", "get", "Name,AdapterRAM,DriverVersion", "/format:csv"])
        .output();
    
    match output {
        Ok(out) => {
            if let Ok(csv_str) = String::from_utf8(out.stdout) {
                parse_windows_gpu_csv(&csv_str)
            } else {
                vec![]
            }
        }
        Err(_) => {
            // 备用方案：PowerShell
            get_gpu_info_windows_powershell()
        }
    }
}

#[cfg(target_os = "windows")]
fn parse_windows_gpu_csv(csv_str: &str) -> Vec<GpuInfo> {
    let mut gpus = vec![];
    
    for line in csv_str.lines().skip(1) {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 3 {
            let adapter_ram = parts.get(1).unwrap_or(&"");
            let name = parts.get(2).unwrap_or(&"Unknown").trim().to_string();
            
            if name.is_empty() || name == "Name" {
                continue;
            }
            
            let vram = if let Ok(bytes) = adapter_ram.parse::<u64>() {
                if bytes > 0 {
                    Some(format!("{} MB", bytes / 1024 / 1024))
                } else {
                    None
                }
            } else {
                None
            };
            
            gpus.push(GpuInfo {
                name,
                vendor: "Unknown".to_string(),
                vram,
            });
        }
    }
    
    gpus
}

#[cfg(target_os = "windows")]
fn get_gpu_info_windows_powershell() -> Vec<GpuInfo> {
    use std::process::Command;
    
    let output = Command::new("powershell")
        .args(["-Command", "Get-WmiObject Win32_VideoController | Select-Object Name, AdapterRAM | ConvertTo-Json"])
        .output();
    
    match output {
        Ok(out) => {
            if let Ok(json_str) = String::from_utf8(out.stdout) {
                parse_windows_gpu_powershell(&json_str)
            } else {
                vec![]
            }
        }
        Err(_) => vec![],
    }
}

#[cfg(target_os = "windows")]
fn parse_windows_gpu_powershell(json_str: &str) -> Vec<GpuInfo> {
    let mut gpus = vec![];
    
    // 可能是单个对象或数组
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(json_str) {
        let items = if json.is_array() {
            json.as_array().map(|v| v.to_vec()).unwrap_or_default()
        } else {
            vec![json]
        };
        
        for item in items {
            let name = item.get("Name")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown GPU")
                .to_string();
            
            let vram = item.get("AdapterRAM")
                .and_then(|v| v.as_u64())
                .filter(|&v| v > 0)
                .map(|v| format!("{} MB", v / 1024 / 1024));
            
            gpus.push(GpuInfo {
                name,
                vendor: "Unknown".to_string(),
                vram,
            });
        }
    }
    
    gpus
}

#[derive(Serialize)]
struct AudioDevices {
    inputs: Vec<String>,
    outputs: Vec<String>,
    default_input: Option<String>,
    default_output: Option<String>,
}

#[tauri::command]
fn list_audio_devices() -> AudioDevices {
    use cpal::traits::{DeviceTrait, HostTrait};

    let host = cpal::default_host();

    let mut inputs = Vec::new();
    if let Ok(mut devs) = host.input_devices() {
        for d in devs.by_ref() {
            inputs.push(d.name().unwrap_or_else(|_| "Unknown".to_string()));
        }
    }

    let mut outputs = Vec::new();
    if let Ok(mut devs) = host.output_devices() {
        for d in devs.by_ref() {
            outputs.push(d.name().unwrap_or_else(|_| "Unknown".to_string()));
        }
    }

    let default_input = host
        .default_input_device()
        .and_then(|d| d.name().ok());
    let default_output = host
        .default_output_device()
        .and_then(|d| d.name().ok());

    AudioDevices {
        inputs,
        outputs,
        default_input,
        default_output,
    }
}

#[tauri::command]
fn list_cameras() -> Vec<String> {
    // 摄像头枚举在跨平台上较复杂，此处返回系统默认信息
    // 可后续通过平台特定 API 扩展
    #[cfg(target_os = "macos")]
    {
        // macOS: 通过 system_profiler 获取摄像头
        if let Ok(output) = std::process::Command::new("system_profiler")
            .args(["SPCameraDataType", "-json"])
            .output()
        {
            if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                if let Some(cameras) = json.get("SPCameraDataType").and_then(|v| v.as_array()) {
                    return cameras
                        .iter()
                        .filter_map(|c| c.get("_name").and_then(|n| n.as_str()).map(|s| s.to_string()))
                        .collect();
                }
            }
        }
        Vec::new()
    }
    #[cfg(target_os = "windows")]
    {
        // Windows: 简单返回提示，可通过 WMI 扩展
        vec!["Windows 摄像头枚举待扩展".to_string()]
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Vec::new()
    }
}

#[derive(Serialize, Default)]
struct NetTestResult {
    external_ip: Option<String>,
    http_latency_ms: Option<u128>,
    download_mbps: Option<f64>,
    upload_mbps: Option<f64>,
    error: Option<String>,
}

#[tauri::command]
async fn run_network_test() -> NetTestResult {
    let client = match reqwest::Client::builder()
        .user_agent("hisen-desk/0.1")
        .timeout(std::time::Duration::from_secs(30))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return NetTestResult {
                error: Some(format!("client error: {}", e)),
                ..Default::default()
            }
        }
    };

    let mut result = NetTestResult::default();

    // External IP (使用国内可访问的服务)
    // 尝试多个备用地址
    let ip_urls = [
        "https://myip.ipip.net/json",
        "https://ip.useragentinfo.com/json",
        "https://whois.pconline.com.cn/ipJson.jsp?json=true",
    ];
    
    for url in ip_urls {
        if let Ok(resp) = client.get(url).send().await {
            if let Ok(text) = resp.text().await {
                // 尝试解析 JSON
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                    // ipip.net 格式: {"ip": "x.x.x.x", ...}
                    if let Some(ip) = v.get("ip").and_then(|x| x.as_str()) {
                        result.external_ip = Some(ip.to_string());
                        break;
                    }
                    // pconline 格式: {"ip": "x.x.x.x", ...}
                    if let Some(ip) = v.get("ip").and_then(|x| x.as_str()) {
                        result.external_ip = Some(ip.to_string());
                        break;
                    }
                }
            }
        }
    }

    // HTTP latency (使用国内网站测试延迟)
    let start = Instant::now();
    let latency = client
        .get("https://www.baidu.com/img/flexible/logo/pc/peak-result.png")
        .send()
        .await
        .map(|_| start.elapsed().as_millis())
        .ok();
    result.http_latency_ms = latency;

    // Approx download speed (使用国内CDN测速，约3MB)
    // 使用阿里云/腾讯云等国内CDN的测试文件
    let download_urls = [
        "https://dldir1.qq.com/qqfile/qq/PCQQ9.7.17/QQ9.7.17.29225.exe", // 腾讯
        "https://npm.taobao.org/mirrors/node/v18.0.0/node-v18.0.0.tar.gz", // 淘宝镜像
    ];
    
    let start_dl = Instant::now();
    for url in download_urls {
        // 只下载前3MB来测速
        if let Ok(resp) = client
            .get(url)
            .header("Range", "bytes=0-3000000")
            .send()
            .await 
        {
            if let Ok(bytes) = resp.bytes().await {
                if bytes.len() > 100000 { // 确保下载了足够数据
                    let size = bytes.len() as f64;
                    let secs = (start_dl.elapsed().as_millis().max(1) as f64) / 1000.0;
                    let mbps = (size * 8.0) / 1_000_000.0 / secs;
                    result.download_mbps = Some(mbps);
                    break;
                }
            }
        }
    }

    // Approx upload speed (使用httpbin.org的国内镜像或备用方案)
    // 由于国内缺少公开上传测速端点，这里使用POST请求测量
    let upload_data = vec![0u8; 500_000]; // 500KB
    let start_ul = Instant::now();
    
    // 尝试使用 httpbin 测试上传
    if let Ok(_resp) = client
        .post("https://httpbin.org/post")
        .body(upload_data.clone())
        .send()
        .await
    {
        let size = upload_data.len() as f64;
        let secs = (start_ul.elapsed().as_millis().max(1) as f64) / 1000.0;
        let mbps = (size * 8.0) / 1_000_000.0 / secs;
        result.upload_mbps = Some(mbps);
    }

    result
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_system_info,
            list_audio_devices,
            list_cameras,
            run_network_test
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
