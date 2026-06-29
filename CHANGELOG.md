# Changelog

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
