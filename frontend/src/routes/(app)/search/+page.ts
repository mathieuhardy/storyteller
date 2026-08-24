// Search screen data (docs/api.md §3 "Search"): forwards the URL's query
// string straight to GET /search — `q` plus whatever `type=`/`tag=`/`page=`
// the toolbar/pager add, same URL-driven approach as the list/table screen.
import type { PageLoad } from './$types';
import { search } from '$api/client';
import type { Page, SearchResult } from '$api/types';

export const load: PageLoad = async ({ url, fetch }) => {
	const q = url.searchParams.get('q') ?? '';
	if (!q.trim()) {
		return { q, results: { items: [], page: 1, per_page: 50, total: 0 } as Page<SearchResult> };
	}

	const results = await search(`?${url.searchParams.toString()}`, fetch);
	return { q, results };
};
