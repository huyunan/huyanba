use chrono::{DateTime, Utc, FixedOffset};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::{Write};
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};
use std::time::Instant;
use tauri::{
    menu::MenuBuilder,
    path::BaseDirectory,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent,
};
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{GetDC, ReleaseDC};
use windows::Win32::UI::ColorSystem::SetDeviceGammaRamp;

#[derive(Default)]
struct LockState {
    labels: Mutex<Vec<String>>,
}
#[derive(Default)]
struct AppState {
    allow_exit: AtomicBool,
}

const TRAY_ICON: tauri::image::Image<'static> = tauri::include_image!("icons/32x32.png");
fn default_source_kind() -> String {
    "legacy".into()
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum RestRemoteSource {
    Palace,
}

impl Default for RestRemoteSource {
    fn default() -> Self {
        Self::Palace
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RestFile {
    path: String,
    added_at: i64,
    source_url: String,
    #[serde(default)]
    last_shown_at: i64,
    #[serde(default = "default_source_kind")]
    source_kind: String,
    #[serde(default)]
    remote_id: String,
    #[serde(default)]
    thumb_url: String,
    #[serde(default)]
    author_name: String,
    #[serde(default)]
    author_url: String,
    #[serde(default)]
    photo_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct RestState {
    files: Vec<RestFile>,
    next_source_index: usize,
    next_show_index: usize,
    last_download_at: i64,
    last_batch_at: i64,
    #[serde(default, alias = "preferredSourceUrl")]
    fixed_source_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct RestStorageConfig {
    #[serde(default)]
    custom_dir: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct RestStorageSettings {
    current_dir: String,
    default_dir: String,
    is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct PalaceStagingState {
    items: Vec<PalaceStagingEntry>,
    last_batch_at: i64,
    current_page: usize,
    has_prev_page: bool,
    has_next_page: bool,
    max_page: usize,
}

impl Default for PalaceStagingState {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            last_batch_at: 0,
            current_page: 1,
            has_prev_page: false,
            has_next_page: false,
            max_page: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PalaceStagingEntry {
    id: String,
    title: String,
    file_name: String,
    source_url: String,
    width: u32,
    height: u32,
    credit_name: String,
    credit_url: String,
    photo_url: String,
    added_at: i64,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct PalaceDownloadRequest {
    id: String,
    title: String,
    image_url: String,
    thumb_url: String,
    photo_url: String,
    credit_name: String,
    credit_url: String,
    width: u32,
    height: u32,
}
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct LockUpdate {
    time_text: String,
    date_text: String,
    rest_countdown: String,
    allow_esc_exit: bool,
}

fn clamp(value: f64, min: f64, max: f64) -> f64 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

fn temperature_to_rgb(temp: f64) -> (f64, f64, f64) {
    let temp = clamp(temp, 1000.0, 40000.0) / 100.0;
    let (mut r, mut g, mut b);
    if temp <= 66.0 {
        r = 255.0;
        g = 99.4708025861 * temp.ln() - 161.1195681661;
        b = if temp <= 19.0 {
            0.0
        } else {
            138.5177312231 * (temp - 10.0).ln() - 305.0447927307
        };
    } else {
        r = 329.698727446 * (temp - 60.0).powf(-0.1332047592);
        g = 288.1221695283 * (temp - 60.0).powf(-0.0755148492);
        b = 255.0;
    }

    r = clamp(r, 0.0, 255.0);
    g = clamp(g, 0.0, 255.0);
    b = clamp(b, 0.0, 255.0);
    (r / 255.0, g / 255.0, b / 255.0)
}

fn apply_gamma(mult_r: f64, mult_g: f64, mult_b: f64) -> Result<(), String> {
    unsafe {
        let hdc = GetDC(HWND(0));
        if hdc.0 == 0 {
            return Err("无法获取显示设备句柄".into());
        }

        let mut ramp = [0u16; 256 * 3];
        for i in 0..256 {
            let base = i as f64 / 255.0;
            ramp[i] = clamp(base * 65535.0 * mult_r, 0.0, 65535.0).round() as u16;
            ramp[i + 256] = clamp(base * 65535.0 * mult_g, 0.0, 65535.0).round() as u16;
            ramp[i + 512] = clamp(base * 65535.0 * mult_b, 0.0, 65535.0).round() as u16;
        }

        let ok = SetDeviceGammaRamp(hdc, ramp.as_ptr() as *const _).as_bool();
        ReleaseDC(HWND(0), hdc);
        if !ok {
            return Err("设置色温失败".into());
        }
    }
    Ok(())
}

#[tauri::command]
fn set_gamma(filter_enabled: bool, strength: f64, color_temp: f64) -> Result<(), String> {
    if !filter_enabled {
        return apply_gamma(1.0, 1.0, 1.0);
    }
    let (r, g, b) = temperature_to_rgb(color_temp);
    let factor = clamp(strength / 100.0, 0.0, 1.0);
    let mut mult_r = (1.0 - factor) + factor * r;
    let mut mult_g = (1.0 - factor) + factor * g;
    let mut mult_b = (1.0 - factor) + factor * b;

    // Greenish bias to avoid reddish tint and reduce blue light.
    let green_boost = 0.08 * factor;
    let red_cut = 0.18 * factor;
    let blue_cut = 0.35 * factor;
    mult_r = clamp(mult_r * (1.0 - red_cut), 0.0, 1.0);
    mult_g = clamp(mult_g * (1.0 + green_boost), 0.0, 1.0);
    mult_b = clamp(mult_b * (1.0 - blue_cut), 0.0, 1.0);
    apply_gamma(mult_r, mult_g, mult_b)
}

#[tauri::command]
fn reset_gamma() -> Result<(), String> {
    apply_gamma(1.0, 1.0, 1.0)
}

#[tauri::command]
async fn show_lock_windows(
    app: tauri::AppHandle,
    state: tauri::State<'_, LockState>,
    end_at_ms: i64,
) -> Result<(), String> {
    let start = Instant::now();
    let mut labels = state.labels.lock().map_err(|_| "锁状态被占用")?;
    if !labels.is_empty() {
        for label in labels.iter() {
            if let Some(window) = app.get_webview_window(label) {
                let _ = window.set_always_on_top(true);
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        return Ok(());
    }

    let monitors = app.available_monitors().map_err(|err| err.to_string())?;
    append_app_log(&app, &format!("锁屏创建开始 monitors={}", monitors.len()));
    for (index, monitor) in monitors.into_iter().enumerate() {
        let label = format!("lockscreen-{}", index);
        let position = monitor.position();
        let size = monitor.size();
        let scale = monitor.scale_factor();
        let width = (size.width as f64 / scale).ceil() + 400.0;
        let height = (size.height as f64 / scale).ceil() + 400.0;
        let x = (position.x as f64 / scale).floor() - 200.0;
        let y = (position.y as f64 / scale).floor() - 200.0;

        let url = format!(
            "index.html?lockscreen=1&end={}",
            end_at_ms,
        );
        let window = WebviewWindowBuilder::new(&app, label.clone(), WebviewUrl::App(url.into()))
            .decorations(false)
            .transparent(false)
            .resizable(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .position(x, y)
            .inner_size(width, height)
            .build()
            .map_err(|err| err.to_string())?;

        apply_default_window_icon(&app, &window);
        let _ = window.set_fullscreen(true);
        let _ = window.set_focus();
        labels.push(label);
    }

    append_app_log(
        &app,
        &format!(
            "锁屏创建完成 labels={} elapsed_ms={}",
            labels.len(),
            start.elapsed().as_millis()
        ),
    );
    Ok(())
}

#[tauri::command]
fn hide_lock_windows(
    app: tauri::AppHandle,
    state: tauri::State<'_, LockState>,
) -> Result<(), String> {
    let start = Instant::now();
    let mut labels = state.labels.lock().map_err(|_| "锁状态被占用")?;
    append_app_log(&app, &format!("锁屏关闭开始 labels={}", labels.len()));
    for label in labels.iter() {
        if let Some(window) = app.get_webview_window(label) {
            let _ = window.close();
        }
    }
    labels.clear();
    append_app_log(
        &app,
        &format!("锁屏关闭完成 elapsed_ms={}", start.elapsed().as_millis()),
    );
    Ok(())
}

#[tauri::command]
fn lockscreen_action(app: tauri::AppHandle, action: String) -> Result<(), String> {
    append_app_log(&app, &format!("锁屏动作: {}", action));
    for (_label, window) in app.webview_windows() {
        let _ = window.emit("lockscreen-action", action.clone());
    }
    Ok(())
}

fn date_time() -> String {
    let utc_time = Utc::now();
    let china_timezone = FixedOffset::east_opt(8 * 3600).unwrap();
    // 将 UTC 时间转换为中国标准时间
    let china_time: DateTime<FixedOffset> = utc_time.with_timezone(&china_timezone);
    // 格式化
    return china_time.format("%Y-%m-%d %H:%M:%S").to_string();
}

fn load_storage_config(path: &Path) -> RestStorageConfig {
    let Ok(data) = fs::read_to_string(path) else {
        return RestStorageConfig::default();
    };
    serde_json::from_str(&data).unwrap_or_default()
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn default_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .resolve("log", BaseDirectory::AppCache)
        .map_err(|err| err.to_string())
}

fn storage_config_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .resolve("", BaseDirectory::AppConfig)
        .map_err(|err| err.to_string())?;
    fs::create_dir_all(&dir).map_err(|err| err.to_string())?;
    Ok(dir.join("wallpaper-storage.json"))
}

fn storage_settings_from_config(
    app: &AppHandle,
    config: &RestStorageConfig,
) -> Result<RestStorageSettings, String> {
    let default_dir = default_dir(app)?;
    let current_dir = if config.custom_dir.trim().is_empty() {
        default_dir.clone()
    } else {
        PathBuf::from(config.custom_dir.trim())
    };
    Ok(RestStorageSettings {
        current_dir: path_to_string(&current_dir),
        default_dir: path_to_string(&default_dir),
        is_default: current_dir == default_dir,
    })
}

fn get_storage_settings_inner(
    app: &AppHandle,
) -> Result<RestStorageSettings, String> {
    let config_path = storage_config_path(app)?;
    let config = load_storage_config(&config_path);
    storage_settings_from_config(app, &config)
}

fn allow_dir_on_scope(app: &AppHandle, dir: &Path) -> Result<(), String> {
    app.asset_protocol_scope()
        .allow_directory(dir, true)
        .map_err(|err| err.to_string())
}

fn ensure_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let settings = get_storage_settings_inner(app)?;
    let dir = PathBuf::from(settings.current_dir);
    fs::create_dir_all(&dir).map_err(|err| err.to_string())?;
    allow_dir_on_scope(app, &dir)?;
    Ok(dir)
}

fn append_line(path: &Path, message: &str) {
    let ts = date_time();
    let line = format!("[{}] {}\n", ts, message);
    if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = file.write_all(line.as_bytes());
    }
}

fn append_app_log(app: &AppHandle, message: &str) {
    let dir = match ensure_dir(app) {
        Ok(dir) => dir,
        Err(_) => return,
    };
    append_line(&dir.join("app.log"), message);
}

#[tauri::command]
fn log_app(app: AppHandle, message: String) -> Result<(), String> {
    append_app_log(&app, &message);
    Ok(())
}

fn apply_default_window_icon<R: tauri::Runtime>(
    app: &AppHandle<R>,
    window: &tauri::WebviewWindow<R>,
) {
    if let Some(icon) = app.default_window_icon().cloned() {
        let _ = window.set_icon(icon);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let dir = ensure_dir(app.handle())?;
            allow_dir_on_scope(app.handle(), &dir)?;
            if let Some(window) = app.get_webview_window("main") {
                apply_default_window_icon(app.handle(), &window);
                let _ = window.center();
                let _ = window.show();
                let _ = window.set_focus();
            }
            let tray_menu = MenuBuilder::new(app)
                .text("tray_show", "显示主界面")
                .text("tray_hide", "隐藏到托盘")
                .separator()
                .text("tray_quit", "退出")
                .build()?;

            let tray = TrayIconBuilder::new()
                .icon(TRAY_ICON.clone())
                .tooltip("护眼吧")
                .menu(&tray_menu)
                .show_menu_on_left_click(false)
                .on_tray_icon_event(|tray, event| {
                    let app = tray.app_handle();
                    let Some(window) = app.get_webview_window("main") else {
                        return;
                    };
                    match event {
                        TrayIconEvent::Click {
                            button,
                            button_state,
                            ..
                        } => {
                            if button == MouseButton::Left && button_state == MouseButtonState::Up {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        TrayIconEvent::DoubleClick { button, .. } => {
                            if button == MouseButton::Left {
                                let visible = window.is_visible().unwrap_or(true);
                                if visible {
                                    let _ = window.hide();
                                } else {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                            }
                        }
                        _ => {}
                    }
                })
                .on_menu_event(|app, event| {
                    let Some(window) = app.get_webview_window("main") else {
                        return;
                    };
                    match event.id().as_ref() {
                        "tray_show" => {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                        "tray_hide" => {
                            let _ = window.hide();
                        }
                        "tray_quit" => {
                            if let Some(state) = app.try_state::<AppState>() {
                                state.allow_exit.store(true, Ordering::SeqCst);
                            }
                            let _ = apply_gamma(1.0, 1.0, 1.0);
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            app.manage(tray);
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() != "main" {
                return;
            }
            match event {
                WindowEvent::CloseRequested { api, .. } => {
                    if let Some(state) = window.app_handle().try_state::<AppState>() {
                        if !state.allow_exit.load(Ordering::SeqCst) {
                            let _ = window.hide();
                            api.prevent_close();
                            return;
                        }
                    }
                    let _ = apply_gamma(1.0, 1.0, 1.0);
                }
                WindowEvent::Destroyed => {
                    let _ = apply_gamma(1.0, 1.0, 1.0);
                }
                _ => {}
            }
        })
        .manage(LockState::default())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            set_gamma,
            reset_gamma,
            lockscreen_action,
            show_lock_windows,
            hide_lock_windows,
            log_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
