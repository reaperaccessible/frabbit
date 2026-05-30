app-title = Outil d'installation et de mise à jour de REAPER accessible
app-short-name = FRABBIT

common-yes = oui
common-no = non

action-install = Sera installé
action-update = Mise à jour disponible
action-keep = Aucune mise à jour disponible

package-reaper = REAPER
package-osara = OSARA
package-sws = Extension SWS
package-reapack = ReaPack
package-reakontrol = ReaKontrol
package-jaws-scripts = Scripts JAWS de Snowman pour REAPER
package-ffmpeg = FFmpeg pour le support vidéo amélioré
package-surge-xt = Surge XT

package-reaper-description = REAPER est la station de travail audio numérique sur laquelle tout le reste repose. FRABBIT peut installer ou mettre à jour REAPER pour vous.
package-osara-description = OSARA est l'extension d'accessibilité open source qui rend REAPER utilisable avec les lecteurs d'écran. NVDA, JAWS et Narrator sous Windows, VoiceOver sous macOS sont tous largement utilisés, d'autres lecteurs d'écran Windows peuvent également fonctionner. Installez OSARA si vous utilisez un lecteur d'écran avec REAPER.
package-sws-description = L'extension SWS est un ensemble d'actions, de scripts et d'outils supplémentaires développés par la communauté qui étendent les fonctionnalités de REAPER. Pour une configuration REAPER la plus accessible possible, que vous soyez sous Windows ou Mac, vous devriez installer SWS aux côtés d'OSARA.
package-reapack-description = ReaPack est un gestionnaire de paquets open source. Il permet de rechercher, installer, suivre et mettre à jour des scripts et extensions tiers directement depuis REAPER. Installez-le si vous souhaitez utiliser des scripts partagés par la communauté REAPER.
package-reakontrol-description = ReaKontrol fournit une intégration open source pour les claviers Native Instruments Komplete Kontrol. Installez-le si vous possédez un clavier série S MK2, série A, M-32 ou Kontrol MK3.
package-jaws-scripts-description = Les scripts de Snowman améliorent la façon dont JAWS gère les différentes fenêtres de REAPER, tout en offrant un support Braille étendu et de nombreuses autres fonctionnalités. Notez que ces scripts sont conçus pour être utilisés avec OSARA, ils ne le remplacent pas. Pour une accessibilité optimale avec JAWS, vous devriez installer les deux.
package-ffmpeg-description = Les bibliothèques partagées de FFmpeg permettent au décodeur vidéo de REAPER d'importer et de lire les formats vidéo et audio courants. FRABBIT installe le dossier bin du build GPL-shared de BtbN dans UserPlugins ; le niveau de correctif n'est pas récupérable à partir des noms de fichiers DLL seuls, donc les FFmpeg installés en externe sont signalés avec un indicateur `<majeur>.0.0`.
package-surge-xt-description = Surge XT est un synthétiseur hybride gratuit et open source. FRABBIT exécute l'installateur du fabricant pour vous — il installe les formats VST3, CLAP, AU (macOS uniquement) et autonome à l'échelle du système pour que REAPER et d'autres DAW puissent charger Surge XT. Suit le canal nightly car la dernière version stable (1.3.4) date d'août 2024 et le projet publie désormais principalement via les nightlies. Installations REAPER standard uniquement : les données d'usine résident en dehors de tout dossier REAPER portable.

# $reason est l'une des chaînes localisées "wizard-package-row-unavailable-*"
# expliquant *pourquoi* la ligne est indisponible.
wizard-package-row-unavailable-suffix = (non disponible : { $reason })
wizard-package-row-unavailable-portable = cible REAPER portable

detect-installed = Installé
detect-not-installed = Non installé
detect-version-unknown = Version inconnue
detect-source-receipt = Reçu FRABBIT
detect-source-files = Présence de fichier dans UserPlugins
detect-source-reapack-registry = Registre ReaPack

# $package est le nom localisé du paquet.
status-package-installed = { $package } installé

wizard-step-target = Cible
wizard-step-version-check = Vérification des versions
wizard-step-packages = Paquets
wizard-step-reapack-acknowledgement = Don ReaPack
wizard-step-review = Vérification
wizard-step-progress = Progression
wizard-step-done = Terminé

wizard-button-back = Précédent
wizard-button-back-mnemonic = P
wizard-button-next = Suivant
wizard-button-next-mnemonic = S
wizard-button-install = Installer
wizard-button-install-mnemonic = I
wizard-button-close = Fermer
wizard-button-close-mnemonic = F

