---
name: portabl-apps-meta
description: Renseigne les infos d'une appli standalone (.html, .exe ou raccourci .lnk / .url) pour qu'elle s'affiche correctement dans le lanceur Porta[b]l Apps (nom, description, version, auteur, catégorie, tags, couleur, icône, notes). À utiliser quand on crée ou publie une appli destinée à Porta[b]l Apps, quand on change sa version, ou quand on demande de « remplir les infos portal », « préparer l'appli pour le portail » ou de vérifier ce que le lanceur va afficher.
---

# Infos d'appli pour Porta[b]l Apps

Porta[b]l Apps est un lanceur local, hors réseau, qui scanne un dossier `apps\` et affiche une carte par appli. Ce skill sert à ce que chaque appli y apparaisse avec les bonnes infos, **en les mettant au bon endroit** : de préférence dans l'appli elle-même, pour qu'elles voyagent avec elle et restent à jour.

## 1. Identifier le type d'appli

| Livrable | Comment le lanceur la voit |
|---|---|
| Un seul fichier `.html` / `.htm` | Posé à la racine de `apps\` |
| Un `.html` avec des fichiers à côté | Sous-dossier de `apps\` contenant `index.html` |
| Un `.exe` autonome | Posé à la racine de `apps\` |
| Un `.exe` avec DLL, config… | Sous-dossier contenant **un seul** `.exe` |
| Plusieurs `.exe` / point d'entrée non standard | Sous-dossier + `portal.json` avec `"entry"` |
| Un raccourci Windows `.lnk` ou internet `.url` | Posé à la racine de `apps\` (ou seul raccourci d'un sous-dossier sans `.exe`) |

Seul le premier niveau de `apps\` est scanné. Les sous-sous-dossiers ne sont jamais des applis.

## 2. Où mettre chaque info (par ordre de préférence)

Le lanceur lit, du moins prioritaire au plus prioritaire : nom du fichier → infos embarquées dans l'appli → fichier `.portal.json`. Le `.portal.json` écrase tout le reste, champ par champ.

**Règle : tout ce que l'appli peut déclarer elle-même va dans l'appli. Le `.portal.json` ne sert qu'à ce qui ne peut pas y être.** En particulier, ne jamais recopier la version dans le `.portal.json` : elle y serait figée et ne suivrait plus les nouvelles versions.

### Appli HTML : balises dans le `<head>`

```html
<head>
  <meta charset="utf-8">
  <title>Nom affiché</title>
  <meta name="description" content="Ce que fait l'appli, en une phrase.">
  <meta name="author" content="Auteur">
  <meta name="version" content="1.2.0">
  <meta name="portal:category" content="Outils">
  <meta name="portal:tags" content="csv, export">
  <meta name="portal:color" content="#00f0ff">
  <meta name="portal:icon" content="🔧">
  <meta name="portal:notes" content="Raccourcis, astuces, limites connues…">
  <link rel="icon" href="data:image/svg+xml,…">
</head>
```

Points de détection à respecter (le lanceur lit ces balises avec une analyse simple, pas un vrai navigateur) :
- Les balises doivent être **avant `</head>`** (ou dans les 256 premiers Ko s'il n'y a pas de `</head>`).
- Attributs entre guillemets (`"` ou `'`), `name` et `content` sur la même balise. Les entités `&amp; &lt; &gt; &quot; &#39;` sont décodées, pas les autres.
- Équivalents acceptés : `application-name` ou `portal:name` (prioritaires sur `<title>`), `keywords` pour les tags, `theme-color` pour la couleur, `portal:description` / `portal:version` / `portal:author`.
- `portal:tags` : liste séparée par des virgules.
- `portal:color` : une couleur CSS ; préférer `#rrggbb`, que l'éditeur du lanceur sait afficher.
- Icône, au choix :
  - `portal:icon` avec un **emoji ou un sigle de 4 caractères max** ;
  - `portal:icon` ou `<link rel="icon">` avec un fichier image **local** relatif à l'appli (`png`, `svg`, `ico`, `jpg`, `gif`, `webp`) ou une URL `data:`.
  - Jamais d'URL `http(s)` : le lanceur tourne hors réseau et l'ignorera.
- La **version** doit venir de la source de version du projet (package.json, constante, Cargo.toml…). Si le HTML est généré par un build, injecter la version au build plutôt que la taper à la main.

### Appli EXE : ressources de version Windows

Le lanceur lit la première traduction de la ressource `VERSIONINFO` et la première icône de l'exe :

| Carte | Champ VERSIONINFO |
|---|---|
| Nom | `FileDescription`, sinon `ProductName` |
| Description | `Comments` |
| Version | `ProductVersion`, sinon `FileVersion` |
| Auteur | `CompanyName`, sinon `LegalCopyright` |
| Icône | première icône de l'exe (la plus grande résolution) |

