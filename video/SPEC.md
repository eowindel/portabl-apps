GOAL:    Le spectateur retient « un dossier, et toutes mes applis sont à un clic », et veut l'essayer. (Mélange A+B de STORYLINES.md : le bazar, puis l'ordre, avec le compte-tours comme pont et comme signature.)
OUTPUT:  HyperFrames, 1920 x 1080, 30 fps
TIME:    28 s x 30 = 840 frames. Grille 120 BPM (15 frames / temps, 60 / mesure) même sans musique. Sans voix.

LOOK:    Deux mondes.
         MONDE 1, le bazar : sol #c9ced6 (gris acier froid, sans texture), encre #1b1d24, tuiles #f4f6f9 avec ombre portée 0 8px 0 rgba(27,29,36,.18) (sans flou).
           Fichiers : badge carré 72 px (HTML #ff8a3d, EXE #3d8bff, URL #2ecf8a), nom en Chakra Petch 600, 30 px.
           Accent unique : #ff2bd6, seulement sur les 4 résultats « kanban ».
         MONDE 2, le garage : la vraie interface de l'appli (captures à 1,75x de sa taille réelle), puis le néon de la marque :
           sol #0b0614, cyan #00f0ff, magenta #ff2bd6, jaune #ffd23f, texte #f6eaff. Orbitron 900 italique, Chakra Petch 600.
           Le néon est le matériau de la marque, il est donc permis ; pas de particules ni de lueurs ajoutées par-dessus la vraie UI.
         Titres : Orbitron 900 italique, 130 px (claim) / 96 px (monde 1), interlettrage +6 %. Corps : Chakra Petch 600, 30 px minimum.
OBJECTS: Dossier jaune #ffd23f (onglet, face avant, bord haut plus clair), étiqueté « apps/ ». Compte-tours du logo (arc magenta, aiguille cyan, moyeu clair, traits 6/4 sur viewBox 64). Damier 24 px. Curseur : flèche blanche cernée d'encre, 78 px (7 % de la hauteur).
MOTION:  enter = power3.out ; move = power2.inOut ; exit = power2.in ; caméra = power3.inOut 0,5 s ; ressort du curseur au clic : scale .86 -> 1 en 6 frames.
         Un seul geste principal par plan ; les gestes secondaires suivent 3 à 6 frames plus tard. Coupes dures sur le temps pour les thèmes.
SHOTS:   (s = secondes)
         1  0,0–3,5   Bazar : 12 fichiers qui se chevauchent, recherche « kanban » -> 4 résultats magenta. « Lequel ? ». Curseur qui cherche. Fin : cut sur whoosh.
         2  3,5–8,0   Un dossier : les 12 tuiles volent dans le dossier « apps/ » (écart 3 frames), « Un dossier. » s'écrit à droite, un iris néon s'ouvre depuis le dossier, le compte-tours se trace et l'aiguille retombe : même cadrage que le plan 3.
         3  8,0–15,5  Le garage : le compte-tours rétrécit jusqu'au logo de la vraie UI (match cut) ; curseur -> recherche, frappe réelle « kanban » (6 captures) ; clic carte, lancement (5 captures), la carte devient la fenêtre de Kanban Turbo ; zoom jauges : 64 -> 65 tours.
         4  15,5–18,5 Six thèmes, un par temps (15 frames), coupes dures, poussée 5 %, nom du thème dans un bandeau plein de 64 px en bas.
         5  18,5–22,5 Claim : « UN DOSSIER. » puis « TOUTES TES APPLIS. », soleil rayé et route en grille, puis trois blocs HTML · EXE · RACCOURCIS.
         6  22,5–28,0 Fin : compte-tours 520 px, aiguille dans le rouge (impact), damier, wordmark par volet, « Windows · Linux · ~4 Mo · MIT », adresse du dépôt tenue 2,5 s.
SOUND:   Pas de musique (aucune source libre disponible ici). Effets : whoosh sur chaque changement de monde, clic sur les clics, pop sur le compteur, riser 4 s -> impact sur le compte-tours final. Le bazar est quasi silencieux.
BANNED:  tout ce que spec.md interdit par défaut, sauf le néon de la marque ; en plus : verre dépoli, UI inclinée en 3D, logo cantonné dans un coin, cartes en rangée de trois, titre posé sur une capture assombrie, textes qui dérivent après leur arrivée.
QA:      0,0 s (tuiles + recherche, jamais vide) · 3,0 s (4 résultats magenta) · 6,0 s (dossier plein, titre) · 7,9 s (compte-tours centré = 8,0 s) · 9,4 s (zoom sur la recherche) · 11,6 s (carte en lancement) · 12,5 s (fenêtre Kanban) · 14,7 s (65) · 16,0 s (thème 2) · 21,0 s (claim) · 27,5 s (fin).
