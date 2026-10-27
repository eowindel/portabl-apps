//! Découverte des applis et lecture de leurs métadonnées.
//!
//! Sources, de la plus prioritaire à la moins prioritaire :
//! 1. un fichier `<nom>.portal.json` à côté de l'appli (ou `portal.json` dans un dossier d'appli) ;
//! 2. ce que l'appli embarque : balises `<meta>` / `<title>` / favicon pour le HTML,
//!    ressources de version et icône pour les .exe ;
//! 3. le nom du fichier.

use base64::{engine::general_purpose::STANDARD as B64, Engine};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::UNIX_EPOCH;

#[derive(Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppEntry {
    pub id: String,
    pub kind: String,
    pub path: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub category: String,
    pub tags: Vec<String>,
    pub color: String,
    /// URL `data:` d'une image, ou un emoji.
    pub icon: String,
    pub notes: String,
    pub size: u64,
    pub modified: u64,
    /// D'où viennent les infos ("sidecar", "html", "exe").
    pub sources: Vec<String>,
}

/// Contenu possible d'un fichier `.portal.json` — tous les champs sont optionnels.
#[derive(Deserialize, Default)]
#[serde(default)]
struct Sidecar {
    entry: Option<String>,
    name: Option<String>,
    description: Option<String>,
    version: Option<String>,
    author: Option<String>,
    category: Option<String>,
    tags: Option<Vec<String>>,
    color: Option<String>,
    icon: Option<String>,
    notes: Option<String>,
    hidden: Option<bool>,
}

pub fn scan(dir: &Path) -> Vec<AppEntry> {
    let mut apps = Vec::new();
    let Ok(read) = std::fs::read_dir(dir) else { return apps };
    for entry in read.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(app) = folder_app(dir, &path) {
                apps.push(app);
            }
        } else if let Some(kind) = kind_of(&path) {
            let sidecar = path.with_file_name(format!("{}.portal.json", stem(&path)));
            if let Some(app) = build(dir, &path, kind, read_sidecar(&sidecar)) {
                apps.push(app);
            }
        }
    }
    apps.sort_by_key(|a| a.name.to_lowercase());
    apps
}

/// Un sous-dossier est une appli s'il contient un `portal.json` avec `entry`,
/// un `index.html`, ou un unique `.exe`.
fn folder_app(root: &Path, folder: &Path) -> Option<AppEntry> {
    let sidecar = read_sidecar(&folder.join("portal.json"));
    let target = match sidecar.as_ref().and_then(|s| s.entry.clone()) {
        Some(entry) => folder.join(entry),
        None if folder.join("index.html").is_file() => folder.join("index.html"),
        None => {
            let exes: Vec<PathBuf> = std::fs::read_dir(folder)
                .ok()?
                .flatten()
                .map(|e| e.path())
                .filter(|p| kind_of(p) == Some("exe"))
                .collect();
            if exes.len() != 1 {
                return None;
            }
            exes.into_iter().next()?
        }
    };
    let kind = kind_of(&target)?;
    let mut app = build(root, &target, kind, sidecar)?;
    // Pour un dossier, le nom du dossier est un meilleur repli que « index ».
    if app.sources.is_empty() || app.name.eq_ignore_ascii_case("index") {
        app.name = file_name(folder);
    }
    Some(app)
}

fn build(root: &Path, path: &Path, kind: &'static str, sidecar: Option<Sidecar>) -> Option<AppEntry> {
    if sidecar.as_ref().and_then(|s| s.hidden).unwrap_or(false) {
        return None;
    }
    let meta = std::fs::metadata(path).ok()?;
    let mut app = AppEntry {
        id: path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/")
            .to_lowercase(),
        kind: kind.into(),
        path: path.to_string_lossy().into_owned(),
        name: stem(path),
        size: meta.len(),
        modified: meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0),
        ..Default::default()
    };

    match kind {
        "html" => read_html_meta(path, &mut app),
        _ => read_exe_meta(path, &mut app),
    }
    if let Some(s) = sidecar {
        apply_sidecar(path, s, &mut app);
    }
    Some(app)
}

