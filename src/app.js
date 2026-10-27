"use strict";

const THEMES = [
  { id: "arcade", name: "Néon Arcade", swatch: "linear-gradient(180deg,#1d0a3a 0 50%,#ff2bd6 50% 60%,#00f0ff 60% 70%,#0b0614 70%)" },
  { id: "superbike", name: "Superbike", swatch: "repeating-linear-gradient(90deg,#ff2a2a 0 18px,#f4f4f4 18px 36px) bottom/100% 10px no-repeat,#141417" },
  { id: "enduro", name: "Enduro", swatch: "linear-gradient(135deg,#2a2216 0 55%,#ff7a00 55% 75%,#c6ff00 75%)" },
  { id: "pixel", name: "8-bit", swatch: "repeating-linear-gradient(0deg,#040904 0 2px,#0b1f0b 2px 4px)" },
  {
    id: "lol", name: "Hextech", swatch: "linear-gradient(135deg,#091428 0 55%,#c8aa6e 55% 68%,#0ac8b9 68%)",
    flavor: {
      tagline: "faille de l'invocateur",
      search: "Chercher un champion…",
      gauges: ["champions", "kills aujourd'hui", "CS total"],
      fav: "★ Mains",
      unit: ["CS", "CS"],
      never: "jamais pick",
      emptyTitle: "Bienvenue dans la Faille",
      welcome: "Bienvenue dans la Faille de l'invocateur !",
      go: lolAnnouncer,
    },
  },
  {
    id: "sommet", name: "Sommet", swatch: "linear-gradient(160deg,#1b4f80 0 45%,#e3edf5 45% 52%,#263d55 52% 75%,#ffd23f 75%)",
    flavor: {
      tagline: "refuge d'applis",
      search: "Chercher un itinéraire…",
      gauges: ["itinéraires", "sorties aujourd'hui", "sommets gravis"],
      fav: "⛰ Incontournables",
      unit: ["sortie", "sorties"],
      never: "jamais parcourue",
      emptyTitle: "Refuge vide",
      welcome: "Salut l'ami ! Bonne rando 🥾",
      number: (n) => `${(1000 + n * 263).toLocaleString("fr-FR")} m`,
      go: summitGuide,
    },
  },
];

// Textes propres à chaque thème ; un thème ne redéfinit que ce qu'il veut.
const DEFAULT_FLAVOR = {
  tagline: "garage d'applis",
  search: "Rechercher une machine…",
  gauges: ["machines", "tours aujourd'hui", "tours au total"],
  fav: "★ Favoris",
  unit: ["tour", "tours"],
  never: "jamais lancée",
  emptyTitle: "Garage vide",
  welcome: "",
  number: (n) => String(n).padStart(2, "0"),
  go: (app) => ({ text: `GO ! ${app.name}` }),
};
const flavor = () => ({ ...DEFAULT_FLAVOR, ...THEMES.find((t) => t.id === document.documentElement.dataset.theme)?.flavor });

// Annonceur de la Faille : lancer plusieurs applis à moins de 10 s d'intervalle enchaîne les multikills.
const killStreak = { count: 0, last: 0, firstBlood: false };
function lolAnnouncer(app) {
  const now = Date.now();
  killStreak.count = now - killStreak.last < 10_000 ? killStreak.count + 1 : 1;
  killStreak.last = now;
  if (!killStreak.firstBlood) {
    killStreak.firstBlood = true;
    return { text: "First Blood !", kill: true };
  }
  const multi = [null, null, "Double Kill !", "Triple Kill !", "Quadra Kill !"][killStreak.count];
  if (killStreak.count >= 5) return { text: "PENTAKILL !!", kill: true };
  if (multi) return { text: multi, kill: true };
  return { text: `${app.name} — ennemi abattu` };
}

// Guide de haute montagne : chaque lancement ajoute du dénivelé, avec quelques paliers mythiques.
const ascent = { dPlus: 0 };
const SUMMITS = [
  [1000, "Premier refuge atteint ⛺ Une myrtille ?"],
  [2000, "Lac d'altitude en vue 💧 Trop froid pour s'y baigner (ou pas)"],
  [3835, "Refuge du Goûter (3 835 m) — tu avais bien réservé ?"],
  [4805, "MONT BLANC ! 4 805 m… ou 4 808, selon la dernière mesure"],
  [8849, "EVEREST ! 8 849 m, sans oxygène, respect 🫡"],
];
function summitGuide(app) {
  const before = ascent.dPlus;
  ascent.dPlus += 150 + Math.round(Math.random() * 45) * 10;
  const passed = SUMMITS.filter(([alt]) => before < alt && ascent.dPlus >= alt).pop();
  const hour = new Date().getHours();
  const frontale = hour >= 21 || hour < 5 ? " — n'oublie pas la frontale 🔦" : "";
  if (passed) return { text: passed[1], kill: true };
  return { text: `${app.name} · +${ascent.dPlus - before} m D+${frontale}` };
}

// Moment de la journée, pour les thèmes dont le ciel suit l'heure.
function setDaypart() {
  const h = new Date().getHours();
  const part = h < 5 ? "night" : h < 9 ? "dawn" : h < 18 ? "day" : h < 21 ? "dusk" : "night";
  document.documentElement.dataset.daypart = part;
}
setDaypart();
setInterval(setDaypart, 10 * 60_000);

// ------------------------------------------------------------------ accès au backend
// Hors de Tauri (aperçu dans un navigateur), on simule le backend pour pouvoir travailler le design.
const api = window.__TAURI__ ? {
  load: () => window.__TAURI__.core.invoke("load"),
  updatePrefs: (prefs) => window.__TAURI__.core.invoke("update_prefs", { prefs }),
  toggleFavorite: (id) => window.__TAURI__.core.invoke("toggle_favorite", { id }),
  launch: (app) => window.__TAURI__.core.invoke("launch", { id: app.id, path: app.path }),
  reveal: (app) => window.__TAURI__.core.invoke("reveal", { path: app.path }),
  openAppsDir: () => window.__TAURI__.core.invoke("open_apps_dir"),
  pickFolder: () => window.__TAURI__.core.invoke("pick_folder"),
  about: () => window.__TAURI__.core.invoke("about"),
  saveMeta: (app, fields) => window.__TAURI__.core.invoke("save_meta", { path: app.path, fields }),
  pickIcon: (app) => window.__TAURI__.core.invoke("pick_icon", { path: app.path }),
} : mockApi();

