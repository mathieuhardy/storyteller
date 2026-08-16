// Rail content store: allows entry pages to provide backlinks, outgoing links,
// and metadata to the collapsible rail panel. The rail reads this reactive state
// and updates when a new entry is viewed.

import type { Entry, OutgoingLink, Backlink } from '$api/types';

interface RailState {
	/** The currently viewed entry, if any. */
	entry: Entry | null;
	/** Outgoing links from the entry. */
	links: OutgoingLink[];
	/** Backlinks are included in entry.backlinks, but we expose them here for clarity. */
	backlinks: Backlink[];
}

let state = $state<RailState>({
	entry: null,
	links: [],
	backlinks: []
});

/** Set the rail content when viewing an entry. */
export function setRailEntry(entry: Entry, links: OutgoingLink[]): void {
	state = {
		entry,
		links,
		backlinks: entry.backlinks ?? []
	};
}

/** Clear the rail content when leaving an entry view. */
export function clearRailEntry(): void {
	state = {
		entry: null,
		links: [],
		backlinks: []
	};
}

/** Get the current rail state (reactive). */
export function getRailState(): RailState {
	return state;
}
