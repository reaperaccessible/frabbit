# Changelog

## [1.0.15] - 2026-08-05

### Corrections

- **Vraie correction du blocage quand le serveur d'un produit est en panne.** La 1.0.14 avait rendu résiliente une première voie de vérification des versions, mais l'interface graphique en utilise une autre (une vérification produit par produit) — et là, **une seule vérification en échec (par exemple FFmpeg quand gyan.dev renvoie 503) affichait une page d'erreur bloquante** : impossible d'atteindre la liste des produits ni d'installer quoi que ce soit. Désormais, l'interface **continue toujours** jusqu'à la liste des produits : un produit dont le serveur ne répond pas apparaît simplement avec une **note d'information non bloquante** (« sa dernière version n'a pas pu être vérifiée »), et tout le reste s'installe normalement. On peut aller au bout même si le site web d'un produit est en erreur.

## [1.0.14] - 2026-08-05

### Corrections

- **La panne d'un seul serveur ne bloque plus tout FRABBIT au démarrage.** Au lancement, FRABBIT vérifie la dernière version de chaque produit en ligne. Si l'un de ces serveurs était momentanément indisponible — par exemple gyan.dev, qui héberge FFmpeg, renvoyant une erreur 503 — **tout le démarrage échouait** et rien ne pouvait être installé. Désormais, un produit dont le serveur ne répond pas s'affiche simplement sans numéro de version disponible, et tous les autres restent utilisables. FRABBIT s'ouvre normalement même si un fournisseur externe est en panne.

## [1.0.13] - 2026-08-05

### Corrections

- **La correction du nom du dépôt ReaperAccessible s'applique enfin sur une installation existante.** Les versions 1.0.11 et 1.0.12 corrigeaient bien le nom, mais l'étape qui le fait n'était **jamais exécutée** quand le dépôt avait déjà été ajouté sous l'ancien nom : FRABBIT vérifiait s'il était « déjà configuré » en ne regardant **que l'adresse** du dépôt, pas son nom. L'adresse étant déjà présente, il concluait « c'est fait » et sautait l'étape — donc le mauvais nom (« ReaperAccessible FR ») restait, et ReaPack gardait le mauvais dossier de scripts. FRABBIT compare désormais **aussi le nom** : si l'adresse est là mais sous un nom erroné, l'étape reste active et corrige le nom au bon (« ReaperAccessible scripts »).

## [1.0.12] - 2026-08-05

### Corrections

- **Le nom du dépôt ReaperAccessible est corrigé même s'il avait déjà été ajouté sous l'ancien nom.** La 1.0.11 écrivait le bon nom (« ReaperAccessible scripts ») uniquement pour une configuration ReaPack neuve. Mais si une version précédente de FRABBIT avait déjà ajouté le dépôt sous « ReaperAccessible FR », FRABBIT n'y touchait pas (il ne re-modifie pas un dépôt dont l'adresse est déjà présente) — le mauvais nom restait, et ReaPack rangeait donc toujours les scripts dans un dossier au mauvais nom. FRABBIT **corrige désormais le nom en place** : si l'adresse du dépôt est déjà là mais sous un nom erroné, il le renomme au nom correct. (Après la mise à jour, ouvrez ReaPack et synchronisez pour que les scripts soient réinstallés dans le bon dossier ; l'ancien dossier peut être supprimé.)

## [1.0.11] - 2026-08-05

### Corrections

- **Le dépôt ReaperAccessible est ajouté à ReaPack sous le bon nom.** FRABBIT déclarait le dépôt sous « ReaperAccessible FR » (et « ReaperAccessible EN »), alors que le dépôt lui-même s'appelle « ReaperAccessible scripts » (et « ReaperAccessible scripts US » en anglais). Comme ReaPack utilise ce nom pour le dossier où il range les scripts, et que REAPER calcule l'identifiant de chaque script à partir de son chemin, un nom incorrect plaçait les scripts dans le mauvais dossier et pouvait casser les raccourcis du keymap qui les visent. FRABBIT utilise désormais exactement le nom déclaré par chaque dépôt.

## [1.0.10] - 2026-08-05

### Nouveautés

