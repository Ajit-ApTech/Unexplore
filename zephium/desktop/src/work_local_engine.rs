//! Native GGUF model runner and downloader for Unexplore.
//! Runs local uncensored GGUF models directly via llama.cpp with Apple Silicon Metal GPU acceleration.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;
use tauri::{AppHandle, Manager, WebviewWindow};
use tokio::io::AsyncWriteExt;

const DEFAULT_LOCAL_PORT: u16 = 11436;

fn find_available_port(preferred: u16) -> u16 {
    for p in preferred..preferred + 100 {
        if let Ok(listener) = std::net::TcpListener::bind(("127.0.0.1", p)) {
            drop(listener);
            return p;
        }
    }
    preferred
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct LocalModelStatus {
    pub is_running: bool,
    pub model_name: Option<String>,
    pub model_path: Option<String>,
    pub port: u16,
    pub endpoint: String,
    pub metal_accelerated: bool,
    pub error: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct ModelFileInfo {
    pub id: String,
    pub name: String,
    pub filename: String,
    pub path: String,
    pub size_bytes: f64,
    pub formatted_size: String,
    pub is_active: bool,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct ModelDownloadProgress {
    pub id: String,
    pub downloaded_bytes: f64,
    pub total_bytes: f64,
    pub percent: f32,
    pub speed_mbps: f32,
    pub status: String,
    pub error: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct CuratedModel {
    pub id: String,
    pub name: String,
    pub filename: String,
    pub url: String,
    pub size_gb: f32,
    pub min_ram_gb: u32,
    pub label: String,
    pub badge: String,
    pub description: String,
    pub recommended: bool,
}

#[derive(Clone)]
struct DownloadTask {
    cancel_flag: Arc<AtomicBool>,
    pause_flag: Arc<AtomicBool>,
    url: String,
    filename: String,
}

struct EngineInner {
    child: Option<std::process::Child>,
    active_model_path: Option<String>,
    active_model_name: Option<String>,
    port: u16,
    endpoint: String,
    error: Option<String>,
    tasks: HashMap<String, DownloadTask>,
    active_downloads: HashMap<String, ModelDownloadProgress>,
}

static ENGINE: OnceLock<Mutex<EngineInner>> = OnceLock::new();

fn engine() -> &'static Mutex<EngineInner> {
    ENGINE.get_or_init(|| {
        Mutex::new(EngineInner {
            child: None,
            active_model_path: None,
            active_model_name: None,
            port: DEFAULT_LOCAL_PORT,
            endpoint: format!("http://127.0.0.1:{DEFAULT_LOCAL_PORT}/v1"),
            error: None,
            tasks: HashMap::new(),
            active_downloads: HashMap::new(),
        })
    })
}

fn models_directory(app: &AppHandle) -> Result<PathBuf, String> {
    let base = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {e}"))?;
    let models_dir = base.join("models");
    std::fs::create_dir_all(&models_dir)
        .map_err(|e| format!("Failed to create models dir {}: {e}", models_dir.display()))?;
    Ok(models_dir)
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.0} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

fn find_llama_server_binary() -> Result<PathBuf, String> {
    let candidates = [
        "/opt/homebrew/bin/llama-server",
        "/usr/local/bin/llama-server",
        "/usr/bin/llama-server",
    ];
    for path_str in candidates {
        let p = Path::new(path_str);
        if p.exists() {
            return Ok(p.to_path_buf());
        }
    }
    // Check PATH via which
    if let Ok(output) = std::process::Command::new("which").arg("llama-server").output() {
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !s.is_empty() && Path::new(&s).exists() {
                return Ok(PathBuf::from(s));
            }
        }
    }
    Err("llama-server binary was not found. Please ensure llama.cpp is installed.".into())
}

/// Native file picker to pick any .gguf model from disk.
#[tauri::command]
#[specta::specta]
pub(crate) async fn local_engine_pick_file(
    caller: WebviewWindow,
    app: AppHandle,
) -> Result<Option<ModelFileInfo>, String> {
    if !super::authorize(&caller, super::CallerPolicy::Main, "local_engine_pick_file") {
        return Err("Unauthorized".into());
    }
    use tauri_plugin_dialog::DialogExt;
    let picker = app.clone();
    let picked = tokio::task::spawn_blocking(move || {
        picker
            .dialog()
            .file()
            .set_title("Select Uncensored GGUF Model")
            .add_filter("GGUF Models (*.gguf)", &["gguf"])
            .blocking_pick_file()
    })
    .await
    .map_err(|e| e.to_string())?;

    let path_buf = match picked {
        Some(tauri_plugin_dialog::FilePath::Path(p)) => p,
        Some(tauri_plugin_dialog::FilePath::Url(u)) => match u.to_file_path() {
            Ok(p) => p,
            Err(()) => return Ok(None),
        },
        None => return Ok(None),
    };

    let filename = path_buf
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "model.gguf".to_string());
    let size_bytes = std::fs::metadata(&path_buf)
        .map(|m| m.len())
        .unwrap_or(0);
    let name = filename
        .trim_end_matches(".gguf")
        .replace(['_', '-'], " ");

    let state = engine().lock().unwrap();
    let is_active = state
        .active_model_path
        .as_deref()
        .map(|p| p == path_buf.to_string_lossy().as_ref())
        .unwrap_or(false);

    Ok(Some(ModelFileInfo {
        id: filename.clone(),
        name,
        filename,
        path: path_buf.to_string_lossy().to_string(),
        size_bytes: size_bytes as f64,
        formatted_size: format_bytes(size_bytes),
        is_active,
    }))
}

/// Starts llama-server with the given model, Metal GPU acceleration, and binds to 127.0.0.1:11435.
#[tauri::command]
#[specta::specta]
pub(crate) async fn local_engine_load_model(
    caller: WebviewWindow,
    app: AppHandle,
    model_path: String,
) -> Result<LocalModelStatus, String> {
    if !super::authorize(&caller, super::CallerPolicy::Main, "local_engine_load_model") {
        return Err("Unauthorized".into());
    }
    let path = PathBuf::from(&model_path);
    if !path.exists() {
        return Err(format!("Model file does not exist at {}", model_path));
    }

    // Kill any existing instance first
    {
        let mut state = engine().lock().unwrap();
        if let Some(mut child) = state.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        state.active_model_path = None;
        state.active_model_name = None;
        state.error = None;
    }

    let binary = find_llama_server_binary()?;
    let filename = path
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "model".to_string());
    let display_name = filename
        .trim_end_matches(".gguf")
        .replace(['_', '-'], " ");

    let model_alias = path
        .file_stem()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "local-model".to_string());

    let port = find_available_port(DEFAULT_LOCAL_PORT);
    let endpoint = format!("http://127.0.0.1:{port}/v1");
    let health_url = format!("http://127.0.0.1:{port}/health");

    let log_path = models_directory(&app)
        .map(|d| d.join("llama_server.log"))
        .unwrap_or_else(|_| PathBuf::from("/tmp/llama_server.log"));
    let log_file = std::fs::File::create(&log_path).ok();
    let err_stdio = if let Some(f) = log_file {
        std::process::Stdio::from(f)
    } else {
        std::process::Stdio::null()
    };

    // Launch llama-server with Metal GPU acceleration (-ngl 99) and 16k context window for agentic browser tools
    let child = std::process::Command::new(&binary)
        .arg("-m")
        .arg(&path)
        .arg("--alias")
        .arg(&model_alias)
        .arg("--host")
        .arg("127.0.0.1")
        .arg("--port")
        .arg(port.to_string())
        .arg("-ngl")
        .arg("99")
        .arg("-c")
        .arg("16384")
        .arg("--flash-attn")
        .arg("auto")
        .stdout(std::process::Stdio::null())
        .stderr(err_stdio)
        .spawn()
        .map_err(|e| format!("Failed to spawn llama-server: {e}"))?;

    {
        let mut state = engine().lock().unwrap();
        state.child = Some(child);
        state.active_model_path = Some(path.to_string_lossy().to_string());
        state.active_model_name = Some(display_name.clone());
        state.port = port;
        state.endpoint = endpoint.clone();
        state.error = None;
    }

    // Poll /health up to 25 seconds for model load
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .map_err(|e| e.to_string())?;

    let start = Instant::now();
    let mut healthy = false;

    while start.elapsed().as_secs() < 25 {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        // Check if process crashed early
        {
            let mut state = engine().lock().unwrap();
            if let Some(child) = state.child.as_mut() {
                if let Ok(Some(status)) = child.try_wait() {
                    let mut log_tail = String::new();
                    if let Ok(content) = std::fs::read_to_string(&log_path) {
                        let lines: Vec<&str> = content.lines().collect();
                        let start = lines.len().saturating_sub(4);
                        log_tail = lines[start..].join(" ");
                    }
                    let err = if log_tail.is_empty() {
                        format!("llama-server exited with {status}")
                    } else {
                        format!("llama-server exited with {status}: {log_tail}")
                    };
                    state.error = Some(err.clone());
                    state.child = None;
                    return Err(err);
                }
            }
        }
        if let Ok(resp) = client.get(&health_url).send().await {
            if resp.status().is_success() {
                healthy = true;
                break;
            }
        }
    }

    if !healthy {
        let mut state = engine().lock().unwrap();
        if let Some(mut child) = state.child.take() {
            let _ = child.kill();
        }
        let err = "llama-server timed out while loading the model into memory.".to_string();
        state.error = Some(err.clone());
        return Err(err);
    }

    // Register local engine model in zephium-app work_models so the browser discovers the model and endpoint
    #[cfg(feature = "work-product")]
    {
        let _ = zephium_app::work_models::register_local_engine_model(&endpoint, &model_alias, &display_name).await;
    }

    // Notify UI that models changed
    super::emit_to_privileged(
        &app,
        super::MAIN_LABEL,
        "zephium:work-models-changed",
        &serde_json::json!({ "version": 1 }),
    );

    Ok(LocalModelStatus {
        is_running: true,
        model_name: Some(display_name),
        model_path: Some(path.to_string_lossy().to_string()),
        port,
        endpoint,
        metal_accelerated: cfg!(target_os = "macos"),
        error: None,
    })
}

