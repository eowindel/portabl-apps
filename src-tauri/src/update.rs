//! Vérification des nouvelles versions sur GitHub (launcher + applis qui déclarent un `repo`).
//!
//! Rien ne part sur le réseau tant que l'utilisateur n'a pas activé l'option. Les requêtes ne
//! visent que `api.github.com` ; on ne télécharge ni n'installe rien : on signale seulement.

use crate::settings::Settings;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};
use std::time::Duration;

/// Dépôt du launcher lui-même.
pub const LAUNCHER_REPO: &str = "eowindel/portabl-apps";
/// Intervalle minimal entre deux interrogations d'un même dépôt (l'API anonyme est limitée à 60/h).
const TTL_MS: u64 = 24 * 3600 * 1000;

/// Dernière release connue d'un dépôt, conservée dans les réglages.
#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Release {
    /// Tag de la dernière release (vide : aucune release publiée).
    pub tag: String,
    pub url: String,
    pub etag: String,
    /// Dernière interrogation réussie (ms depuis l'epoch Unix).
    pub checked: u64,
}

/// Accepte `owner/nom` ou une URL `https://github.com/owner/nom[...]` ; renvoie `owner/nom`.
pub fn normalize_repo(input: &str) -> Option<String> {
    let t = input.trim();
    let t = t
        .strip_prefix("https://github.com/")
        .or_else(|| t.strip_prefix("http://github.com/"))
        .or_else(|| t.strip_prefix("github.com/"))
        .unwrap_or(t);
    let mut parts = t.split('/');
    let (owner, name) = (parts.next()?, parts.next()?.trim_end_matches(".git"));
    let ok = |s: &str| !s.is_empty() && s != "." && s != ".." && s.chars().all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c));
    (ok(owner) && ok(name)).then(|| format!("{owner}/{name}"))
}

/// Seules les pages GitHub s'ouvrent depuis l'interface.
pub fn is_github_url(url: &str) -> bool {
    url.starts_with("https://github.com/") && !url.contains(char::is_whitespace)
}

#[derive(Deserialize)]
struct ApiRelease {
    tag_name: String,
    html_url: String,
}

enum Fetched {
    Fresh(Release),
    NotModified,
    NoRelease,
}

fn fetch(repo: &str, etag: &str) -> Result<Fetched, String> {
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(10))
        .user_agent(concat!("portabl-apps/", env!("CARGO_PKG_VERSION")))
        .build();
    let mut req = agent
        .get(&format!("https://api.github.com/repos/{repo}/releases/latest"))
        .set("Accept", "application/vnd.github+json");
    if !etag.is_empty() {
        req = req.set("If-None-Match", etag);
    }
    match req.call() {
        // ureq ne range pas le 304 parmi les erreurs : réponse vide, rien à lire.
        Ok(resp) if resp.status() == 304 => Ok(Fetched::NotModified),
        Ok(resp) => {
            let etag = resp.header("ETag").unwrap_or_default().to_string();
            let r: ApiRelease = resp.into_json().map_err(|e| e.to_string())?;
            let url = if is_github_url(&r.html_url) { r.html_url } else { format!("https://github.com/{repo}/releases") };
            Ok(Fetched::Fresh(Release { tag: r.tag_name, url, etag, checked: 0 }))
        }
        Err(ureq::Error::Status(304, _)) => Ok(Fetched::NotModified),
        Err(ureq::Error::Status(404, _)) => Ok(Fetched::NoRelease),
        Err(ureq::Error::Status(code, _)) => Err(format!("GitHub a répondu {code} pour {repo}")),
        Err(e) => Err(format!("Réseau indisponible ({e})")),
    }
}

/// Interroge les dépôts périmés (ou tous si `force`) et renvoie la table complète des dernières releases.
/// Les erreurs réseau n'effacent rien : on garde la dernière réponse connue.
pub fn check(settings: &Settings, repos: BTreeSet<String>, force: bool, now: u64) -> (HashMap<String, Release>, Option<String>) {
    let mut cache = settings.update_cache.clone();
    let mut error = None;
    for repo in repos {
        let known = cache.get(&repo).cloned().unwrap_or_default();
        if !force && known.checked > 0 && now.saturating_sub(known.checked) < TTL_MS {
            continue;
        }
        match fetch(&repo, &known.etag) {
            Ok(Fetched::Fresh(mut r)) => {
                r.checked = now;
                cache.insert(repo, r);
            }
            Ok(Fetched::NotModified) => {
                cache.insert(repo, Release { checked: now, ..known });
            }
            Ok(Fetched::NoRelease) => {
                cache.insert(repo, Release { checked: now, ..Default::default() });
            }
            Err(e) => {
                error = Some(e);
                // Un réseau coupé ne sert à rien de le réessayer pour chaque dépôt.
                if e_is_network(error.as_deref()) {
                    break;
                }
            }
        }
    }
    (cache, error)
}

fn e_is_network(e: Option<&str>) -> bool {
    e.is_some_and(|e| e.starts_with("Réseau"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_repos() {
        assert_eq!(normalize_repo("eowindel/portabl-apps").as_deref(), Some("eowindel/portabl-apps"));
        assert_eq!(normalize_repo(" https://github.com/a/b.git ").as_deref(), Some("a/b"));
        assert_eq!(normalize_repo("github.com/a/b/releases").as_deref(), Some("a/b"));
        assert_eq!(normalize_repo("a"), None);
        assert_eq!(normalize_repo("../x"), None);
        assert_eq!(normalize_repo("a/b c"), None);
        assert_eq!(normalize_repo(""), None);
    }

    #[test]
    fn only_github_urls_open() {
        assert!(is_github_url("https://github.com/a/b/releases/tag/v1"));
        assert!(!is_github_url("https://evil.example/https://github.com/"));
        assert!(!is_github_url("file:///etc/passwd"));
    }

    /// Test réseau manuel : `cargo test -- --ignored`.
    #[test]
    #[ignore]
    fn fetches_launcher_release() {
        let mut s = Settings::default();
        s.check_updates = true;
        let repos = BTreeSet::from([LAUNCHER_REPO.to_string(), "eowindel/depot-inexistant-xyz".to_string()]);
        let (cache, err) = check(&s, repos, true, 1);
        assert!(err.is_none(), "{err:?}");
        let r = &cache[LAUNCHER_REPO];
        println!("{} {}", r.tag, r.url);
        assert!(r.tag.starts_with('v') && is_github_url(&r.url));
        assert!(cache["eowindel/depot-inexistant-xyz"].tag.is_empty());
        // Seconde passe avec l'ETag : 304, la release est conservée.
        s.update_cache = cache.clone();
        let (again, err) = check(&s, BTreeSet::from([LAUNCHER_REPO.to_string()]), true, 2);
        assert!(err.is_none(), "{err:?}");
        assert_eq!(again[LAUNCHER_REPO].tag, r.tag);
        assert_eq!(again[LAUNCHER_REPO].checked, 2);
    }
}
