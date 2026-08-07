<script lang="ts">
	// Top bar (docs/ui/layout.md §2.1): brand · breadcrumb · flexible space ·
	// search (reserved for M5) · links-panel toggle · language · theme · primary
	// action. The search field is present but disabled until FTS lands (M5).
	import Icon from './Icon.svelte';
	import IconButton from './IconButton.svelte';
	import Button from './Button.svelte';
	import ThemeToggle from './ThemeToggle.svelte';
	import LanguageSelect from './LanguageSelect.svelte';
	import { t } from '$i18n/index.svelte';

	let {
		breadcrumb = [],
		linksOpen = false,
		onToggleLinks
	}: {
		breadcrumb?: { label: string; href?: string }[];
		linksOpen?: boolean;
		onToggleLinks?: () => void;
	} = $props();
</script>

<header class="topbar">
	<a class="brand" href="/">
		<Icon name="feather" size={18} />
		<span class="brand-name">{t('app.name')}</span>
	</a>

	<nav class="crumbs" aria-label="Breadcrumb">
		{#each breadcrumb as crumb, i (crumb.label)}
			{#if i > 0}<span class="sep" aria-hidden="true"><Icon name="chevron-right" size={13} /></span
				>{/if}
			{#if crumb.href && i < breadcrumb.length - 1}
				<a class="crumb" href={crumb.href}>{crumb.label}</a>
			{:else}
				<span class="crumb current">{crumb.label}</span>
			{/if}
		{/each}
	</nav>

	<div class="spacer"></div>

	<label class="search" title={t('search.comingSoon')}>
		<Icon name="search" size={15} />
		<input type="search" placeholder={t('search.placeholder')} disabled />
		<span class="soon">M5</span>
	</label>

	<IconButton label={t('links.toggle')} active={linksOpen} onclick={() => onToggleLinks?.()}>
		<Icon name="link" />
	</IconButton>

	<LanguageSelect />
	<ThemeToggle />

	<Button variant="primary" size="sm" disabled title={t('search.comingSoon')}>
		<Icon name="plus" size={15} />
		{t('action.newEntry')}
	</Button>
</header>

<style>
	.topbar {
		display: flex;
		align-items: center;
		gap: 10px;
		height: 48px;
		padding: 0 12px;
		background: var(--surface);
		border-bottom: 1px solid var(--border);
	}
	.brand {
		display: flex;
		align-items: center;
		gap: 7px;
		color: var(--accent);
		text-decoration: none;
		font-weight: 700;
	}
	.brand-name {
		color: var(--text);
		font-size: 15px;
	}
	.crumbs {
		display: flex;
		align-items: center;
		gap: 4px;
		min-width: 0;
		overflow: hidden;
	}
	.sep {
		display: inline-flex;
		color: var(--faint);
	}
	.crumb {
		font-size: 13px;
		color: var(--muted);
		text-decoration: none;
		white-space: nowrap;
	}
	.crumb:hover {
		color: var(--text);
	}
	.crumb.current {
		color: var(--text);
		font-weight: 550;
	}
	.spacer {
		flex: 1;
	}
	.search {
		display: flex;
		align-items: center;
		gap: 7px;
		height: 30px;
		padding: 0 9px;
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
		background: var(--surface-2);
		color: var(--faint);
		max-width: 260px;
	}
	.search input {
		border: 0;
		background: transparent;
		font: inherit;
		font-size: 13px;
		color: var(--text);
		width: 150px;
	}
	.search input:disabled {
		cursor: not-allowed;
	}
	.soon {
		font-family: var(--font-mono);
		font-size: 10px;
		color: var(--faint);
		border: 1px solid var(--border);
		border-radius: 4px;
		padding: 1px 4px;
	}
	@media (max-width: 900px) {
		.search input,
		.search .soon {
			display: none;
		}
	}
	@media (max-width: 720px) {
		.crumbs {
			display: none;
		}
	}
</style>
