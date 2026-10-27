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
    /// `"hidden": true` dans le .portal.json : l'interface ne l'affiche qu'à la demande.
    pub hidden: bool,
    /// Appli rangée dans un sous-dossier (champ `entry` possible).
    pub folder: bool,
    /// Fichier .portal.json de l'appli, qu'il existe déjà ou non.
    pub meta_file: String,
    /// Contenu brut du .portal.json (objet vide s'il n'existe pas), pour l'éditeur.
    pub sidecar: serde_json::Value,
    /// Infos trouvées sans le .portal.json : ce que l'éditeur propose en grisé.
    pub detected: Detected,
}

#[derive(Serialize, Clone, Default)]
pub struct Detected {
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub category: String,
    pub tags: Vec<String>,
    pub color: String,
    pub icon: String,
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
            let meta_file = sidecar_path(dir, &path);
            if let Some(app) = build(dir, &path, kind, &meta_file, None) {
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
    let meta_file = folder.join("portal.json");
    let entry = read_sidecar(&meta_file).and_then(|(s, _)| s.entry);
    let target = match entry {
        Some(entry) => folder.join(entry),
        None if folder.join("index.html").is_file() => folder.join("index.html"),
        None => {
            let files: Vec<PathBuf> = std::fs::read_dir(folder).ok()?.flatten().map(|e| e.path()).collect();
            let only = |kinds: &[&str]| {
                let found: Vec<&PathBuf> = files.iter().filter(|p| kind_of(p).is_some_and(|k| kinds.contains(&k))).collect();
                (found.len() == 1).then(|| found[0].clone())
            };
            // Un seul .exe ; à défaut, un seul raccourci (.lnk / .url).
            let has_exe = files.iter().any(|p| kind_of(p) == Some("exe"));
            if has_exe { only(&["exe"])? } else { only(&["lnk", "url"])? }
        }
    };
    let kind = kind_of(&target)?;
    build(root, &target, kind, &meta_file, Some(file_name(folder)))
}

/// Emplacement du .portal.json d'une appli : `portal.json` dans son sous-dossier,
/// ou `<nom>.portal.json` à côté d'un fichier posé à la racine.
pub fn sidecar_path(root: &Path, app: &Path) -> PathBuf {
    let rel = app.strip_prefix(root).unwrap_or(app);
    match rel.components().next() {
        Some(first) if rel.components().count() > 1 => root.join(first).join("portal.json"),
        _ => app.with_file_name(format!("{}.portal.json", stem(app))),
    }
}

fn build(root: &Path, path: &Path, kind: &'static str, meta_file: &Path, folder_name: Option<String>) -> Option<AppEntry> {
    let sidecar = read_sidecar(meta_file);
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
        "lnk" => read_lnk_meta(path, &mut app),
        "url" => read_url_meta(path, &mut app),
        _ => read_exe_meta(path, &mut app),
    }
    // Pour un dossier, le nom du dossier est un meilleur repli que « index ».
    if let Some(name) = &folder_name {
        if app.sources.is_empty() || app.name.eq_ignore_ascii_case("index") {
            app.name = name.clone();
        }
    }
    app.folder = folder_name.is_some();
    app.meta_file = meta_file.to_string_lossy().into_owned();
    app.detected = Detected {
        name: app.name.clone(),
        description: app.description.clone(),
        version: app.version.clone(),
        author: app.author.clone(),
        category: app.category.clone(),
        tags: app.tags.clone(),
        color: app.color.clone(),
        icon: app.icon.clone(),
        sources: app.sources.clone(),
    };
    app.sidecar = serde_json::Value::Object(Default::default());
    if let Some((s, raw)) = sidecar {
        app.hidden = s.hidden.unwrap_or(false);
        app.sidecar = raw;
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

    if let Some(icon) = exe_icon(path) {
        app.icon = icon;
    }
}

/// Première icône d'un .exe / .dll, plus grande résolution disponible, en URL `data:`.
fn exe_icon(path: &Path) -> Option<String> {
    use pelite::{FileMap, PeFile};
    let map = FileMap::open(path).ok()?;
    let pe = PeFile::from_bytes(&map).ok()?;
    let res = pe.resources().ok()?;
    let first_icon = res.icons().next();
    let Some(Ok((_, group))) = first_icon else { return None };
    let size = |w: u8| if w == 0 { 256u32 } else { w as u32 };
    let best = group.entries().iter().max_by_key(|e| (size(e.bWidth), e.wBitCount))?;
    let img = group.image(best.nId).ok()?;
    Some(if img.starts_with(b"\x89PNG") {
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
    })
}

// ---------------------------------------------------------------- raccourcis

/// Icône d'un fichier : image locale, ou première icône d'un .exe / .dll.
fn icon_from_file(file: &Path) -> Option<String> {
    let ext = file.extension()?.to_str()?.to_ascii_lowercase();
    match ext.as_str() {
        "exe" | "dll" | "scr" => exe_icon(file),
        _ => resolve_icon(file.parent()?, file.file_name()?.to_str()?),
    }
}

/// Remplace les `%VARIABLES%` d'un chemin (les raccourcis en contiennent souvent).
fn expand_env(value: &str) -> String {
    static VAR: OnceLock<Regex> = OnceLock::new();
    let re = VAR.get_or_init(|| Regex::new(r"%([^%]+)%").unwrap());
    re.replace_all(value, |c: &regex::Captures| std::env::var(&c[1]).unwrap_or_else(|_| c[0].to_string())).into_owned()
}

/// Ce qu'on lit d'un fichier `.lnk`.
#[derive(Default)]
struct Lnk {
    description: Option<String>,
    relative: Option<String>,
    target: Option<String>,
    icon: Option<String>,
}

/// Lecture minimale d'un .lnk (spécification MS-SHLLINK) : jamais de panique, `None` si le fichier est abîmé.
fn parse_lnk(b: &[u8]) -> Option<Lnk> {
    let u16_at = |o: usize| b.get(o..o + 2).map(|s| u16::from_le_bytes([s[0], s[1]]));
    let u32_at = |o: usize| b.get(o..o + 4).map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]]));
    if u32_at(0)? != 0x4C {
        return None;
    }
    let flags = u32_at(0x14)?;
    let mut pos = 0x4C;
    if flags & 0x01 != 0 {
        pos += 2 + u16_at(pos)? as usize; // LinkTargetIDList
    }
    let mut target = None;
    if flags & 0x02 != 0 {
        let size = u32_at(pos)? as usize;
        target = lnk_target(b.get(pos..pos.checked_add(size)?)?);
        pos += size;
    }
    let unicode = flags & 0x80 != 0;
    // StringData : nom, chemin relatif, dossier de travail, arguments, icône.
    let mut strings: [Option<String>; 5] = Default::default();
    for (i, bit) in [0x04u32, 0x08, 0x10, 0x20, 0x40].into_iter().enumerate() {
        if flags & bit == 0 {
            continue;
        }
        let n = u16_at(pos)? as usize;
        pos += 2;
        let len = if unicode { n * 2 } else { n };
        let raw = b.get(pos..pos.checked_add(len)?)?;
        pos += len;
        strings[i] = Some(if unicode {
            String::from_utf16_lossy(&raw.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect::<Vec<_>>())
        } else {
            raw.iter().map(|&c| c as char).collect()
        });
    }
    let [description, relative, _, _, icon] = strings;
    Some(Lnk { description, relative, target, icon })
}

