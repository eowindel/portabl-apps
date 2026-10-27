#!/usr/bin/env node
// Montre ce que Porta[b]l Apps affichera pour une appli, en reproduisant sa détection
// (src-tauri/src/scan.rs), et signale les pièges courants.
// Usage : node check.mjs <appli.html | appli.exe | dossier-de-l-appli>
import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { basename, dirname, extname, join, resolve, sep } from "node:path";

const arg = process.argv[2];
if (!arg) {
  console.error("Usage : node check.mjs <appli.html | appli.exe | dossier-de-l-appli>");
  process.exit(2);
}

const warnings = [];
const warn = (msg) => warnings.push(msg);
const kindOf = (p) => ({ ".html": "html", ".htm": "html", ".exe": "exe", ".lnk": "lnk", ".url": "url" })[extname(p).toLowerCase()];
const stem = (p) => basename(p, extname(p));

// ------------------------------------------------------------------ cible et .portal.json
let target = resolve(arg);
let metaFile;
let folder = null;
if (!existsSync(target)) fail(`Introuvable : ${target}`);

if (statSync(target).isDirectory()) {
  folder = target;
  metaFile = join(folder, "portal.json");
  const entry = readSidecar(metaFile)?.entry;
  if (entry) {
    if (/^([a-z]:|[\\/])/i.test(entry) || entry.split(/[\\/]/).includes("..")) warn(`"entry" doit être un chemin relatif dans le dossier : ${entry}`);
    target = join(folder, entry);
    if (!existsSync(target)) fail(`"entry" pointe vers un fichier absent : ${entry}`);
  } else if (existsSync(join(folder, "index.html"))) {
    target = join(folder, "index.html");
  } else {
    const files = readdirSync(folder);
    const exes = files.filter((f) => kindOf(f) === "exe");
    const links = files.filter((f) => ["lnk", "url"].includes(kindOf(f)));
    const pick = exes.length ? exes : links;
    if (pick.length !== 1) fail(`Dossier ignoré par le lanceur : ni "entry" dans portal.json, ni index.html, et ${exes.length} .exe / ${links.length} raccourci(s) (il en faut exactement un).`);
    target = join(folder, pick[0]);
  }
} else {
  metaFile = join(dirname(target), `${stem(target)}.portal.json`);
  if (existsSync(join(dirname(target), "portal.json")) || basename(target).toLowerCase() === "index.html") {
    warn("Ce fichier semble faire partie d'une appli en dossier : passez plutôt le dossier au script.");
  }
}
const kind = kindOf(target);
if (!kind) fail(`Extension non reconnue (seuls .html, .htm et .exe sont des applis) : ${basename(target)}`);

// ------------------------------------------------------------------ détection
const app = { name: stem(target), description: "", version: "", author: "", category: "", tags: [], color: "", icon: "", notes: "" };
const sources = [];
const detected = {};

if (kind === "html") readHtml(target);
else if (kind === "url") readUrl(target);
else if (kind === "lnk") warn("Raccourci .lnk : description et icône sont lues par le lanceur (champ « Commentaire » du raccourci, icône ou cible) ; non vérifiées ici.");
else warn("Exe : les ressources de version ne sont pas lues ici. Vérifiez-les avec (Get-Item <exe>).VersionInfo en PowerShell.");
if (folder && (!sources.length || app.name.toLowerCase() === "index")) app.name = basename(folder);
Object.assign(detected, app);

const sidecar = readSidecar(metaFile);
if (sidecar) applySidecar(sidecar);