wizard-target-heading = Choisir une tâche
wizard-target-language-label = Langue
wizard-target-language-restart-note = Le changement de langue redémarre FRABBIT pour que la nouvelle langue prenne effet.
wizard-locale-name-fr-FR = Français (France)
wizard-locale-name-en-US = Anglais (États-Unis)
wizard-target-choice-label = Chemin d'installation
wizard-target-details-label = Détails de la cible
wizard-target-empty = Aucune cible d'installation REAPER n'est sélectionnée.
wizard-target-portable-choice = Créer ou mettre à jour une version portable de REAPER
wizard-target-portable-folder-label = Dossier portable
wizard-target-portable-folder-message = Choisissez un dossier REAPER portable si vous en avez déjà un, ou un dossier vide pour créer une nouvelle version portable.
wizard-target-portable-folder-browse-label = Parcourir…
wizard-target-portable-pending-details = Utilisez le bouton Parcourir pour définir l'emplacement d'une version portable existante, ou pour choisir un dossier vide si vous souhaitez créer une nouvelle version portable de REAPER.
wizard-target-custom-portable-label = Dossier REAPER portable
wizard-target-custom-portable-app-path-label = Chemin de l'application REAPER
wizard-target-custom-portable-path-label = Chemin des ressources portables
wizard-target-custom-portable-version-label = Version de REAPER
wizard-target-custom-portable-writable-label = Accessible en écriture
wizard-target-custom-portable-note = FRABBIT créera le chemin des ressources REAPER ici s'il est manquant.

# $version est la version de REAPER et $path est le chemin des ressources.
wizard-target-row = REAPER { $version } dans { $path }

# $app_path est le chemin de l'application REAPER, $path est le chemin des ressources,
# $version est la version de REAPER, et $writable est oui/non.
wizard-target-details = Chemin d'installation REAPER : { $app_path }
    Version : { $version }
    Chemin des ressources : { $path }
    Accessible en écriture : { $writable }

wizard-packages-heading = Choisir les paquets
wizard-packages-list-label = Paquets à installer ou à mettre à jour
wizard-packages-tree-group-label = Paquets
wizard-configuration-tree-group-label = Configuration
# $package est le nom localisé du paquet dont l'étape de configuration dépend.
wizard-configuration-row-unavailable = Non disponible : nécessite que { $package } soit installé.
wizard-configuration-row-already-applied = Déjà appliqué sur cette cible REAPER.
# $reason est l'une des chaînes "wizard-configuration-row-status-*" ci-dessous.
wizard-configuration-row-summary-suffix = ({ $reason })
# $package est le nom localisé du paquet dépendant.
wizard-configuration-row-status-requires = nécessite { $package }
wizard-configuration-row-status-already-applied = déjà appliqué
config-reapack-reaper-accessibility-name = Ajouter le dépôt ReaPack ReaperAccessible
config-reapack-reaper-accessibility-description = Ajoute le dépôt ReaPack ReaperAccessible. Une fois ajouté, allez dans le menu Extensions, ReaPack, Parcourir les paquets pour obtenir des scripts et plugins accessibles supplémentaires.

wizard-reapack-ack-heading = Avis de don ReaPack
wizard-reapack-ack-body = ReaPack est un logiciel libre publié sous licence LGPL. Son auteur Christian Fillion accepte des dons facultatifs pour soutenir le développement continu. Christian maintient également les extensions SWS et a contribué du code spécifiquement pour améliorer la compatibilité avec OSARA par le passé. Tout soutien que vous pouvez apporter est bien mérité.
wizard-reapack-ack-link-label = Ouvrir la page de don ReaPack
wizard-reapack-ack-confirm-label = Passer le don cette fois, juste installer ou mettre à jour ReaPack
cli-reapack-ack-prompt-summary = ReaPack est un logiciel libre (LGPL). Les dons à son auteur Christian Fillion sont acceptés sur https://reapack.com/donate pour soutenir le développement continu.
cli-reapack-ack-flag-required = ReaPack fait partie de ce plan mais l'accusé de réception du don est manquant. Relancez avec --accept-reapack-donation-notice pour confirmer que vous avez lu https://reapack.com/donate et que vous souhaitez que FRABBIT installe ou mette à jour ReaPack.