// ------------------------------------------------------------------ état
const state = {
  apps: [],
  settings: { favorites: [], stats: {} },
  appsDir: "",
  filter: "all",
  query: "",
  sort: store("sort") || "recent",
  signature: "",
  detail: null,
};

const $ = (sel, root = document) => root.querySelector(sel);
const grid = $("#grid");

function store(key, value) {
  try {
    if (value === undefined) return localStorage.getItem("portabl." + key);
    localStorage.setItem("portabl." + key, value);
  } catch { /* stockage indisponible : sans importance */ }
}

function esc(s) {
  return String(s ?? "").replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]));
}

const norm = (s) => String(s ?? "").normalize("NFD").replace(/[\u0300-\u036f]/g, "").toLowerCase();

function hue(str) {
  let h = 0;
  for (const ch of str) h = (h * 31 + ch.codePointAt(0)) >>> 0;
  return h % 360;
}
const colorOf = (app) => app.color || `hsl(${hue(app.name)} 90% 62%)`;
const isFav = (app) => state.settings.favorites.includes(app.id);
const statOf = (app) => state.settings.stats[app.id] || { count: 0, last: 0 };

const rtf = new Intl.RelativeTimeFormat("fr", { numeric: "auto" });
function ago(ms) {
  if (!ms) return "jamais lancée";
  const s = (ms - Date.now()) / 1000;
  const steps = [[60, "second"], [60, "minute"], [24, "hour"], [7, "day"], [4.35, "week"], [12, "month"], [Infinity, "year"]];
  let v = s;
  for (const [n, unit] of steps) {
    if (Math.abs(v) < n) return rtf.format(Math.round(v), unit);
    v /= n;
  }
}
const fmtSize = (b) => b > 1 << 20 ? (b / (1 << 20)).toFixed(1) + " Mo" : Math.max(1, Math.round(b / 1024)) + " Ko";
const fmtDate = (ms) => ms ? new Date(ms).toLocaleString("fr-FR", { dateStyle: "medium", timeStyle: "short" }) : "—";

function iconHtml(app) {
  const icon = app.icon || "";
  if (icon.startsWith("data:")) return `<div class="app-icon"><img src="${esc(icon)}" alt=""></div>`;
  if (icon) return `<div class="app-icon emoji">${esc(icon)}</div>`;
  const initials = app.name.split(/[\s_\-.]+/).filter(Boolean).slice(0, 2).map((w) => w[0]).join("").toUpperCase();
  return `<div class="app-icon">${esc(initials || "?")}</div>`;
}

// ------------------------------------------------------------------ chargement
async function load({ quiet = false } = {}) {
  try {
    const snap = await api.load();
    state.settings = snap.settings;
    state.appsDir = snap.appsDir;
    applyTheme(snap.settings.theme);
    const signature = JSON.stringify(snap.apps.map((a) => [a.id, a.modified, a.name]));
    if (quiet && signature === state.signature) return;
    state.signature = signature;
    state.apps = snap.apps;
    render();
  } catch (e) {
    toast(String(e), true);
  }
}

function applyTheme(id) {
  const next = THEMES.some((t) => t.id === id) ? id : "arcade";
  const changed = document.documentElement.dataset.theme !== next || !state.themeApplied;
  if (!changed) return;
  state.themeApplied = true;
  document.documentElement.dataset.theme = next;
  const f = flavor();
  $(".tagline").textContent = f.tagline;
  search.placeholder = f.search;
  document.querySelectorAll(".gauge-lbl").forEach((el, i) => { el.textContent = f.gauges[i]; });
  if (state.apps.length) render();
  if (f.welcome) toast(f.welcome);
}

// ------------------------------------------------------------------ rendu
function visibleApps() {
  const q = norm(state.query).trim();
  const words = q.split(/\s+/).filter(Boolean);
  let list = state.apps.filter((a) => {
    if (a.hidden !== (state.filter === "hidden")) return false;
    if (state.filter === "fav" && !isFav(a)) return false;
    if (state.filter === "html" && a.kind !== "html") return false;
    if (state.filter === "exe" && a.kind !== "exe") return false;
    if (state.filter === "link" && a.kind !== "lnk" && a.kind !== "url") return false;
    if (state.filter.startsWith("cat:") && a.category !== state.filter.slice(4)) return false;
    if (!words.length) return true;
    const hay = norm([a.name, a.description, a.category, a.author, a.tags.join(" "), a.id].join(" "));
    return words.every((w) => hay.includes(w));
  });
  const by = {
    recent: (a, b) => statOf(b).last - statOf(a).last,
    laps: (a, b) => statOf(b).count - statOf(a).count,
    modified: (a, b) => b.modified - a.modified,
    name: () => 0,
  }[state.sort];
  list = list.slice().sort((a, b) =>
    (isFav(b) - isFav(a)) || by(a, b) || a.name.localeCompare(b.name, "fr", { sensitivity: "base" }));
  return list;
}