/// Unloads the current model and stops the local engine.
#[tauri::command]
#[specta::specta]
pub(crate) async fn local_engine_unload_model(
    caller: WebviewWindow,
    app: AppHandle,
) -> Result<LocalModelStatus, String> {
    if !super::authorize(&caller, super::CallerPolicy::Main, "local_engine_unload_model") {
        return Err("Unauthorized".into());
    }
    let (port, endpoint) = {
        let mut state = engine().lock().unwrap();
        if let Some(mut child) = state.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        state.active_model_path = None;
        state.active_model_name = None;
        state.error = None;
        (state.port, state.endpoint.clone())
    };

    #[cfg(feature = "work-product")]
    {
        let _ = zephium_app::work_models::unregister_local_engine_model().await;
    }

    super::emit_to_privileged(
        &app,
        super::MAIN_LABEL,
        "zephium:work-models-changed",
        &serde_json::json!({ "version": 1 }),
    );

    Ok(LocalModelStatus {
        is_running: false,
        model_name: None,
        model_path: None,
        port,
        endpoint,
        metal_accelerated: cfg!(target_os = "macos"),
        error: None,
    })
}

/// Returns the current running status of the local model engine.
#[tauri::command]
#[specta::specta]
pub(crate) async fn local_engine_get_status(caller: WebviewWindow) -> LocalModelStatus {
    if !super::authorize(&caller, super::CallerPolicy::Main, "local_engine_get_status") {
        return LocalModelStatus {
            is_running: false,
            model_name: None,
            model_path: None,
            port: DEFAULT_LOCAL_PORT,
            endpoint: format!("http://127.0.0.1:{DEFAULT_LOCAL_PORT}/v1"),
            metal_accelerated: false,
            error: Some("Unauthorized".into()),
        };
    }
    let mut state = engine().lock().unwrap();
    let mut is_running = false;
    if let Some(child) = state.child.as_mut() {
        match child.try_wait() {
            Ok(None) => is_running = true,
            Ok(Some(status)) => {
                state.error = Some(format!("Engine stopped with {status}"));
                state.child = None;
            }
            Err(e) => {
                state.error = Some(e.to_string());
                state.child = None;
            }
        }
    }
    LocalModelStatus {
        is_running,
        model_name: if is_running { state.active_model_name.clone() } else { None },
        model_path: if is_running { state.active_model_path.clone() } else { None },
        port: state.port,
        endpoint: state.endpoint.clone(),
        metal_accelerated: cfg!(target_os = "macos"),
        error: state.error.clone(),
    }
}