fn apply_sidecar(path: &Path, s: Sidecar, app: &mut AppEntry) {
    let base = path.parent().unwrap_or(Path::new("."));
    let set = |field: &mut String, v: Option<String>| {
        if let Some(v) = v.filter(|v| !v.trim().is_empty()) {
            *field = v;
        }
    };
    set(&mut app.name, s.name);
    set(&mut app.description, s.description);
    set(&mut app.version, s.version);
    set(&mut app.author, s.author);
    set(&mut app.category, s.category);
    set(&mut app.color, s.color);
    set(&mut app.notes, s.notes);
    if let Some(tags) = s.tags {
        app.tags = tags;
    }
    if let Some(icon) = s.icon.and_then(|i| resolve_icon(base, &i)) {
        app.icon = icon;
    }
    app.sources.push("sidecar".into());
}

// ---------------------------------------------------------------- HTML

fn read_html_meta(path: &Path, app: &mut AppEntry) {
    static META: OnceLock<Regex> = OnceLock::new();
    static LINK: OnceLock<Regex> = OnceLock::new();
    static TITLE: OnceLock<Regex> = OnceLock::new();
    static ATTR: OnceLock<Regex> = OnceLock::new();
    let meta_re = META.get_or_init(|| Regex::new(r"(?is)<meta\b[^>]*>").unwrap());
    let link_re = LINK.get_or_init(|| Regex::new(r"(?is)<link\b[^>]*>").unwrap());
    let title_re = TITLE.get_or_init(|| Regex::new(r"(?is)<title[^>]*>(.*?)</title>").unwrap());
    let attr_re = ATTR.get_or_init(|| Regex::new(r#"(?s)([a-zA-Z_:-]+)\s*=\s*(?:"([^"]*)"|'([^']*)')"#).unwrap());

    let Ok(bytes) = std::fs::read(path) else { return };
    let text = String::from_utf8_lossy(&bytes);
    // Les métadonnées sont dans le <head> ; inutile de parcourir le reste d'une appli de 5 Mo.
    let head = match text.to_ascii_lowercase().find("</head>") {
        Some(end) => &text[..end],
        None => &text[..text.len().min(256 * 1024)],
    };

    let attrs = |tag: &str| -> Vec<(String, String)> {
        attr_re
            .captures_iter(tag)
            .map(|c| {
                let v = c.get(2).or_else(|| c.get(3)).map(|m| m.as_str()).unwrap_or("");
                (c[1].to_lowercase(), decode_entities(v))
            })
            .collect()
    };
    let get = |a: &[(String, String)], k: &str| a.iter().find(|(n, _)| n == k).map(|(_, v)| v.clone());

    let mut found = false;
    if let Some(t) = title_re.captures(head) {
        let t = decode_entities(t[1].trim());
        if !t.is_empty() {
            app.name = t;
            found = true;
        }
    }
    let base = path.parent().unwrap_or(Path::new("."));
    for tag in meta_re.find_iter(head) {
        let a = attrs(tag.as_str());
        let (Some(name), Some(content)) = (get(&a, "name").or_else(|| get(&a, "property")), get(&a, "content")) else {
            continue;
        };
        let content = content.trim().to_string();
        if content.is_empty() {
            continue;
        }
        let key = name.to_lowercase();
        found |= key.starts_with("portal:") || matches!(key.as_str(), "description" | "author" | "version");
        match key.as_str() {
            "portal:name" | "application-name" => app.name = content,
            "description" | "portal:description" => app.description = content,
            "version" | "portal:version" => app.version = content,
            "author" | "portal:author" => app.author = content,
            "portal:category" => app.category = content,
            "keywords" | "portal:tags" => {
                app.tags = content.split(',').map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect()
            }
            "portal:color" | "theme-color" => app.color = content,
            "portal:notes" => app.notes = content,
            "portal:icon" => {
                if let Some(i) = resolve_icon(base, &content) {
                    app.icon = i
                }
            }
            _ => {}
        }
    }
    if app.icon.is_empty() {
        for tag in link_re.find_iter(head) {
            let a = attrs(tag.as_str());
            let is_icon = get(&a, "rel").map(|r| r.to_lowercase().contains("icon")).unwrap_or(false);
            if let (true, Some(href)) = (is_icon, get(&a, "href")) {
                if let Some(i) = resolve_icon(base, &href) {
                    app.icon = i;
                    break;
                }
            }
        }
    }
    if found {
        app.sources.push("html".into());
    }
}

fn decode_entities(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
}

// ---------------------------------------------------------------- EXE

fn read_exe_meta(path: &Path, app: &mut AppEntry) {
    use pelite::{FileMap, PeFile};
    let Ok(map) = FileMap::open(path) else { return };
    let Ok(pe) = PeFile::from_bytes(&map) else { return };
    let Ok(res) = pe.resources() else { return };

    if let Ok(vi) = res.version_info() {
        if let Some(&lang) = vi.translation().first() {
            let v = |k: &str| vi.value(lang, k).map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
            if let Some(n) = v("FileDescription").or_else(|| v("ProductName")) {
                app.name = n;
            }
            if let Some(d) = v("Comments") {
                app.description = d;
            }
            if let Some(ver) = v("ProductVersion").or_else(|| v("FileVersion")) {
                app.version = ver;
            }
            if let Some(a) = v("CompanyName").or_else(|| v("LegalCopyright")) {
                app.author = a;
            }
            app.sources.push("exe".into());
        }
    }

    // Première icône du fichier, plus grande résolution disponible.
    let first_icon = res.icons().next();
    if let Some(Ok((_, group))) = first_icon {
        let size = |w: u8| if w == 0 { 256u32 } else { w as u32 };
        if let Some(best) = group
            .entries()
            .iter()
            .max_by_key(|e| (size(e.bWidth), e.wBitCount))
        {
            if let Ok(img) = group.image(best.nId) {
                app.icon = if img.starts_with(b"\x89PNG") {
                    format!("data:image/png;base64,{}", B64.encode(img))
                } else {
                    // Image BMP brute : on la ré-emballe dans un .ico à une seule entrée.
                    let mut ico = Vec::with_capacity(22 + img.len());
                    ico.extend_from_slice(&[0, 0, 1, 0, 1, 0]);
                    ico.extend_from_slice(&[best.bWidth, best.bHeight, best.bColorCount, 0]);
                    ico.extend_from_slice(&1u16.to_le_bytes());
                    ico.extend_from_slice(&best.wBitCount.to_le_bytes());
                    ico.extend_from_slice(&(img.len() as u32).to_le_bytes());
                    ico.extend_from_slice(&22u32.to_le_bytes());
                    ico.extend_from_slice(img);
                    format!("data:image/x-icon;base64,{}", B64.encode(&ico))
                };
            }
        }
    }
}

// ---------------------------------------------------------------- utilitaires

fn kind_of(path: &Path) -> Option<&'static str> {
    match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
        "html" | "htm" => Some("html"),
        "exe" => Some("exe"),
        _ => None,
    }
}

