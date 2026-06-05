# Changelog

## [1.14.1] - 2026-06-05

- Fix: les installateurs configures via `installer_silent_args` sont maintenant promus PlannedUnattended -> AvailableUnattended et reellement executes par FRABBIT (CSI etait telecharge mais jamais lance en v1.14.0)

## [1.14.0] - 2026-06-05

- CSI: migration vers le nouvel installeur Inno Setup CSIInstaller.exe (v7.0.1-test)
- FRABBIT télécharge l'installeur et le lance en mode silencieux (`/VERYSILENT /SUPPRESSMSGBOXES /NORESTART`); l'installeur gère l'extraction, le dépôt ReaPack et le runtime VC++
- Nouveau détecteur manifeste `inno_setup_registry`: lit `DisplayVersion` depuis la clé Uninstall `<AppId>_is1`
- Nouveau champ manifeste `inno_setup_app_id`: GUID AppId Inno Setup (sans suffixe `_is1`)
- Nouveau champ manifeste `installer_silent_args`: arguments silencieux pour l'installeur vendeur, configurables par paquet
- Suppression du pipeline post-install générique (zip routes, dépôt ReaPack, fichier de version): rendu obsolète par l'installeur Inno Setup
- Suppression des champs manifeste morts: `compare_by_file_mtime`, `version_from_github_published_at`, `post_install_zip_routes`, `post_install_reapack_repo`, `post_install_version_file`, `version_file_documents_relative`
- Suppression des modules morts: `generic_post_install.rs`, `date_version.rs`, `csi.rs`
- Suppression du dossier `Contents/CSI For Behringer X-Touch Universal/` (1700+ fichiers): plus livré par FRABBIT

## [1.13.0] - 2026-05-31

- CSI: detection par date de modification du DLL
- Comparaison: mtime du DLL local vs published_at du release GitHub
- Nouveau champ manifeste: compare_by_file_mtime
- Nouveau champ manifeste: version_from_github_published_at
- Nouveau module date_version.rs pour conversion timestamps en YYYY.MM.DD

## [1.12.0] - 2026-05-31

- Simplification majeure: -2479 lignes de code
- Paquets declaratifs: ajouter un paquet simple = JSON + locales, zero Rust
- Self-update simplifie: verification de version uniquement, plus de staging/apply
- Rollback reduit au minimum (backup manifest)
- Signature.rs supprime
- Module native_tree_checkboxes extrait de wx_app.rs
- 16 cles de locale mortes supprimees

## [1.11.0] - 2026-05-31

- CSI integre dans le systeme de paquets complet (comme REAPER, OSARA, SWS)
- Detection automatique via .frabbit-version (CsiVersionFile)
- Version disponible via GitHub API (CsiGithubRelease)
- Resolution d'artefact automatique (CsiGithubReleaseZip)
- Installation: DLL extraite par le pipeline standard, post-install pour CSI/ et Documents/
- Suppression de l'ancien install_csi: bool, checkbox separee et progress events CSI

## [1.10.0] - 2026-05-30

- Integration CSI complete: progression en temps reel, resume/review, rapport Done
- Fichier de version .frabbit-version ecrit apres installation CSI
- Evenements de progression CsiDownloadStarted/CsiDownloadCompleted/CsiInstallCompleted
- Page Review affiche la section CSI quand la case est cochee
- Rapport de fin (Done) inclut le statut CSI dans les details
- CLI: ajout du flag --install-csi pour la commande Setup
- Locales FR/EN: ajout des textes review, progression et resume CSI

## [1.9.0] - 2026-05-30

- Ajout case a cocher CSI (Control Surface Integrator) pour X-Touch Universal
- Installation CSI: DLL + config + dossier Documents
- Depot ReaPack CSI ajoute automatiquement

## [1.8.0] - 2026-05-30

- Release complete: Windows x64 + macOS universel

## [1.7.0] - 2026-05-30

- Le KeyMap ReaperAccessible est place dans le dossier KeyMaps/ ET applique dans reaper-kb.ini

## [1.6.0] - 2026-05-30

- Fix: le KeyMap est maintenant applique meme si OSARA n est pas coche

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