/// Lists all stored .gguf files in the app's models directory.
#[tauri::command]
#[specta::specta]
pub(crate) async fn local_engine_list_models(
    caller: WebviewWindow,
    app: AppHandle,
) -> Result<Vec<ModelFileInfo>, String> {
    if !super::authorize(&caller, super::CallerPolicy::Main, "local_engine_list_models") {
        return Err("Unauthorized".into());
    }
    let dir = models_directory(&app)?;
    let active_path = {
        let state = engine().lock().unwrap();
        state.active_model_path.clone()
    };

    let mut result = Vec::new();
    let entries = std::fs::read_dir(&dir).map_err(|e| e.to_string())?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "gguf" {
                    let filename = path
                        .file_name()
                        .map(|f| f.to_string_lossy().to_string())
                        .unwrap_or_default();
                    let name = filename
                        .trim_end_matches(".gguf")
                        .replace(['_', '-'], " ");
                    let size_bytes = entry.metadata().map(|m| m.len()).unwrap_or(0);
                    let path_str = path.to_string_lossy().to_string();
                    let is_active = active_path
                        .as_deref()
                        .map(|p| p == path_str)
                        .unwrap_or(false);

                    result.push(ModelFileInfo {
                        id: filename.clone(),
                        name,
                        filename,
                        path: path_str,
                        size_bytes: size_bytes as f64,
                        formatted_size: format_bytes(size_bytes),
                        is_active,
                    });
                }
            }
        }
    }
    result.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(result)
}

