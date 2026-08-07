# Foundations — Tokens

Valeurs de base de la direction **Atelier**. Définies en variables CSS sur `:root`, redéfinies sous
`@media (prefers-color-scheme: dark)` **et** sous `:root[data-theme="dark"|"light"]` (le sélecteur de thème
prime sur la préférence système, dans les deux sens). Les composants ne référencent **que** ces tokens,
jamais une couleur en dur.

## 1. Couleurs

### Neutres (ardoise froide — choisie, pas un gris par défaut)

| Token | Clair | Sombre | Usage |
|---|---|---|---|
| `--bg` | `#f6f7f9` | `#0c0e12` | Fond application |
| `--surface` | `#ffffff` | `#14171d` | Cartes, barres, panneaux |
| `--surface-2` | `#f1f3f6` | `#191d24` | Fonds secondaires, champs, survols |
| `--surface-3` | `#e9edf2` | `#20252e` | Pistes, avatars, aplats tertiaires |
| `--border` | `#e1e5eb` | `#262c36` | Séparateurs, contours |
| `--border-strong` | `#d2d8e0` | `#333a46` | Contours au survol / focus |
| `--text` | `#1b2027` | `#e6e8ee` | Texte principal |
| `--muted` | `#626b78` | `#949ca8` | Texte secondaire |
| `--faint` | `#9aa2ad` | `#656d79` | Texte tertiaire, placeholders |

### Accent (iris)

| Token | Clair | Sombre | Usage |
|---|---|---|---|
| `--accent` | `#5b5bd6` | `#8f90ee` | Liens résolus, sélection, action primaire |
| `--accent-fg` | `#ffffff` | `#10121a` | Texte sur accent |
| `--accent-soft` | `#edeefc` | `#1d2030` | Fonds actifs, puces de lien |
| `--accent-line` | `#c9cbf3` | `#3a3d63` | Soulignés/contours d'accent |

### Couleurs sémantiques (distinctes de l'accent)

Elles **encodent un état**, jamais une simple décoration. Chaque état de lien a sa teinte propre.

| État | Token | Clair | Sombre | Sens |
|---|---|---|---|---|
| **Stub** (à créer) | `--stub` / `--stub-soft` / `--stub-line` | `#a75c11` / `#fbf0df` / `#e6cfa4` | `#d9a35f` / `#2a2113` / `#4a3a20` | Lien non résolu |
| **Ambigu** | `--amb` / `--amb-soft` / `--amb-line` | `#b6244a` / `#fce8ed` / `#f2c0cd` | `#f0728c` / `#2c1620` / `#4d2632` | ≥ 2 cibles |
| **OK / succès** | `--ok` / `--ok-soft` | `#2f8f5b` / `#e6f4ec` | `#57b37e` / `#14241b` | Sain, watcher actif |
| **Avertissement** | `--warn` / `--warn-soft` | `#a75c11` / `#fbf0df` | `#d9a35f` / `#2a2113` | Diagnostic non bloquant, asset manquant |
| **Erreur** | `--danger` | `#cf3838` | `#e8615f` | Diagnostic bloquant, champ requis manquant |

> **Correspondance états ↔ couleurs de lien** : `resolved` → accent ; `stub` → stub ; `ambiguous` → amb.
> C'est le socle visuel du [linking](../linking.md). Voir [states.md](states.md).

### Ombres

| Token | Clair | Sombre |
|---|---|---|
| `--shadow` | `0 1px 2px rgba(20,25,35,.05), 0 1px 3px rgba(20,25,35,.04)` | `0 1px 2px rgba(0,0,0,.4)` |
| `--shadow-pop` | `0 8px 30px rgba(20,25,35,.16), 0 2px 8px rgba(20,25,35,.10)` | `0 10px 34px rgba(0,0,0,.55)…` |
| `--shadow-lg` | `0 10px 40px rgba(20,25,35,.10)…` | `0 14px 50px rgba(0,0,0,.55)…` |

## 2. Typographie

Pas de webfont (CSP des artefacts + démarrage rapide) : piles système, choix assumé.

| Token | Pile |
|---|---|
| `--font-ui` | `system-ui, -apple-system, "Segoe UI", Roboto, Helvetica, Arial, sans-serif` |
| `--font-mono` | `ui-monospace, "SF Mono", "JetBrains Mono", "Cascadia Mono", Menlo, Consolas, monospace` |

**Mono** est réservé au **technique** : clés de frontmatter (`role`, `pov`…), slugs, dates, chemins,
compteurs (avec `font-variant-numeric: tabular-nums`). Le reste est en `--font-ui`.

### Échelle de taille (px)

| Rôle | Taille / graisse |
|---|---|
| Micro-label (uppercase, `letter-spacing .06em`) | 10.5–11 / 600 |
| Légende, méta | 11.5–12.5 / 400 |
| Corps UI (base) | 14 / 400 |
| Corps de lecture (fiche) | 14.5 / 400, `line-height 1.72` |
| Sous-titre / valeur forte | 15 / 500–600 |
| Titre d'écran (liste) | 21 / 660 |
| Titre d'entrée (fiche) | 27 / 680 |
| Titre projet (dashboard) | 28 / 700 |
| Accroche lanceur | 40 / 700 |

Titres avec `text-wrap: balance`. Colonne de lecture visée ≈ 60–65 caractères.

## 3. Espacements, rayons

- **Rayons** : `--radius: 8px` (défaut), `--radius-sm: 6px` (champs, boutons, puces), `--radius-lg: 14px`
  (modales, grandes cartes).
- **Espacements** : échelle informelle multiple de 2/4 (2, 4, 6, 8, 12, 14, 18, 22, 26). Les groupes de
  frères se posent en `flex`/`grid` + `gap`, jamais en marges par élément (pas de collapse silencieux).

## 4. Accessibilité

- Focus clavier **toujours visible** (contour accent : `box-shadow: 0 0 0 3px var(--accent-soft)` +
  `border-color: var(--accent)` sur les champs).
- Respect de `prefers-reduced-motion` (animations décoratives, ex. pulse du watcher, désactivées).
- Contrastes vérifiés sur les deux thèmes ; le second thème n'est pas une inversion naïve.
- L'état n'est jamais porté par la **seule** couleur : un stub porte aussi un `+`, un ambigu un `?`, un
  diagnostic une icône. Voir [states.md](states.md).
