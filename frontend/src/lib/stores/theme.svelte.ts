// Theme preference: system (follow the OS), light, or dark. Like the language,
// it is a local UI preference in localStorage — never part of a project.
// `data-theme` on the root wins over `prefers-color-scheme` in app.css; `system`
// removes the attribute so the OS preference applies.

export type Theme = 'system' | 'light' | 'dark';

const STORAGE_KEY = 'storyteller:theme';

let theme = $state<Theme>('system');

function apply(next: Theme): void {
	if (typeof document === 'undefined') return;
	const root = document.documentElement;
	if (next === 'system') root.removeAttribute('data-theme');
	else root.setAttribute('data-theme', next);
}

/** Reads the saved theme and applies it (called from the root layout). The
 * inline script in app.html already applied it before paint; this syncs state. */
export function initTheme(): void {
	if (typeof localStorage !== 'undefined') {
		const saved = localStorage.getItem(STORAGE_KEY);
		if (saved === 'light' || saved === 'dark' || saved === 'system') theme = saved;
	}
	apply(theme);
}

export function getTheme(): Theme {
	return theme;
}

export function setTheme(next: Theme): void {
	theme = next;
	if (typeof localStorage !== 'undefined') localStorage.setItem(STORAGE_KEY, next);
	apply(next);
}

/** Cycles system → light → dark → system, for a single toggle control. */
export function cycleTheme(): void {
	const order: Theme[] = ['system', 'light', 'dark'];
	setTheme(order[(order.indexOf(theme) + 1) % order.length]);
}
