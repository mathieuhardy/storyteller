# Layout — Shell & grille

Structure spatiale de l'interface **Atelier**. Voir les [tokens](foundations.md) pour les valeurs,
[components.md](components.md) pour les briques, [screens.md](screens.md) pour l'usage par écran.

## 1. Deux contextes

L'application a deux contextes de plus haut niveau :

1. **Lanceur** (aucun projet ouvert) — plein écran, sans shell. Ouvre un **dossier** de projet ou en crée un,
   liste les **projets récents**. Local-first : pas de compte, pas de cloud. Voir
   [accueil-dashboard.html](mockups/accueil-dashboard.html).
2. **Shell applicatif** (projet ouvert) — la coque persistante décrite ci-dessous, dans laquelle défilent
   tous les écrans (tableau de bord, liste, fiche, éditeur, chantier).

## 2. Le shell (projet ouvert)

Grille CSS à trois zones, une barre en haut sur toute la largeur :

```
┌───────────────────────────────────────────────┐
│ topbar  (48px)                                  │
├──────────┬───────────────────────┬─────────────┤
│  nav     │  main                 │  rail        │
│  244px   │  1fr                  │  0 / 312px   │
│  (types) │  (contenu de l'écran) │ (escamotable)│
└──────────┴───────────────────────┴─────────────┘
```

`grid-template-columns: 244px minmax(0,1fr) 0` ; l'état **panneau ouvert** passe la 3ᵉ colonne à `312px`
avec une transition. Hauteur `100vh`, `min-height: 640px`.

### 2.1 Topbar (48px)

De gauche à droite : **marque** · **fil d'Ariane** (projet / type / entrée, l'élément courant en `--text`) ·
espace flexible · **recherche** (voir §4) · bascule du **panneau liens** (compteur) · **sélecteur de langue**
([i18n.md](i18n.md)) · **thème** · action primaire contextuelle (« Nouvelle entrée », « Enregistrer »…).

En mode édition, la topbar accueille l'indicateur **« Non enregistré »** et les actions Annuler / Enregistrer.

### 2.2 Nav (244px) — navigation par type

- **En-tête projet** cliquable (couverture, nom, nb d'entrées) = **sélecteur de projet** (rouvre le lanceur).
- Section **Types** : un item par type activé, avec **icône**, **label** (localisé) et **compteur**
  (`GET /project` → `stats.by_type`). L'item actif est sur fond `--accent-soft`.
- Section **Chantier** : entrée **« À créer (stubs) »** avec un badge de compte en teinte `--stub`.
- Seuls les **types activés** (`enabled_types`) sont listés (voir [data-model.md](../data-model.md) §7).

### 2.3 Main (1fr)

Zone défilante. Chaque écran centre son contenu dans une **colonne de largeur maîtrisée** selon la densité :

| Écran | `max-width` |
|---|---|
| Fiche entrée | 780px |
| Éditeur | 800px |
| Liste/table | pleine largeur (table `overflow-x`) |
| Tableau de bord | 940px |
| Chantier | 840px |

### 2.4 Rail (312px) — panneau escamotable

**Caché par défaut.** Ouvert via le bouton **« Liens · N »** de la topbar (état actif accentué) ou fermé par
son ✕. Contient, pour la fiche : **Mentionné dans** (rétroliens), **Liens sortants** (avec statut), et
**Métadonnées**. Voir [screens.md](screens.md).

- Sur large écran, l'ouverture **pousse** la colonne `main` (transition douce).
- Sous **1080px**, le panneau ouvert passe en **overlay** (`position: fixed`, ombre) au lieu de comprimer la
  fiche.

## 3. Responsive

| Palier | Comportement |
|---|---|
| ≥ 1080px | Trois zones ; rail en colonne poussante |
| < 1080px | Nav **réduite en icônes** (labels/compteurs masqués) ; rail en **overlay** |
| < 900px | Idem ; recherche réduite à l'icône |
| < 720px | Fil d'Ariane raccourci |

Règle générale : le corps de page **ne défile jamais horizontalement**. Les contenus larges (tables,
diagrammes) scrollent dans leur propre conteneur `overflow-x: auto`.

## 4. Recherche (emplacement réservé)

Le champ de recherche est présent dans la topbar mais marqué **« à venir »** : la recherche plein-texte
(`GET /entities?q=`) renvoie **501** jusqu'au jalon M5 ([api.md](../api.md)). L'emplacement et l'affordance
sont posés dès le MVP ; l'activation viendra sans réagencement.

## 5. Lanceur

Deux colonnes : à gauche l'**accroche** produit + actions (Ouvrir un dossier, Nouveau projet) et la mention
local-first ; à droite la liste des **projets récents** (nom, chemin `~/…`, nb d'entrées, dernière ouverture).
Cliquer un projet entre dans le shell sur son **tableau de bord**. Sous 820px, les deux colonnes s'empilent.
