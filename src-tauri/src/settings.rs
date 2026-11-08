//! Réglages persistés dans `portabl-apps.json`, à côté de l'exécutable (mode portable).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Stat {
    pub count: u64,
    /// Horodatage du dernier lancement (ms depuis l'epoch Unix).
    pub last: u64,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// Dossier scanné. Vide = `<dossier de l'exe>/apps`.
    pub apps_dir: String,
    pub theme: String,
    /// "app" (fenêtre Edge dédiée), "default" (navigateur par défaut) ou "custom".
    pub browser_mode: String,
    pub browser_path: String,
    /// Arguments du navigateur perso ; `{url}` est remplacé par l'URL du fichier.
    pub browser_args: String,
    pub favorites: Vec<String>,
    pub stats: HashMap<String, Stat>,
    /// Autorisation de contacter GitHub pour chercher les mises à jour (désactivée par défaut).
    pub check_updates: bool,
    /// Dernière release connue de chaque dépôt (voir `update.rs`).
    pub update_cache: HashMap<String, crate::update::Release>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            apps_dir: String::new(),
            theme: "arcade".into(),
            browser_mode: "app".into(),
            browser_path: String::new(),
            browser_args: "{url}".into(),
            favorites: Vec::new(),
            stats: HashMap::new(),
            check_updates: false,
            update_cache: HashMap::new(),
        }
    }
}

pub fn exe_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn settings_path() -> PathBuf {
    exe_dir().join("portabl-apps.json")
}

impl Settings {
    pub fn load() -> Settings {
        // `pitlane.json` : nom de l'appli avant son renommage, repris s'il existe encore.
        std::fs::read_to_string(settings_path())
            .or_else(|_| std::fs::read_to_string(exe_dir().join("pitlane.json")))
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(settings_path(), json).map_err(|e| format!("Impossible d'écrire les réglages : {e}"))
    }

    pub fn apps_dir(&self) -> PathBuf {
        if self.apps_dir.trim().is_empty() {
            exe_dir().join("apps")
        } else {
            let p = PathBuf::from(self.apps_dir.trim());
            if p.is_absolute() { p } else { exe_dir().join(p) }
        }
    }
}
