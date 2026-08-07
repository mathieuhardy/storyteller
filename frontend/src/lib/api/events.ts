// Server-Sent Events client (docs/api.md §5). The watcher pushes change events
// so views refresh without polling; the frontend treats each as a cache
// invalidation. `EventSource` reconnects on its own, so a dropped connection
// (server restart, sleep) recovers by itself — matching "a lagging client just
// reloads from the index" (docs/ui/states.md §6).

import { writable } from 'svelte/store';
import type { ServerEvent, ServerEventName } from './types';

const NAMES: ServerEventName[] = [
	'entity.created',
	'entity.updated',
	'entity.deleted',
	'assets.changed',
	'index.rebuilt'
];

/** The most recent event, for components that want to react in place. */
export const lastEvent = writable<ServerEvent | null>(null);

let source: EventSource | null = null;

/** Opens the event stream once. `onEvent` fires for every change (e.g. to
 * invalidate SvelteKit loads). Safe to call from `onMount`; a second call is a
 * no-op while a stream is live. */
export function connectEvents(onEvent?: (event: ServerEvent) => void): void {
	if (source || typeof window === 'undefined') return;
	source = new EventSource('/api/v1/events');
	for (const name of NAMES) {
		source.addEventListener(name, (raw) => {
			let data: unknown = null;
			try {
				data = JSON.parse((raw as MessageEvent).data);
			} catch {
				// A malformed payload is not worth breaking the stream over.
			}
			const event: ServerEvent = { name, data };
			lastEvent.set(event);
			onEvent?.(event);
		});
	}
}

/** Closes the stream (e.g. on teardown). */
export function disconnectEvents(): void {
	source?.close();
	source = null;
}
