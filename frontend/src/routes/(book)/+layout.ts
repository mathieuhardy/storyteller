// Book mode layout data loader. No project required — books are standalone.
import type { LayoutLoad } from './$types';

export const load: LayoutLoad = async () => {
	// Books don't require project info — they work with their own file API.
	return {};
};