/// Cible locale d'un raccourci, d'après sa section LinkInfo.
fn lnk_target(info: &[u8]) -> Option<String> {
    let u32_at = |o: usize| info.get(o..o + 4).map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]])).map(|v| v as usize);
    let ansi = |o: usize| {
        let tail = info.get(o..)?;
        let end = tail.iter().position(|&c| c == 0)?;
        Some(tail[..end].iter().map(|&c| c as char).collect::<String>())
    };
    let wide = |o: usize| {
        let tail = info.get(o..)?;
        let units: Vec<u16> = tail.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).take_while(|&u| u != 0).collect();
        Some(String::from_utf16_lossy(&units))
    };
    let header = u32_at(4)?;
    if u32_at(8)? & 1 == 0 {
        return None; // cible sur le réseau : pas de chemin local
    }
    let (base, suffix) = if header >= 0x24 {
        (wide(u32_at(28)?), wide(u32_at(32)?))
    } else {
        (ansi(u32_at(16)?), ansi(u32_at(24)?))
    };
    let (base, suffix) = (base.filter(|b| !b.is_empty()), suffix.unwrap_or_default());
    let mut path = base?;
    if !suffix.is_empty() {
        if !path.ends_with('\\') {
            path.push('\\');
        }
        path.push_str(&suffix);
    }
    Some(path)
}

