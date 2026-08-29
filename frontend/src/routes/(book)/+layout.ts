// Book mode layout data loader. Fetches project info for file tree.
import type { LayoutLoad } from './$types';
import { getProject } from '$api/client';

export const load: LayoutLoad = async ({ fetch }) => {
	const project = await getProject(fetch);
	return { project };
};
