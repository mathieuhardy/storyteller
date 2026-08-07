import type { LayoutLoad } from './$types';
import { getProject, getStubs } from '$api/client';

// Feeds the shell's navigation: the project (types + counts) and the workshop
// badge (stub count). Re-runs on every SSE change via `invalidateAll`.
export const load: LayoutLoad = async ({ fetch }) => {
	const project = await getProject(fetch);
	let stubsCount = 0;
	try {
		stubsCount = (await getStubs(fetch)).length;
	} catch {
		stubsCount = 0;
	}
	return { project, stubsCount };
};