function render() {
  renderGauges();
  renderFilters();
  const list = visibleApps();
  grid.innerHTML = list.map((app, i) => cardHtml(app, i)).join("");
  const empty = $("#empty");
  empty.hidden = list.length > 0;
  if (!list.length) {
    // Textes et pictogrammes neutres : seul le « dossier vide » suit l'ambiance du thème.
    const none = shownApps().length === 0 && state.filter !== "hidden";
    const searching = state.query.trim() !== "";
    const [icon, title, text] = none
      ? ["📂", flavor().emptyTitle, `Déposez vos applis (<code>.html</code>, <code>.exe</code>, raccourcis <code>.lnk</code> / <code>.url</code>) dans<br><code>${esc(state.appsDir)}</code>`]
      : searching
        ? ["🔎", "Aucun résultat", "Essayez une autre recherche ou un autre filtre."]
        : state.filter === "fav"
          ? ["☆", "Aucun favori", "Cliquez sur ☆ au bas d'une carte, ou touche <kbd>F</kbd>, pour l'épingler ici."]
          : state.filter === "hidden"
            ? ["👁", "Aucune appli masquée", "Une appli masquée apparaît ici, grisée."]
            : ["🔎", "Aucun résultat", "Aucune appli ne correspond à ce filtre."];
    $("#empty-icon").textContent = icon;
    $("#empty-title").textContent = title;
    $("#empty-text").innerHTML = text;
    $("#empty-open").hidden = !none;
  }
  if (state.detail) {
    const fresh = state.apps.find((a) => a.id === state.detail);
    fresh ? renderDrawer(fresh) : closeDrawer();
  }
}

const lapsHtml = (app) => {
  const { count } = statOf(app);
  return `🚀 <b>${count}</b> ${esc(flavor().unit[count > 1 ? 1 : 0])}`;
};
const lastText = (app) => statOf(app).last ? ago(statOf(app).last) : flavor().never;

function cardHtml(app, i) {
  const stat = statOf(app);
  const fav = isFav(app);
  const f = flavor();
  const num = f.number(state.apps.indexOf(app) + 1);
  return `
  <article class="card${app.hidden ? " is-hidden" : ""}" tabindex="0" data-id="${esc(app.id)}" style="--c:${esc(colorOf(app))};animation-delay:${Math.min(i, 20) * 35}ms">
    <div class="card-head">
      ${iconHtml(app)}
      <div class="card-title">
        <h3 title="${esc(app.name)}">${esc(app.name).replace(/([_\-.\/])/g, "$1<wbr>")}</h3>
        <div class="meta-line">
          <span class="badge ${app.kind}">${app.kind}</span>
          ${app.version ? `<span>v${esc(app.version.replace(/^v/i, ""))}</span>` : ""}
          ${app.category ? `<span>· ${esc(app.category)}</span>` : ""}
        </div>
      </div>
    </div>
    ${app.description ? `<p class="desc">${esc(app.description)}</p>` : ""}
    ${app.tags.length ? `<div class="tags">${app.tags.slice(0, 5).map((t) => `<span class="tag">#${esc(t)}</span>`).join("")}</div>` : ""}
    <div class="card-foot">
      <span class="laps">${lapsHtml(app)}</span>
      <span class="last">${esc(lastText(app))}</span>
      <div class="card-btns">
        <button class="mini ${fav ? "on" : ""}" data-act="fav" title="Favori (F)">${fav ? "★" : "☆"}</button>
        <button class="mini" data-act="info" title="Détails (I)">ⓘ</button>
      </div>
    </div>
    <span class="race-number">${num}</span>
  </article>`;
}

function renderGauges() {
  const stats = Object.values(state.settings.stats);
  const dayStart = new Date().setHours(0, 0, 0, 0);
  // Approximation volontaire : on ne garde que la date du dernier tour, donc « aujourd'hui » = applis lancées aujourd'hui.
  animateNumber($("#g-apps"), shownApps().length);
  animateNumber($("#g-today"), stats.filter((s) => s.last >= dayStart).length);
  animateNumber($("#g-total"), stats.reduce((n, s) => n + s.count, 0));
}

function animateNumber(el, to) {
  const from = Number(el.textContent) || 0;
  if (from === to) return;
  const t0 = performance.now();
  const step = (t) => {
    const k = Math.min(1, Math.max(0, (t - t0) / 500));
    el.textContent = Math.round(from + (to - from) * (1 - (1 - k) ** 3));
    if (k < 1) requestAnimationFrame(step);
  };
  requestAnimationFrame(step);
}

const shownApps = () => state.apps.filter((a) => !a.hidden);

function renderFilters() {
  const apps = shownApps();
  const count = (f) => apps.filter(f).length;
  const cats = [...new Set(apps.map((a) => a.category).filter(Boolean))].sort((a, b) => a.localeCompare(b, "fr"));
  const chips = [
    ["all", "Tout", apps.length],
    ["fav", flavor().fav, count(isFav)],
    ["html", "HTML", count((a) => a.kind === "html")],
    ["exe", "EXE", count((a) => a.kind === "exe")],
    ["link", "Raccourcis", count((a) => a.kind === "lnk" || a.kind === "url")],
    ...cats.map((c) => ["cat:" + c, c, count((a) => a.category === c)]),
    ["hidden", "👁 Masquées", state.apps.length - apps.length],
  ];
  if (!chips.some(([id]) => id === state.filter)) state.filter = "all";
  $("#filters").innerHTML = chips
    .filter(([id, , n]) => n > 0 || id === "all" || id === "fav")
    .map(([id, label, n]) => `<button class="chip ${state.filter === id ? "active" : ""}" data-filter="${esc(id)}">${esc(label)}<span class="n">${n}</span></button>`)
    .join("");
}

// ------------------------------------------------------------------ actions
const appById = (id) => state.apps.find((a) => a.id === id);

async function launch(app, card) {
  card ??= grid.querySelector(`[data-id="${CSS.escape(app.id)}"]`);
  try {
    state.settings = await api.launch(app);
  } catch (e) {
    toast(String(e), true);
    return;
  }
  const f = flavor();
  document.body.classList.remove("revving");
  void document.body.offsetWidth;
  document.body.classList.add("revving");
  setTimeout(() => document.body.classList.remove("revving"), 800);
  if (card) {
    card.classList.remove("launch");
    void card.offsetWidth;
    card.classList.add("launch");
    card.addEventListener("animationend", () => card.classList.remove("launch"), { once: true });
    updateFoot(app, card);
  }
  renderGauges();
  const msg = f.go(app);
  toast(msg.text, false, msg.kill ? "kill" : "");
}

