# À faire

- [ ] **Retirer l'option « Bruit de moteur au lancement »** (Réglages ⚙ > Ambiance).
  - Où : la case `#set-sound` dans `src/index.html`, sa gestion dans `src/app.js` (`savePrefs({ sound })`, tests `state.settings.sound`), et le champ `sound` dans `src-tauri/src/settings.rs`.
  - À trancher avant de le faire : supprimer le son complètement, ou garder le son (moteur / carillon / cloche selon le thème) mais sans possibilité de le couper ?

- [ ] **Le compteur de tours ne se met pas à jour tout de suite** quand on clique pour lancer une appli.
  - Piste : dans `launch()` (`src/app.js`), la carte n'est redessinée (`refreshCard`) qu'à la fin de l'animation de lancement (`.card.launch`, 0,75 s). Si l'animation ne se joue pas (Windows avec animations réduites → `prefers-reduced-motion`), `animationend` ne part jamais et le compteur reste figé jusqu'au prochain rescan.
  - Idée : mettre à jour le compteur et la date dès le retour de `api.launch`, sans attendre l'animation.
