<script lang="ts">
	// Left navigation (docs/ui/layout.md §2.2): a clickable project header that
	// reopens the launcher, the enabled types with their counts, and the workshop
	// ("to create" / stubs). Only enabled types are listed.
	import Icon from './Icon.svelte';
	import { typeLabel, formatNumber, t } from '$i18n/index.svelte';

	let {
		projectName,
		entriesCount,
		types,
		stubsCount = 0,
		activeType = null,
		activeWorkshop = false,
		activeGallery = false,
		activeGraph = false,
		activeSettings = false
	}: {
		projectName: string;
		entriesCount: number;
		types: { name: string; count: number }[];
		stubsCount?: number;
		activeType?: string | null;
		activeWorkshop?: boolean;
		activeGallery?: boolean;
		activeGraph?: boolean;
		activeSettings?: boolean;
	} = $props();

	function monogram(label: string): string {
		return label.trim().charAt(0).toUpperCase() || '?';
	}
</script>

<nav class="nav">
	<a class="project" href="/" title={projectName}>
		<span class="mono-tile" aria-hidden="true">{monogram(projectName)}</span>
		<span class="pmeta">
			<span class="pname">{projectName}</span>
			<span class="pcount mono">{t('launcher.entries', { count: formatNumber(entriesCount) })}</span>
		</span>
	</a>

	<div class="section">
		<p class="label">{t('nav.types')}</p>
		<ul>
			{#each types as type (type.name)}
				<li>
					<a
						class="item"
						class:active={activeType === type.name}
						href="/type/{type.name}"
						aria-current={activeType === type.name ? 'page' : undefined}
					>
						<span class="mono-tile sm" aria-hidden="true">{monogram(typeLabel(type.name))}</span>
						<span class="iname">{typeLabel(type.name)}</span>
						<span class="count mono">{formatNumber(type.count)}</span>
					</a>
				</li>
			{/each}
		</ul>
	</div>

	<div class="section">
		<p class="label">{t('nav.workshop')}</p>
		<a class="item" class:active={activeWorkshop} href="/stubs">
			<span class="stub-ico" aria-hidden="true"><Icon name="square-plus" size={15} /></span>
			<span class="iname">{t('nav.stubs')}</span>
			{#if stubsCount > 0}
				<span class="count stub mono">{formatNumber(stubsCount)}</span>
			{/if}
		</a>
	</div>

	<div class="section">
		<p class="label">{t('nav.media')}</p>
		<a class="item" class:active={activeGallery} href="/gallery">
			<span class="media-ico" aria-hidden="true"><Icon name="image" size={15} /></span>
			<span class="iname">{t('nav.gallery')}</span>
		</a>
	</div>

	<div class="section">
		<p class="label">{t('nav.explore')}</p>
		<a class="item" class:active={activeGraph} href="/graph">
			<span class="media-ico" aria-hidden="true"><Icon name="network" size={15} /></span>
			<span class="iname">{t('nav.graph')}</span>
		</a>
	</div>

	<div class="section">
		<a class="item writing" href="/book">
			<span class="media-ico" aria-hidden="true"><Icon name="edit" size={15} /></span>
			<span class="iname">{t('nav.writing')}</span>
		</a>
	</div>

	<div class="section">
		<p class="label">{t('nav.settings')}</p>
		<a class="item" class:active={activeSettings} href="/settings/types">
			<span class="media-ico" aria-hidden="true"><Icon name="settings" size={15} /></span>
			<span class="iname">{t('settings.types.title')}</span>
		</a>
	</div>
</nav>

<style>
	.nav {
		display: flex;
		flex-direction: column;
		gap: 6px;
		height: 100%;
		padding: 10px;
		overflow-y: auto;
		background: var(--surface);
		border-right: 1px solid var(--border);
	}
	.project {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 8px;
		border-radius: var(--radius);
		text-decoration: none;
		color: var(--text);
	}
	.project:hover {
		background: var(--surface-2);
	}
	.pmeta {
		min-width: 0;
	}
	.pname {
		display: block;
		font-weight: 640;
		font-size: 14px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.pcount {
		font-size: 11.5px;
		color: var(--faint);
	}
	.section {
		display: flex;
		flex-direction: column;
		gap: 2px;
		margin-top: 6px;
	}
	.label {
		margin: 6px 8px 2px;
		font-size: 10.5px;
		font-weight: 600;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		color: var(--faint);
	}
	ul {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 2px;
	}
	.item {
		display: flex;
		align-items: center;
		gap: 9px;
		padding: 6px 8px;
		border-radius: var(--radius-sm);
		text-decoration: none;
		color: var(--text);
		font-size: 13.5px;
	}
	.item:hover {
		background: var(--surface-2);
	}
	.item.active {
		background: var(--accent-soft);
		color: var(--accent);
		font-weight: 600;
	}
	.iname {
		flex: 1;
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.count {
		font-size: 11.5px;
		color: var(--faint);
	}
	.count.stub {
		color: var(--stub);
		background: var(--stub-soft);
		padding: 1px 7px;
		border-radius: 999px;
	}
	.mono-tile {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		flex: none;
		width: 32px;
		height: 32px;
		border-radius: 9px;
		background: var(--accent-soft);
		color: var(--accent);
		font-weight: 700;
		font-size: 14px;
	}
	.mono-tile.sm {
		width: 24px;
		height: 24px;
		border-radius: 7px;
		font-size: 12px;
	}
	.stub-ico {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border-radius: 7px;
		background: var(--stub-soft);
		color: var(--stub);
	}
	.media-ico {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border-radius: 7px;
		background: var(--accent-soft);
		color: var(--accent);
	}
</style>
