<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import { invalidateAll, beforeNavigate, afterNavigate } from '$app/navigation';
	import { initTheme } from '$lib/stores/theme.svelte';
	import { initLocale } from '$i18n/index.svelte';
	import { connectEvents, disconnectEvents } from '$api/events';

	let { children } = $props();

	// Track in-flight navigation so an SSE refresh never clobbers it (see below).
	let navigating = false;
	beforeNavigate(() => (navigating = true));
	afterNavigate(() => (navigating = false));

	let refreshTimer: ReturnType<typeof setTimeout> | undefined;

	// The watcher's SSE stream drives refreshes: a change re-runs the load
	// functions so nav counts and lists stay in sync without polling
	// (docs/ui/states.md §6). Two guards make this safe:
	//  - debounce, to fold a burst of watcher events into one refresh;
	//  - skip while navigating, because a concurrent `invalidateAll()` supersedes
	//    an in-flight `goto` (e.g. opening a project emits `index.rebuilt` right
	//    as the launcher navigates to the dashboard). We reschedule until the
	//    navigation settles, then refresh the destination.
	function scheduleRefresh() {
		clearTimeout(refreshTimer);
		refreshTimer = setTimeout(() => {
			if (navigating) {
				scheduleRefresh();
				return;
			}
			invalidateAll();
		}, 150);
	}

	onMount(() => {
		initTheme();
		initLocale();
		connectEvents(() => scheduleRefresh());
		return () => {
			clearTimeout(refreshTimer);
			disconnectEvents();
		};
	});
</script>

{@render children()}
