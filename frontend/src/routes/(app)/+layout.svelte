<script lang="ts">
	// Application shell (docs/ui/layout.md §2): a full-width topbar over a
	// three-zone grid — nav · main · collapsible rail. Business screens render in
	// `main`; the rail pushes it open on wide screens and overlays below 1080px.
	import { page } from '$app/state';
	import Topbar from '$components/Topbar.svelte';
	import Nav from '$components/Nav.svelte';
	import Rail from '$components/Rail.svelte';
	import { typeLabel, t } from '$i18n/index.svelte';
	import type { LayoutData } from './$types';

	let { data, children }: { data: LayoutData; children: import('svelte').Snippet } = $props();

	let railOpen = $state(false);

	const projectName = $derived(
		(typeof data.project.entry?.frontmatter?.title === 'string' &&
			(data.project.entry.frontmatter.title as string)) ||
			data.project.root.split('/').filter(Boolean).pop() ||
			data.project.root
	);

	const types = $derived(
		data.project.enabled_types.map((name) => ({
			name,
			count: data.project.stats.by_type[name] ?? 0
		}))
	);

	const activeType = $derived(page.params.type ?? null);
	const activeSlug = $derived(page.params.slug ?? null);
	const activeWorkshop = $derived(page.url.pathname === '/stubs');
	const activeSearch = $derived(page.url.pathname === '/search');
	const activeGallery = $derived(page.url.pathname === '/gallery');
	const activeGraph = $derived(page.url.pathname === '/graph');
	const activeSettings = $derived(page.url.pathname.startsWith('/settings'));

	const breadcrumb = $derived.by(() => {
		const crumbs: { label: string; href?: string }[] = [
			{ label: projectName, href: '/dashboard' }
		];
		if (activeSlug) {
			// Entry view: we don't have the entry type here, so just show the slug
			// A proper implementation would require loading the entry data at layout level
			crumbs.push({ label: activeSlug });
		} else if (activeType) {
			crumbs.push({ label: typeLabel(activeType) });
		} else if (activeWorkshop) {
			crumbs.push({ label: t('nav.stubs') });
		} else if (activeSearch) {
			crumbs.push({ label: t('search.action') });
		} else if (activeGallery) {
			crumbs.push({ label: t('nav.gallery') });
		} else if (activeGraph) {
			crumbs.push({ label: t('nav.graph') });
		} else if (activeSettings) {
			crumbs.push({ label: t('nav.settings') });
			if (page.url.pathname === '/settings/types') {
				crumbs.push({ label: t('settings.types.title') });
			}
		} else {
			crumbs.push({ label: t('dashboard.title') });
		}
		return crumbs;
	});
</script>

<div class="shell">
	<Topbar
		{breadcrumb}
		linksOpen={railOpen}
		newEntryTypes={data.project.enabled_types}
		onToggleLinks={() => (railOpen = !railOpen)}
	/>
	<div class="cols" class:rail-open={railOpen}>
		<Nav
			{projectName}
			entriesCount={data.project.stats.entries}
			{types}
			stubsCount={data.stubsCount}
			{activeType}
			{activeWorkshop}
			{activeGallery}
			{activeGraph}
			{activeSettings}
		/>
		<main>
			{@render children()}
		</main>
		<div class="rail-slot" inert={!railOpen} aria-hidden={!railOpen}>
			<Rail onClose={() => (railOpen = false)} />
		</div>
	</div>
</div>

<style>
	.shell {
		display: grid;
		grid-template-rows: 48px 1fr;
		height: 100vh;
		min-height: 640px;
	}
	.cols {
		display: grid;
		grid-template-columns: 244px minmax(0, 1fr) 0;
		min-height: 0;
		transition: grid-template-columns 0.2s ease;
	}
	.cols.rail-open {
		grid-template-columns: 244px minmax(0, 1fr) 312px;
	}
	main {
		min-width: 0;
		overflow-y: auto;
		overflow-x: hidden;
	}
	.rail-slot {
		min-width: 0;
		overflow: hidden;
	}

	@media (max-width: 1080px) {
		/* Rail overlays instead of compressing the content. */
		.cols,
		.cols.rail-open {
			grid-template-columns: 244px minmax(0, 1fr) 0;
		}
		.rail-slot {
			position: fixed;
			top: 48px;
			right: 0;
			bottom: 0;
			width: 312px;
			box-shadow: var(--shadow-lg);
			transform: translateX(100%);
			transition: transform 0.2s ease;
			z-index: 20;
		}
		.cols.rail-open .rail-slot {
			transform: translateX(0);
		}
	}
</style>
