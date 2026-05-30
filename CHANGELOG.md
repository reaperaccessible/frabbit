# Changelog

## [1.5.0] - 2026-05-30

- Fix: tolere les fichiers reaper-kb.ini non-UTF-8 (Windows-1252)

## [1.4.0] - 2026-05-30

- KeyMap decoupled: installable independently of OSARA update
- Resume affiche le KeyMap selectionne avec son nom

## [1.3.0] - 2026-05-30

- Textes ameliores: actions claires, review simplifie, dropdown KeyMaps
- Dropdown keymap nomme 'KeyMaps' avec labels courts
- Resume affiche clairement le KeyMap selectionne

## [1.2.0] - 2026-05-30

- Interface: dropdown de selection du keymap (Preserver, OSARA, ReaperAccessible x3)
- Suppression du support Windows ARM64 (instable)

## [1.1.0] - 2026-05-30

- Ajout des keymaps ReaperAccessible (Windows) : USA, Francais France, Francais Canada
- Choix du keymap a l'installation : Preserver, OSARA (USA), ou ReaperAccessible (3 variantes)
- Les keymaps sont embarques dans le binaire, pas de telechargement supplementaire
- Sauvegarde automatique du reaper-kb.ini avant remplacement

## [1.0.0] - 2026-05-30

Premiere version de FRABBIT, basee sur le code source de RABBIT (Timtam/rabbit).

- Interface en francais et en anglais avec detection automatique de la langue
- Installation et mise a jour de REAPER, OSARA, SWS, ReaPack, ReaKontrol, scripts JAWS, FFmpeg, Surge XT
- Depots ReaPack ReaperAccessible selon la langue (francais ou anglais)
- Support Windows x64, Windows ARM64 et macOS (universel)
- Mise a jour automatique de FRABBIT
