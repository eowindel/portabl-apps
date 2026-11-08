mod scan;
mod settings;
mod update;

use scan::AppEntry;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use settings::{Settings, Stat};
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{Manager, State};
use tauri_plugin_dialog::DialogExt;

#[cfg(windows)]
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
    check_updates: Option<bool>,
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
    if let Some(v) = prefs.check_updates { s.check_updates = v }
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

/// Navigateur capable d'ouvrir une fenêtre dédiée (`--app=`) : Edge sous Windows.
#[cfg(windows)]
fn find_app_browser() -> Option<PathBuf> {
    ["ProgramFiles(x86)", "ProgramFiles", "LOCALAPPDATA"]
        .iter()
        .filter_map(|v| std::env::var(v).ok())
        .map(|base| Path::new(&base).join(r"Microsoft\Edge\Application\msedge.exe"))
        .find(|p| p.is_file())
}

/// Sous Linux : le premier navigateur Chromium / Chrome / Brave / Edge trouvé dans le PATH.
#[cfg(unix)]
fn find_app_browser() -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    let dirs: Vec<PathBuf> = std::env::split_paths(&path).collect();
    [
        "chromium",
        "chromium-browser",
        "google-chrome",
        "google-chrome-stable",
        "brave-browser",
        "microsoft-edge",
        "microsoft-edge-stable",
    ]
    .iter()
    .find_map(|name| dirs.iter().map(|d| d.join(name)).find(|p| p.is_file()))
}

/// Ouvre un fichier ou un dossier avec l'application par défaut du système.
fn open_default(target: &Path) -> Result<(), String> {
    #[cfg(windows)]
    let mut cmd = Command::new("explorer.exe");
    #[cfg(unix)]
    let mut cmd = Command::new("xdg-open");
    cmd.arg(target).spawn().map(|_| ()).map_err(|e| e.to_string())
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
        "app" => match find_app_browser() {
            // Fenêtre dédiée sans barre d'onglets : chaque appli HTML ressemble à une appli native.
            Some(browser) => spawn(&browser, vec![format!("--app={url}"), "--new-window".into()]),
            None => open_default(path),
        },
        _ => open_default(path),
    }
}

