# 🏁 Porta[b]l Apps — garage d'applis

Lanceur local, hors réseau, pour applis standalone `.html` et `.exe`.
Un seul exe de ~4 Mo, sans installation (Tauri 2 + WebView2, déjà présent sur Windows 10/11).

## Utilisation

```
PortablApps\
├── portabl-apps.exe
├── portabl-apps.json ← réglages, favoris, compteurs (créé automatiquement)
└── apps\             ← déposez vos applis ici
    ├── MonOutil.html
    ├── MonOutil.portal.json   (optionnel)
    ├── Convertisseur.exe
    └── GrosseAppli\           (sous-dossier : index.html, un seul .exe, ou portal.json avec "entry")
```

Le dossier est rescanné à chaque retour sur la fenêtre (ou F5). Le dossier scanné se change dans les réglages ⚙.

**Raccourcis** : taper = rechercher · `Ctrl K` · flèches pour naviguer · `Entrée` lancer · `F` favori · `I` / clic droit détails · `E` modifier la fiche · `Échap`.

**Applis HTML** : par défaut, chacune s'ouvre dans sa propre fenêtre Edge en mode appli (sans onglets), pour en lancer plusieurs tout en gardant le lanceur à portée. Alternatives dans les réglages : navigateur par défaut, ou navigateur perso avec arguments (`{url}`).

## Décrire ses applis

Par ordre de priorité :

1. **`MonAppli.portal.json`** à côté du fichier (ou `portal.json` dans le dossier de l'appli), à écrire à la main ou avec l'éditeur ✏️ de l'appli (touche `E`) :
   ```json
   {
     "name": "Mon Appli", "description": "…", "version": "1.2.0", "author": "Moi",
     "category": "Outils", "tags": ["csv"], "color": "#00f0ff",
     "icon": "🏍️", "notes": "…", "hidden": false, "entry": "app.html"
   }
   ```
   `icon` : emoji ou fichier image (`png`, `svg`, `ico`…). `entry` : seulement pour un dossier.
2. **Balises du HTML** : `<title>`, favicon, `<meta name="description|author|version">`
   et `<meta name="portal:category|portal:tags|portal:color|portal:icon|portal:notes">`.
3. **Infos de version de l'exe** : description, version, société, commentaires + icône.

## Développement

Prérequis : Node.js, Rust (`rustup`), MSVC Build Tools.

```bash
npm install
npm run dev      # appli en mode dev
npm run build    # → src-tauri/target/release/portabl-apps.exe
```

- `src/` : interface (HTML/CSS/JS sans framework ni bundler). Thèmes dans `src/themes.css` + liste `THEMES` dans `src/app.js`.
- `src-tauri/src/scan.rs` : découverte et métadonnées · `lib.rs` : lancement et commandes · `settings.rs` : réglages.
- `node scripts/preview-server.mjs` : aperçu de l'interface dans un navigateur, avec un backend simulé.
- `demo-apps/` : applis de démo (les `.exe` de test ne sont pas versionnés).
- `npm run fonts` : recopie les polices et leurs licences OFL depuis `node_modules` vers `src/fonts`.

## Publier une version

1. Changer `version` dans `src-tauri/Cargo.toml` : c'est la seule source, affichée dans le badge de l'appli (garder `package.json` aligné).
2. Ajouter une section en haut de `CHANGELOG.md` (`## [x.y.z] - AAAA-MM-JJ`, puis `### Ajouté` / `### Modifié` / `### Corrigé`). Le fichier est embarqué dans l'exe et affiché en cliquant sur le badge de version.
3. `npm run build`, puis copier `src-tauri/target/release/portabl-apps.exe` dans le dossier à distribuer.
4. Commit + tag : `git tag vx.y.z`.

## Sur un PC d'entreprise

- WebView2 est requis : il est présent d'office sur Windows 11 et sur les Windows 10 à jour.
- L'exe n'est pas signé : SmartScreen ou AppLocker peuvent le bloquer selon la politique de l'entreprise.
- Si le dossier de l'exe n'est pas accessible en écriture, les réglages ne sont pas sauvegardés : placez l'appli dans un dossier utilisateur.
