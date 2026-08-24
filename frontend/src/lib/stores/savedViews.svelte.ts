// Saved views (docs/api.md §4, milestone M5): "a view = a persisted set of
// parameters on the frontend side" — a named `sort=`/`tag=`/`<field>=`
// combination for one type's list/table screen. Like the theme and language,
// this is a local UI preference in localStorage, never written to the
// project: two people opening the same project folder don't share views.

export interface SavedView {
	id: string;
	name: string;
	type: string;
	/** The query string as it appears in the URL (no leading `?`), minus `page`. */
	query: string;
}

const STORAGE_KEY = 'storyteller:savedViews';

let views = $state<SavedView[]>([]);
let loaded = false;

function load(): void {
	if (loaded) return;
	loaded = true;
	if (typeof localStorage === 'undefined') return;
	try {
		const raw = localStorage.getItem(STORAGE_KEY);
		if (raw) views = JSON.parse(raw);
	} catch {
		views = [];
	}
}

function persist(): void {
	if (typeof localStorage === 'undefined') return;
	localStorage.setItem(STORAGE_KEY, JSON.stringify(views));
}

/** Saved views for one type, in save order. */
export function viewsFor(type: string): SavedView[] {
	load();
	return views.filter((v) => v.type === type);
}

export function saveView(name: string, type: string, query: string): void {
	load();
	views = [...views, { id: crypto.randomUUID(), name, type, query }];
	persist();
}

export function deleteView(id: string): void {
	load();
	views = views.filter((v) => v.id !== id);
	persist();
}