Comment les renseigner selon la chaîne de build :
- **.NET** (`.csproj`) : `<AssemblyTitle>` (→ FileDescription), `<Product>`, `<Version>`, `<Company>`, `<Description>` (→ Comments) et `<ApplicationIcon>` pour l'icône. Pour un exe unique, publier avec `PublishSingleFile` et `SelfContained`.
- **Rust** : crate `winresource` (ou `embed-resource` avec un `.rc`) dans `build.rs`, en reprenant `CARGO_PKG_VERSION` ; Tauri le fait seul à partir de `productName`, de la version et des icônes.
- **Python / PyInstaller** : `--version-file` (fichier généré par `pyi-grab_version` ou écrit à la main) et `--icon`.
- **Go** : `goversioninfo` (`versioninfo.json`) ou `rsrc`.
- **C / C++** : bloc `VERSIONINFO` dans le `.rc`, avec `Comments` dans `StringFileInfo`.
- **AutoHotkey, Electron, etc.** : utiliser leur option de métadonnées Windows ; à défaut, `rcedit`.

Un exe ne peut pas porter catégorie, tags, couleur, emoji ni notes : ces champs vont dans le `.portal.json`.

### Raccourcis `.lnk` et `.url`

Le nom de la carte est celui du fichier : nommer le raccourci comme on veut le voir affiché. Le lanceur lit :
- `.lnk` : la description (champ « Commentaire » du raccourci) et l'icône du raccourci, sinon celle de sa cible ; sans description, le chemin de la cible sert de description.
- `.url` : la ligne `URL=` comme description, et `IconFile=` pour l'icône.

Un clic ouvre le raccourci comme dans l'explorateur. Catégorie, tags, couleur, notes et nom différent vont dans le `.portal.json` (`Mon outil.lnk` → `Mon outil.portal.json`).

### Le fichier `.portal.json`

Emplacement :
- appli posée à la racine : même nom que le fichier + `.portal.json` (`Kanban.html` → `Kanban.portal.json`, `Calc.exe` → `Calc.portal.json`) ;
- appli en sous-dossier : `portal.json` à la racine de ce sous-dossier.

```json
{
  "name": "Mon Appli",
  "description": "Ce qu'elle fait en une phrase",
  "author": "Moi",
  "category": "Outils",
  "tags": ["csv", "export"],
  "color": "#00f0ff",
  "icon": "🔧",
  "notes": "Raccourcis, astuces…",
  "entry": "Lanceur.exe"
}
```

- JSON strict : guillemets doubles, pas de virgule finale, UTF-8. Un fichier invalide est ignoré **en entier**, et un mauvais type (`"tags": "a, b"` au lieu d'une liste) aussi.
- `icon` : emoji, ou image relative au fichier lancé (`"logo.png"`).
- `entry` : seulement pour un sous-dossier, chemin relatif au dossier, vers un `.html`, `.htm` ou `.exe` existant.
- `hidden: true` masque l'appli ; à ne pas mettre sans demande explicite.
- N'y mettre que les champs nécessaires ; ne pas dupliquer ce que l'appli déclare déjà (surtout `version`).
- L'utilisateur peut aussi éditer ce fichier depuis le lanceur (bouton ✏️ Modifier, touche E) : préserver les champs existants quand on le modifie.

## 3. Choisir les valeurs

- **Nom** : court (la carte le coupe vers 25 caractères), sans le mot « app ».
- **Description** : une phrase, ce que l'appli fait pour l'utilisateur ; la carte en montre deux lignes.
- **Version** : SemVer `x.y.z`, sans préfixe `v` (le lanceur l'ajoute).
- **Catégorie** : une seule ; chaque catégorie devient un filtre. Réutiliser celles qui existent déjà dans le dossier `apps\` de l'utilisateur s'il est accessible (lire les autres `.portal.json` et balises) ; sinon, demander plutôt qu'inventer une nouvelle catégorie.
- **Tags** : 2 à 5, en minuscules, utiles à la recherche.
- **Couleur** : facultative (sinon une couleur est déduite du nom) ; si elle est mise, reprendre la couleur principale de l'appli.
- **Icône** : un emoji évocateur suffit ; une vraie icône si l'appli en a une.
- Ne jamais inventer un auteur : le demander, ou reprendre celui du projet (package.json, Cargo.toml, git config).

## 4. Vérifier

Lancer le script de contrôle fourni avec ce skill, qui reproduit la détection du lanceur et affiche la carte telle qu'elle apparaîtra :

```bash
node "<dossier de ce skill>/check.mjs" <chemin vers l'appli .html, .exe ou son dossier>
```

Il signale aussi les pièges courants : balises après `</head>`, icône distante, `.portal.json` invalide, version dupliquée, `entry` introuvable. Pour un `.exe`, il ne lit que le `.portal.json` : vérifier les ressources de version dans l'explorateur (clic droit › Propriétés › Détails) ou avec `(Get-Item app.exe).VersionInfo` en PowerShell.

## 5. À rappeler à l'utilisateur

Une appli destinée à Porta[b]l Apps doit fonctionner **hors réseau** : pas de CDN, de polices ou de scripts distants dans un HTML, pas de dépendance à installer pour un exe. Si le projet en a, le signaler (sans le corriger d'office si ce n'est pas demandé).