// ------------------------------------------------------------------ rapport
const show = (v) => (Array.isArray(v) ? v.join(", ") : v) || "—";
console.log(`\nCarte Porta[b]l Apps pour ${target}`);
console.log(`Type : ${kind}${folder ? " (appli en dossier)" : ""}`);
console.log(`Fichier .portal.json : ${metaFile}${sidecar ? "" : " (absent)"}\n`);
for (const k of ["name", "description", "version", "author", "category", "tags", "color", "icon", "notes"]) {
  const v = k === "icon" && app.icon.startsWith("data:") ? `image (${app.icon.slice(5, app.icon.indexOf(";"))})` : app[k];
  const from = sidecar && k in sidecar && String(sidecar[k]).trim() ? " ← .portal.json" : "";
  console.log(`  ${k.padEnd(12)} ${show(v)}${from}`);
}
console.log(`\nInfos lues : ${[...sources, ...(sidecar ? ["sidecar"] : [])].join(", ") || "nom du fichier uniquement"}`);
if (!app.description) warn("Pas de description : la carte sera vide sous le titre.");
if (kind === "html" && !detected.version && !sidecar?.version) warn("Pas de version déclarée.");
if ((kind === "lnk" || kind === "url") && !sidecar?.name) warn(`Le nom de la carte est le nom du fichier (« ${stem(target)} »).`);
if (app.name.length > 25) warn(`Nom long (${app.name.length} caractères) : il sera coupé sur la carte.`);
if (/^v/i.test(app.version)) warn(`Version « ${app.version} » : retirez le « v », le lanceur l'ajoute.`);
if (warnings.length) console.log(`\n⚠ ${warnings.join("\n⚠ ")}`);
else console.log("\n✔ Rien à signaler.");

// ------------------------------------------------------------------ fonctions
function fail(msg) {
  console.error(`✖ ${msg}`);
  process.exit(1);
}

function readSidecar(file) {
  if (!existsSync(file)) return null;
  let raw;
  try {
    raw = JSON.parse(readFileSync(file, "utf8").replace(/^﻿/, ""));
  } catch (e) {
    warn(`${basename(file)} n'est pas un JSON valide, il sera ignoré en entier : ${e.message}`);
    return null;
  }
  if (!raw || typeof raw !== "object" || Array.isArray(raw)) {
    warn(`${basename(file)} doit contenir un objet { … } : il sera ignoré.`);
    return null;
  }
  const types = { name: "string", description: "string", version: "string", author: "string", category: "string", color: "string", icon: "string", notes: "string", entry: "string", hidden: "boolean" };
  for (const [k, t] of Object.entries(types)) {
    if (k in raw && raw[k] !== null && typeof raw[k] !== t) {
      warn(`${basename(file)} : "${k}" doit être de type ${t}. Le fichier entier sera ignoré.`);
      return null;
    }
  }
  if ("tags" in raw && raw.tags !== null && !(Array.isArray(raw.tags) && raw.tags.every((t) => typeof t === "string"))) {
    warn(`${basename(file)} : "tags" doit être une liste de textes (["a", "b"]). Le fichier entier sera ignoré.`);
    return null;
  }
  const known = new Set([...Object.keys(types), "tags"]);
  const unknown = Object.keys(raw).filter((k) => !known.has(k));
  if (unknown.length) warn(`${basename(file)} : champs inconnus, sans effet : ${unknown.join(", ")}`);
  if (raw.entry && basename(file) !== "portal.json") warn(`"entry" n'a d'effet que dans le portal.json d'une appli en dossier.`);
  if (raw.hidden === true) warn("L'appli est masquée (\"hidden\": true).");
  return raw;
}

function applySidecar(s) {
  for (const k of ["name", "description", "version", "author", "category", "color", "notes"]) {
    if (typeof s[k] === "string" && s[k].trim()) app[k] = s[k];
  }
  if (Array.isArray(s.tags)) app.tags = s.tags;
  if (typeof s.icon === "string") {
    const icon = resolveIcon(dirname(target), s.icon, ".portal.json");
    if (icon) app.icon = icon;
  }
  if (s.version && detected.version) {
    warn(`Version écrite dans le .portal.json (${s.version}) alors que l'appli déclare déjà ${detected.version} : elle restera figée. Retirez-la du .portal.json.`);
  }
}

