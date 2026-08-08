<script lang="ts">
	// Dashboard screen (docs/ui/screens.md §1): project overview with hero, stats,
	// recent entries, distribution chart, stubs preview, and index health.
	import type { PageData } from './$types';
	import { t } from '$i18n/index.svelte';

	import HeroSection from '$components/dashboard/HeroSection.svelte';
	import StatCard from '$components/dashboard/StatCard.svelte';
	import RecentEntries from '$components/dashboard/RecentEntries.svelte';
	import DistributionChart from '$components/dashboard/DistributionChart.svelte';
	import StubsPreview from '$components/dashboard/StubsPreview.svelte';
	import IndexHealth from '$components/dashboard/IndexHealth.svelte';

	let { data }: { data: PageData } = $props();

	// Parent layout provides project data
	const project = $derived(data.project);
	const stats = $derived(project.stats);

	// Dashboard-specific data from our load
	const recentEntries = $derived(data.recentEntries);
	const stubs = $derived(data.stubs);

	// Calculate derived stats
	const chapterCount = $derived(stats.by_type['chapter'] ?? 0);
	const stubCount = $derived(stubs.length);
	const errorCount = $derived(
		recentEntries.items.filter((e) => e.has_errors).length
	);
</script>

<div class="dashboard">
	<!-- Hero: project overview -->
	<HeroSection entry={project.entry} projectRoot={project.root} />

	<!-- Stats grid: 4 key metrics -->
	<div class="stats-grid">
		<StatCard
			value={stats.entries}
			label={t('dashboard.totalEntries')}
			href="/type/note"
		/>
		<StatCard
			value={chapterCount}
			label={t('dashboard.chapters')}
			href="/type/chapter"
		/>
		<StatCard
			value={stubCount}
			label={t('dashboard.toCreate')}
			variant="stub"
			href="/stubs"
		/>
		<StatCard
			value={errorCount}
			label={t('dashboard.toReview')}
			variant={errorCount > 0 ? 'danger' : 'default'}
		/>
	</div>

	<!-- Content grid: two columns -->
	<div class="content-grid">
		<!-- Left column: recent entries + distribution -->
		<div class="left-col">
			<RecentEntries entries={recentEntries.items} />
			<DistributionChart byType={stats.by_type} />
		</div>

		<!-- Right column: stubs preview + index health -->
		<div class="right-col">
			<StubsPreview {stubs} />
			<IndexHealth />
		</div>
	</div>
</div>

<style>
	.dashboard {
		max-width: 940px;
		margin: 0 auto;
		padding: 24px 20px;
	}

	.stats-grid {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		gap: 12px;
		margin-bottom: 22px;
	}

	.content-grid {
		display: grid;
		grid-template-columns: 1.4fr 1fr;
		gap: 22px;
	}

	.left-col,
	.right-col {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}

	/* Responsive: tablet and below */
	@media (max-width: 820px) {
		.stats-grid {
			grid-template-columns: repeat(2, 1fr);
		}
		.content-grid {
			grid-template-columns: 1fr;
		}
	}
</style>
