# Changelog

## [1.0.0] - 2026-06-14

Premiere version officielle de FRABBIT, l'outil d'installation et de mise a jour de REAPER accessible.

### Fonctionnalites

- Interface graphique en francais et en anglais avec detection automatique de la langue
- Installation et mise a jour automatique de :
  - REAPER (l'application elle-meme)
  - OSARA (extension d'accessibilite pour les lecteurs d'ecran)
  - SWS Extension (actions et outils supplementaires)
  - ReaPack (gestionnaire de paquets)
  - ReaKontrol (integration Native Instruments Komplete Kontrol)
  - Scripts JAWS de Snowman pour REAPER
  - FFmpeg (support video ameliore)
  - Surge XT (synthetiseur hybride)
- Choix du KeyMap a l'installation : Preserver l'actuel, OSARA, ou ReaperAccessible (USA / Francais France / Francais Canada)
- Sauvegarde automatique du KeyMap existant dans `KeyMaps/<Variant>ReplacedBackup.ReaperKeyMap` avant remplacement (comportement identique a l'installateur OSARA)
- Copie de reference du KeyMap installe dans `KeyMaps/<Variant>.ReaperKeyMap`
- Page Review (bilan) detaillee avant installation, listant les paquets coches ET le KeyMap selectionne
- Page Done (rapport) avec message adaptatif selon le contexte (paquets seuls, KeyMap seul, ou les deux)
- Support de l'option "Ajouter le depot ReaPack ReaperAccessible" pour acceder aux scripts et plugins accessibles supplementaires
- Mise a jour automatique de FRABBIT lui-meme (verification de version au demarrage)

### Plateformes supportees

- Windows x64 (installation standard, mode portable non supporte)
- macOS universel (Intel + Apple Silicon)

### Accessibilite

- Compatible avec les lecteurs d'ecran NVDA, JAWS, et Narrator sous Windows
- Compatible avec VoiceOver sous macOS
- Navigation complete au clavier
