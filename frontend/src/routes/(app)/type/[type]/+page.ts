// List/table screen data (docs/ui/screens.md §3): the type's field schema
// (columns/filters/sort options) plus the current page of entries. Filters,
// sort and pagination all live in the URL's query string — this load simply
// forwards it to the API, adding the path's `type` and a default sort so the
// screen opens on something sensible.
import type { PageLoad } from './$types';
import { getEntities, getType } from '$api/client';
import type { EntrySummary, Page } from '$api/types';

export const load: PageLoad = async ({ params, url, fetch }) => {
	const type = params.type;

	const apiParams = new URLSearchParams(url.searchParams);
	apiParams.set('type', type);
	if (!apiParams.has('sort')) apiParams.set('sort', '-updated');

	const [schema, entries] = await Promise.all([
		getType(type, fetch),
		getEntities(`?${apiParams.toString()}`, fetch).catch(
			(): Page<EntrySummary> => ({ items: [], page: 1, per_page: 50, total: 0 })
		)
	]);

	return { type, schema, entries };
};
