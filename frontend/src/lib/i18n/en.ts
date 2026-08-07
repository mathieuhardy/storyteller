// English UI catalog (ADR 0014). Holds the app chrome and the built-in type
// labels, keyed by their stable English `name` from `GET /types`. It never
// translates the author's content — only what the *application* says.
//
// `en` is the source of truth for the message-key set; `fr` is typed against it,
// so a missing translation is a compile error.

export const en = {
	'app.name': 'Storyteller',

	'action.newEntry': 'New entry',
	'action.save': 'Save',
	'action.cancel': 'Cancel',
	'action.open': 'Open',
	'action.create': 'Create',
	'action.openFolder': 'Open a folder…',
	'action.newProject': 'New project',

	'search.placeholder': 'Search…',
	'search.comingSoon': 'Search arrives in a later milestone',

	'links.toggle': 'Links',

	'theme.label': 'Theme',
	'theme.system': 'System',
	'theme.light': 'Light',
	'theme.dark': 'Dark',

	'lang.label': 'Language',
	'lang.en': 'English',
	'lang.fr': 'Français',

	'nav.types': 'Types',
	'nav.workshop': 'Workshop',
	'nav.stubs': 'To create',

	'breadcrumb.project': 'Project',

	'launcher.tagline': 'Your story bible,\nin plain markdown.',
	'launcher.subtitle':
		'Characters, world, chapters and resources — typed, linked, and yours on disk.',
	'launcher.localFirst': 'Local-first. No account, no cloud — just a folder.',
	'launcher.recent': 'Recent projects',
	'launcher.noRecent': 'No project opened yet.',
	'launcher.entries': '{count} entries',
	'launcher.openTitle': 'Open a project folder',
	'launcher.pathLabel': 'Folder path on this machine',
	'launcher.pathHint':
		'In browser mode, type the server-side folder path. A native picker comes with the desktop app.',
	'launcher.opening': 'Opening…',
	'launcher.openError': 'Could not open that folder.',

	'dashboard.title': 'Dashboard',
	'dashboard.placeholderTitle': 'Project shell ready',
	'dashboard.placeholderBody':
		'The dashboard, entry views and editor land in the next milestone steps. The navigation, links panel, theme and language are live.',

	'screen.comingSoonTitle': 'Coming soon',
	'screen.comingSoonBody': 'This view is part of a later step of the frontend milestone.',

	'type.project': 'Project',
	'type.character': 'Character',
	'type.location': 'Location',
	'type.faction': 'Faction',
	'type.object': 'Object',
	'type.culture': 'Culture',
	'type.system': 'System',
	'type.species': 'Species',
	'type.chapter': 'Chapter',
	'type.note': 'Note',
	'type.concept': 'Concept'
};

export type MessageKey = keyof typeof en;
export type Messages = Record<MessageKey, string>;
