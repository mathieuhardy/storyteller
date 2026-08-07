// French UI catalog (ADR 0014). Typed against `Messages`, so it must cover every
// key `en` defines. Type/field labels mirror the server's French labels.

import type { Messages } from './en';

export const fr: Messages = {
	'app.name': 'Storyteller',

	'action.newEntry': 'Nouvelle entrée',
	'action.save': 'Enregistrer',
	'action.cancel': 'Annuler',
	'action.open': 'Ouvrir',
	'action.create': 'Créer',
	'action.openFolder': 'Ouvrir un dossier…',
	'action.newProject': 'Nouveau projet',

	'search.placeholder': 'Rechercher…',
	'search.comingSoon': 'La recherche arrive dans un jalon ultérieur',

	'links.toggle': 'Liens',

	'theme.label': 'Thème',
	'theme.system': 'Système',
	'theme.light': 'Clair',
	'theme.dark': 'Sombre',

	'lang.label': 'Langue',
	'lang.en': 'English',
	'lang.fr': 'Français',

	'nav.types': 'Types',
	'nav.workshop': 'Chantier',
	'nav.stubs': 'À créer',

	'breadcrumb.project': 'Projet',

	'launcher.tagline': 'Votre bible narrative,\nen markdown brut.',
	'launcher.subtitle':
		'Personnages, monde, chapitres et ressources — typés, liés, et à vous sur le disque.',
	'launcher.localFirst': "Local d'abord. Aucun compte, aucun cloud — juste un dossier.",
	'launcher.recent': 'Projets récents',
	'launcher.noRecent': 'Aucun projet ouvert pour le moment.',
	'launcher.entries': '{count} entrées',
	'launcher.openTitle': 'Ouvrir un dossier de projet',
	'launcher.pathLabel': 'Chemin du dossier sur cette machine',
	'launcher.pathHint':
		"En mode navigateur, saisissez le chemin du dossier côté serveur. Un sélecteur natif viendra avec l'application de bureau.",
	'launcher.opening': 'Ouverture…',
	'launcher.openError': "Impossible d'ouvrir ce dossier.",

	'dashboard.title': 'Tableau de bord',
	'dashboard.placeholderTitle': "Coque du projet prête",
	'dashboard.placeholderBody':
		"Le tableau de bord, les vues d'entrées et l'éditeur arrivent aux étapes suivantes. La navigation, le panneau de liens, le thème et la langue sont actifs.",

	'screen.comingSoonTitle': 'Bientôt',
	'screen.comingSoonBody': 'Cette vue fait partie d’une étape ultérieure du jalon frontend.',

	'type.project': 'Projet',
	'type.character': 'Personnage',
	'type.location': 'Lieu',
	'type.faction': 'Faction',
	'type.object': 'Objet',
	'type.culture': 'Culture',
	'type.system': 'Système',
	'type.species': 'Espèce',
	'type.chapter': 'Chapitre',
	'type.note': 'Note',
	'type.concept': 'Concept'
};