pub(crate) fn curated_models_list() -> Vec<CuratedModel> {
    vec![
        CuratedModel {
            id: "qwen2.5-3b-instruct".into(),
            name: "Qwen 2.5 3B (Browser Agent)".into(),
            filename: "qwen2.5-3b-instruct-q4_k_m.gguf".into(),
            url: "https://huggingface.co/Qwen/Qwen2.5-3B-Instruct-GGUF/resolve/main/qwen2.5-3b-instruct-q4_k_m.gguf".into(),
            size_gb: 2.19,
            min_ram_gb: 4,
            label: "AGENTIC".into(),
            badge: "BROWSER AI".into(),
            description: "Exceptional tool-calling and browser navigation capabilities. Built specifically for agentic web browsing and coding.".into(),
            recommended: true,
        },
        CuratedModel {
            id: "llama-3.1-8b-abliterated".into(),
            name: "Llama 3.1 8B (Uncensored Agent)".into(),
            filename: "llama-3.1-8b-instruct-abliterated-q4_k_m.gguf".into(),
            url: "https://huggingface.co/failspy/Llama-3.1-8B-Instruct-abliterated-GGUF/resolve/main/llama-3.1-8b-instruct-abliterated-q4_k_m.gguf".into(),
            size_gb: 4.92,
            min_ram_gb: 8,
            label: "UNCENSORED".into(),
            badge: "TOOL MASTER".into(),
            description: "Uncensored Llama 3.1 with state-of-the-art function calling, browser automation, and deep reasoning.".into(),
            recommended: false,
        },
        CuratedModel {
            id: "gemma-2-2b-abliterated".into(),
            name: "Gemma 2 2B Abliterated".into(),
            filename: "gemma-2-2b-it-abliterated-Q4_K_M.gguf".into(),
            url: "https://huggingface.co/bartowski/gemma-2-2b-it-abliterated-GGUF/resolve/main/gemma-2-2b-it-abliterated-Q4_K_M.gguf".into(),
            size_gb: 1.6,
            min_ram_gb: 4,
            label: "UNCENSORED".into(),
            badge: "FASTEST".into(),
            description: "Ultra-fast, lightweight 2B uncensored model for quick creative writing and chat.".into(),
            recommended: false,
        },
        CuratedModel {
            id: "dolphin-llama3-8b".into(),
            name: "Dolphin 2.9 Llama 3 8B".into(),
            filename: "dolphin-2.9-llama3-8b-Q4_K_M.gguf".into(),
            url: "https://huggingface.co/bartowski/dolphin-2.9-llama3-8b-GGUF/resolve/main/dolphin-2.9-llama3-8b-Q4_K_M.gguf".into(),
            size_gb: 4.9,
            min_ram_gb: 8,
            label: "UNCENSORED".into(),
            badge: "POPULAR".into(),
            description: "Dolphin fine-tune of Llama 3 8B. High accuracy instruction following with no guardrails.".into(),
            recommended: false,
        },
    ]
}

