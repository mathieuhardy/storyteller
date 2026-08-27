// Links workshop data (docs/ui/screens.md §5): stubs come straight from
// `GET /stubs`. Ambiguous links have no dedicated aggregate endpoint — the
// screen builds it the way the spec describes, from `GET /entities/{slug}/links`
// across every entry — fine at the single-user/local scale this app targets
// (docs/principles.md); a real search index (M5) would be the place to move
// this server-side if projects grow far past that.
import type { PageLoad } from './$types';
import { getStubs, getEntities, getLinks, getTypes } from '$api/client';
import type { EntrySummary, OutgoingLink, Page } from '$api/types';
import type { AmbiguousGroup } from '$lib/workshop/link-rewrite';

async function fetchAllEntities(fetchImpl: typeof fetch): Promise<EntrySummary[]> {
	const items: EntrySummary[] = [];
	let page = 1;
	// eslint-disable-next-line no-constant-condition
	while (true) {
		const result: Page<EntrySummary> = await getEntities(`?per_page=200&page=${page}`, fetchImpl);
		items.push(...result.items);
		if (items.length >= result.total || result.items.length === 0) break;
		page += 1;
	}
	return items;
}

export const load: PageLoad = async ({ fetch }) => {
	const [stubs, entities, types] = await Promise.all([
		getStubs(fetch),
		fetchAllEntities(fetch),
		getTypes({}, fetch)
	]);

	const linksByEntity = await Promise.all(
		entities.map((entity) => getLinks(entity.slug, fetch).catch((): OutgoingLink[] => []))
	);

	const groups = new Map<string, AmbiguousGroup>();
	entities.forEach((entity, i) => {
		for (const link of linksByEntity[i]) {
			if (link.resolution !== 'ambiguous') continue;
			let group = groups.get(link.target_raw);
			if (!group) {
				group = { targetRaw: link.target_raw, candidateSlugs: link.candidates ?? [], occurrences: [] };
				groups.set(link.target_raw, group);
			}
			group.occurrences.push({
				slug: entity.slug,
				title: entity.title,
				type: entity.type,
				field: link.field
			});
		}
	});

	return {
		stubs,
		ambiguous: Array.from(groups.values()),
		entities,
		types: types.filter((t) => t.enabled)
	};
};
