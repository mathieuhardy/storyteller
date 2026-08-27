// Settings / Types data: all types (enabled or not) plus entry counts.
import type { PageLoad } from './$types';
import { getTypes, getProject } from '$api/client';

export const load: PageLoad = async ({ fetch }) => {
	const [allTypes, project] = await Promise.all([
		getTypes({ all: true }, fetch),
		getProject(fetch)
	]);
	return { allTypes, countsByType: project.stats.by_type };
};
