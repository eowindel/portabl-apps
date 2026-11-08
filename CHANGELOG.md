# Changelog

Toutes les évolutions notables de Porta[b]l Apps.
Format inspiré de [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/), versions en [SemVer](https://semver.org/lang/fr/).

## [0.9.0] - 2026-11-08

### Ajouté
- **Mises à jour** : en activant l'option dans les réglages (désactivée par défaut), le lanceur cherche la dernière release GitHub de lui-même et des applis qui déclarent un dépôt (`repo` dans le `.portal.json`, ou `<meta name="portal:repo">`). Une pastille ⬆ apparaît sur la carte, dans la fiche et sur le badge de version, avec un lien vers la release. Seul `api.github.com` est contacté, au plus une fois par jour et par dépôt ; rien n'est téléchargé ni installé.
- Champ « Dépôt GitHub » dans l'éditeur de fiche.

## [0.8.3] - 2026-11-08

### Corrigé
- Linux : l'appli n'avait pas d'icône dans la barre des tâches (un « W » générique) et l'épingler ne permettait pas de la relancer. Elle crée maintenant son lanceur (`~/.local/share/applications`) et son icône au démarrage.

## [0.8.2] - 2026-11-04

### Corrigé
- Linux : la liste déroulante de tri avait un fond blanc et un texte illisible ; elle reprend maintenant les couleurs du thème.

## [0.8.1] - 2026-11-04

### Corrigé
- Linux : le **[b]** de « Porta[b]l Apps » n'apparaissait pas dans le titre.
- Linux : la roue dentée des réglages était minuscule ; c'est maintenant une icône dessinée, de même taille partout.

## [0.8.0] - 2026-11-03

### Ajouté
- Porta[b]l Apps fonctionne aussi sous **Linux** : les applis HTML s'ouvrent dans une fenêtre dédiée (Chromium, Chrome, Brave ou Edge trouvé dans le PATH), sinon dans l'application par défaut du système. « Afficher dans le dossier » et « Ouvrir le dossier des applis » passent par `xdg-open`.
- Une build automatique (GitHub Actions) compile et teste Windows et Linux, et publie les binaires dans une release à chaque tag `vX.Y.Z`.

### Modifié
- Sous Linux, les applis `.exe` et les raccourcis `.lnk` / `.url` s'affichent mais ne se lancent que sous Windows, avec un message clair.

## [0.7.0] - 2026-10-27

### Ajouté
- Les raccourcis Windows (`.lnk`) et internet (`.url`) deviennent des cartes, avec leur description, leur icône (ou celle de leur cible) et un filtre **Raccourcis**. Un clic les lance comme un double-clic dans l'explorateur : arguments et dossier de départ sont respectés.
- Dans un sous-dossier sans `.exe`, un unique raccourci suffit à en faire une appli.

## [0.6.1] - 2026-10-27

### Modifié
- Le pied de page « © 2026 casarou · Licence MIT » reste collé en bas de la fenêtre, au lieu de flotter sous les cartes quand il y a peu d'applis.

## [0.6.0] - 2026-10-27

### Ajouté
- Pied de page « © 2026 casarou · Licence MIT » : un clic ouvre les licences (la MIT, puis celles des polices embarquées).

### Modifié
- Les sons de lancement et l'option « Bruit de moteur » sont retirés.

### Corrigé
- Le compteur de tours et la date se mettent à jour dès le lancement, sans attendre la fin de l'animation de la carte.

## [0.5.2] - 2026-10-27

### Corrigé
- Les écrans « rien à afficher » (aucun favori, aucun résultat, aucune appli masquée) ont un texte et un pictogramme neutres, plus de drapeau à damier hors thème.

## [0.5.1] - 2026-10-27

### Modifié
- Les boutons ★ et ⓘ passent dans le pied de la carte : ils ne se superposent plus au titre.
- Un titre trop long passe sur deux lignes, à la limite des mots (ou après `_`, `-`, `.`), au lieu d'être coupé tout de suite.
- Le drapeau à damier du compteur de tours devient une fusée 🚀, plus neutre selon les thèmes.

## [0.5.0] - 2026-10-27

### Ajouté
- Éditeur de fiche : bouton **✏️ Modifier** dans la fiche détaillée ou touche **E** sur une carte. Il écrit le `.portal.json` à votre place, avec un aperçu de la carte en direct.
- Les valeurs détectées (balises HTML, infos de l'exe) s'affichent en grisé : seul ce que vous changez est enregistré.
- Choix d'une image pour l'icône ; une image prise ailleurs est copiée à côté de l'appli.
- Filtre **👁 Masquées** pour retrouver et réafficher les applis masquées.

## [0.4.0] - 2026-10-27

### Ajouté
- Mode d'emploi intégré : bouton **?**, touche **F1** ou depuis les réglages.
- Badge de version cliquable avec la liste des nouveautés.

## [0.3.0] - 2026-10-27

### Ajouté
- Thème **Sommet**, pour les amoureux de la montagne : le ciel suit l'heure de la journée.

### Modifié
- L'appli s'appelle désormais **Porta[b]l Apps** (`portabl-apps.exe`).
- Les réglages sont enregistrés dans `portabl-apps.json` ; l'ancien fichier `pitlane.json` est repris automatiquement.

## [0.2.0] - 2026-10-27

### Ajouté
- Thème **Hextech**, pour les invocateurs.
- Chaque thème peut avoir ses propres textes, son son de lancement et ses annonces.

## [0.1.0] - 2026-10-27

### Ajouté
- Lanceur portable, hors réseau, pour applis `.html` et `.exe`.
- Scan automatique du dossier `apps\`, applis rangées en fichiers seuls ou en sous-dossiers.
- Infos des applis lues depuis un fichier `.portal.json`, les balises du HTML ou les propriétés de l'exe (icône comprise).
- Recherche, filtres par type et catégorie, favoris, tri, compteur de lancements, navigation au clavier.
- Applis HTML ouvertes dans une fenêtre Edge dédiée, le navigateur par défaut ou un navigateur personnalisé.
- Thèmes **Néon Arcade**, **Superbike**, **Enduro** et **8-bit**.