// Compteur et date de dernier tour, mis à jour sur place : la carte garde son animation de lancement.
function updateFoot(app, card) {
  card.querySelector(".laps").innerHTML = lapsHtml(app);
  card.querySelector(".last").textContent = lastText(app);
}

function refreshCard(app) {
  const card = grid.querySelector(`[data-id="${CSS.escape(app.id)}"]`);
  if (!card) return;
  const tmp = document.createElement("div");
  tmp.innerHTML = cardHtml(app, 0);
  const fresh = tmp.firstElementChild;
  fresh.style.animation = "none";
  const focused = document.activeElement === card;
  card.replaceWith(fresh);
  if (focused) fresh.focus();
}

async function toggleFav(app) {
  try {
    state.settings = await api.toggleFavorite(app.id);
  } catch (e) {
    return toast(String(e), true);
  }
  renderFilters();
  if (state.filter === "fav") render(); else refreshCard(app);
  if (state.detail === app.id) renderDrawer(app);
}

let toastTimer;
function toast(msg, error = false, extra = "") {
  const t = $("#toast");
  t.textContent = msg;
  t.className = `toast show ${error ? "error" : ""} ${extra}`;
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => t.classList.remove("show"), error ? 4000 : 1800);
}

// ------------------------------------------------------------------ fiche détaillée
function openDrawer(app) {
  state.detail = app.id;
  renderDrawer(app);
  $("#drawer").classList.add("open");
  $("#drawer").setAttribute("aria-hidden", "false");
}

function closeDrawer() {
  state.detail = null;
  $("#drawer").classList.remove("open");
  $("#drawer").setAttribute("aria-hidden", "true");
}

function renderDrawer(app) {
  const stat = statOf(app);
  const sources = { sidecar: "fichier .portal.json", html: "balises du HTML", exe: "infos de version de l'exe", lnk: "raccourci Windows", url: "raccourci internet" };
  const facts = [
    ["Version", app.version],
    ["Auteur", app.author],
    ["Catégorie", app.category],
    ["Tags", app.tags.join(", ")],
    ["Tours", `${stat.count} — dernier : ${ago(stat.last)}`],
    ["Modifiée", fmtDate(app.modified)],
    ["Taille", fmtSize(app.size)],
    ["Fichier", app.path],
    ["Infos lues", app.sources.map((s) => sources[s] || s).join(", ") || "nom du fichier uniquement"],
  ].filter(([, v]) => v);
  $("#drawer-body").innerHTML = `
    <div style="--c:${esc(colorOf(app))}">${iconHtml(app)}</div>
    <h2>${esc(app.name)}</h2>
    <div class="meta-line"><span class="badge ${app.kind}">${app.kind}</span>${app.hidden ? "<span>· masquée</span>" : ""}${app.version ? `<span>v${esc(app.version.replace(/^v/i, ""))}</span>` : ""}</div>
    ${app.description ? `<p class="desc">${esc(app.description)}</p>` : ""}
    <div class="drawer-actions">
      <button class="btn primary" data-dact="launch">▶ Lancer</button>
      <button class="btn" data-dact="fav">${isFav(app) ? "★ Favori" : "☆ Favori"}</button>
      <button class="btn" data-dact="edit" title="Modifier la fiche (E)">✏️ Modifier</button>
      <button class="btn" data-dact="reveal" title="Afficher dans l'explorateur">📂</button>
    </div>
    <dl class="facts">${facts.map(([k, v]) => `<dt>${k}</dt><dd>${esc(v)}</dd>`).join("")}</dl>
    ${app.notes ? `<div class="notes">${esc(app.notes)}</div>` : ""}`;
}

$("#drawer").addEventListener("click", async (e) => {
  if (e.target.closest("[data-close]")) return closeDrawer();
  const act = e.target.closest("[data-dact]")?.dataset.dact;
  const app = appById(state.detail);
  if (!act || !app) return;
  if (act === "launch") launch(app);
  if (act === "fav") toggleFav(app);
  if (act === "edit") openEditor(app);
  if (act === "reveal") api.reveal(app).catch((err) => toast(String(err), true));
});

// ------------------------------------------------------------------ réglages
function openSettings() {
  const s = state.settings;
  $("#set-dir").value = s.appsDir || state.appsDir;
  $("#set-bpath").value = s.browserPath || "";
  $("#set-bargs").value = s.browserArgs || "{url}";
  for (const r of document.querySelectorAll("input[name=bmode]")) r.checked = r.value === s.browserMode;
  $(".custom-browser").hidden = s.browserMode !== "custom";
  renderThemePicker();
  $("#settings").classList.add("open");
  $("#settings").setAttribute("aria-hidden", "false");
}

function closeSettings() {
  $("#settings").classList.remove("open");
  $("#settings").setAttribute("aria-hidden", "true");
}

function renderThemePicker() {
  $("#theme-picker").innerHTML = THEMES.map((t) => `
    <button class="theme-opt ${state.settings.theme === t.id ? "active" : ""}" data-theme-id="${t.id}">
      <div class="swatch" style="background:${t.swatch}"></div><span>${esc(t.name)}</span>
    </button>`).join("");
}

async function savePrefs(prefs, { rescan = false } = {}) {
  try {
    state.settings = await api.updatePrefs(prefs);
  } catch (e) {
    return toast(String(e), true);
  }
  applyTheme(state.settings.theme);
  if (rescan) {
    state.signature = "";
    await load();
  }
}

