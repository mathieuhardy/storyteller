// Dashboard data fetching: recent entries and full stubs list. The parent
// layout already provides `project` and `stubsCount`, so we just add the
// details needed for the dashboard cards.

import type { PageLoad } from './$types';
import { getEntities, getStubs } from '$api/client';
import type { Page, EntrySummary, Stub } from '$api/types';

export const load: PageLoad = async ({ fetch }) => {
	// Fetch recent entries and stubs in parallel
	const [recentEntries, stubs] = await Promise.all([
		getEntities('?sort=-updated&per_page=5', fetch).catch(
			(): Page<EntrySummary> => ({
				items: [],
				page: 1,
				per_page: 5,
				total: 0
			})
		),
		getStubs(fetch).catch((): Stub[] => [])
	]);

	return {
		recentEntries,
		stubs
	};
};