/// Returns curated uncensored model specifications.
#[tauri::command]
#[specta::specta]
pub(crate) async fn local_engine_curated_models(
    caller: WebviewWindow,
) -> Result<Vec<CuratedModel>, String> {
    if !super::authorize(&caller, super::CallerPolicy::Main, "local_engine_curated_models") {
        return Err("Unauthorized".into());
    }
    Ok(curated_models_list())
}

fn spawn_download_task(
    app: AppHandle,
    id: String,
    url: String,
    filename: String,
    cancel_flag: Arc<AtomicBool>,
    pause_flag: Arc<AtomicBool>,
) {
    tokio::spawn(async move {
        let emit_progress = |progress: ModelDownloadProgress| {
            {
                let mut state = engine().lock().unwrap();
                if progress.status == "completed"
                    || progress.status == "cancelled"
                    || progress.status == "error"
                {
                    state.active_downloads.remove(&progress.id);
                } else {
                    state
                        .active_downloads
                        .insert(progress.id.clone(), progress.clone());
                }
            }
            super::emit_to_privileged(
                &app,
                super::MAIN_LABEL,
                "unexplore:model-download-progress",
                &progress,
            );
            use tauri::Emitter;
            let _ = app.emit("unexplore:model-download-progress", &progress);
        };

        let dir = match models_directory(&app) {
            Ok(d) => d,
            Err(e) => {
                emit_progress(ModelDownloadProgress {
                    id: id.clone(),
                    downloaded_bytes: 0.0,
                    total_bytes: 0.0,
                    percent: 0.0,
                    speed_mbps: 0.0,
                    status: "error".into(),
                    error: Some(e),
                });
                return;
            }
        };

        let target_path = dir.join(&filename);
        let partial_path = dir.join(format!("{filename}.downloading"));

        let existing_bytes = if partial_path.exists() {
            tokio::fs::metadata(&partial_path)
                .await
                .map(|m| m.len())
                .unwrap_or(0)
        } else {
            0
        };

        let client = match reqwest::Client::builder().build() {
            Ok(c) => c,
            Err(e) => {
                emit_progress(ModelDownloadProgress {
                    id: id.clone(),
                    downloaded_bytes: existing_bytes as f64,
                    total_bytes: 0.0,
                    percent: 0.0,
                    speed_mbps: 0.0,
                    status: "error".into(),
                    error: Some(e.to_string()),
                });
                return;
            }
        };

        let mut req = client.get(&url);
        if existing_bytes > 0 {
            req = req.header("Range", format!("bytes={existing_bytes}-"));
        }

        let mut resp = match req.send().await {
            Ok(r) => r,
            Err(e) => {
                emit_progress(ModelDownloadProgress {
                    id: id.clone(),
                    downloaded_bytes: existing_bytes as f64,
                    total_bytes: 0.0,
                    percent: 0.0,
                    speed_mbps: 0.0,
                    status: "error".into(),
                    error: Some(e.to_string()),
                });
                return;
            }
        };

        let (mut file, mut downloaded_bytes, total_bytes) =
            if resp.status() == reqwest::StatusCode::PARTIAL_CONTENT {
                let remaining = resp.content_length().unwrap_or(0);
                let total = existing_bytes + remaining;
                let file = match tokio::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&partial_path)
                    .await
                {
                    Ok(f) => f,
                    Err(e) => {
                        emit_progress(ModelDownloadProgress {
                            id: id.clone(),
                            downloaded_bytes: existing_bytes as f64,
                            total_bytes: total as f64,
                            percent: 0.0,
                            speed_mbps: 0.0,
                            status: "error".into(),
                            error: Some(e.to_string()),
                        });
                        return;
                    }
                };
                (file, existing_bytes, total)
            } else if resp.status().is_success() {
                let total = resp.content_length().unwrap_or(0);
                let file = match tokio::fs::File::create(&partial_path).await {
                    Ok(f) => f,
                    Err(e) => {
                        emit_progress(ModelDownloadProgress {
                            id: id.clone(),
                            downloaded_bytes: 0.0,
                            total_bytes: total as f64,
                            percent: 0.0,
                            speed_mbps: 0.0,
                            status: "error".into(),
                            error: Some(e.to_string()),
                        });
                        return;
                    }
                };
                (file, 0, total)
            } else {
                emit_progress(ModelDownloadProgress {
                    id: id.clone(),
                    downloaded_bytes: 0.0,
                    total_bytes: 0.0,
                    percent: 0.0,
                    speed_mbps: 0.0,
                    status: "error".into(),
                    error: Some(format!("HTTP error {}", resp.status())),
                });
                return;
            };

        let start_time = Instant::now();
        let mut last_emit = Instant::now();
        let initial_downloaded = downloaded_bytes;

        // Emit initial downloading state
        let initial_percent = if total_bytes > 0 {
            (downloaded_bytes as f32 / total_bytes as f32) * 100.0
        } else {
            0.0
        };
        emit_progress(ModelDownloadProgress {
            id: id.clone(),
            downloaded_bytes: downloaded_bytes as f64,
            total_bytes: total_bytes as f64,
            percent: initial_percent,
            speed_mbps: 0.0,
            status: "downloading".into(),
            error: None,
        });

        loop {
            if cancel_flag.load(Ordering::Relaxed) {
                let _ = tokio::fs::remove_file(&partial_path).await;
                emit_progress(ModelDownloadProgress {
                    id: id.clone(),
                    downloaded_bytes: 0.0,
                    total_bytes: total_bytes as f64,
                    percent: 0.0,
                    speed_mbps: 0.0,
                    status: "cancelled".into(),
                    error: None,
                });
                return;
            }

            if pause_flag.load(Ordering::Relaxed) {
                let _ = file.flush().await;
                let percent = if total_bytes > 0 {
                    (downloaded_bytes as f32 / total_bytes as f32) * 100.0
                } else {
                    0.0
                };
                emit_progress(ModelDownloadProgress {
                    id: id.clone(),
                    downloaded_bytes: downloaded_bytes as f64,
                    total_bytes: total_bytes as f64,
                    percent,
                    speed_mbps: 0.0,
                    status: "paused".into(),
                    error: None,
                });
                return;
            }

            match resp.chunk().await {
                Ok(Some(chunk)) => {
                    downloaded_bytes += chunk.len() as u64;
                    if let Err(e) = file.write_all(&chunk).await {
                        emit_progress(ModelDownloadProgress {
                            id: id.clone(),
                            downloaded_bytes: downloaded_bytes as f64,
                            total_bytes: total_bytes as f64,
                            percent: 0.0,
                            speed_mbps: 0.0,
                            status: "error".into(),
                            error: Some(e.to_string()),
                        });
                        return;
                    }

                    if last_emit.elapsed().as_millis() > 200 {
                        last_emit = Instant::now();
                        let percent = if total_bytes > 0 {
                            (downloaded_bytes as f32 / total_bytes as f32) * 100.0
                        } else {
                            0.0
                        };
                        let elapsed_sec = start_time.elapsed().as_secs_f32();
                        let session_bytes = downloaded_bytes.saturating_sub(initial_downloaded);
                        let speed_mbps = if elapsed_sec > 0.0 {
                            (session_bytes as f32 / (1024.0 * 1024.0)) / elapsed_sec
                        } else {
                            0.0
                        };
                        emit_progress(ModelDownloadProgress {
                            id: id.clone(),
                            downloaded_bytes: downloaded_bytes as f64,
                            total_bytes: total_bytes as f64,
                            percent,
                            speed_mbps,
                            status: "downloading".into(),
                            error: None,
                        });
                    }
                }
                Ok(None) => break,
                Err(e) => {
                    emit_progress(ModelDownloadProgress {
                        id: id.clone(),
                        downloaded_bytes: downloaded_bytes as f64,
                        total_bytes: total_bytes as f64,
                        percent: 0.0,
                        speed_mbps: 0.0,
                        status: "error".into(),
                        error: Some(e.to_string()),
                    });
                    return;
                }
            }
        }

        let _ = file.flush().await;
        drop(file);

        if let Err(e) = tokio::fs::rename(&partial_path, &target_path).await {
            emit_progress(ModelDownloadProgress {
                id: id.clone(),
                downloaded_bytes: downloaded_bytes as f64,
                total_bytes: total_bytes as f64,
                percent: 0.0,
                speed_mbps: 0.0,
                status: "error".into(),
                error: Some(format!("Failed to finalize file: {e}")),
            });
            return;
        }

        emit_progress(ModelDownloadProgress {
            id: id.clone(),
            downloaded_bytes: total_bytes as f64,
            total_bytes: total_bytes as f64,
            percent: 100.0,
            speed_mbps: 0.0,
            status: "completed".into(),
            error: None,
        });

        super::emit_to_privileged(
            &app,
            super::MAIN_LABEL,
            "zephium:work-models-changed",
            &serde_json::json!({ "version": 1 }),
        );
    });
}

