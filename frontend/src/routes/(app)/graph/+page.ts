// Link graph data (docs/ui/screens.md §7): one request, `GET /graph`, gives
// every node and edge — layout is computed client-side (ADR 0018).
import type { PageLoad } from './$types';
import { getGraph } from '$api/client';

export const load: PageLoad = async ({ fetch }) => {
	const graph = await getGraph(fetch);
	return { graph };
};