fn stem(path: &Path) -> String {
    path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()
}

fn file_name(path: &Path) -> String {
    path.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()
}

fn read_sidecar(path: &Path) -> Option<Sidecar> {
    let text = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(text.trim_start_matches('\u{feff}')).ok()
}

/// Transforme une valeur d'icône en quelque chose d'affichable :
/// URL `data:` telle quelle, fichier image local → `data:`, sinon emoji / texte court.
fn resolve_icon(base: &Path, value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    if value.starts_with("data:") {
        return Some(value.to_string());
    }
    let mime = match Path::new(value).extension().and_then(|e| e.to_str()).map(|e| e.to_ascii_lowercase()) {
        Some(ext) => match ext.as_str() {
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "svg" => "image/svg+xml",
            "ico" => "image/x-icon",
            _ => return None,
        },
        // Pas d'extension : emoji ou sigle (« 🏍️ », « XL »).
        None => return (value.chars().count() <= 4).then(|| value.to_string()),
    };
    let file = base.join(value.trim_start_matches("./"));
    let bytes = std::fs::read(file).ok()?;
    Some(format!("data:{mime};base64,{}", B64.encode(bytes)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_windows_exe_metadata() {
        let mut app = AppEntry::default();
        read_exe_meta(Path::new(r"C:\Windows\System32\cmd.exe"), &mut app);
        assert!(app.sources.contains(&"exe".to_string()));
        assert!(!app.version.is_empty());
        assert!(app.icon.starts_with("data:image/"));
    }
}