/// Downloads a model in the background with streaming progress events.
#[tauri::command]
#[specta::specta]
pub(crate) async fn local_engine_download_model(
    caller: WebviewWindow,
    app: AppHandle,
    id: String,
    url: String,
    filename: String,
) -> Result<(), String> {
    if !super::authorize(&caller, super::CallerPolicy::Main, "local_engine_download_model") {
        return Err("Unauthorized".into());
    }
    let dir = models_directory(&app)?;
    let target_path = dir.join(&filename);
    if target_path.exists() {
        return Ok(());
    }

    let cancel_flag = Arc::new(AtomicBool::new(false));
    let pause_flag = Arc::new(AtomicBool::new(false));
    {
        let mut state = engine().lock().unwrap();
        state.tasks.insert(
            id.clone(),
            DownloadTask {
                cancel_flag: cancel_flag.clone(),
                pause_flag: pause_flag.clone(),
                url: url.clone(),
                filename: filename.clone(),
            },
        );
    }

    spawn_download_task(app, id, url, filename, cancel_flag, pause_flag);
    Ok(())
}

/// Pauses an active download.
#[tauri::command]
#[specta::specta]
pub(crate) async fn local_engine_pause_download(
    caller: WebviewWindow,
    id: String,
) -> Result<(), String> {
    if !super::authorize(&caller, super::CallerPolicy::Main, "local_engine_pause_download") {
        return Err("Unauthorized".into());
    }
    let state = engine().lock().unwrap();
    if let Some(task) = state.tasks.get(&id) {
        task.pause_flag.store(true, Ordering::Relaxed);
    }
    Ok(())
}

