// Disambiguation rewrite helpers (docs/linking.md §3.3, docs/ui/screens.md §5):
// given the literal text of a body or a frontmatter field, replace every
// `[[TargetRaw]]` / `[[TargetRaw|Display]]` occurrence with `[[chosenSlug|Display]]`
// (keeping the displayed text), operating on the real string content rather than
// reconstructing it — `GET .../links` does not expose the original display text,
// only the bare target.
import type { Frontmatter } from '$api/types';

export interface AmbiguousOccurrence {
	slug: string;
	title: string;
	type: string;
	field: string | null;
}

export interface AmbiguousGroup {
	targetRaw: string;
	candidateSlugs: string[];
	occurrences: AmbiguousOccurrence[];
}

function escapeRegExp(text: string): string {
	return text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

export function rewriteWikilinkText(text: string, targetRaw: string, chosenSlug: string): string {
	const pattern = '\\[\\[\\s*' + escapeRegExp(targetRaw) + '\\s*(\\|[^\\]]*)?\\s*\\]\\]';
	const re = new RegExp(pattern, 'g');
	return text.replace(re, (_match, displayGroup: string | undefined) => {
		const display = displayGroup ? displayGroup.slice(1).trim() : targetRaw;
		return `[[${chosenSlug}|${display}]]`;
	});
}

/** Rewrites a frontmatter value (string, or array of strings for a link-list).
 * Returns `undefined` when nothing in it matched (no patch needed). */
function rewriteFrontmatterField(value: unknown, targetRaw: string, chosenSlug: string): unknown {
	if (typeof value === 'string') {
		const updated = rewriteWikilinkText(value, targetRaw, chosenSlug);
		return updated !== value ? updated : undefined;
	}
	if (Array.isArray(value)) {
		let changed = false;
		const updated = value.map((item) => {
			if (typeof item !== 'string') return item;
			const rewritten = rewriteWikilinkText(item, targetRaw, chosenSlug);
			if (rewritten !== item) changed = true;
			return rewritten;
		});
		return changed ? updated : undefined;
	}
	return undefined;
}

/** Builds the PATCH payload for one entry, given the distinct fields (or the
 * body, `field: null`) where it cites the ambiguous target. */
export function buildDisambiguationPatch(
	entry: { body: string; frontmatter: Frontmatter },
	fields: (string | null)[],
	targetRaw: string,
	chosenSlug: string
): { frontmatter?: Frontmatter; body?: string } {
	const patch: { frontmatter?: Frontmatter; body?: string } = {};
	const frontmatterPatch: Frontmatter = {};
	for (const field of new Set(fields)) {
		if (field === null) {
			const rewritten = rewriteWikilinkText(entry.body, targetRaw, chosenSlug);
			if (rewritten !== entry.body) patch.body = rewritten;
		} else {
			const updated = rewriteFrontmatterField(entry.frontmatter[field], targetRaw, chosenSlug);
			if (updated !== undefined) frontmatterPatch[field] = updated;
		}
	}
	if (Object.keys(frontmatterPatch).length > 0) patch.frontmatter = frontmatterPatch;
	return patch;
}
