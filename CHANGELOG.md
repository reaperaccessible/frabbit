# Changelog

## [1.0.9] - 2026-08-04

### Modifications

- **ReaPack n'impose plus ses dépôts par défaut.** Lors d'une **première** installation de ReaPack, FRABBIT écrit désormais d'office deux réglages dans un `reapack.ini` neuf : il empêche ReaPack d'ajouter ses dépôts par défaut (ReaTeam Extensions, ReaTeam Scripts, MPL, etc.) au premier lancement de REAPER, et il désactive l'installation automatique. Résultat : synchroniser ReaPack n'installe plus des milliers de scripts que vous n'avez jamais demandés. Vous gardez le contrôle total — il suffit de parcourir ReaPack et d'installer uniquement ce que vous voulez. Ce comportement est appliqué automatiquement, sans option à cocher : FRABBIT n'impose aucun dépôt. (Le dépôt « ReaPack » de reapack.com, que ReaPack réajoute toujours lui-même pour ses propres mises à jour, reste présent mais inoffensif puisque l'installation automatique est coupée.)
- **Vos réglages ReaPack existants sont respectés.** Si ReaPack a déjà été configuré (vous l'avez déjà lancé, ou vous avez volontairement gardé certains dépôts), une réinstallation ou une mise à jour de ReaPack via FRABBIT **ne touche à rien** : aucun dépôt n'est supprimé et aucun réglage n'est modifié. FRABBIT n'agit que sur une configuration ReaPack neuve.

### Corrections

- **Un échec sur un paquet n'interrompt plus toute l'installation, et le bilan dit enfin la vérité.** Avant, si un paquet échouait (typiquement l'invite administrateur de Windows refusée ou manquée), FRABBIT s'arrêtait net : les paquets suivants n'étaient jamais installés et le message affichait « Rien n'a été installé » — trompeur, car certains paquets l'étaient déjà. Désormais, chaque paquet est traité indépendamment : un échec est signalé **sur ce paquet précis, avec sa raison**, et les autres paquets continuent de s'installer. La page de fin liste le résultat réel de chaque paquet (« installé », « échec : approbation administrateur refusée », etc.), affiche en tête « Terminé avec des erreurs : N paquet(s) non installés », et un compteur d'échecs. Fini de croire que tout est installé alors que non.
- **L'installation standard fonctionne enfin quand UAC est désactivé.** Si FRABBIT tourne déjà avec les droits administrateur (parce que le contrôle de compte d'utilisateur — UAC — est désactivé et que le compte est administrateur, ou parce que FRABBIT a été lancé « en tant qu'administrateur »), il lance désormais les installateurs **directement**, sans passer par le verbe « runas ». C'était la vraie cause de l'échec de l'installation de REAPER dans `C:\Program Files` : avec UAC désactivé, il n'existe plus de service d'élévation à solliciter, donc « runas » revenait en `ERROR_CANCELLED` (1223) et l'installation semblait « annulée » alors que l'utilisateur n'avait rien vu ni refusé. FRABBIT n'élève maintenant que lorsque c'est réellement nécessaire.
- **Fiabilité de l'invite administrateur (UAC) améliorée** (quand elle est bien activée). L'invite d'élévation était déclenchée depuis un thread de travail sans initialisation COM ni demande de premier plan, ce qui pouvait empêcher la fenêtre d'approbation de s'afficher ou de recevoir le focus (donc d'être lue par le lecteur d'écran). FRABBIT initialise désormais COM sur le thread, autorise la fenêtre à passer au premier plan, et attend la fin réelle de l'opération (`SEE_MASK_NOASYNC`).

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