/// Resumes a paused download with HTTP Range header.
#[tauri::command]
#[specta::specta]
pub(crate) async fn local_engine_resume_download(
    caller: WebviewWindow,
    app: AppHandle,
    id: String,
) -> Result<(), String> {
    if !super::authorize(&caller, super::CallerPolicy::Main, "local_engine_resume_download") {
        return Err("Unauthorized".into());
    }
    let (url, filename) = {
        let state = engine().lock().unwrap();
        if let Some(task) = state.tasks.get(&id) {
            (task.url.clone(), task.filename.clone())
        } else if let Some(curated) = curated_models_list().into_iter().find(|m| m.id == id) {
            (curated.url, curated.filename)
        } else {
            return Err("Download task not found".into());
        }
    };

    let cancel_flag = Arc::new(AtomicBool::new(false));
    let pause_flag = Arc::new(AtomicBool::new(false));
    {
        let mut state = engine().lock().unwrap();
        state.tasks.insert(
            id.clone(),
            DownloadTask {
                cancel_flag: cancel_flag.clone(),
                pause_flag: pause_flag.clone(),
                url: url.clone(),
                filename: filename.clone(),
            },
        );
    }

    spawn_download_task(app, id, url, filename, cancel_flag, pause_flag);
    Ok(())
}