- **Mise à jour automatique de FRABBIT sur macOS.** L'équivalent de ce qui existe sous Windows depuis la 1.0.8 : au démarrage, FRABBIT vérifie s'il existe une version plus récente et propose de l'installer dans une boîte de dialogue accessible. Sur Mac, FRABBIT télécharge et vérifie la nouvelle version, remplace **l'application entière** (`Frabbit.app`), puis se relance. Remplacer l'application complète plutôt que son seul programme interne est nécessaire : c'est l'application qui porte le numéro de version affiché par le Finder, les repères de langue que VoiceOver utilise pour choisir sa voix, et la signature qui scelle le tout. Si l'application se trouve dans un dossier en lecture seule, FRABBIT propose d'ouvrir la page de téléchargement ; en cas d'échec à n'importe quelle étape, l'application en place est restaurée telle quelle et FRABBIT continue de fonctionner dans sa version actuelle.
- FRABBIT lancé en ligne de commande sur Mac (un binaire seul, hors application) continue de se mettre à jour comme avant, en remplaçant ce seul fichier.

### Corrections

- L'application macOS n'annonce plus l'allemand au système ; elle déclare le français et l'anglais, les deux langues réellement proposées.

## [1.0.9] - 2026-08-04

### Modifications

- **ReaPack n'impose plus ses dépôts par défaut.** Lors d'une **première** installation de ReaPack, FRABBIT écrit désormais d'office deux réglages dans un `reapack.ini` neuf : il empêche ReaPack d'ajouter ses dépôts par défaut (ReaTeam Extensions, ReaTeam Scripts, MPL, etc.) au premier lancement de REAPER, et il désactive l'installation automatique. Résultat : synchroniser ReaPack n'installe plus des milliers de scripts que vous n'avez jamais demandés. Vous gardez le contrôle total — il suffit de parcourir ReaPack et d'installer uniquement ce que vous voulez. Ce comportement est appliqué automatiquement, sans option à cocher : FRABBIT n'impose aucun dépôt. (Le dépôt « ReaPack » de reapack.com, que ReaPack réajoute toujours lui-même pour ses propres mises à jour, reste présent mais inoffensif puisque l'installation automatique est coupée.)
- **Vos réglages ReaPack existants sont respectés.** Si ReaPack a déjà été configuré (vous l'avez déjà lancé, ou vous avez volontairement gardé certains dépôts), une réinstallation ou une mise à jour de ReaPack via FRABBIT **ne touche à rien** : aucun dépôt n'est supprimé et aucun réglage n'est modifié. FRABBIT n'agit que sur une configuration ReaPack neuve.

### Corrections

- **Fin du « REAPER Version inconnue » quand REAPER n'est pas installé.** La ligne de cible affichait « REAPER Version inconnue dans C:\Program Files\REAPER (x64) » lorsqu'aucun REAPER n'était présent — un « inconnue » trompeur qui donnait l'impression que FRABBIT ne maîtrisait pas l'état de la machine. Elle indique désormais clairement « REAPER — non installé (sera installé dans …) ». Une version « inconnue » n'apparaît plus que dans le cas rare d'un REAPER bien présent mais dont la version n'a pas pu être lue (affichée « version indéterminée »).
- **Un échec sur un paquet n'interrompt plus toute l'installation, et le bilan dit enfin la vérité.** Avant, si un paquet échouait (typiquement l'invite administrateur de Windows refusée ou manquée), FRABBIT s'arrêtait net : les paquets suivants n'étaient jamais installés et le message affichait « Rien n'a été installé » — trompeur, car certains paquets l'étaient déjà. Désormais, chaque paquet est traité indépendamment : un échec est signalé **sur ce paquet précis, avec sa raison**, et les autres paquets continuent de s'installer. La page de fin liste le résultat réel de chaque paquet (« installé », « échec : approbation administrateur refusée », etc.), affiche en tête « Terminé avec des erreurs : N paquet(s) non installés », et un compteur d'échecs. Fini de croire que tout est installé alors que non.
- **L'installation de REAPER fonctionne enfin quand le contrôle de compte d'utilisateur (UAC) est désactivé.** C'était la cause du faux « REAPER : approbation administrateur refusée » alors que rien n'était refusé. Sur un compte administrateur avec UAC désactivé, le processus tourne déjà avec un jeton administrateur complet, mais l'indicateur Windows `TokenIsElevated` renvoie « faux » — FRABBIT en déduisait qu'il fallait élever et passait par le verbe « runas », lequel échoue en `ERROR_CANCELLED` faute de service d'élévation quand UAC est coupé. FRABBIT décide désormais d'après le **type de jeton** (`TokenElevationType`) et l'appartenance au groupe Administrateurs : il ne demande une élévation que lorsque c'est réellement nécessaire (jeton administrateur filtré sous UAC actif), et lance sinon l'installateur **directement**.
- **REAPER est installé au bon endroit (`C:\Program Files\REAPER (x64)`).** Pour une **première** installation (aucun REAPER détecté), FRABBIT retombait sur un chemin par défaut codé en dur `C:\Program Files\REAPER`, au lieu de l'emplacement 64 bits réel de REAPER `C:\Program Files\REAPER (x64)`. Cela pouvait créer une installation en double, mal placée, à côté du REAPER existant, et affichait « Architecture : unknown ». Le défaut pointe désormais sur `%ProgramFiles%\REAPER (x64)`, là où l'installateur x64 écrit et où FRABBIT re-détecte ensuite. (La détection d'un REAPER **déjà installé**, elle, était déjà correcte.)
- **Robustesse de la vérification et de l'invite d'élévation** : FRABBIT re-vérifie la présence des fichiers pendant quelques secondes après un installateur silencieux (un succès qui rend la main un peu tard n'est plus pris pour un échec), et l'invite « runas » — encore utilisée sous UAC actif — est déclenchée avec initialisation COM, passage au premier plan et attente de fin réelle (`SEE_MASK_NOASYNC`) pour qu'elle s'affiche et soit lue par le lecteur d'écran.

## [1.0.8] - 2026-08-04

### Nouveautés

- **Mise à jour automatique de FRABBIT.** Au démarrage, FRABBIT vérifie s'il existe une version plus récente et, le cas échéant, propose de l'installer dans une boîte de dialogue accessible. Si vous acceptez, FRABBIT télécharge et vérifie la nouvelle version, se remplace lui-même, supprime l'ancienne version, puis redémarre — le tout sans installateur. Si le dossier est en lecture seule, il propose d'ouvrir la page de téléchargement ; en cas d'échec, il reste sur la version actuelle. (Windows ; la version macOS suivra.)

## [1.0.7] - 2026-08-04

### Corrections

- Les messages d'échec d'installation sont désormais **localisés** (français et anglais) au lieu d'afficher du texte technique brut. Notamment, quand l'invite d'approbation administrateur est annulée, le message est clair et traduit (avec, sous Windows, le rappel qu'une cible REAPER portable évite l'élévation).
- Ajout de messages dédiés et clairs pour les échecs spécifiques à macOS (installateur `.pkg` en erreur, application non copiable). Contribution de math65.

## [1.0.6] - 2026-08-03

### Corrections

- Windows : quand l'installateur de REAPER (ou d'un autre produit) est annulé après l'accord administrateur, FRABBIT affiche désormais un message clair (« l'invite d'approbation administrateur a été annulée ou refusée ; relancez et approuvez, ou choisissez une cible REAPER portable ») au lieu de « process failed … exit code 1223 ».
- Cocher un produit déjà installé et à jour affichait « Mise à jour disponible. Vous avez X. La dernière version est X » — incohérent. Ce cas indique maintenant « Réinstaller (version X) », sans comparaison de version inutile.

## [1.0.5] - 2026-08-03

### Modifications

- Dans la liste des produits, **aucun produit n'est plus coché automatiquement** : chaque ligne démarre décochée et c'est à l'utilisateur de cocher exactement ce qu'il veut installer ou mettre à jour. Les produits déjà installés pour lesquels une mise à jour existe affichent toujours « Update available » (mise à jour disponible) même décochés, pour que l'information reste visible.

## [1.0.4] - 2026-08-03

### Corrections

- Windows : les produits qui passent par leur propre installateur (OSARA, SWS, scripts JAWS) ou qui déposent des fichiers échouaient lorsque REAPER avait été installé mais jamais ouvert, car les sous-dossiers de son dossier ressources (`Scripts`, `Effects`, etc.) n'existaient pas encore. FRABBIT crée désormais lui-même la structure de dossiers standard de REAPER avant l'installation, il n'est donc plus nécessaire d'ouvrir REAPER au préalable.

## [1.0.3] - 2026-08-03

### Modifications

- Suppression de l'étape « Don ReaPack » : FRABBIT n'affiche plus d'avis de don ni de lien lors de l'installation de ReaPack. ReaPack s'installe et se met à jour directement, sans étape intermédiaire. L'assistant compte désormais six étapes au lieu de sept.

## [1.0.2] - 2026-07-06

### Corrections

- Windows : FRABBIT ne démarrait pas sur une installation de Windows fraîchement formatée, avec des erreurs « VCRUNTIME140.dll est introuvable » (ainsi que MSVCP140.dll et VCRUNTIME140_1.dll). L'exécutable dépendait du Redistribuable Visual C++, absent d'un Windows neuf. Le runtime C/C++ est désormais lié statiquement : FRABBIT est un exécutable autonome qui démarre sur un Windows vierge, sans aucune installation préalable.

## [1.0.1] - 2026-06-29

Première version macOS de FRABBIT, en plus de Windows.

### Nouveautés

- Prise en charge complète de macOS (universel : Apple Silicon + Intel)
- Trois KeyMaps ReaperAccessible disponibles sous macOS (USA / Français France / Français Canada). REAPER adapte automatiquement les touches de modification à l'affichage (Ctrl devient Cmd, Alt devient Option), si bien que les raccourcis correspondent aux conventions Mac sans configuration.

### Corrections

- macOS : la liste des paquets était inaccessible sous VoiceOver lorsque le bloc de choix du KeyMap était affiché en dessous (la table était comprimée sous une hauteur exploitable). La hauteur de la fenêtre passe de 600 à 720 pixels, avec une taille minimale de garde, pour que VoiceOver puisse de nouveau parcourir la liste.

## [1.0.0] - 2026-06-14

Première version officielle de FRABBIT, l'outil d'installation et de mise à jour de REAPER accessible.

### Fonctionnalités

- Interface graphique en français et en anglais avec détection automatique de la langue
- Installation et mise à jour automatique de :
  - REAPER (l'application elle-même)
  - OSARA (extension d'accessibilité pour les lecteurs d'écran)
  - SWS Extension (actions et outils supplémentaires)
  - ReaPack (gestionnaire de paquets)
  - ReaKontrol (intégration Native Instruments Komplete Kontrol)
  - Scripts JAWS de Snowman pour REAPER
  - FFmpeg (support vidéo amélioré)
  - Surge XT (synthétiseur hybride)
- Choix du KeyMap à l'installation : préserver l'actuel, OSARA, ou ReaperAccessible (USA / Français France / Français Canada)
- Sauvegarde automatique du KeyMap existant dans `KeyMaps/<Variant>ReplacedBackup.ReaperKeyMap` avant remplacement (comportement identique à l'installateur OSARA)
- Copie de référence du KeyMap installé dans `KeyMaps/<Variant>.ReaperKeyMap`
- Page Review (bilan) détaillée avant installation, listant les paquets cochés ET le KeyMap sélectionné
- Page Done (rapport) avec message adaptatif selon le contexte (paquets seuls, KeyMap seul, ou les deux)
- Support de l'option « Ajouter le dépôt ReaPack ReaperAccessible » pour accéder aux scripts et plugins accessibles supplémentaires
- Mise à jour automatique de FRABBIT lui-même (vérification de version au démarrage)

### Plateformes supportées

- Windows x64 (installation standard, mode portable non supporté)
- macOS universel (Intel + Apple Silicon)

### Accessibilité

- Compatible avec les lecteurs d'écran NVDA, JAWS, et Narrator sous Windows
- Compatible avec VoiceOver sous macOS
- Navigation complète au clavier
