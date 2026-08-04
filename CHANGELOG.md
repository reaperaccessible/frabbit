# Changelog

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
