import type { PageLoad } from './$types';
import { listProjects } from '$api/client';

// The launcher lists previously-opened projects and books. A failure here
// (server down) degrades to an empty list rather than a broken screen.
export const load: PageLoad = async ({ fetch }) => {
	try {
		return { projects: await listProjects(fetch) };
	} catch {
		return { projects: { items: [], books: [], active: '', active_kind: null } };
	}
};
