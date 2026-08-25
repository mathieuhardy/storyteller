# Screens — Écrans & endpoints

Les surfaces de l'application, chacune avec son rôle, sa structure et les **endpoints** consommés
([api.md](../api.md)). L'UI ne fait que consommer l'API. Maquettes de référence dans [`mockups/`](mockups/)
pour les cinq écrans du MVP (§1-5) ; §6 (v2/M7) n'a pas de maquette dédiée, même traitement que
Recherche (M5) — spec ici, implémentation directe contre l'API existante.

## Mapping écran → endpoints (vue d'ensemble)

| Écran | Endpoints principaux |
|---|---|
| Lanceur | *(local)* ouverture de dossier ; `GET /version` (compat) par projet |
| Tableau de bord | `GET /project` (racine + `stats.by_type`), `GET /entities?sort=-updated`, `GET /stubs`, `GET /events` |
| Fiche entrée | `GET /entities/{slug}?include=backlinks`, `GET /entities/{slug}/links`, `GET /types/{type}` |
| Vue liste/table | `GET /entities?type=…&tag=…&<field>=…&sort=…&page=…`, `GET /types/{type}` (colonnes) |
| Éditeur | `GET /types/{type}` (formulaire), `POST`/`PATCH /entities`, `POST /entities/{slug}/rename` |
| Chantier | `GET /stubs`, `GET /entities/{slug}/links` (ambigus), `POST /entities` (création), réécriture de lien |
| Galerie médias | `GET /assets`, `POST /assets` (upload) |

Toutes les vues écoutent `GET /events` pour se rafraîchir ([states.md](states.md) §6).

---

## 1. Accueil / tableau de bord

**Maquette :** [accueil-dashboard.html](mockups/accueil-dashboard.html)

Deux temps (voir [layout.md](layout.md)) :

