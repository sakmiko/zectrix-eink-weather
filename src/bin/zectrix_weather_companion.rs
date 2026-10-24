use std::error::Error;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{
    Arc, Condvar, Mutex,
    atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering},
};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use zectrix_eink_weather::{
    FramePublisher, PublishDecision, WeatherProviderConfig, WeatherRenderConfig, WeatherSnapshot,
    ZectrixPublisher, ZectrixPublisherConfig, env_or, env_parse, fetch_weather_snapshot,
    load_dotenv_if_present, load_location_catalog, preset_cold_snow, preset_rain_radar,
    preset_sunny_clear, render_weather_dashboard, search_locations,
    validate_weather_page_id_with_reserved,
};

struct RuntimeControl {
    started_at: u64,
    device_id: String,
    allow_page_3: bool,
    reserved_pages: Vec<u8>,
    page_id: AtomicU8,
    last_ok_at: AtomicU64,
    push_success_count: AtomicU64,
    push_fail_streak: AtomicU64,
    max_fail_streak: u64,
    force_refresh: AtomicBool,
    provider_cfg: Mutex<WeatherProviderConfig>,
    render_cfg: Mutex<WeatherRenderConfig>,
    last_snapshot: Mutex<Option<WeatherSnapshot>>,
    last_frame_hash: Mutex<Option<String>>,
    wake_signal: (Mutex<()>, Condvar),
}

impl RuntimeControl {
    fn new(
        device_id: String,
        page_id: u8,
        allow_page_3: bool,
        reserved_pages: Vec<u8>,
        max_fail_streak: u64,
        provider_cfg: WeatherProviderConfig,
        render_cfg: WeatherRenderConfig,
    ) -> Arc<Self> {
        Arc::new(Self {
            started_at: unix_time_u64(),
            device_id,
            allow_page_3,
            reserved_pages,
            page_id: AtomicU8::new(page_id),
            last_ok_at: AtomicU64::new(0),
            push_success_count: AtomicU64::new(0),
            push_fail_streak: AtomicU64::new(0),
            max_fail_streak,
            force_refresh: AtomicBool::new(false),
            provider_cfg: Mutex::new(provider_cfg),
            render_cfg: Mutex::new(render_cfg),
            last_snapshot: Mutex::new(None),
            last_frame_hash: Mutex::new(None),
            wake_signal: (Mutex::new(()), Condvar::new()),
        })
    }

    fn trigger_immediate_refresh(&self) {
        self.force_refresh.store(true, Ordering::SeqCst);
        let (lock, cvar) = &self.wake_signal;
        if let Ok(_guard) = lock.lock() {
            cvar.notify_all();
        }
    }
}

fn main() {
    if let Err(err) = dispatch_cli() {
        eprintln!("ZECTRIX 气象站服务异常退出：{err}");
        std::process::exit(1);
    }
}