#[tauri::command]
fn launch(id: String, path: String, state: State<AppState>) -> Result<Settings, String> {
    let mut s = state.0.lock().unwrap();
    let p = plain(&checked_path(&s, &path)?);
    let ext = p.extension().map(|e| e.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
    match ext.as_str() {
        #[cfg(windows)]
        "exe" => {
            Command::new(&p)
                .current_dir(p.parent().unwrap_or(Path::new(".")))
                .creation_flags(CREATE_NEW_CONSOLE)
                .spawn()
                .map_err(|e| format!("Lancement impossible : {e}"))?;
        }
        // Windows ouvre le raccourci lui-même : cible, arguments et dossier de départ sont respectés.
        #[cfg(windows)]
        "lnk" | "url" => {
            Command::new("explorer.exe")
                .arg(&p)
                .spawn()
                .map_err(|e| format!("Lancement impossible : {e}"))?;
        }
        // Sous Linux, seules les applis HTML se lancent.
        #[cfg(unix)]
        "exe" | "lnk" | "url" => {
            return Err("Les applis .exe et les raccourcis .lnk / .url ne se lancent que sous Windows".into());
        }
        _ => open_html(&s, &p)?,
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
    #[cfg(windows)]
    {
        Command::new("explorer.exe")
            .raw_arg(format!("/select,\"{}\"", p.display()))
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
    // xdg-open ne sait pas sélectionner un fichier : on ouvre son dossier.
    #[cfg(unix)]
    {
        open_default(p.parent().unwrap_or(&p))
    }
}

#[tauri::command]
fn open_apps_dir(state: State<AppState>) -> Result<(), String> {
    let dir = state.0.lock().unwrap().apps_dir();
    let _ = std::fs::create_dir_all(&dir);
    open_default(&dir)
}

/// Champs qu'on accepte d'écrire dans un .portal.json depuis l'éditeur.
const META_FIELDS: [&str; 12] =
    ["name", "description", "version", "author", "category", "tags", "color", "icon", "notes", "repo", "hidden", "entry"];

/// Enregistre les champs modifiés dans le .portal.json de l'appli.
/// Un champ vide est retiré du fichier ; les champs inconnus déjà présents sont conservés ;
/// un fichier qui ne contient plus rien est supprimé.
#[tauri::command]
fn save_meta(path: String, fields: Map<String, Value>, state: State<AppState>) -> Result<(), String> {
    let s = state.0.lock().unwrap();
    let app = checked_path(&s, &path)?;
    let root = s.apps_dir().canonicalize().map_err(|e| e.to_string())?;
    let file = scan::sidecar_path(&root, &app);
    let in_folder = file.file_name().is_some_and(|n| n == "portal.json");

    let mut obj = match scan::read_json_object(&file)? {
        Some(Value::Object(o)) => o,
        _ => Map::new(),
    };
    for (key, value) in fields {
        if !META_FIELDS.contains(&key.as_str()) {
            continue;
        }
        let value = match value {
            Value::String(v) => Value::String(v.trim().to_string()),
            Value::Array(tags) => Value::Array(
                tags.into_iter()
                    .filter_map(|t| t.as_str().map(|t| t.trim().to_string()))
                    .filter(|t| !t.is_empty())
                    .map(Value::String)
                    .collect(),
            ),
            v => v,
        };
        let empty = match &value {
            Value::Null | Value::Bool(false) => true,
            Value::String(v) => v.is_empty(),
            Value::Array(v) => v.is_empty(),
            _ => false,
        };
        if empty {
            obj.shift_remove(&key);
            continue;
        }
        if key == "repo" {
            match value.as_str().and_then(update::normalize_repo) {
                Some(repo) => {
                    obj.insert(key, Value::String(repo));
                    continue;
                }
                None => return Err("Dépôt GitHub invalide : attendu « propriétaire/nom »".into()),
            }
        }
        if key == "entry" {
            let entry = value.as_str().unwrap_or_default();
            let target = file.parent().unwrap_or(&root).join(entry);
            let inside = Path::new(entry).components().all(|c| matches!(c, std::path::Component::Normal(_)));
            let launchable = Path::new(entry).extension().is_some_and(|e| {
                ["html", "htm", "exe"].iter().any(|x| e.eq_ignore_ascii_case(x))
            });
            if !in_folder {
                return Err("« entry » ne sert que pour une appli rangée dans un sous-dossier".into());
            }
            if !inside || !launchable || !target.is_file() {
                return Err(format!("Fichier de lancement introuvable dans le dossier : {entry}"));
            }
        }
        obj.insert(key, value);
    }

    if obj.is_empty() {
        if file.exists() {
            std::fs::remove_file(&file).map_err(|e| format!("Suppression impossible : {e}"))?;
        }
        return Ok(());
    }
    let json = serde_json::to_string_pretty(&Value::Object(obj)).map_err(|e| e.to_string())?;
    std::fs::write(&file, json + "\n").map_err(|e| format!("Écriture impossible ({}) : {e}", plain(&file).display()))
}

#[derive(Serialize)]
struct IconPick {
    /// Valeur à mettre dans `icon` (chemin relatif à l'appli).
    value: String,
    /// Aperçu affichable (`data:`).
    preview: String,
}

/// Choix d'une image pour l'icône. Une image prise ailleurs est copiée à côté de l'appli,
/// pour que l'appli et son icône voyagent ensemble.
#[tauri::command]
async fn pick_icon(app: tauri::AppHandle, path: String) -> Result<Option<IconPick>, String> {
    let target = checked_path(&app.state::<AppState>().0.lock().unwrap(), &path)?;
    let base = target.parent().ok_or("Dossier introuvable")?.to_path_buf();
    let Some(picked) = app
        .dialog()
        .file()
        .set_title("Icône de l'appli")
        .set_directory(plain(&base))
        .add_filter("Images", &["png", "svg", "ico", "jpg", "jpeg", "gif", "webp"])
        .blocking_pick_file()
        .and_then(|f| f.into_path().ok())
    else {
        return Ok(None);
    };
    let picked = picked.canonicalize().map_err(|e| e.to_string())?;
    let rel = match picked.strip_prefix(&base) {
        Ok(rel) => rel.to_path_buf(),
        Err(_) => {
            let name = picked.file_name().ok_or("Fichier invalide")?.to_string_lossy().into_owned();
            let stem = target.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
            let name = if base.join(&name).exists() { format!("{stem}-{name}") } else { name };
            std::fs::copy(&picked, base.join(&name)).map_err(|e| format!("Copie de l'image impossible : {e}"))?;
            PathBuf::from(name)
        }
    };
    let value = rel.to_string_lossy().replace('\\', "/");
    let preview = scan::resolve_icon(&base, &value).ok_or("Image illisible")?;
    Ok(Some(IconPick { value, preview }))
}

#[derive(Serialize)]
struct License {
    name: &'static str,
    text: &'static str,
}

#[derive(Serialize)]
struct About {
    version: &'static str,
    changelog: &'static str,
    /// Licence de l'appli, puis celles des polices embarquées (OFL : elle doit voyager avec elles).
    licenses: Vec<License>,
}

/// Version (celle de Cargo.toml), changelog et licences, embarqués dans l'exe à la compilation.
#[tauri::command]
fn about() -> About {
    let license = |name, text| License { name, text };
    About {
        version: env!("CARGO_PKG_VERSION"),
        changelog: include_str!("../../CHANGELOG.md"),
        licenses: vec![
            license("Porta[b]l Apps — MIT", include_str!("../../LICENSE")),
            license("Orbitron — SIL OFL 1.1", include_str!("../../src/fonts/LICENSE-orbitron.txt")),
            license("Chakra Petch — SIL OFL 1.1", include_str!("../../src/fonts/LICENSE-chakra-petch.txt")),
            license("Press Start 2P — SIL OFL 1.1", include_str!("../../src/fonts/LICENSE-press-start-2p.txt")),
            license("Cinzel — SIL OFL 1.1", include_str!("../../src/fonts/LICENSE-cinzel.txt")),
            license("Oswald — SIL OFL 1.1", include_str!("../../src/fonts/LICENSE-oswald.txt")),
        ],
    }
}

/// Dernières releases GitHub du launcher et des applis qui déclarent un `repo`.
/// Sans l'autorisation des réglages, rien ne part sur le réseau.
#[tauri::command]
async fn check_updates(force: bool, state: State<'_, AppState>) -> Result<UpdateReport, String> {
    let (settings, dir) = {
        let s = state.0.lock().unwrap();
        if !s.check_updates {
            return Err("Vérification des mises à jour désactivée dans les réglages".into());
        }
        (s.clone(), s.apps_dir())
    };
    let mut repos: BTreeSet<String> = scan::scan(&dir).into_iter().filter(|a| !a.repo.is_empty()).map(|a| a.repo).collect();
    repos.insert(update::LAUNCHER_REPO.to_string());
    let now = now_ms();
    let (releases, error) = tauri::async_runtime::spawn_blocking(move || update::check(&settings, repos, force, now))
        .await
        .map_err(|e| e.to_string())?;
    let mut s = state.0.lock().unwrap();
    s.update_cache = releases.clone();
    s.save()?;
    Ok(UpdateReport { launcher_repo: update::LAUNCHER_REPO, releases, error })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateReport {
    launcher_repo: &'static str,
    releases: HashMap<String, update::Release>,
    /// Dernière erreur rencontrée (réseau coupé, limite GitHub…), pour l'afficher sans bloquer.
    error: Option<String>,
}

/// Ouvre une page GitHub (page de release) dans le navigateur par défaut.
#[tauri::command]
fn open_url(url: String) -> Result<(), String> {
    if !update::is_github_url(&url) {
        return Err("Seules les pages github.com s'ouvrent depuis ici".into());
    }
    #[cfg(windows)]
    let mut cmd = Command::new("explorer.exe");
    #[cfg(unix)]
    let mut cmd = Command::new("xdg-open");
    cmd.arg(url).spawn().map(|_| ()).map_err(|e| e.to_string())
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

/// Linux : un binaire nu n'a ni icône ni lanceur pour le dock. On installe donc, au
/// démarrage, une entrée `.desktop` (nom = identifiant de l'appli = app_id Wayland) et
/// l'icône dans `~/.local/share`, avec `Exec` pointant vers le binaire en cours.
#[cfg(target_os = "linux")]
fn install_desktop_entry() {
    use std::fs;
    const ID: &str = "dev.portablapps.launcher";
    let (Some(home), Ok(exe)) = (std::env::var_os("HOME"), std::env::current_exe()) else { return };
    let share = std::path::Path::new(&home).join(".local/share");
    let icon_path = share.join("icons/hicolor/128x128/apps").join(format!("{ID}.png"));
    let desktop_path = share.join("applications").join(format!("{ID}.desktop"));
    let exec = exe.to_string_lossy().replace('\\', "\\\\").replace('"', "\\\"");
    let desktop = format!(
        "[Desktop Entry]\nType=Application\nName=Porta[b]l Apps\n\
         Comment=Lanceur local pour applis standalone\nExec=\"{exec}\"\nIcon={ID}\n\
         Terminal=false\nCategories=Utility;\nStartupWMClass={ID}\n"
    );
    let icon = include_bytes!("../icons/128x128.png");
    let _ = (|| -> std::io::Result<()> {
        if fs::read(&icon_path).ok().as_deref() != Some(icon.as_slice()) {
            fs::create_dir_all(icon_path.parent().unwrap())?;
            fs::write(&icon_path, icon)?;
        }
        if fs::read_to_string(&desktop_path).ok().as_deref() != Some(desktop.as_str()) {
            fs::create_dir_all(desktop_path.parent().unwrap())?;
            fs::write(&desktop_path, &desktop)?;
        }
        Ok(())
    })();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "linux")]
    {
        // GDK en déduit l'app_id Wayland / la classe X11 : doit égaler le nom du .desktop.
        glib::set_prgname(Some("dev.portablapps.launcher"));
        install_desktop_entry();
    }
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
            check_updates,
            open_url,
            save_meta,
            pick_icon,
            about
        ])
        .run(tauri::generate_context!())
        .expect("erreur au démarrage de Porta[b]l Apps");
}