fn read_lnk_meta(path: &Path, app: &mut AppEntry) {
    let Ok(bytes) = std::fs::read(path) else { return };
    let Some(link) = parse_lnk(&bytes) else { return };
    let base = path.parent().unwrap_or(Path::new("."));
    let target = link
        .target
        .or_else(|| link.relative.map(|r| base.join(expand_env(&r)).to_string_lossy().into_owned()))
        .map(|t| expand_env(&t));
    match link.description.map(|d| d.trim().to_string()).filter(|d| !d.is_empty()) {
        Some(d) => app.description = d,
        None => {
            if let Some(t) = &target {
                app.description = t.clone();
            }
        }
    }
    // Icône du raccourci, sinon celle de sa cible.
    let own = link.icon.map(|i| expand_env(&i)).filter(|i| !i.trim().is_empty());
    for source in own.into_iter().chain(target) {
        let file = base.join(source.trim());
        if let Some(icon) = icon_from_file(&file) {
            app.icon = icon;
            break;
        }
    }
    app.sources.push("lnk".into());
}

/// Raccourci internet `.url` : fichier texte de type INI (`URL=`, `IconFile=`).
fn read_url_meta(path: &Path, app: &mut AppEntry) {
    let Ok(bytes) = std::fs::read(path) else { return };
    let text = String::from_utf8_lossy(&bytes);
    let (mut url, mut icon) = (None, None);
    for line in text.trim_start_matches('\u{feff}').lines() {
        let Some((key, value)) = line.split_once('=') else { continue };
        let value = value.trim();
        match key.trim().to_ascii_lowercase().as_str() {
            "url" if !value.is_empty() => url = Some(value.to_string()),
            "iconfile" if !value.is_empty() => icon = Some(expand_env(value)),
            _ => {}
        }
    }
    if let Some(u) = url {
        app.description = u;
        app.sources.push("url".into());
    }
    let base = path.parent().unwrap_or(Path::new("."));
    if let Some(i) = icon.and_then(|i| icon_from_file(&base.join(i))) {
        app.icon = i;
    }
}

// ---------------------------------------------------------------- utilitaires

fn kind_of(path: &Path) -> Option<&'static str> {
    match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
        "html" | "htm" => Some("html"),
        "exe" => Some("exe"),
        "lnk" => Some("lnk"),
        "url" => Some("url"),
        _ => None,
    }
}

fn stem(path: &Path) -> String {
    path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()
}

fn file_name(path: &Path) -> String {
    path.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()
}

/// Lit un .portal.json : les champs connus, plus l'objet brut pour l'éditeur.
fn read_sidecar(path: &Path) -> Option<(Sidecar, serde_json::Value)> {
    let raw = read_json_object(path).ok()??;
    Some((serde_json::from_value(raw.clone()).ok()?, raw))
}

