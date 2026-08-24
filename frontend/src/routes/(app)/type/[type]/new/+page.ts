// Entry editor data, create mode (docs/ui/screens.md §4): only the type schema
// is needed — there is no entry yet.
import type { PageLoad } from './$types';
import { getType } from '$api/client';

export const load: PageLoad = async ({ params, fetch }) => {
	const type = params.type;
	const schema = await getType(type, fetch);
	return { type, schema };
};