- **Lanceur** — ouvrir un **dossier** de projet, en créer un, **projets récents** (nom, chemin, nb
  d'entrées, dernière ouverture). Local-first, aucun compte.
- **Tableau de bord** — vue d'ensemble une fois le projet ouvert :
  - **Hero** = l'entrée racine `project.md` : couverture, `title`, `logline` en exergue, `status`
    (pill), `genres` (chips). Source : `GET /project` → `entry`.
  - **Stats** : total, chapitres, **chantier** (stubs), entrées **à vérifier** (diagnostics). Source :
    `stats.by_type`, `GET /stubs`.
  - **Modifié récemment** : `GET /entities?sort=-updated&per_page=5` (drapeau ▲ si `has_errors`).
  - **Répartition par type** : barres depuis `stats.by_type`.
  - **Chantier** (aperçu) et **Santé de l'index** (watcher/SSE, dernier réindex).

## 2. Fiche entrée

**Maquette :** [fiche-entree.html](mockups/fiche-entree.html)

Lecture d'une entrée dans la colonne `main` (780px) ; rétroliens dans le **panneau escamotable**.

- **En-tête** : `cover`/`portrait`, `type` (badge), `title`, `aliases`, `tags`, statut. Actions : Éditer,
  Renommer, Ouvrir le .md.
- **Bandeau diagnostic** éventuel (asset introuvable, liens non résolus…).
- **Attributs** : frontmatter typé rendu par `kind` (texte, nombre, **liens** avec état, link-list, enum…),
  ordre et labels depuis `GET /types/{type}`. Sous-bloc **« Champs conservés »** pour les clés inconnues.
- **Corps** : markdown rendu, **wikilinks cliquables** (états résolu/stub/ambigu), embeds d'assets.
- **Panneau (rail)** : **Mentionné dans** (`GET …/backlinks`, groupé par source, avec type + champ/contexte
  + extrait), **Liens sortants** (`GET …/links`, statut par lien), **Métadonnées** (identité, chemin,
  `created`/`updated`).

> **Blocs dérivés** : certaines relations inverses (ex. **Membres** d'une faction) se **lisent** depuis les
> backlinks `field:factions` sans être stockées — [linking.md](../linking.md) §5.

Source principale : `GET /entities/{slug}?include=backlinks` (relecture depuis le fichier, fraîche même
après édition externe).

## 3. Vue liste/table

**Maquette :** [vue-liste-table.html](mockups/vue-liste-table.html)

Navigation d'un **type**. Bascule **Table / Liste** :

- **Table** : colonnes = **champs du type** (`GET /types/{type}`), en-têtes cliquables pour trier
  (`sort=` / `-`), **colonne d'état** (diagnostics), cellules de **liens** cliquables. `overflow-x` si large.
- **Liste** : cartes avec `excerpt` (de `EntrySummary`), tags, drapeau d'erreur.
- **Barre d'outils** : tri (menu), **« Ajouter un filtre »** (par champ), réglage des colonnes.
- **Filtres actifs** en chips retirables → correspondent aux paramètres d'URL :
  `type=` (répétable, OR), `tag=` (répétable, AND), `<field>=<valeur>` (égalité, AND).
- **Pagination** : `page` / `per_page` (défaut 50, max 200), `total` affiché.

Source : `GET /entities` (servi depuis l'index) → `Page<EntrySummary>`.

## 4. Éditeur d'entrée

**Maquette :** [editeur-entree.html](mockups/editeur-entree.html)

Création et édition. Continuité visuelle avec la fiche.

- **En-tête éditable** : dépôt de couverture, `title` inline, rappel **identité = fichier** (le slug ne
  change que par **Renommer**). `aliases`/`tags` en tokens.
- **Attributs** : formulaire **généré depuis le schéma de type** (`GET /types/{type}` : `label`, `kind`,
  `required`, `enum_values`, `link_targets`). Contrôles par `kind` — voir [components.md](components.md) §4.
  - **Liens** : tokens + **autocomplétion** (correspondances title/alias) avec option **« Créer … »** inline.
  - **Champs conservés** en lecture seule (clés inconnues préservées).
- **Corps** : éditeur markdown + barre d'outils (dont `[[ ]]`) + bascule **Éditer / Aperçu**.
- **Barre d'enregistrement** collante : « non enregistré », rappel non-destructif, ⌘S.

Écritures :

| Action | Endpoint | Notes |
|---|---|---|
| Créer | `POST /entities` `{type,title,frontmatter?,body?}` | slug dérivé du titre ; 409 si pris ; 201 + `Entry` |
| Modifier | `PATCH /entities/{slug}` `{frontmatter?,body?}` | fusion non destructive ; `created` immuable, `updated` rafraîchi |
| Renommer | `POST /entities/{slug}/rename` `{new_title?,new_slug?}` | réécrit les liens qui casseraient ([ADR 0012](../adr/0012-rename-link-rewriting.md)) |

## 5. Chantier des liens {#chantier}

**Maquette :** [chantier-liens.html](mockups/chantier-liens.html)

Deux onglets : **À créer (stubs)** et **Liens ambigus**.

- **Stubs** : `GET /stubs` → pour chaque cible non résolue : `labels` (variantes d'écriture), `count`,
  `sources` (entrées mentionnant, avec le champ). Bouton **Créer** →
  **création depuis stub** : titre pré-rempli (`labels[0]`), **sélecteur de type** (grille), **slug dérivé**
  en direct ; à la validation, `POST /entities`. Les liens passent de `stub` à `resolved` **au réindex
  suivant** (SSE `entity.created`), sans réécrire les fichiers sources.
- **Ambigus** : cibles à ≥ 2 candidats (`GET …/links` → `resolution: "ambiguous"`, `candidates[]`). Bouton
  **Lever l'ambiguïté** → **désambiguïsation** : choix du candidat, aperçu de réécriture
  `[[Cible]]` → `[[slug|Affiché]]` (vers l'identité fichier, unique), portée *cette occurrence / toutes*.
  Édition **non destructive** du fichier source ([linking.md](../linking.md) §3.3).

> Il n'existe **pas** d'endpoint « create-from-stub » dédié : c'est un `POST /entities` ordinaire alimenté
> par les données du stub.

## 6. Galerie médias {#galerie}

*(v2, [M7](../roadmap.md#m7))* — le backend (`/assets`, [M4](../roadmap.md#m4--what-landed)) exposait déjà
lister/servir/uploader ; il manquait un écran pour **parcourir visuellement** ce que l'éditeur accumule
côté champs image ([features.md](../features.md) §E).

Route `/gallery`, listée dans le nav (section « Médias », sous « Chantier »).

- **Grille** : une **carte** par fichier sous `assets/` (`GET /assets` → `{ path, kind, size }`), triée par
  `path`. `kind: "image"` affiche une vignette (`GET /assets/{path}`, `object-fit: cover`) ; `kind: "file"`
  affiche une icône générique + l'extension. Pas de `width`/`height` server-side (§3 de l'API) — la vignette
  se contente de recadrer, aucune dimension intrinsèque affichée.
- **Filtre nom de fichier** : champ texte, **filtrage client** (pas de paramètre serveur sur `/assets`) —
  même raisonnement d'échelle que l'agrégation client du Chantier (§5) : correct au volume
  mono-utilisateur/local visé ([principles.md](../principles.md)), à pousser côté serveur si un projet
  dépasse largement cette échelle.
- **Import** : bouton **Ajouter** → `POST /assets` (`multipart/form-data`), même flux que
  `ImageFieldEditor` de l'éditeur (composants.md §4) ; nom aplati à un nom de fichier nu par le serveur,
  `409` sur collision signalé inline. Rafraîchit la grille sur `assets.changed` ([SSE](../api.md#5-event-stream-sse)).
- **Détail** (clic sur une carte) : modale avec aperçu plein cadre, `path`, `kind`, taille formatée, et un
  bouton **Copier le lien** qui place `![[path]]` dans le presse-papiers (syntaxe d'embed —
  [linking.md](../linking.md)) pour un collage direct dans le corps d'une fiche.
- **Vide** : aucun asset → message + action **Ajouter**. Recherche sans résultat → « Aucun résultat » +
  effacement du filtre ([states.md](states.md) §1).

Pas de suppression depuis cet écran : un asset référencé ailleurs (`cover`, embed…) ne doit pas disparaître
silencieusement sans vérifier ses usages — hors scope MVP de cette itération, laissé à une gestion manuelle
du dossier `assets/` (cohérent avec « markdown = source de vérité », [principles.md](../principles.md)).