/// `Ok(None)` si le fichier n'existe pas, `Err` s'il existe mais n'est pas un objet JSON valide.
pub fn read_json_object(path: &Path) -> Result<Option<serde_json::Value>, String> {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.to_string()),
    };
    match serde_json::from_str::<serde_json::Value>(text.trim_start_matches('\u{feff}')) {
        Ok(v) if v.is_object() => Ok(Some(v)),
        _ => Err(format!("{} n'est pas un JSON valide", file_name(path))),
    }
}

/// Transforme une valeur d'icône en quelque chose d'affichable :
/// URL `data:` telle quelle, fichier image local → `data:`, sinon emoji / texte court.
pub fn resolve_icon(base: &Path, value: &str) -> Option<String> {
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

    /// .lnk minimal : en-tête, drapeaux « nom » + « Unicode », puis la chaîne de description.
    fn fake_lnk(description: &str) -> Vec<u8> {
        let mut b = vec![0u8; 0x4C];
        b[0] = 0x4C;
        b[0x14..0x18].copy_from_slice(&(0x04u32 | 0x80).to_le_bytes());
        let units: Vec<u16> = description.encode_utf16().collect();
        b.extend_from_slice(&(units.len() as u16).to_le_bytes());
        for u in units {
            b.extend_from_slice(&u.to_le_bytes());
        }
        b
    }

    #[test]
    fn parses_lnk_description_and_survives_garbage() {
        assert_eq!(parse_lnk(&fake_lnk("Outil perso é")).unwrap().description.as_deref(), Some("Outil perso é"));
        assert!(parse_lnk(b"pas un raccourci").is_none());
        let mut cut = fake_lnk("coupé");
        cut.truncate(cut.len() - 3);
        assert!(parse_lnk(&cut).is_none());
    }

    #[test]
    fn shortcuts_become_cards() {
        let root = std::env::temp_dir().join(format!("portabl-lnk-{}", std::process::id()));
        std::fs::create_dir_all(root.join("Dossier")).unwrap();
        std::fs::write(root.join("Intranet.url"), "[InternetShortcut]\r\nURL=http://intranet/outil\r\n").unwrap();
        std::fs::write(root.join("Outil.lnk"), fake_lnk("Mon outil")).unwrap();
        std::fs::write(root.join("Dossier").join("Seul.lnk"), fake_lnk("Dans un dossier")).unwrap();
        let apps = scan(&root);
        std::fs::remove_dir_all(&root).unwrap();
        let by_name = |n: &str| apps.iter().find(|a| a.name == n).unwrap_or_else(|| panic!("{n} absent"));
        assert_eq!(by_name("Intranet").kind, "url");
        assert_eq!(by_name("Intranet").description, "http://intranet/outil");
        assert_eq!(by_name("Outil").kind, "lnk");
        assert_eq!(by_name("Outil").description, "Mon outil");
        assert_eq!(by_name("Seul").description, "Dans un dossier");
    }

    #[test]
    fn sidecar_next_to_file_or_in_folder() {
        let root = Path::new(r"C:\apps");
        assert_eq!(sidecar_path(root, &root.join("Kanban.html")), root.join("Kanban.portal.json"));
        assert_eq!(sidecar_path(root, &root.join(r"Suite\bin\Lanceur.exe")), root.join(r"Suite\portal.json"));
    }

    #[test]
    fn hidden_apps_are_listed_with_detected_values() {
        let root = std::env::temp_dir().join(format!("portabl-test-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("a.html"), "<title>Détectée</title><meta name=\"version\" content=\"2.0\">").unwrap();
        std::fs::write(root.join("a.portal.json"), r#"{"name":"Perso","hidden":true,"extra":1}"#).unwrap();
        let apps = scan(&root);
        std::fs::remove_dir_all(&root).unwrap();
        let app = &apps[0];
        assert!(app.hidden);
        assert_eq!(app.name, "Perso");
        assert_eq!(app.detected.name, "Détectée");
        assert_eq!(app.detected.version, "2.0");
        assert_eq!(app.sidecar["extra"], 1);
    }
}
