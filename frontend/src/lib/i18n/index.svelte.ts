// Locale state + translation helpers (ADR 0014). Language is a local UI
// preference (like the theme), persisted to localStorage, never written to the
// project. `t()` reads the reactive `locale` rune, so any markup calling it
// re-renders when the language switches.

import { en, type MessageKey } from './en';
import { fr } from './fr';

const catalogs = { en, fr } as const;
export type Locale = keyof typeof catalogs;

const STORAGE_KEY = 'storyteller:locale';

function initialLocale(): Locale {
	if (typeof localStorage !== 'undefined') {
		const saved = localStorage.getItem(STORAGE_KEY);
		if (saved === 'en' || saved === 'fr') return saved;
	}
	if (typeof navigator !== 'undefined' && navigator.language?.toLowerCase().startsWith('fr')) {
		return 'fr';
	}
	return 'en';
}

let locale = $state<Locale>('en');

/** Applies the initial locale on the client (called from the root layout). */
export function initLocale(): void {
	setLocale(initialLocale());
}

export function getLocale(): Locale {
	return locale;
}

export function setLocale(next: Locale): void {
	locale = next;
	if (typeof localStorage !== 'undefined') localStorage.setItem(STORAGE_KEY, next);
	if (typeof document !== 'undefined') document.documentElement.lang = next;
}

type Params = Record<string, string | number>;

/** Translates a chrome key, interpolating `{name}` placeholders. */
export function t(key: MessageKey, params?: Params): string {
	const template = catalogs[locale][key] ?? en[key] ?? String(key);
	if (!params) return template;
	return template.replace(/\{(\w+)\}/g, (_, name: string) =>
		name in params ? String(params[name]) : `{${name}}`
	);
}

/** Localized label of a built-in type, keyed by its English `name`. */
export function typeLabel(name: string): string {
	const key = `type.${name}` as MessageKey;
	return catalogs[locale][key] ?? en[key] ?? name;
}

export function formatNumber(value: number): string {
	return new Intl.NumberFormat(locale).format(value);
}

/** Formats an ISO/RFC-3339 date per the current locale; passes through if it is
 * not a parseable date (tolerance — never show `Invalid Date`). */
export function formatDate(iso: string): string {
	const date = new Date(iso);
	if (Number.isNaN(date.getTime())) return iso;
	return new Intl.DateTimeFormat(locale, { dateStyle: 'medium', timeStyle: 'short' }).format(date);
}

/** Relative "opened 3 days ago"-style label for the launcher. */
export function formatRelative(iso: string): string {
	const date = new Date(iso);
	if (Number.isNaN(date.getTime())) return iso;
	const seconds = Math.round((date.getTime() - Date.now()) / 1000);
	const rtf = new Intl.RelativeTimeFormat(locale, { numeric: 'auto' });
	const units: [Intl.RelativeTimeFormatUnit, number][] = [
		['year', 31536000],
		['month', 2592000],
		['day', 86400],
		['hour', 3600],
		['minute', 60]
	];
	for (const [unit, size] of units) {
		if (Math.abs(seconds) >= size) return rtf.format(Math.round(seconds / size), unit);
	}
	return rtf.format(seconds, 'second');
}