function decode(s) {
  return s.replace(/&lt;/g, "<").replace(/&gt;/g, ">").replace(/&quot;/g, '"').replace(/&#39;|&apos;/g, "'").replace(/&nbsp;/g, " ").replace(/&amp;/g, "&");
}

function readHtml(file) {
  const text = readFileSync(file, "utf8");
  const end = text.toLowerCase().indexOf("</head>");
  const head = end >= 0 ? text.slice(0, end) : text.slice(0, 256 * 1024);
  if (end < 0) warn("Pas de </head> : seuls les 256 premiers Ko sont lus.");
  const tail = end >= 0 ? text.slice(end) : "";
  if (/<meta\b[^>]*\bname\s*=\s*["'](portal:|description|version|author)/i.test(tail)) {
    warn("Des balises <meta> d'infos sont après </head> : le lanceur ne les voit pas.");
  }

  const attrs = (tag) => Object.fromEntries([...tag.matchAll(/([a-zA-Z_:-]+)\s*=\s*(?:"([^"]*)"|'([^']*)')/gs)]
    .map((m) => [m[1].toLowerCase(), decode(m[2] ?? m[3] ?? "")]));
  let found = false;
  const title = head.match(/<title[^>]*>([\s\S]*?)<\/title>/i);
  if (title && decode(title[1].trim())) { app.name = decode(title[1].trim()); found = true; }
  for (const [tag] of head.matchAll(/<meta\b[^>]*>/gis)) {
    const a = attrs(tag);
    const key = (a.name ?? a.property ?? "").toLowerCase();
    const content = (a.content ?? "").trim();
    if (!key || !content) continue;
    if (key.startsWith("portal:") || ["description", "author", "version"].includes(key)) found = true;
    switch (key) {
      case "portal:name": case "application-name": app.name = content; break;
      case "description": case "portal:description": app.description = content; break;
      case "version": case "portal:version": app.version = content; break;
      case "author": case "portal:author": app.author = content; break;
      case "portal:category": app.category = content; break;
      case "keywords": case "portal:tags": app.tags = content.split(",").map((t) => t.trim()).filter(Boolean); break;
      case "portal:color": case "theme-color": app.color = content; break;
      case "portal:notes": app.notes = content; break;
      case "portal:icon": { const i = resolveIcon(dirname(file), content, "portal:icon"); if (i) app.icon = i; break; }
    }
  }
  if (!app.icon) {
    for (const [tag] of head.matchAll(/<link\b[^>]*>/gis)) {
      const a = attrs(tag);
      if ((a.rel ?? "").toLowerCase().includes("icon") && a.href) {
        const i = resolveIcon(dirname(file), a.href, "favicon");
        if (i) { app.icon = i; break; }
      }
    }
  }
  if (found) sources.push("html");
}

function readUrl(file) {
  const text = readFileSync(file, "utf8").replace(/^﻿/, "");
  const url = text.match(/^\s*url\s*=\s*(.+?)\s*$/im)?.[1];
  if (url) { app.description = url; sources.push("url"); }
  else warn("Pas de ligne URL= : le raccourci internet sera vide.");
}

function resolveIcon(base, value, origin) {
  value = value.trim();
  if (!value) return null;
  if (value.startsWith("data:")) return value;
  if (/^https?:/i.test(value)) {
    warn(`Icône distante ignorée (${origin}) : le lanceur fonctionne hors réseau. Utilisez un fichier local ou une URL data:.`);
    return null;
  }
  const ext = extname(value).slice(1).toLowerCase();
  const mime = { png: "image/png", jpg: "image/jpeg", jpeg: "image/jpeg", gif: "image/gif", webp: "image/webp", svg: "image/svg+xml", ico: "image/x-icon" }[ext];
  if (!ext) {
    if ([...value].length <= 4) return value;
    warn(`Icône « ${value} » ignorée (${origin}) : un emoji ou sigle fait 4 caractères au plus.`);
    return null;
  }
  if (!mime) {
    warn(`Icône « ${value} » ignorée (${origin}) : format non géré.`);
    return null;
  }
  const file = join(base, value.replace(/^\.\//, "").split("/").join(sep));
  if (!existsSync(file)) {
    warn(`Icône introuvable (${origin}) : ${file}`);
    return null;
  }
  return `data:${mime};base64,…`;
}
