# Components — Catalogue

Briques réutilisables de la direction **Atelier**. Elles s'appuient sur les [tokens](foundations.md). La
colonne *Shadcn* indique la correspondance visée côté implémentation (sous réserve du point ouvert
Shadcn/Tailwind — [ADR 0009](../adr/0009-rust-sveltekit-stack.md)).

## 1. Chrome & actions

| Composant | Description | Shadcn |
|---|---|---|
| **Bouton** | `.btn` en variantes `primary` (accent), `ghost` (surface + bordure), `stub` (teinte stub). Tailles `sm`. | `Button` |
| **Icon-button** | Bouton carré 30px (thème, colonnes…). | `Button` (icon) |
| **Segment** | Bascule inline (Table/Liste, Éditer/Aperçu, panneaux du chantier). Option active en `surface` + ombre. | `Tabs` / `ToggleGroup` |
| **Fil d'Ariane** | Projet / type / entrée ; courant en `--text`. | `Breadcrumb` |
| **Sélecteur de langue** | EN/FR dans la topbar — voir [i18n.md](i18n.md). | `DropdownMenu` |

## 2. Marqueurs & étiquettes

| Composant | Description | Shadcn |
|---|---|---|
| **Type badge** | Pastille type d'entrée (icône + label), fond `accent-soft`. | `Badge` |
| **Tag / pill** | Étiquette libre (`tags`) ou valeur d'état (statut) ; pill avec point coloré pour un statut. | `Badge` |
| **Chip de filtre** | Filtre actif retirable : `champ` (mono) + valeur + ✕. | `Badge` + close |
| **Compteur** | Nombre en mono, `tabular-nums`, fond pill (nav, panneaux). | — |

## 3. Liens (le cœur du produit)

Rendu unifié des [wikilinks](../linking.md), même vocabulaire partout (corps, frontmatter, listes, rail) :

| État | Rendu inline | Rendu « puce » (frontmatter/token) |
|---|---|---|
| `resolved` | Texte accent + souligné `accent-line` (survol : fond `accent-soft`) | Puce `accent-soft` |
| `stub` | Texte stub + souligné pointillé + exposant `+` | Puce `stub-soft`, exposant `+` |
| `ambiguous` | Texte amb + souligné pointillé + exposant `?` | — |

- Cliquer un **résolu** navigue vers l'entrée ; cliquer un **stub** ouvre la **création depuis stub** ; un
  **ambigu** ouvre la **désambiguïsation** ([screens.md](screens.md#chantier)).
- Un **embed d'asset** `![[…]]` non trouvé est un bloc d'avertissement, pas un lien (voir [states.md](states.md)).

## 4. Champs de formulaire (générés par type)

Rendu piloté par `FieldSchema.kind` (`GET /types/{type}` — [api.md](../api.md)). Label = `label` du schéma
(clé mono en repère), obligation via `required`.

| `kind` | Contrôle |
|---|---|
| `text` | `input` ou `textarea` (multi-lignes pour les champs longs : desire, wound…) |
| `number` / `number-or-text` | `input` étroit, mono |
| `boolean` | Interrupteur (toggle) |
| `enum` | `select` alimenté par `enum_values` |
| `list` / `list-or-text` | Champ à **tokens** (ajout/suppression) |
| `link` | **Token de lien** + bouton « lier… » ouvrant l'**autocomplétion** |
| `link-list` | Plusieurs tokens de lien |
| `image` / `image-list` | Zone de dépôt / miniature (copie vers `assets/`) |

Composants transverses du formulaire :

| Composant | Description | Shadcn |
|---|---|---|
| **Token input** | Puces éditables (aliases, tags, listes). | `Input` custom / `Badge` |
| **Autocomplétion de lien** | Popover de résolution : correspondances (`title`/alias) + option **« Créer … »** inline (voie stub). | `Command` (combobox) |
| **Champs conservés** | Bloc en lecture seule pour les **clés inconnues** préservées (ex. `obsidian_note_id`) ; cadenas + mention « édition dans le .md ». | `Card` |
| **Barre d'enregistrement** | Barre collante bas d'écran : état « non enregistré », rappel écriture non destructive, ⌘S, Annuler/Enregistrer. | — |

## 5. Collections

| Composant | Description | Shadcn |
|---|---|---|
| **Table** | En-têtes cliquables (tri, flèche sur la colonne active), colonnes = champs du type, `overflow-x` ; **colonne d'état** à gauche (`●` sain / `▲` diagnostic). Lignes cliquables. | `Table` |
| **Carte de liste** | Alternative « Liste » : avatar, titre, rôle, **excerpt** (API), tags, drapeau d'erreur éventuel. | `Card` |
| **Ligne de rétrolien** | Entrée source + **type** + **champ/contexte** (`via pov`) + extrait. | — |
| **Ligne de lien sortant** | Point de statut (résolu/stub) + nom + champ d'origine. | — |
| **Ligne de stub** | Icône `+`, libellé(s), nombre de mentions, sources en chips, bouton **Créer**. | — |
| **Carte de stat** | Grand nombre + libellé (tableau de bord) ; variante `chantier` teintée stub. | `Card` |
| **Barre de répartition** | Nom + piste + remplissage accent + valeur mono (dashboard). | — |

## 6. Surfaces flottantes

| Composant | Description | Shadcn |
|---|---|---|
| **Popover** | Petites actions ancrées (création depuis stub compacte, menus tri/filtre). | `Popover` / `DropdownMenu` |
| **Modale** | Actions structurantes centrées : création depuis stub complète, désambiguïsation. En-tête typé (kicker + titre), corps, pied d'actions. | `Dialog` |
| **Panneau escamotable** | Rail droit (rétroliens/liens/méta) — voir [layout.md](layout.md#24-rail-312px--panneau-escamotable). | `Sheet` (desktop inline) |
| **Bandeau diagnostic** | Encart non bloquant en tête de contenu (asset manquant, liens non résolus…). | `Alert` |
| **Indicateur temps réel** | Pastille « Watcher actif » pulsée (respecte `reduced-motion`). | — |