fn dispatch_cli() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    // --env 优先于默认 .env；已有系统环境变量始终优先于环境文件。
    if let Some(pos) = args.iter().position(|a| a == "--env")
        && let Some(env_file) = args.get(pos + 1)
    {
        zectrix_eink_weather::load_dotenv_from_path(env_file);
    } else {
        load_dotenv_if_present();
    }

    // 0.0 帮助与版本说明：--help / -h / --version / -v
    if args.iter().any(|a| a == "--help" || a == "-h" || a == "help") {
        print_cli_help();
        return Ok(());
    }
    if args.iter().any(|a| a == "--version" || a == "-v" || a == "version") {
        println!("zectrix_weather_companion v{}", env!("CARGO_PKG_VERSION"));
        println!("ZECTRIX NOTE4 400x300 1-bit E-Ink Keyless Multi-Source Weather Station");
        return Ok(());
    }

    // 0. 快速地址簿检索：--list-locations [keyword]
    if let Some(pos) = args.iter().position(|a| a == "--list-locations") {
        let kw = args.get(pos + 1).map(String::as_str).unwrap_or("");
        let cfg = WeatherProviderConfig::from_env();
        let catalog = load_location_catalog(cfg.city_map_file.as_deref());
        let matches = search_locations(kw, cfg.city_map_file.as_deref());
        println!(
            "地址簿匹配结果（共 {} 条，关键词=\"{}\"）：",
            matches.len(),
            kw
        );
        for loc in matches {
            println!(
                "  - {} · {} · {} | 坐标={:.4},{:.4} | NMC={} ({}) | 中国天气网={} | 墨迹={}",
                loc.province,
                loc.city,
                loc.district,
                loc.lon,
                loc.lat,
                loc.nmc_station_id,
                loc.nmc_path,
                loc.weather_cn_code,
                loc.moji_slug
            );
        }
        return Ok(());
    }

    // 0.5 命令行快速天气聚合查询：--query <城市/区县/地标/经纬度>
    if let Some(pos) = args.iter().position(|a| a == "--query") {
        let q = args
            .get(pos + 1)
            .map(String::as_str)
            .unwrap_or("北京·海淀区");
        let pub_cfg = ZectrixPublisherConfig::from_env();
        let mut cfg = WeatherProviderConfig::from_env();
        cfg.apply_location(q);
        let snap = fetch_weather_snapshot(&cfg, pub_cfg.page_id)?;
        println!(
            "【{} · {}】({:.4}, {:.4}) 更新于 {}",
            snap.city, snap.district, cfg.longitude, cfg.latitude, snap.updated_at
        );
        println!(
            "  实况：{} {}°C（体感 {}°C，最低 {}°C ~ 最高 {}°C）",
            snap.condition, snap.temp_c, snap.feels_like, snap.low_c, snap.high_c
        );
        println!(
            "  环境：湿度 {}% | {} {}级 | 空气质量 {} ({}) | 气压 {}hPa",
            snap.humidity, snap.wind_dir, snap.wind_scale, snap.aqi, snap.aqi_grade, snap.pressure
        );
        println!("  短临：{}", snap.radar_summary);
        for d in &snap.daily_forecast {
            println!(
                "  预报：{} ({}) {} {}°~{}°C {} {}级",
                d.day_cn, d.date, d.cond, d.low, d.high, d.wind_cn, d.wind_scale
            );
        }
        return Ok(());
    }

    // 1. 本地预览模式：--preview [rain|sunny|snow|live] [output.png]
    if let Some(pos) = args.iter().position(|a| a == "--preview") {
        let default_mode = env_or("WEATHER_DEFAULT_PREVIEW_MODE", "rain");
        let mode = args.get(pos + 1).map(String::as_str).unwrap_or(&default_mode);
        let default_out = env_or("WEATHER_PREVIEW_PATH", "weather_preview.png");
        let out_path = args.get(pos + 2).map(String::as_str).unwrap_or(&default_out);
        let pub_cfg = ZectrixPublisherConfig::from_env();
        let mut cfg = WeatherProviderConfig::from_env();
        if let Some(lpos) = args.iter().position(|a| a == "--location")
            && let Some(loc_q) = args.get(lpos + 1)
        {
            cfg.apply_location(loc_q);
        }
        let snap = match mode {
            "sunny" => preset_sunny_clear(pub_cfg.page_id),
            "snow" => preset_cold_snow(pub_cfg.page_id),
            "live" => fetch_weather_snapshot(&cfg, pub_cfg.page_id)?,
            _ => preset_rain_radar(pub_cfg.page_id),
        };
        let rendered = render_weather_dashboard(snap, WeatherRenderConfig::from_env())?;
        rendered.frame.write_png(out_path)?;
        println!("已生成 400x300 墨水屏气象站预览图：{out_path}");
        return Ok(());
    }

    // 2. 外部健康检查探针：--check
    if args.iter().any(|a| a == "--check") {
        return query_local_control("GET /healthz");
    }

    // 3. 外部立即刷新控制：--trigger-refresh
    if args.iter().any(|a| a == "--trigger-refresh") {
        return query_local_control("POST /refresh");
    }

    let run_once = args.iter().any(|a| a == "--once");
    run_daemon(run_once)
}

fn print_cli_help() {
    println!(
        r#"ZECTRIX NOTE4 纯代码免Key多源气象站伴侣 CLI (v{})

用法:
  zectrix_weather_companion [命令/选项]

基础选项:
  --help, -h               显示此帮助信息
  --version, -v            显示版本信息
  --env <path>             指定自定义环境配置文件路径（默认加载 .env / 系统环境）

操作模式:
  --query [城市/区县/坐标] 实时从配置源拉取并打印指定地点天气明细终端摘要
  --list-locations [关键词] 检索内置全国区县经纬度与站点代码地址簿
  --preview [模式] [文件]   在本地离线生成 400x300 1-bit 墨水屏预览图（模式可选: live|rain|sunny|snow，默认 rain）
  --check                  查询后台常驻服务的健康状态探针（默认 127.0.0.1:9597）
  --trigger-refresh        触发后台服务立即拉取最新气象数据并推送上屏
  --once                   单次运行：完成数据抓取和推送后立即退出

常驻运行:
  直接运行程序无参数时，将作为守护服务启动，按照环境变量 WEATHER_REFRESH_SECONDS 持续同步并推送。
"#,
        env!("CARGO_PKG_VERSION")
    );
}