wizard-version-check-heading = Vérification des dernières versions
wizard-version-check-status-pending = Préparation de la vérification des versions…
# $package est le nom localisé du paquet.
wizard-version-check-status-checking = Vérification de { $package }…
# $error_count est le nombre de vérifications échouées.
wizard-version-check-status-error = { $error_count } vérification(s) de version échouée(s). Utilisez Précédent pour essayer une autre cible, ou fermez FRABBIT.
wizard-version-check-progress-label = Progression
wizard-version-check-error-heading = Vérifications échouées
# $package est le nom localisé du paquet ; $message est le message d'erreur.
wizard-version-check-error-line = { $package } : { $message }
wizard-package-details-label = Détails du paquet
wizard-packages-keymap-heading = KeyMaps
wizard-packages-keymap-replace-label = KeyMaps
wizard-packages-keymap-unavailable-note = Sélectionnez OSARA pour configurer le comportement des raccourcis clavier.
wizard-packages-keymap-preserve-note = Pour les utilisateurs avancés : vos raccourcis clavier actuels seront préservés. FRABBIT ne touchera pas reaper-kb.ini, vous devrez gérer manuellement la mise à jour avec les derniers ajouts du KeyMaps.
wizard-packages-keymap-replace-note = FRABBIT sauvegardera une copie de votre fichier reaper-kb.ini actuel, puis le remplacera par le KeyMap sélectionné.
wizard-package-details-handling-prefix = Traitement
wizard-package-handling-automatic = FRABBIT peut installer ce paquet directement.
wizard-package-handling-unattended = FRABBIT peut installer ce paquet sans intervention, y compris en lançant son installateur si nécessaire.
wizard-package-handling-planned = FRABBIT est conçu pour exécuter l'installateur ou la routine d'installation de ce paquet lui-même et terminer l'installation sans intervention, mais cette version rapporte les étapes au lieu de les exécuter.
wizard-package-handling-manual = FRABBIT téléchargera ce paquet et rapportera les étapes manuelles après l'exécution.
wizard-package-handling-unavailable = Ce paquet n'est pas disponible pour la plateforme ou l'architecture sélectionnée.

# $package est le nom du paquet, $action est l'action planifiée, $installed est la version installée, $available est la version disponible.
wizard-package-row = { $package } : { $action }. Vous avez { $installed }. La dernière version est { $available }

wizard-review-heading = Vérifiez vos choix avant confirmation
wizard-review-target-prefix = Cible
wizard-review-package-heading = Paquets sélectionnés
wizard-review-keymap-heading = KeyMaps
wizard-review-keymap-preserve = Aucun KeyMap ne sera installé. Vos raccourcis clavier actuels seront préservés.
wizard-review-keymap-replace = Le KeyMap sélectionné sera installé. Vos raccourcis clavier actuels seront sauvegardés avant remplacement.
wizard-review-notes-heading = Notes
wizard-review-preflight-prefix = Installation impossible pour le moment

# $path est le chemin des ressources REAPER sélectionné.
wizard-review-target = Cible : { $path }
wizard-review-no-target = Aucune cible sélectionnée.
wizard-review-no-package = Aucun paquet sélectionné.

# $package est le nom du paquet et $action est l'action planifiée.
wizard-review-package = { $package } : { $action }

wizard-progress-heading = Progression de l'installation
wizard-progress-status-idle = Prêt à installer.
wizard-progress-status-running = Installation des paquets sélectionnés. Cela peut prendre quelques minutes.
wizard-progress-details-label = Détails de la progression
wizard-progress-details-idle = Aucune installation en cours.
wizard-progress-details-starting = Démarrage de l'opération d'installation.
wizard-progress-details-cache-prefix = Cache

# $package est le nom localisé du paquet.
wizard-progress-status-downloading = Téléchargement de { $package }…
# $downloaded et $total sont des tailles en octets lisibles.
wizard-progress-status-downloading-with-bytes = Téléchargement de { $package }… { $downloaded } / { $total }
wizard-progress-status-installing = Installation de { $package }…
# $step est le nom de l'étape de configuration.
wizard-progress-status-configuring = Application de l'étape de configuration : { $step }

wizard-progress-log-download-started = Téléchargement de { $package }…
wizard-progress-log-download-completed = { $package } téléchargé.
wizard-progress-log-install-started = Installation de { $package }…
wizard-progress-log-install-completed = { $package } installé.
wizard-progress-log-configuration-started = Application de { $step }…
wizard-progress-log-configuration-completed = { $step } appliqué.

