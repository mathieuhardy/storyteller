// Media gallery data (docs/ui/screens.md §6): the full asset list, straight
// from `GET /assets` — no pagination on that endpoint, filtering is client-side.
import type { PageLoad } from './$types';
import { getAssets } from '$api/client';

export const load: PageLoad = async ({ fetch }) => {
	const assets = await getAssets(fetch);
	return { assets };
};
