# States — États normalisés

L'interface **signale, ne cache jamais** (dégradation gracieuse — [principles.md](../principles.md) §7). Chaque
état ci-dessous a un traitement visuel fixe, réutilisé partout. Les couleurs viennent de
[foundations.md](foundations.md) ; l'état ne repose jamais sur la **seule** couleur (icône/forme en renfort).

## 1. États de chargement / vide

| État | Traitement |
|---|---|
| **Chargement** | Squelettes (blocs `surface-2` animés) aux emplacements réels ; jamais de spinner plein écran une fois le shell monté. |
| **Vide (type sans entrée)** | Message + action primaire (« Aucun personnage — créer le premier »). |
| **Vide (recherche/filtre)** | « Aucun résultat » + bouton d'effacement des filtres. |
| **Recherche indisponible** | Champ marqué **« à venir »** (M5, `q=` → 501). L'affordance existe, désactivée. |

## 2. États de lien (cœur métier)

Reflètent `resolution` de `GET /entities/{slug}/links` — voir [linking.md](../linking.md) §3.4.

| État | Marqueur | Action au clic |
|---|---|---|
| `resolved` | Accent + souligné plein | Navigue vers l'entrée |
| `stub` | Stub + pointillé + exposant `+` | Ouvre **création depuis stub** (titre pré-rempli) |
| `ambiguous` | Amb + pointillé + exposant `?` | Ouvre **désambiguïsation** (candidats) |

Comptes agrégés : badge **« Liens · N »** (topbar) et entrée **« À créer »** (nav) reprennent les mêmes teintes.

## 3. Diagnostics par entrée (non bloquants)

`Entry.errors: Diagnostic[]` et `EntrySummary.has_errors` — [api.md](../api.md). Toujours affichés **sans
empêcher** la lecture/l'édition.

| Code | Sévérité | Rendu |
|---|---|---|
| `missing_required_field` | error | Ligne d'état `▲` (`--danger`) en liste ; message inline sous le champ dans l'éditeur |
| `invalid_field_value` | error | Idem, sur le champ concerné |
| `duplicate_slug` | error | Bandeau : « même identité que `…` » + lien vers l'autre fichier |
| `yaml_parse_error` | error | Bandeau en tête de fiche ; corps affiché tel quel |
| `unknown_type` | warning | Traité comme `note` + mention « type inconnu » |
| `encoding_error` | warning | Bandeau d'avertissement |

- **Liste/table** : colonne d'état à gauche — `●` discret (sain) ou `▲` (`--danger`, tooltip = nombre + nature).
- **Fiche/éditeur** : **bandeau** en tête (dismissible) pour les diagnostics d'entrée ; **message inline**
  sous le champ pour les diagnostics de champ.

## 4. Assets manquants

Un embed `![[fichier]]` ou un champ image (`cover`, `portrait`, `map`…) pointant vers un fichier absent
n'est **pas** un stub (voir [linking.md](../linking.md) §1) : c'est un **« asset introuvable »**.

- **Couverture/portrait** : cadre `surface-2` avec le nom de fichier et l'état, à la place de l'image.
- **Embed dans le corps** : bloc d'avertissement pointillé (`--warn`) citant le chemin.
- Un **bandeau** peut regrouper « illustration introuvable » en tête de fiche.

## 5. Identité, champs préservés, non-destructif

| Situation | Rendu |
|---|---|
| **Identité vs titre** | Le titre s'édite librement ; l'identité **fichier** (`aria-solane.md`) est montrée en mono avec la mention « passez par *Renommer* ». |
| **Clés inconnues préservées** | Section **« Champs conservés »** (cadenas), lecture seule, éditables uniquement dans le .md. |
| **Écriture non destructive** | La barre d'enregistrement rappelle : clés inconnues, ordre des clés et corps **préservés** ; seules les valeurs modifiées sont réécrites. |
| **Non enregistré** | Indicateur `--warn` (topbar + barre basse) dès la première modification ; ⌘S pour enregistrer. |

## 6. Temps réel (watcher / SSE)

Le flux `GET /events` ([api.md](../api.md)) pilote les rafraîchissements :

| Événement | Effet UI |
|---|---|
| `entity.created` / `entity.updated` / `entity.deleted` | Mise à jour ciblée des listes/compteurs ; toast discret si l'entrée courante change sous les pieds de l'utilisateur |
| `index.rebuilt` (`reason: "watch"`) | Rafraîchissement global (édition externe Obsidian/vim/`git pull`) |

- Indicateur **« Watcher actif »** (pastille `--ok` pulsée) + « réindexé il y a N s » sur le tableau de bord.
- Client à la traîne : on **recharge** depuis l'index (source unique), pas de réconciliation fine.

## 7. Erreurs d'API (échec de requête)

Distinctes des diagnostics (qui, eux, voyagent dans un 200). Corps `{ error: { code, message, details } }` :

| Cas | Rendu |
|---|---|
| `not_found` (404) | Écran « entrée introuvable » + retour à la liste |
| `conflict` (409) | Message inline (ex. slug déjà pris à la création/renommage) |
| `unprocessable` (422) | Message inline (ex. titre non « slugifiable ») |
| `not_implemented` (501) | Fonction marquée « à venir » (recherche, rendu HTML) |
| `internal_error` (500) | Bandeau global réessayable, sans perte de saisie |
