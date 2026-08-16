// Entry detail screen data (docs/ui/screens.md §2): the full entry with
// backlinks, plus outgoing links and type schema for attribute rendering.
import type { PageLoad } from './$types';
import { getEntity, getLinks, getType } from '$api/client';

export const load: PageLoad = async ({ params, fetch }) => {
	const slug = params.slug;

	const entry = await getEntity(slug, { backlinks: true }, fetch);

	const [links, schema] = await Promise.all([
		getLinks(slug, fetch),
		getType(entry.type, fetch)
	]);

	return { entry, links, schema };
};
