# FRABBIT

**Outil d'installation et de mise à jour de REAPER, OSARA, SWS, ReaPack, ReaKontrol et plus en un clic. Inclut les ressources ReaperAccessible.**

FRABBIT installe et maintient à jour un REAPER pleinement accessible sous Windows et macOS en quelques clics. Au lieu de chercher sur plusieurs pages de téléchargement, copier des fichiers dans les bons dossiers et combattre des installateurs incompatibles avec votre lecteur d'écran, vous lancez un seul programme et il fait le travail.

## Paquets pris en charge

- **REAPER** -- la station de travail audio
- **OSARA** -- l'extension d'accessibilité pour lecteurs d'écran
- **SWS** -- l'extension communautaire populaire
- **ReaPack** -- le gestionnaire de paquets
- **ReaKontrol** -- support claviers Native Instruments Komplete Kontrol
- **Scripts JAWS pour REAPER** *(Windows uniquement, si JAWS est detecte)*
- **FFmpeg** *(Windows uniquement, optionnel)* -- support video ameliore
- **Surge XT** *(optionnel)* -- synthetiseur hybride open source

## Telecharger

Choisissez le fichier correspondant a votre machine. Ces liens pointent toujours vers la derniere version :

- **Windows (Intel/AMD 64 bits)** :
  [frabbit-windows-x86_64.exe](https://github.com/ReaperAccessible/frabbit/releases/latest/download/frabbit-windows-x86_64.exe)
- **Windows (ARM 64 bits)** :
  [frabbit-windows-aarch64.exe](https://github.com/ReaperAccessible/frabbit/releases/latest/download/frabbit-windows-aarch64.exe)
- **macOS (universel -- Apple Silicon + Intel)** :
  [frabbit-macos-universal.app.zip](https://github.com/ReaperAccessible/frabbit/releases/latest/download/frabbit-macos-universal.app.zip)

Pour telecharger une version specifique ou verifier les sommes SHA-256, consultez la [page des versions](https://github.com/ReaperAccessible/frabbit/releases).

## Utilisation

Lancez l'executable telecharge. L'assistant vous guide :

1. **Choisissez une cible REAPER** -- FRABBIT detecte automatiquement les installations existantes ; choisissez "portable" si vous voulez un dossier REAPER autonome.
2. **FRABBIT verifie les dernieres versions** de REAPER et des extensions d'accessibilite.
3. **Choisissez les paquets** que vous voulez installer ou mettre a jour. Des choix par defaut sont deja coches.
4. **Verifiez et installez.** FRABBIT telecharge, verifie et installe tout sans intervention supplementaire.

Une fois termine, vous pouvez lancer REAPER directement depuis l'assistant.

## Site web

https://reaperaccessible.fr

## Licence

FRABBIT est sous double licence **MIT OU Apache-2.0**.

## Compilation depuis les sources

Vous avez besoin d'une toolchain Rust stable recente. Pour la GUI wxDragon sous Windows, vous avez egalement besoin des outils de compilation C++ Visual Studio, d'un `libclang.dll` accessible via `LIBCLANG_PATH`, et de Ninja dans le `PATH`.

```
cargo fmt
cargo test --workspace
.\scripts\build-wxdragon-test.ps1
```
