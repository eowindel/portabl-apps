mod scan;
mod settings;

use scan::AppEntry;
use serde::{Deserialize, Serialize};
use settings::{Settings, Stat};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;
use tauri_plugin_dialog::DialogExt;

const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;

struct AppState(Mutex<Settings>);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Snapshot {
    settings: Settings,
    apps_dir: String,
    apps: Vec<AppEntry>,
}

/// Réglages modifiables depuis l'interface (les favoris et stats ont leurs propres commandes).
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Prefs {
    apps_dir: Option<String>,
    theme: Option<String>,
    browser_mode: Option<String>,
    browser_path: Option<String>,
    browser_args: Option<String>,
    sound: Option<bool>,
}

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

#[tauri::command]
fn load(state: State<AppState>) -> Snapshot {
    let settings = state.0.lock().unwrap().clone();
    let dir = settings.apps_dir();
    let _ = std::fs::create_dir_all(&dir);
    Snapshot {
        apps: scan::scan(&dir),
        apps_dir: dir.to_string_lossy().into_owned(),
        settings,
    }
}

#[tauri::command]
fn update_prefs(prefs: Prefs, state: State<AppState>) -> Result<Settings, String> {
    let mut s = state.0.lock().unwrap();
    if let Some(v) = prefs.apps_dir { s.apps_dir = v }
    if let Some(v) = prefs.theme { s.theme = v }
    if let Some(v) = prefs.browser_mode { s.browser_mode = v }
    if let Some(v) = prefs.browser_path { s.browser_path = v }
    if let Some(v) = prefs.browser_args { s.browser_args = v }
    if let Some(v) = prefs.sound { s.sound = v }
    s.save()?;
    Ok(s.clone())
}

#[tauri::command]
fn toggle_favorite(id: String, state: State<AppState>) -> Result<Settings, String> {
    let mut s = state.0.lock().unwrap();
    if let Some(i) = s.favorites.iter().position(|f| f == &id) {
        s.favorites.remove(i);
    } else {
        s.favorites.push(id);
    }
    s.save()?;
    Ok(s.clone())
}

/// Vérifie que le chemin demandé est bien dans le dossier d'applis (le front ne peut pas lancer n'importe quoi).
fn checked_path(s: &Settings, path: &str) -> Result<PathBuf, String> {
    let root = s.apps_dir().canonicalize().map_err(|e| e.to_string())?;
    let p = Path::new(path).canonicalize().map_err(|_| "Fichier introuvable".to_string())?;
    if p.starts_with(&root) { Ok(p) } else { Err("Chemin hors du dossier d'applis".into()) }
}

/// `canonicalize` renvoie des chemins `\\?\C:\...` que certains programmes n'aiment pas.
fn plain(p: &Path) -> PathBuf {
    PathBuf::from(p.to_string_lossy().trim_start_matches(r"\\?\"))
}

fn find_edge() -> Option<PathBuf> {
    ["ProgramFiles(x86)", "ProgramFiles", "LOCALAPPDATA"]
        .iter()
        .filter_map(|v| std::env::var(v).ok())
        .map(|base| Path::new(&base).join(r"Microsoft\Edge\Application\msedge.exe"))
        .find(|p| p.is_file())
}

fn open_html(s: &Settings, path: &Path) -> Result<(), String> {
    let url = url::Url::from_file_path(path).map_err(|_| "Chemin invalide")?.to_string();
    let spawn = |exe: &Path, args: Vec<String>| {
        Command::new(exe).args(args).spawn().map(|_| ()).map_err(|e| e.to_string())
    };
    match s.browser_mode.as_str() {
        "custom" if !s.browser_path.trim().is_empty() => {
            let args = if s.browser_args.contains("{url}") { s.browser_args.clone() } else { format!("{} {{url}}", s.browser_args) };
            let args = args.split_whitespace().map(|a| a.replace("{url}", &url)).collect();
            spawn(Path::new(s.browser_path.trim()), args)
        }
        "app" => match find_edge() {
            // Fenêtre dédiée sans barre d'onglets : chaque appli HTML ressemble à une appli native.
            Some(edge) => spawn(&edge, vec![format!("--app={url}"), "--new-window".into()]),
            None => spawn(Path::new("explorer.exe"), vec![path.to_string_lossy().into()]),
        },
        _ => spawn(Path::new("explorer.exe"), vec![path.to_string_lossy().into()]),
    }
}

#[tauri::command]
fn launch(id: String, path: String, state: State<AppState>) -> Result<Settings, String> {
    let mut s = state.0.lock().unwrap();
    let p = plain(&checked_path(&s, &path)?);
    let is_exe = p.extension().map(|e| e.eq_ignore_ascii_case("exe")).unwrap_or(false);
    if is_exe {
        Command::new(&p)
            .current_dir(p.parent().unwrap_or(Path::new(".")))
            .creation_flags(CREATE_NEW_CONSOLE)
            .spawn()
            .map_err(|e| format!("Lancement impossible : {e}"))?;
    } else {
        open_html(&s, &p)?;
    }
    let stat = s.stats.entry(id).or_insert_with(Stat::default);
    stat.count += 1;
    stat.last = now_ms();
    s.save()?;
    Ok(s.clone())
}

#[tauri::command]
fn reveal(path: String, state: State<AppState>) -> Result<(), String> {
    let s = state.0.lock().unwrap();
    let p = plain(&checked_path(&s, &path)?);
    Command::new("explorer.exe")
        .raw_arg(format!("/select,\"{}\"", p.display()))
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn open_apps_dir(state: State<AppState>) -> Result<(), String> {
    let dir = state.0.lock().unwrap().apps_dir();
    let _ = std::fs::create_dir_all(&dir);
    Command::new("explorer.exe").arg(dir).spawn().map(|_| ()).map_err(|e| e.to_string())
}

#[derive(Serialize)]
struct About {
    version: &'static str,
    changelog: &'static str,
}

/// Version (celle de Cargo.toml) et changelog, embarqués dans l'exe à la compilation.
#[tauri::command]
fn about() -> About {
    About {
        version: env!("CARGO_PKG_VERSION"),
        changelog: include_str!("../../CHANGELOG.md"),
    }
}

#[tauri::command]
async fn pick_folder(app: tauri::AppHandle) -> Option<String> {
    app.dialog()
        .file()
        .set_title("Dossier des applis")
        .blocking_pick_folder()
        .and_then(|f| f.into_path().ok())
        .map(|p| p.to_string_lossy().into_owned())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState(Mutex::new(Settings::load())))
        .invoke_handler(tauri::generate_handler![
            load,
            update_prefs,
            toggle_favorite,
            launch,
            reveal,
            open_apps_dir,
            pick_folder,
            about
        ])
        .run(tauri::generate_context!())
        .expect("erreur au démarrage de Porta[b]l Apps");
}
