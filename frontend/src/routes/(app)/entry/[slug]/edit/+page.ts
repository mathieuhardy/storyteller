// Entry editor data, edit mode (docs/ui/screens.md §4): the full entry, its
// outgoing links (to render existing wikilink tokens with their resolution
// state), and the type schema that drives the generated form.
import type { PageLoad } from './$types';
import { getEntity, getLinks, getType } from '$api/client';

export const load: PageLoad = async ({ params, fetch }) => {
	const slug = params.slug;

	const entry = await getEntity(slug, {}, fetch);
	const [links, schema] = await Promise.all([getLinks(slug, fetch), getType(entry.type, fetch)]);

	return { entry, links, schema };
};