fn query_local_control(request_line: &str) -> Result<(), Box<dyn Error>> {
    let addr = env_or("WEATHER_CONTROL_LISTEN", "127.0.0.1:9597");
    let mut stream = TcpStream::connect_timeout(&addr.parse()?, Duration::from_secs(3))?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    write!(
        stream,
        "{request_line} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n"
    )?;
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    let status_line = response.lines().next().unwrap_or("");
    let body = response.split("\r\n\r\n").nth(1).unwrap_or("");
    println!("{body}");
    if status_line.contains("200 OK") {
        Ok(())
    } else {
        Err(format!("气象站状态异常：{status_line}").into())
    }
}

fn spawn_control_server(ctrl: Arc<RuntimeControl>) -> Option<thread::JoinHandle<()>> {
    let addr = match std::env::var("WEATHER_CONTROL_LISTEN") {
        Ok(v) if v.trim().is_empty() => return None,
        Ok(v) => v,
        Err(_) => "127.0.0.1:9597".to_owned(),
    };
    // 页面切换和地区修改接口没有鉴权，只允许本机访问。
    if !addr
        .parse::<std::net::SocketAddr>()
        .is_ok_and(|address| address.ip().is_loopback())
    {
        eprintln!("天气控制端只能监听 loopback 地址：{addr}");
        return None;
    }
    let listener = match TcpListener::bind(&addr) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("气象站控制探针=绑定失败 {addr}: {e}");
            return None;
        }
    };
    eprintln!(
        "气象站控制探针=监听 http://{addr} (/healthz, /refresh, /locations?q=..., /location?q=..., /page?id=N, /preview.png)"
    );
    Some(thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            handle_control_client(stream, &ctrl);
        }
    }))
}

