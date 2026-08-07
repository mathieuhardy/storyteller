# UI — Documentation d'interface

Référence de conception de l'interface **Storyteller** (jalon [M4](../roadmap.md)). Ce dossier fixe la
**direction visuelle**, le **système de composants** et les **patterns d'interaction** qui guideront
l'implémentation SvelteKit. Il complète [usage.md](../usage.md) (guide utilisateur, à remplir après M4) et
s'appuie sur [data-model.md](../data-model.md), [linking.md](../linking.md) et [api.md](../api.md).

> **Statut : conception.** Rien n'est encore implémenté. Les maquettes de [`mockups/`](mockups/) sont des
> pages HTML autonomes servant de **référence visuelle**, pas de code de production. Les points de styling et
> d'i18n sont tranchés : Tailwind + shadcn-svelte contenu par des classes de composants
> ([ADR 0013](../adr/0013-tailwind-with-component-classes.md)) et catalogue i18n côté front
> ([ADR 0014](../adr/0014-ui-i18n-front-catalog.md)).

## Direction retenue — « Atelier »

Registre **outil dense et efficace** (proche d'un IDE / d'un Notion), pensé pour **consulter et naviguer
vite** une bible qui grossit. Ardoise froide neutre, accent iris, information dense mais lisible.

Décisions cadrées avec l'auteur :

| Sujet | Décision |
|---|---|
| Ambiance | Outil dense « Atelier » (variante littéraire « Verre » écartée — voir [`mockups/_variante-verre-ecartee.html`](mockups/_variante-verre-ecartee.html)) |
| Liens dans le texte | **Couleur d'accent** (résolus très repérables) |
| Rétroliens | **Panneau escamotable** (caché par défaut, ouvert à la demande) |
| Thème | Clair **et** sombre, plus suivi du système |
| Langue d'interface | **Sélecteur EN/FR** (voir [i18n.md](i18n.md)) |

## Les documents

| Fichier | Contenu |
|---|---|
| [layout.md](layout.md) | Shell applicatif, grille, panneau escamotable, responsive, lanceur |
| [foundations.md](foundations.md) | Tokens : couleurs (clair/sombre), typographie, espacements, rayons, ombres |
| [components.md](components.md) | Catalogue de composants + correspondance Shadcn |
| [states.md](states.md) | États normalisés (chargement, vide, erreur, stub, ambigu, champ préservé…) |
| [screens.md](screens.md) | Les 5 écrans détaillés + mapping écran → endpoints |
| [i18n.md](i18n.md) | Sélecteur de langue EN/FR, périmètre de traduction |

## Inventaire des écrans (MVP)

Les cinq surfaces maquettées, dans un langage visuel unique :

| Écran | Rôle | Maquette |
|---|---|---|
| **Accueil / tableau de bord** | Lanceur multi-projets + vue d'ensemble d'un projet | [accueil-dashboard.html](mockups/accueil-dashboard.html) |
| **Fiche entrée** | Lecture d'une entrée (attributs + corps + rétroliens) | [fiche-entree.html](mockups/fiche-entree.html) |
| **Vue liste/table** | Navigation d'un type (filtres, tri, colonnes) | [vue-liste-table.html](mockups/vue-liste-table.html) |
| **Éditeur d'entrée** | Création/édition (formulaire + corps markdown) | [editeur-entree.html](mockups/editeur-entree.html) |
| **Chantier des liens** | Stubs à créer + désambiguïsation | [chantier-liens.html](mockups/chantier-liens.html) |

## Principes directeurs (rappel)

L'UI **consomme** l'API et n'embarque aucune logique métier ([architecture](../architecture.md)). Elle doit
respecter les invariants produit :

- **Markdown = source de vérité.** L'interface est une vue confortable de fichiers qui appartiennent à
  l'auteur. Elle affiche l'identité **fichier (slug)** distincte du **titre** affiché.
- **Non-destructif & dégradation gracieuse.** Les [diagnostics](states.md) par entrée s'affichent **sans
  bloquer** ; les clés inconnues sont montrées comme **préservées** ; un asset manquant est signalé, pas caché.
- **Local-first.** Aucun compte, aucun cloud ; le lanceur ouvre un **dossier**.
- **Temps réel.** Le [watcher](../glossary.md) pousse des événements [SSE](../api.md) : les vues se
  rafraîchissent sans polling (édition externe Obsidian/vim prise en compte).

Voir [screens.md](screens.md) pour le détail par écran et le mapping vers les endpoints.
