# À faire

- [ ] **Retirer l'option « Bruit de moteur au lancement »** (Réglages ⚙ > Ambiance).
  - Où : la case `#set-sound` dans `src/index.html`, sa gestion dans `src/app.js` (`savePrefs({ sound })`, tests `state.settings.sound`), et le champ `sound` dans `src-tauri/src/settings.rs`.
  - À trancher avant de le faire : supprimer le son complètement, ou garder le son (moteur / carillon / cloche selon le thème) mais sans possibilité de le couper ?