/// Cancels an active download and cleans up any partial files.
#[tauri::command]
#[specta::specta]
pub(crate) async fn local_engine_cancel_download(
    caller: WebviewWindow,
    id: String,
) -> Result<(), String> {
    if !super::authorize(&caller, super::CallerPolicy::Main, "local_engine_cancel_download") {
        return Err("Unauthorized".into());
    }
    let (filename_opt, cancel_flag_opt) = {
        let mut state = engine().lock().unwrap();
        state.active_downloads.remove(&id);
        if let Some(task) = state.tasks.remove(&id) {
            (Some(task.filename.clone()), Some(task.cancel_flag.clone()))
        } else if let Some(curated) = curated_models_list().into_iter().find(|m| m.id == id) {
            (Some(curated.filename), None)
        } else {
            (None, None)
        }
    };

    if let Some(flag) = cancel_flag_opt {
        flag.store(true, Ordering::Relaxed);
    }

    let app = caller.app_handle();
    if let Some(filename) = filename_opt {
        if let Ok(dir) = models_directory(&app) {
            let partial = dir.join(format!("{filename}.downloading"));
            if partial.exists() {
                let _ = tokio::fs::remove_file(partial).await;
            }
        }
    }

    let cancel_prog = ModelDownloadProgress {
        id: id.clone(),
        downloaded_bytes: 0.0,
        total_bytes: 0.0,
        percent: 0.0,
        speed_mbps: 0.0,
        status: "cancelled".into(),
        error: None,
    };
    super::emit_to_privileged(
        &app,
        super::MAIN_LABEL,
        "unexplore:model-download-progress",
        &cancel_prog,
    );
    use tauri::Emitter;
    let _ = app.emit("unexplore:model-download-progress", &cancel_prog);

    Ok(())
}

/// Deletes a stored model file from the models directory.
#[tauri::command]
#[specta::specta]
pub(crate) async fn local_engine_delete_model(
    caller: WebviewWindow,
    app: AppHandle,
    filename: String,
) -> Result<(), String> {
    if !super::authorize(&caller, super::CallerPolicy::Main, "local_engine_delete_model") {
        return Err("Unauthorized".into());
    }
    let dir = models_directory(&app)?;
    let target = dir.join(&filename);
    if target.exists() {
        let active_path = {
            let state = engine().lock().unwrap();
            state.active_model_path.clone()
        };
        if let Some(active) = active_path {
            if active == target.to_string_lossy().as_ref() {
                let _ = local_engine_unload_model(caller, app.clone()).await;
            }
        }
        std::fs::remove_file(&target).map_err(|e| e.to_string())?;
    }
    super::emit_to_privileged(
        &app,
        super::MAIN_LABEL,
        "zephium:work-models-changed",
        &serde_json::json!({ "version": 1 }),
    );
    Ok(())
}

/// Returns all currently active downloads and their latest progress.
#[tauri::command]
#[specta::specta]
pub(crate) async fn local_engine_get_downloads(
    caller: WebviewWindow,
) -> Result<Vec<ModelDownloadProgress>, String> {
    if !super::authorize(&caller, super::CallerPolicy::Main, "local_engine_get_downloads") {
        return Err("Unauthorized".into());
    }
    let state = engine().lock().unwrap();
    Ok(state.active_downloads.values().cloned().collect())
}