fn handle_control_client(mut stream: TcpStream, ctrl: &Arc<RuntimeControl>) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    let mut buf = [0_u8; 1024];
    let n = stream.read(&mut buf).unwrap_or(0);
    let req = String::from_utf8_lossy(&buf[..n]);
    let first_line = req.lines().next().unwrap_or("");
    let path = first_line.split_whitespace().nth(1).unwrap_or("/");

    if path.starts_with("/healthz") || path.starts_with("/status") {
        let now_u64 = unix_time_u64();
        let last_ok = ctrl.last_ok_at.load(Ordering::Relaxed);
        let fails = ctrl.push_fail_streak.load(Ordering::Relaxed);
        let ok_cnt = ctrl.push_success_count.load(Ordering::Relaxed);
        let page_id = ctrl.page_id.load(Ordering::Relaxed);
        let last_hash = ctrl
            .last_frame_hash
            .lock()
            .ok()
            .and_then(|g| g.clone())
            .unwrap_or_default();
        let (city_str, temp_val, cond_str) = ctrl
            .last_snapshot
            .lock()
            .ok()
            .and_then(|g| g.clone())
            .map(|s| (format!("{}·{}", s.city, s.district), s.temp_c, s.condition))
            .unwrap_or_else(|| ("未拉取".into(), 0, "-".into()));

        let healthy = fails < ctrl.max_fail_streak && last_ok > 0;
        let code = if healthy { 200 } else { 503 };
        let status_str = if healthy { "ok" } else { "degraded" };
        let body = serde_json::json!({
            "status": status_str,
            "healthy": healthy,
            "device_id": &ctrl.device_id,
            "page_id": page_id,
            "location": city_str,
            "temp_c": temp_val,
            "condition": cond_str,
            "last_frame_sha256": last_hash,
            "uptime_seconds": now_u64.saturating_sub(ctrl.started_at),
            "last_ok_age_seconds": if last_ok == 0 {
                u64::MAX
            } else {
                now_u64.saturating_sub(last_ok)
            },
            "push_success_count": ok_cnt,
            "push_fail_streak": fails,
        }).to_string();
        write_http_response(&mut stream, code, "application/json", body.as_bytes());
    } else if path.starts_with("/refresh") {
        ctrl.trigger_immediate_refresh();
        let body = br#"{"status":"triggered","message":"immediate weather refresh scheduled"}"#;
        write_http_response(&mut stream, 200, "application/json", body);
    } else if let Some(query) = path.strip_prefix("/locations") {
        let kw_raw = query
            .strip_prefix("?q=")
            .or_else(|| query.strip_prefix("?keyword="))
            .unwrap_or("");
        let Some(kw) = decode_query_component(kw_raw) else {
            write_http_response(&mut stream, 400, "application/json", br#"{"error":"invalid query encoding"}"#);
            return;
        };
        let cfg = ctrl.provider_cfg.lock().map(|g| g.clone()).unwrap();
        let matches = search_locations(&kw, cfg.city_map_file.as_deref());
        let body = serde_json::to_string(&matches).unwrap_or_else(|_| "[]".into());
        write_http_response(&mut stream, 200, "application/json", body.as_bytes());
    } else if let Some(query) = path.strip_prefix("/location?") {
        let mut location = None;
        for pair in query.split('&') {
            if let Some((key, value)) = pair.split_once('=')
                && (key == "q" || key == "name" || key == "coord")
            {
                location = decode_query_component(value);
                break;
            }
        }
        let Some(location) = location else {
            write_http_response(&mut stream, 400, "application/json", br#"{"error":"missing or invalid location"}"#);
            return;
        };
        if let Ok(mut cfg) = ctrl.provider_cfg.lock() {
            cfg.apply_location(&location);
        }
        if let Ok(mut guard) = ctrl.last_frame_hash.lock() {
            *guard = None;
        }
        ctrl.trigger_immediate_refresh();
        write_http_response(
            &mut stream,
            200,
            "application/json",
            br#"{"status":"ok","message":"location resolved and immediate refresh scheduled"}"#,
        );
    } else if let Some(query) = path.strip_prefix("/page?id=") {
        let parsed = query
            .split('&')
            .next()
            .unwrap_or("")
            .parse::<u8>()
            .ok()
            .and_then(|pid| {
                validate_weather_page_id_with_reserved(
                    pid,
                    &ctrl.reserved_pages,
                    ctrl.allow_page_3,
                )
                .ok()
            });
        if let Some(new_page) = parsed {
            ctrl.page_id.store(new_page, Ordering::SeqCst);
            if let Ok(mut guard) = ctrl.last_frame_hash.lock() {
                *guard = None;
            }
            ctrl.trigger_immediate_refresh();
            let body = format!("{{\"status\":\"ok\",\"page_id\":{new_page}}}");
            write_http_response(&mut stream, 200, "application/json", body.as_bytes());
        } else {
            let body =
                br#"{"status":"error","message":"invalid or reserved page_id (check ZECTRIX_RESERVED_PAGES)"}"#;
            write_http_response(&mut stream, 400, "application/json", body);
        }
    } else if let Some(query) = path.strip_prefix("/city?") {
        if let Ok(mut cfg) = ctrl.provider_cfg.lock() {
            let mut city_q = None;
            let mut dist_q = None;
            let mut lon_q = None;
            let mut lat_q = None;
            for pair in query.split('&') {
                if let Some((k, v)) = pair.split_once('=') {
                    match k {
                        "q" | "location" => cfg.apply_location(v),
                        "name" | "city" => city_q = Some(v.to_owned()),
                        "district" => dist_q = Some(v.to_owned()),
                        "lon" => lon_q = v.parse::<f64>().ok(),
                        "lat" => lat_q = v.parse::<f64>().ok(),
                        _ => {}
                    }
                }
            }
            if let (Some(lon), Some(lat)) = (lon_q, lat_q) {
                cfg.apply_location(&format!("{lon},{lat}"));
            } else if city_q.is_some() || dist_q.is_some() {
                let combined = format!(
                    "{}{}",
                    city_q.as_deref().unwrap_or(&cfg.city),
                    dist_q.as_deref().unwrap_or("")
                );
                cfg.apply_location(&combined);
            }
        }
        if let Ok(mut guard) = ctrl.last_frame_hash.lock() {
            *guard = None;
        }
        ctrl.trigger_immediate_refresh();
        write_http_response(
            &mut stream,
            200,
            "application/json",
            br#"{"status":"ok","message":"city/district updated and refresh scheduled"}"#,
        );
    } else if path.starts_with("/preview.png") {
        let page_id = ctrl.page_id.load(Ordering::Relaxed);
        let snap = ctrl
            .last_snapshot
            .lock()
            .ok()
            .and_then(|g| g.clone())
            .unwrap_or_else(|| preset_rain_radar(page_id));
        let r_cfg = ctrl
            .render_cfg
            .lock()
            .map(|g| g.clone())
            .unwrap_or_default();
        if let Ok(out) = render_weather_dashboard(snap, r_cfg)
            && let Ok(png) = out.frame.png_bytes()
        {
            write_http_response(&mut stream, 200, "image/png", &png);
        } else {
            write_http_response(&mut stream, 500, "text/plain", b"render error");
        }
    } else {
        write_http_response(
            &mut stream,
            404,
            "application/json",
            br#"{"status":"not_found"}"#,
        );
    }
}

fn write_http_response(stream: &mut TcpStream, code: u16, content_type: &str, body: &[u8]) {
    let reason = match code {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        503 => "Service Unavailable",
        _ => "Error",
    };
    let header = format!(
        "HTTP/1.1 {code} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(header.as_bytes());
    let _ = stream.write_all(body);
}

fn decode_query_component(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'%' => {
                let hex = |b: u8| match b {
                    b'0'..=b'9' => Some(b - b'0'),
                    b'a'..=b'f' => Some(b - b'a' + 10),
                    b'A'..=b'F' => Some(b - b'A' + 10),
                    _ => None,
                };
                let high = hex(*bytes.get(index + 1)?)?;
                let low = hex(*bytes.get(index + 2)?)?;
                decoded.push(high << 4 | low);
                index += 3;
            }
            b'+' => {
                decoded.push(b' ');
                index += 1;
            }
            byte => {
                decoded.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8(decoded).ok()
}

#[cfg(test)]
mod control_tests {
    use super::*;

    #[test]
    fn chinese_location_query_is_decoded_and_invalid_hex_is_rejected() {
        assert_eq!(decode_query_component("%E5%85%B0%E5%B7%9E"), Some("兰州".into()));
        assert_eq!(decode_query_component("A+B"), Some("A B".into()));
        assert_eq!(decode_query_component("%G0"), None);
        assert_eq!(decode_query_component("%E5%FF"), None);
    }
}

struct WeatherPauseWindow {
    start_minute: u16,
    end_minute: u16,
}

struct WeatherPauseSchedule {
    windows: Vec<WeatherPauseWindow>,
}

impl WeatherPauseSchedule {
    fn parse(value: &str) -> Result<Self, String> {
        let mut windows = Vec::new();
        for entry in value
            .split(',')
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
        {
            let (start, end) = entry
                .split_once('-')
                .ok_or("WEATHER_PAUSE_WINDOWS 格式应为 HH:MM-HH:MM")?;
            let start_minute =
                parse_clock_minute(start).ok_or("WEATHER_PAUSE_WINDOWS 中的开始时间无效")?;
            let end_minute =
                parse_clock_minute(end).ok_or("WEATHER_PAUSE_WINDOWS 中的结束时间无效")?;
            if start_minute == end_minute {
                return Err("WEATHER_PAUSE_WINDOWS 不接受起止时间相同的时段".into());
            }
            windows.push(WeatherPauseWindow {
                start_minute,
                end_minute,
            });
        }
        Ok(Self { windows })
    }

    fn seconds_until_resume_tz(&self, tz_offset_hours: i32) -> Option<u64> {
        if self.windows.is_empty() {
            return None;
        }
        let now_utc = chrono::Utc::now();
        let tz = chrono::FixedOffset::east_opt(tz_offset_hours * 3600)
            .unwrap_or_else(|| chrono::FixedOffset::east_opt(8 * 3600).unwrap());
        let now = now_utc.with_timezone(&tz).time();
        use chrono::Timelike;
        let current_minute = (now.hour() * 60 + now.minute()) as u16;
        let current_second =
            u64::from(now.hour()) * 3_600 + u64::from(now.minute()) * 60 + u64::from(now.second());
        self.windows
            .iter()
            .filter(|window| window.contains(current_minute))
            .map(|window| {
                let end_second = u64::from(window.end_minute) * 60;
                (end_second + 86_400 - current_second) % 86_400
            })
            .max()
            .map(|seconds| seconds.max(1))
    }
}

impl WeatherPauseWindow {
    fn contains(&self, minute: u16) -> bool {
        if self.start_minute < self.end_minute {
            (self.start_minute..self.end_minute).contains(&minute)
        } else {
            minute >= self.start_minute || minute < self.end_minute
        }
    }
}

fn parse_clock_minute(value: &str) -> Option<u16> {
    let bytes = value.as_bytes();
    if bytes.len() != 5
        || bytes[2] != b':'
        || !bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| index == 2 || byte.is_ascii_digit())
    {
        return None;
    }
    let hour = value[..2].parse::<u16>().ok()?;
    let minute = value[3..].parse::<u16>().ok()?;
    if hour > 23 || minute > 59 {
        return None;
    }
    Some(hour * 60 + minute)
}

fn run_daemon(run_once: bool) -> Result<(), Box<dyn Error>> {
    let api_key = read_api_key()?;
    let pub_cfg = ZectrixPublisherConfig::from_env();
    validate_weather_page_id_with_reserved(
        pub_cfg.page_id,
        &pub_cfg.reserved_pages,
        pub_cfg.allow_reserved_override,
    )?;

    let refresh_interval: u64 = env_parse("WEATHER_REFRESH_SECONDS", 1800_u64).max(60);
    let retry_interval: u64 = env_parse("WEATHER_RETRY_SECONDS", 60_u64).max(10);
    let max_fail_streak: u64 = env_parse("WEATHER_MAX_FAIL_STREAK", 5_u64).max(1);
    let pause_schedule = WeatherPauseSchedule::parse(&env_or("WEATHER_PAUSE_WINDOWS", "00:00-06:00"))
        .unwrap_or_else(|e| {
            eprintln!("警告: WEATHER_PAUSE_WINDOWS 解析失败: {e}，将不暂停");
            WeatherPauseSchedule { windows: vec![] }
        });
    let tz_offset_hours = env_parse("WEATHER_TZ_OFFSET_HOURS", 8_i32);
    let preview_output = std::env::var("WEATHER_PREVIEW_PATH").ok();

    let mut provider_cfg = WeatherProviderConfig::from_env();
    let args: Vec<String> = std::env::args().collect();
    if let Some(lpos) = args.iter().position(|a| a == "--location")
        && let Some(loc_q) = args.get(lpos + 1)
    {
        provider_cfg.apply_location(loc_q);
    }
    let render_cfg = WeatherRenderConfig::from_env();
    let ctrl = RuntimeControl::new(
        pub_cfg.device_id.clone(),
        pub_cfg.page_id,
        pub_cfg.allow_reserved_override,
        pub_cfg.reserved_pages.clone(),
        max_fail_streak,
        provider_cfg,
        render_cfg,
    );
    let _control_handle = if !run_once {
        spawn_control_server(Arc::clone(&ctrl))
    } else {
        None
    };

    eprintln!(
        "ZECTRIX 气象站守护服务已启动：设备={} 通道=Page {} 刷新间隔={refresh_interval}s",
        pub_cfg.device_id, pub_cfg.page_id
    );

    let mut pause_active = false;
    loop {
        if let Some(until_resume) = pause_schedule.seconds_until_resume_tz(tz_offset_hours) {
            if !pause_active {
                eprintln!("天气轮询=夜间暂停 (UTC+{tz_offset_hours} 窗口内)");
                pause_active = true;
            }
            let wait_secs = until_resume.min(60);
            let (lock, cvar) = &ctrl.wake_signal;
            if let Ok(guard) = lock.lock() {
                let _ = cvar.wait_timeout(guard, Duration::from_secs(wait_secs));
            }
            continue;
        } else if pause_active {
            eprintln!("天气轮询=夜间恢复 (UTC+{tz_offset_hours})");
            pause_active = false;
        }

        let active_page_id = ctrl.page_id.load(Ordering::SeqCst);
        let forced = ctrl.force_refresh.swap(false, Ordering::SeqCst);
        let prev_hash = if forced {
            None
        } else {
            ctrl.last_frame_hash.lock().ok().and_then(|g| g.clone())
        };
        let active_cfg = ctrl.provider_cfg.lock().map(|g| g.clone()).unwrap();
        let mut active_render_cfg = ctrl.render_cfg.lock().map(|g| g.clone()).unwrap();
        active_render_cfg.previous_frame_hash = prev_hash;

        let mut push_succeeded = true;
        match fetch_weather_snapshot(&active_cfg, active_page_id) {
            Ok(snap) => {
                if let Ok(mut s_guard) = ctrl.last_snapshot.lock() {
                    *s_guard = Some(snap.clone());
                }
                let output = render_weather_dashboard(snap.clone(), active_render_cfg)?;
                if let Some(path) = preview_output.as_deref()
                    && !path.trim().is_empty()
                {
                    let _ = output.frame.write_png(path);
                }

                if output.publish_decision == PublishDecision::Publish || forced {
                    let mut publisher = ZectrixPublisher::new(
                        &api_key,
                        &pub_cfg.base_url,
                        &pub_cfg.device_id,
                        active_page_id,
                        pub_cfg.allow_reserved_override,
                    )?;
                    match publisher.publish(&output) {
                        Ok(()) => {
                            eprintln!(
                                "气象站推送=成功 地区={}·{} 温度={}°C 通道=Page {} sha256={}",
                                snap.city,
                                snap.district,
                                snap.temp_c,
                                active_page_id,
                                &output.frame.sha256[..12]
                            );
                            ctrl.last_ok_at.store(unix_time_u64(), Ordering::Relaxed);
                            ctrl.push_success_count.fetch_add(1, Ordering::Relaxed);
                            ctrl.push_fail_streak.store(0, Ordering::Relaxed);
                            if let Ok(mut h_guard) = ctrl.last_frame_hash.lock() {
                                *h_guard = Some(output.frame.sha256);
                            }
                        }
                        Err(err) => {
                            push_succeeded = false;
                            let streak = ctrl.push_fail_streak.fetch_add(1, Ordering::Relaxed) + 1;
                            eprintln!("气象站推送=失败({streak}/{max_fail_streak}) 原因={err}");
                        }
                    }
                } else {
                    ctrl.last_ok_at.store(unix_time_u64(), Ordering::Relaxed);
                    eprintln!("气象站推送=跳过（画面未变化）");
                }
            }
            Err(err) => {
                push_succeeded = false;
                let streak = ctrl.push_fail_streak.fetch_add(1, Ordering::Relaxed) + 1;
                eprintln!("天气聚合拉取=失败({streak}/{max_fail_streak}) 原因={err}");
            }
        }

        if !push_succeeded
            && ctrl.push_fail_streak.load(Ordering::Relaxed) >= max_fail_streak
            && !run_once
        {
            return Err("连续失败次数达到阈值，触发进程自愈重启".into());
        }

        if run_once {
            if !push_succeeded {
                return Err("单次执行模式下气象站更新失败".into());
            }
            return Ok(());
        }

        let sleep_secs = if push_succeeded {
            refresh_interval
        } else {
            retry_interval
        };
        let (lock, cvar) = &ctrl.wake_signal;
        if let Ok(guard) = lock.lock() {
            let _ = cvar.wait_timeout(guard, Duration::from_secs(sleep_secs));
        }
    }
}

fn read_api_key() -> Result<String, Box<dyn Error>> {
    if let Ok(key) = std::env::var("ZECTRIX_API_KEY")
        && !key.trim().is_empty()
    {
        return Ok(key.trim().to_owned());
    }
    if let Ok(path) = std::env::var("ZECTRIX_API_KEY_FILE")
        && !path.trim().is_empty()
    {
        let content = fs::read_to_string(path)?;
        let trimmed = content.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_owned());
        }
    }
    // 兜底读取本机存在的密钥文件
    for fallback in [
        "/var/lib/zectrix-weather/secrets/zectrix_api_key",
        "/var/lib/zectrix-calendar/secrets/zectrix_api_key",
        "/var/lib/codex-quota/secrets/zectrix_api_key",
    ] {
        if let Ok(content) = fs::read_to_string(fallback) {
            let trimmed = content.trim();
            if !trimmed.is_empty() {
                return Ok(trimmed.to_owned());
            }
        }
    }
    Err("请设置 ZECTRIX_API_KEY 或 ZECTRIX_API_KEY_FILE 环境变量（或在 .env 中配置）".into())
}

fn unix_time_u64() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