$("#settings").addEventListener("click", async (e) => {
  if (e.target === e.currentTarget || e.target.closest("[data-close]")) return closeSettings();
  const theme = e.target.closest("[data-theme-id]")?.dataset.themeId;
  if (theme) {
    await savePrefs({ theme });
    renderThemePicker();
  }
});
$("#set-dir").addEventListener("change", (e) => savePrefs({ appsDir: e.target.value.trim() }, { rescan: true }));
$("#set-dir-pick").addEventListener("click", async () => {
  const dir = await api.pickFolder();
  if (!dir) return;
  $("#set-dir").value = dir;
  savePrefs({ appsDir: dir }, { rescan: true });
});
for (const r of document.querySelectorAll("input[name=bmode]")) {
  r.addEventListener("change", () => {
    $(".custom-browser").hidden = r.value !== "custom";
    savePrefs({ browserMode: r.value });
  });
}
$("#set-bpath").addEventListener("change", (e) => savePrefs({ browserPath: e.target.value.trim() }));
$("#set-bargs").addEventListener("change", (e) => savePrefs({ browserArgs: e.target.value.trim() }));

// ------------------------------------------------------------------ mode d'emploi
const manual = $("#manual");
const isOpen = (el) => el.classList.contains("open");

function openManual() {
  closeSettings();
  closeChangelog();
  manual.classList.add("open");
  manual.setAttribute("aria-hidden", "false");
}

function closeManual() {
  manual.classList.remove("open");
  manual.setAttribute("aria-hidden", "true");
}

manual.addEventListener("click", (e) => {
  if (e.target === e.currentTarget || e.target.closest("[data-close]")) return closeManual();
  const link = e.target.closest(".manual-toc a");
  if (link) {
    e.preventDefault();
    $(link.getAttribute("href"), manual).scrollIntoView({ block: "start" });
  }
});
$("#btn-help").addEventListener("click", openManual);

// ------------------------------------------------------------------ éditeur de fiche
// Le formulaire ne contient que ce qui est écrit dans le .portal.json ; un champ vide
// laisse la place à la valeur détectée (balises HTML, infos de l'exe), affichée en grisé.
const editor = $("#editor");
const editorForm = $("#editor-form");
const TEXT_FIELDS = ["name", "description", "version", "author", "category", "color", "icon", "notes", "entry"];
const ed = { app: null, tags: [], icon: { value: "", preview: "" } };
const field = (name) => editorForm.elements[name];

function openEditor(app) {
  const raw = app.sidecar || {};
  const d = app.detected;
  ed.app = app;
  for (const name of TEXT_FIELDS) {
    field(name).value = typeof raw[name] === "string" ? raw[name] : "";
    field(name).placeholder = d[name] || "";
  }
  field("entry").placeholder = app.path.split(/[\\/]/).pop();
  field("color").placeholder = "auto";
  field("icon").placeholder = d.icon.startsWith("data:") ? "(icône détectée)" : d.icon || "emoji ou image.png";
  field("hidden").checked = raw.hidden === true;
  const tags = Array.isArray(raw.tags) ? raw.tags.map(String) : typeof raw.tags === "string" ? raw.tags.split(",") : [];
  ed.tags = tags.map((t) => t.trim()).filter(Boolean);
  // Seul le backend sait lire une image : on garde l'aperçu de l'icône actuelle.
  ed.icon = { value: field("icon").value, preview: raw.icon ? app.icon : "" };
  $("#ed-entry-field").hidden = !app.folder;
  $("#ed-categories").innerHTML = [...new Set(state.apps.map((a) => a.category).filter(Boolean))]
    .map((c) => `<option value="${esc(c)}">`).join("");
  $("#ed-file").innerHTML = `Enregistré dans <code>${esc(app.metaFile)}</code>${Object.keys(raw).length ? "" : " (sera créé)"}`;
  renderTags();
  editorChanged();
  closeDrawer();
  editor.classList.add("open");
  editor.setAttribute("aria-hidden", "false");
  setTimeout(() => field("name").focus(), 50);
}

function closeEditor() {
  editor.classList.remove("open");
  editor.setAttribute("aria-hidden", "true");
}

function renderTags() {
  const box = $("#ed-tags");
  box.querySelectorAll(".tag-chip").forEach((c) => c.remove());
  box.insertAdjacentHTML("afterbegin", ed.tags.map((t, i) =>
    `<span class="tag-chip">#${esc(t)}<button type="button" data-untag="${i}" title="Retirer">✕</button></span>`).join(""));
  const detected = ed.app.detected.tags;
  $("#ed-tag-new").placeholder = ed.tags.length ? "" : detected.length ? detected.map((t) => "#" + t).join(" ") : "Entrée pour ajouter un tag";
}