wizard-done-heading = Terminé
wizard-done-status-idle = Aucune installation n'a été lancée depuis cette fenêtre.
wizard-done-status-success = FRABBIT a terminé son travail ! Consultez les détails ci-dessous.
wizard-done-status-error = L'installation a échoué. Consultez l'erreur ci-dessous.
wizard-done-status-no-packages = Aucun paquet n'a été sélectionné pour l'installation ou la mise à jour.
wizard-done-show-details = Afficher les détails
wizard-done-launch-reaper = Ouvrir REAPER et fermer FRABBIT
wizard-done-launch-reaper-mnemonic = O
wizard-done-open-resource = Ouvrir le dossier des ressources (uniquement pour la maintenance manuelle avancée)
wizard-done-open-resource-mnemonic = R
wizard-done-no-reaper-app = Aucune application REAPER lançable n'est connue pour cette cible.
wizard-done-launch-reaper-error-prefix = REAPER n'a pas pu être lancé
wizard-done-open-resource-error-prefix = Le dossier des ressources n'a pas pu être ouvert
wizard-done-self-update-apply-running = Application de la mise à jour de FRABBIT…
wizard-done-self-update-error-prefix = La mise à jour automatique de FRABBIT a échoué
wizard-done-self-update-relaunch-prefix = FRABBIT relancé
wizard-self-update-status-checking = Recherche de mises à jour de FRABBIT…

wizard-self-update-prompt-title = Mise à jour de FRABBIT disponible
wizard-self-update-prompt-body = FRABBIT { $latest } est disponible. Vous avez actuellement { $current }. Mettre à jour maintenant ? FRABBIT redémarrera une fois la mise à jour terminée.

# $current est la version actuelle, $latest est la version proposée, $channel est le canal.
self-update-status-update-available = Mise à jour de FRABBIT disponible : { $current } → { $latest } (canal { $channel }). Relancez FRABBIT pour être à nouveau invité.
self-update-status-up-to-date = FRABBIT est à jour (version actuelle { $current }, canal { $channel }).

# $version est la version ciblée mais non écrite.
self-update-apply-no-files-replaced = La mise à jour automatique n'a remplacé aucun fichier (version cible { $version }).
# $count est le nombre de fichiers remplacés, $root est le répertoire d'installation,
# $version est la nouvelle version de FRABBIT.
self-update-apply-replaced-summary = { $count } fichier(s) remplacé(s) sous { $root } ; relancez FRABBIT pour utiliser { $version }.

# $signed / $unsigned sont les nombres de binaires signés/non signés.
self-update-apply-signature-summary-signed-only = Vérification de signature : { $signed } signé(s).
self-update-apply-signature-summary-unsigned-only = Vérification de signature : { $unsigned } non signé(s).
self-update-apply-signature-summary-mixed = Vérification de signature : { $signed } signé(s), { $unsigned } non signé(s).

# $pid est l'identifiant du processus de l'autre installation FRABBIT.
self-update-lock-blocking = Une autre installation de FRABBIT est en cours (PID { $pid }). L'application est en pause jusqu'à sa fin.

wizard-summary-target = Cible : { $path }
wizard-summary-portable = Cible portable : { $value }
wizard-summary-dry-run = Simulation : { $value }
wizard-summary-packages-selected = Paquets sélectionnés : { $packages }
wizard-summary-cache = Cache : { $path }
wizard-summary-planned-app = Chemin d'application prévu : { $path }
wizard-summary-error = Erreur : { $message }
wizard-summary-resource-items-created = Éléments de ressource créés : { $count }
wizard-summary-packages-installed-or-checked = Paquets installés ou vérifiés : { $count }
wizard-summary-packages-current = Paquets déjà à jour : { $count }
wizard-summary-packages-manual = Paquets nécessitant une attention manuelle : { $count }
wizard-summary-backup-files-created = Fichiers de sauvegarde créés : { $count }
wizard-summary-backup-file = Fichier de sauvegarde : { $path }
wizard-summary-receipt-backup = Sauvegarde du reçu : { $path }
wizard-summary-backup-manifest = Manifeste de sauvegarde : { $path }
wizard-summary-package-message = { $package } : { $message }
# $action est l'une des étiquettes "action-*" localisées.
wizard-summary-package-plan-action =   Action planifiée : { $action }
# $status est l'une des étiquettes "status-*" localisées.
wizard-summary-package-status =   Statut : { $status }
# $version est la version que FRABBIT vient d'installer.
wizard-summary-package-installed-version =   Version installée : { $version }
# $architecture est l'architecture REAPER détectée.
wizard-summary-architecture = Architecture : { $architecture }
status-installed-or-checked = Installé ou vérifié
status-planned-unattended = Planifié sans intervention
status-deferred-unattended = Reporté sans intervention
status-skipped-current = Ignoré (déjà à jour)