function addTag() {
  const input = $("#ed-tag-new");
  for (const t of input.value.split(",").map((t) => t.trim().replace(/^#/, "")).filter(Boolean)) {
    if (!ed.tags.some((x) => x.toLowerCase() === t.toLowerCase())) ed.tags.push(t);
  }
  input.value = "";
  renderTags();
  editorChanged();
}

// Icône de l'aperçu : image déjà lue par le backend, emoji tapé, ou icône détectée.
function previewIcon() {
  const v = field("icon").value.trim();
  if (!v) return ed.app.detected.icon;
  if (v === ed.icon.value && ed.icon.preview) return ed.icon.preview;
  if (!/\.[a-z0-9]+$/i.test(v) && [...v].length <= 4) return v;
  return ed.app.detected.icon;
}

function editorChanged() {
  const app = ed.app;
  const d = app.detected;
  const v = (name) => field(name).value.trim();
  const preview = {
    ...app,
    name: v("name") || d.name,
    description: v("description") || d.description,
    version: v("version") || d.version,
    category: v("category") || d.category,
    tags: ed.tags.length ? ed.tags : d.tags,
    color: v("color") || d.color,
    icon: previewIcon(),
    hidden: field("hidden").checked,
  };
  $("#ed-preview").innerHTML = cardHtml(preview, 0);
  $("#ed-color-pick").value = /^#[0-9a-f]{6}$/i.test(preview.color) ? preview.color : "#888888";
  // Une version écrite dans le .portal.json masque celle du HTML ou de l'exe, même mise à jour ensuite.
  const warn = $("#ed-version-warn");
  warn.hidden = !(v("version") && d.version);
  warn.textContent = `⚠ La version déclarée par ${app.kind === "exe" ? "l'exe" : "le HTML"} (${d.version}) ne sera plus suivie, même mise à jour. Laissez vide pour la suivre.`;
}

editorForm.addEventListener("input", (e) => {
  if (e.target.id === "ed-color-pick") field("color").value = e.target.value;
  if (e.target.id !== "ed-tag-new") editorChanged();
});
$("#ed-tag-new").addEventListener("keydown", (e) => {
  if (e.key === "Enter" || e.key === ",") { e.preventDefault(); addTag(); }
  if (e.key === "Backspace" && !e.target.value && ed.tags.length) { ed.tags.pop(); renderTags(); editorChanged(); }
});
$("#ed-tag-new").addEventListener("blur", () => { if ($("#ed-tag-new").value.trim()) addTag(); });
$("#ed-tags").addEventListener("click", (e) => {
  const i = e.target.closest("[data-untag]")?.dataset.untag;
  if (i !== undefined) { ed.tags.splice(Number(i), 1); renderTags(); editorChanged(); }
  $("#ed-tag-new").focus();
});
$("#ed-color-auto").addEventListener("click", () => { field("color").value = ""; editorChanged(); });
$("#ed-icon-pick").addEventListener("click", async () => {
  try {
    const picked = await api.pickIcon(ed.app);
    if (!picked) return;
    ed.icon = picked;
    field("icon").value = picked.value;
    editorChanged();
  } catch (e) {
    toast(String(e), true);
  }
});

editor.addEventListener("click", (e) => {
  if (e.target === e.currentTarget || e.target.closest("[data-close]")) closeEditor();
});

editorForm.addEventListener("submit", async (e) => {
  e.preventDefault();
  if ($("#ed-tag-new").value.trim()) addTag();
  const fields = Object.fromEntries(TEXT_FIELDS.map((name) => [name, field(name).value.trim()]));
  if (!ed.app.folder) delete fields.entry;
  fields.tags = ed.tags;
  fields.hidden = field("hidden").checked;
  try {
    await api.saveMeta(ed.app, fields);
  } catch (err) {
    return toast(String(err), true);
  }
  const hiddenNow = fields.hidden && !ed.app.hidden;
  closeEditor();
  state.signature = "";
  await load();
  toast(hiddenNow ? "Appli masquée : filtre « 👁 Masquées » pour la retrouver" : "Fiche enregistrée ✔");
});

// ------------------------------------------------------------------ version et nouveautés
const changelog = $("#changelog");
const versionBadge = $("#version-badge");
let about = { version: "", changelog: "" };

api.about().then((a) => {
  about = a;
  versionBadge.textContent = `v${a.version}`;
  versionBadge.classList.toggle("new", store("seenVersion") !== a.version);
}).catch(() => { versionBadge.hidden = true; });

// Rendu minimal du CHANGELOG.md : titres de version, sections, listes, `code` et **gras**.
function renderChangelog(md, current) {
  const inline = (s) => esc(s)
    .replace(/`([^`]+)`/g, "<code>$1</code>")
    .replace(/\*\*([^*]+)\*\*/g, "<b>$1</b>")
    .replace(/\[([^\]]+)\]\(https?:[^)]+\)/g, "$1");
  const day = (d) => { const t = new Date(d); return isNaN(t) ? d : t.toLocaleDateString("fr-FR", { dateStyle: "long" }); };
  let html = "";
  let inList = false;
  const closeList = () => { if (inList) { html += "</ul>"; inList = false; } };
  for (const line of md.split(/\r?\n/)) {
    let m;
    if (line.startsWith("# ")) continue;
    if ((m = line.match(/^## \[?([^\]\s]+)\]?(?:\s*-\s*(\S+))?/))) {
      closeList();
      html += `<div class="changelog-version"><h3>v${esc(m[1])}</h3>${m[2] ? `<time>${esc(day(m[2]))}</time>` : ""}${m[1] === current ? '<span class="current">installée</span>' : ""}</div>`;
    } else if ((m = line.match(/^### (.+)/))) {
      closeList();
      html += `<h4>${inline(m[1])}</h4>`;
    } else if ((m = line.match(/^- (.+)/))) {
      if (!inList) { html += "<ul>"; inList = true; }
      html += `<li>${inline(m[1])}</li>`;
    } else if (line.trim()) {
      closeList();
      html += `<p>${inline(line)}</p>`;
    }
  }
  closeList();
  return html;
}

function openChangelog() {
  closeSettings();
  closeManual();
  $("#changelog-body").innerHTML = renderChangelog(about.changelog, about.version);
  changelog.classList.add("open");
  changelog.setAttribute("aria-hidden", "false");
  store("seenVersion", about.version);
  versionBadge.classList.remove("new");
}

function closeChangelog() {
  changelog.classList.remove("open");
  changelog.setAttribute("aria-hidden", "true");
}

versionBadge.addEventListener("click", openChangelog);

// ------------------------------------------------------------------ licences
const license = $("#license");
function openLicense() {
  closeSettings();
  closeManual();
  closeChangelog();
  $("#license-body").innerHTML = (about.licenses || []).map((l, i) =>
    `<details ${i === 0 ? "open" : ""}><summary>${esc(l.name)}</summary><pre>${esc(l.text)}</pre></details>`).join("");
  license.classList.add("open");
  license.setAttribute("aria-hidden", "false");
}
function closeLicense() {
  license.classList.remove("open");
  license.setAttribute("aria-hidden", "true");
}
$("#btn-license").addEventListener("click", openLicense);
license.addEventListener("click", (e) => {
  if (e.target === e.currentTarget || e.target.closest("[data-close]")) closeLicense();
});
changelog.addEventListener("click", (e) => {
  if (e.target === e.currentTarget || e.target.closest("[data-close]")) closeChangelog();
});
$("#settings-help").addEventListener("click", openManual);

// ------------------------------------------------------------------ événements globaux
const speed = document.createElement("div");
speed.className = "speed-lines";
document.body.append(speed);

grid.addEventListener("click", (e) => {
  const card = e.target.closest(".card");
  if (!card) return;
  const app = appById(card.dataset.id);
  const act = e.target.closest("[data-act]")?.dataset.act;
  if (act === "fav") return toggleFav(app);
  if (act === "info") return openDrawer(app);
  launch(app, card);
});
grid.addEventListener("contextmenu", (e) => {
  const card = e.target.closest(".card");
  if (!card) return;
  e.preventDefault();
  openDrawer(appById(card.dataset.id));
});
grid.addEventListener("pointermove", (e) => {
  const card = e.target.closest(".card");
  if (!card) return;
  const r = card.getBoundingClientRect();
  card.style.setProperty("--mx", `${e.clientX - r.left}px`);
  card.style.setProperty("--my", `${e.clientY - r.top}px`);
});

$("#filters").addEventListener("click", (e) => {
  const f = e.target.closest("[data-filter]")?.dataset.filter;
  if (!f) return;
  state.filter = f;
  render();
});

const search = $("#search");
search.addEventListener("input", () => {
  state.query = search.value;
  render();
  const typed = norm(search.value).trim();
  if (isTheme("lol") && /^\/?ff(15)?$/.test(typed)) {
    toast("Vote de reddition échoué (1/4)", true);
  }
  if (isTheme("sommet")) {
    if (/^(ohe|hého|heho|youhou|ouhou)$/.test(typed)) toast(`${search.value}… ${search.value}… ${search.value.toLowerCase()}…`);
    if (typed === "meteo") {
      toast(new Date().getHours() >= 13
        ? "Orages prévus en fin de journée ⛈️ Ne traîne pas au sommet"
        : "Grand beau, iso 0° à 3 800 m ☀️ Départ à l'aube conseillé");
    }
  }
});

const isTheme = (id) => document.documentElement.dataset.theme === id;
const isLol = () => isTheme("lol");

// La marmotte sort la tête de temps en temps ; un clic la fait siffler (alerte : aigle en vue !).
const marmot = $("#marmot");
function marmotPeek() {
  if (isTheme("sommet") && !document.hidden) {
    marmot.style.left = `${10 + Math.random() * 75}vw`;
    marmot.classList.add("peek");
    setTimeout(() => marmot.classList.remove("peek"), 3500);
  }
  setTimeout(marmotPeek, 40_000 + Math.random() * 50_000);
}
setTimeout(marmotPeek, 15_000);
marmot.addEventListener("click", () => {
  marmot.classList.remove("peek");
  toast("Fiiiiuuu ! La marmotte a donné l'alerte 🦅");
});

// Touche B : rappel à la base (en accéléré, personne n'a 8 secondes à perdre).
let recalling = false;
function recall() {
  if (recalling) return;
  recalling = true;
  document.body.classList.add("recalling");
  setTimeout(() => {
    document.body.classList.remove("recalling");
    recalling = false;
    window.scrollTo({ top: 0, behavior: "smooth" });
    grid.querySelector(".card")?.focus({ preventScroll: true });
    toast("Retour à la base");
  }, 1600);
}

const teemoLines = ["Capitaine Teemo, au rapport !", "Taille ne veut rien dire.", "Hut, deux, trois, quatre !"];
let teemoIdx = 0;
$("#teemo").addEventListener("click", () => toast(teemoLines[teemoIdx++ % teemoLines.length]));

const sort = $("#sort");
sort.value = state.sort;
sort.addEventListener("change", () => { state.sort = sort.value; store("sort", sort.value); render(); });

$("#btn-refresh").addEventListener("click", (e) => {
  e.currentTarget.classList.remove("spin");
  void e.currentTarget.offsetWidth;
  e.currentTarget.classList.add("spin");
  state.signature = "";
  load();
});
$("#btn-folder").addEventListener("click", () => api.openAppsDir().catch((err) => toast(String(err), true)));
$("#empty-open").addEventListener("click", () => api.openAppsDir().catch((err) => toast(String(err), true)));
$("#btn-settings").addEventListener("click", openSettings);

// Retour sur la fenêtre : on rescanne en silence pour voir les nouvelles applis.
window.addEventListener("focus", () => load({ quiet: true }));

function moveFocus(card, key) {
  const cards = [...grid.querySelectorAll(".card")];
  if (!cards.length) return;
  if (!card) return cards[0].focus();
  const cols = getComputedStyle(grid).gridTemplateColumns.split(" ").length;
  const i = cards.indexOf(card);
  const next = { ArrowRight: i + 1, ArrowLeft: i - 1, ArrowDown: i + cols, ArrowUp: i - cols }[key];
  cards[Math.max(0, Math.min(cards.length - 1, next))].focus();
}

document.addEventListener("keydown", (e) => {
  const inField = e.target.matches("input, select, textarea");
  if ((e.ctrlKey && e.key.toLowerCase() === "k") || (e.key === "/" && !inField)) {
    e.preventDefault();
    search.focus();
    search.select();
    return;
  }
  if (e.key === "F1") {
    e.preventDefault();
    isOpen(manual) ? closeManual() : openManual();
    return;
  }
  if (e.key === "F5") {
    e.preventDefault();
    $("#btn-refresh").click();
    return;
  }
  if (isOpen(editor)) {
    if (e.key === "Escape") closeEditor();
    if (e.ctrlKey && e.key.toLowerCase() === "s") { e.preventDefault(); editorForm.requestSubmit(); }
    return;
  }
  if (e.key === "Escape") {
    if (isOpen(license)) return closeLicense();
    if (isOpen(changelog)) return closeChangelog();
    if (isOpen(manual)) return closeManual();
    if ($("#settings").classList.contains("open")) return closeSettings();
    if (state.detail) return closeDrawer();
    if (search.value) { search.value = ""; state.query = ""; render(); }
    search.blur();
    return;
  }
  if ($("#settings").classList.contains("open") || isOpen(manual) || isOpen(changelog) || isOpen(license)) return;

  const card = e.target.closest?.(".card");
  if (e.target === search && (e.key === "ArrowDown" || e.key === "Enter")) {
    e.preventDefault();
    const first = grid.querySelector(".card");
    if (e.key === "Enter" && first) launch(appById(first.dataset.id), first);
    else moveFocus(null);
    return;
  }
  if (isLol() && !inField && e.key.toLowerCase() === "b" && !e.ctrlKey && !e.altKey) {
    e.preventDefault();
    return recall();
  }
  if (e.key.startsWith("Arrow") && !inField) {
    e.preventDefault();
    return moveFocus(card, e.key);
  }
  if (card) {
    const app = appById(card.dataset.id);
    if (e.key === "Enter") return launch(app, card);
    if (e.key.toLowerCase() === "f") return toggleFav(app);
    if (e.key.toLowerCase() === "i") return openDrawer(app);
    if (e.key.toLowerCase() === "e") { e.preventDefault(); return openEditor(app); }
  }
  // Taper n'importe où lance la recherche.
  if (!inField && e.key.length === 1 && !e.ctrlKey && !e.altKey && !e.metaKey) {
    search.focus();
  }
});

load();

// ------------------------------------------------------------------ backend simulé (aperçu navigateur)
function mockApi() {
  const day = 864e5, now = Date.now();
  const settings = {
    appsDir: "", theme: store("mock-theme") || "arcade", browserMode: "app", browserPath: "", browserArgs: "{url}",
    favorites: ["kanban.html"],
    stats: { "kanban.html": { count: 42, last: now - 3e5 }, "csv-lab.exe": { count: 17, last: now - day }, "json-garage/index.html": { count: 5, last: now - 9 * day } },
  };
  const a = (id, kind, name, extra = {}) => {
    const app = {
      id, kind, name, path: "C:\\Apps\\" + id.replace(/\//g, "\\"), description: "", version: "", author: "", category: "", tags: [], color: "", icon: "", notes: "",
      size: 180000, modified: now - Math.random() * 30 * day, sources: [], hidden: false, folder: id.includes("/"), sidecar: {}, ...extra,
    };
    app.metaFile = app.folder ? app.path.replace(/[^\\]+$/, "portal.json") : app.path.replace(/\.\w+$/, ".portal.json");
    app.detected = { name: app.name, description: app.description, version: app.version, author: app.author, category: app.category,
      tags: app.tags, color: app.color, icon: app.icon, sources: app.sources };
    return app;
  };
  const apps = [
    a("kanban.html", "html", "Kanban Turbo", { description: "Tableau de tâches ultra-rapide, tout tient dans un seul fichier.", version: "2.4.1", category: "Productivité", tags: ["tâches", "perso"], icon: "🏁", sources: ["html"] }),
    a("csv-lab.exe", "exe", "CSV Lab", { description: "Nettoie, fusionne et convertit des CSV à la volée.", version: "1.0.3", author: "Moi", category: "Data", tags: ["csv", "excel"], color: "#00f0ff", sources: ["exe", "sidecar"] }),
    a("json-garage/index.html", "html", "JSON Garage", { description: "Visualiseur et formateur JSON avec diff.", version: "0.9", category: "Data", tags: ["json"], icon: "🔧", sources: ["sidecar"] }),
    a("pomodoro.html", "html", "Pit Stop Timer", { description: "Minuteur pomodoro façon arrêt au stand.", category: "Productivité", icon: "⏱️", sources: ["html"] }),
    a("regex.html", "html", "Regex Drift", { description: "Testeur d'expressions régulières avec surlignage en direct.", version: "3.1", category: "Dev", tags: ["regex"], sources: ["html"] }),
    a("intranet.url", "url", "Intranet RH", { description: "http://intranet.local/rh", category: "Outils", sources: ["url"] }),
    a("screenshot.exe", "exe", "SnapShot", { description: "Capture d'écran annotée.", version: "5.2.0.0", author: "ACME", category: "Outils", sources: ["exe"] }),
    a("notes.html", "html", "notes"),
    a("color-picker.html", "html", "Livrée — nuancier", { description: "Génère des palettes de couleurs pour vos thèmes.", category: "Design", icon: "🎨", color: "#ffd23f", sources: ["html"] }),
  ];
  const clone = (o) => JSON.parse(JSON.stringify(o));
  return {
    load: async () => ({ settings: clone(settings), appsDir: "C:\\Apps", apps: clone(apps) }),
    updatePrefs: async (p) => {
      for (const [k, v] of Object.entries(p)) if (v !== undefined) settings[k] = v;
      store("mock-theme", settings.theme);
      return clone(settings);
    },
    toggleFavorite: async (id) => {
      const i = settings.favorites.indexOf(id);
      i >= 0 ? settings.favorites.splice(i, 1) : settings.favorites.push(id);
      return clone(settings);
    },
    launch: async (app) => {
      const s = (settings.stats[app.id] ??= { count: 0, last: 0 });
      s.count++; s.last = Date.now();
      return clone(settings);
    },
    reveal: async () => {},
    openAppsDir: async () => {},
    pickFolder: async () => null,
    pickIcon: async () => null,
    saveMeta: async (target, fields) => {
      const app = apps.find((x) => x.id === target.id);
      app.sidecar = Object.fromEntries(Object.entries(fields).filter(([, v]) => v && v.length !== 0));
      for (const k of ["name", "description", "version", "author", "category", "tags", "color", "icon", "notes"]) {
        app[k] = app.sidecar[k] ?? app.detected[k] ?? "";
      }
      app.hidden = !!app.sidecar.hidden;
    },
    about: async () => ({ version: "0.0.0-dev", licenses: [{ name: "Porta[b]l Apps — MIT", text: "(texte lu dans l'appli)" }], changelog: "## [0.0.0-dev] - 2026-01-01\n\n### Ajouté\n- Aperçu navigateur : le vrai changelog n'est lu que dans l'appli." }),
  };
}