package-status-extension-binary-installed = Binaire d'extension unique géré par l'installateur FRABBIT.
# $installed est la version sur disque ; $available est la dernière version disponible.
package-status-skipped-current = La version installée { $installed } est à jour ou plus récente que la version disponible { $available }.
# $automation est l'un des libellés "package-automation-*".
package-status-dry-run-would-run-unattended = Simulation : FRABBIT téléchargerait et exécuterait ce { $automation } sans intervention.
package-status-deferred-unattended-staged = Cette version n'a pas encore implémenté le chemin d'exécution sans intervention pour { $automation }. FRABBIT a placé l'artefact dans le cache mais ne l'a pas exécuté.
package-status-deferred-unattended-not-staged = Cette version n'a pas encore implémenté le chemin d'exécution sans intervention pour { $automation }. FRABBIT n'a ni téléchargé ni exécuté l'artefact.
package-status-unattended-installed = FRABBIT a exécuté l'installateur sans intervention, vérifié les chemins cibles attendus et mis à jour le reçu FRABBIT.
package-status-osara-unattended-keymap-backed-up = FRABBIT a exécuté l'installateur sans intervention, sauvegardé reaper-kb.ini, appliqué le remplacement des raccourcis OSARA et mis à jour le reçu FRABBIT.
package-status-osara-unattended-keymap-replaced = FRABBIT a exécuté l'installateur sans intervention, appliqué le remplacement des raccourcis OSARA et mis à jour le reçu FRABBIT.

package-automation-installer = installateur du fabricant
package-automation-archive = extraction d'archive
package-automation-disk-image = installation depuis une image disque
package-automation-extension-binary = installation directe de fichier

# $name est le nom lisible du dépôt distant ; $url est l'URL du fichier XML d'index.
config-message-reapack-remote-already-present = Le dépôt distant ReaPack { $name } ({ $url }) est déjà configuré dans reapack.ini.
config-message-reapack-remote-added = Le dépôt distant ReaPack { $name } ({ $url }) a été ajouté à reapack.ini.
config-message-reapack-remote-created-file = reapack.ini a été créé avec le dépôt distant ReaPack { $name } ({ $url }). ReaPack ajoutera ses dépôts par défaut au prochain lancement de REAPER.
config-message-reapack-remote-dry-run = Ajouterait le dépôt distant ReaPack { $name } ({ $url }) à reapack.ini.
# $step est l'identifiant de l'étape de configuration.
config-message-skipped = L'étape de configuration { $step } n'a pas été sélectionnée.
# $step est l'identifiant de l'étape ; $dependency est l'identifiant du paquet dépendant.
config-message-skipped-dependency-missing = L'étape de configuration { $step } a été ignorée car le paquet dépendant { $dependency } n'est pas installé et ne fait pas partie de ce plan.
config-message-applied-no-op = Étape de configuration appliquée sans modification.

wizard-summary-configuration-message = { $step } : { $message }
wizard-summary-configuration-status =   Statut : { $status }

config-status-applied = Appliqué
config-status-skipped = Ignoré
config-status-skipped-dependency-missing = Ignoré (dépendance manquante)
config-status-dry-run = Simulation
wizard-summary-planned-execution-title = Exécution sans intervention planifiée :
wizard-summary-planned-execution-runner =   Exécuteur : { $runner }
wizard-summary-planned-execution-artifact =   Artefact : { $artifact }
wizard-summary-planned-execution-program =   Programme : { $program }
wizard-summary-planned-execution-arguments =   Arguments : { $arguments }
wizard-summary-planned-execution-working-directory =   Répertoire de travail : { $path }
wizard-summary-planned-execution-verify =   Vérifier : { $path }
wizard-summary-manual-title = { $title } :
wizard-summary-manual-step =   { $step }
wizard-summary-manual-note =   Note : { $note }
wizard-summary-status-finished = Terminé. { $installed } élément(s) de paquet installé(s) ou vérifié(s) ; { $manual } nécessitent une attention manuelle.

wizard-planned-runner-launch-installer = Lancer l'installateur
wizard-planned-runner-extract-archive = Extraire l'archive et exécuter l'installateur contenu
wizard-planned-runner-extract-archive-copy-osara = Extraire l'archive et copier les fichiers d'installation OSARA
wizard-planned-runner-mount-disk-image = Monter l'image disque et exécuter l'installateur contenu
wizard-planned-runner-mount-disk-image-copy-app = Monter l'image disque et copier le paquet d'application contenu
wizard-planned-runner-mount-disk-image-run-pkg = Monter l'image disque et exécuter l'installateur pkg contenu

wizard-packages-csi-label = Installer CSI (Control Surface Integrator) pour Behringer X-Touch Universal
wizard-packages-csi-note = CSI permet l'intégration de surfaces de contrôle matérielles avec REAPER. Nécessite ReaPack.
