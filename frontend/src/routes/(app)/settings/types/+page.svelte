<script lang="ts">
	// Settings / Types screen: enable or disable types for entry creation.
	// Types are grouped into built-in and custom. Each row shows the type
	// monogram, label, folder, entry count, and a toggle switch.
	import { t, typeLabel, formatNumber } from '$i18n/index.svelte';
	import { setTypeEnabled, ApiError } from '$api/client';
	import Icon from '$components/Icon.svelte';
	import type { TypeResponse } from '$api/types';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	// Local state mirrors server state; optimistic updates with rollback.
	let types = $state<TypeResponse[]>([...data.allTypes]);
	let error = $state<string | null>(null);

	const builtIn = $derived(types.filter((t) => !t.custom));
	const custom = $derived(types.filter((t) => t.custom));

	function monogram(label: string): string {
		return label.trim().charAt(0).toUpperCase() || '?';
	}

	async function toggle(type: TypeResponse) {
		const prev = type.enabled;
		const idx = types.findIndex((t) => t.name === type.name);
		if (idx === -1) return;

		// Optimistic update
		types[idx] = { ...type, enabled: !prev };
		error = null;

		try {
			const updated = await setTypeEnabled(type.name, !prev);
			types[idx] = updated;
		} catch (e) {
			// Rollback
			types[idx] = { ...type, enabled: prev };
			error = e instanceof ApiError ? e.message : t('settings.types.errorUpdating');
		}
	}
</script>

<div class="screen">
	<div class="head">
		<h1><Icon name="settings" size={19} />{t('settings.types.title')}</h1>
		<p class="sub">{t('settings.types.subtitle')}</p>
	</div>

	{#if error}
		<p class="error">{error}</p>
	{/if}

	<section class="group">
		<h2 class="group-label">{t('settings.types.builtIn')}</h2>
		<ul class="list">
			{#each builtIn as type (type.name)}
				{@const count = data.countsByType[type.name] ?? 0}
				<li class="row">
					<span class="mono-tile" aria-hidden="true">{monogram(typeLabel(type.name))}</span>
					<span class="info">
						<span class="label">{typeLabel(type.name)}</span>
						<span class="meta">
							<span class="folder">{t('settings.types.folder', { folder: type.folder })}</span>
							<span class="count">{t('settings.types.entryCount', { count: formatNumber(count) })}</span>
						</span>
					</span>
					<button
						type="button"
						role="switch"
						aria-checked={type.enabled}
						class="toggle"
						class:on={type.enabled}
						onclick={() => toggle(type)}
					>
						<span class="toggle-knob"></span>
						<span class="sr-only">
							{type.enabled ? t('settings.types.enabled') : t('settings.types.disabled')}
						</span>
					</button>
				</li>
			{/each}
		</ul>
	</section>

	<section class="group">
		<h2 class="group-label">{t('settings.types.custom')}</h2>
		{#if custom.length === 0}
			<p class="empty">{t('settings.types.noCustom')}</p>
		{:else}
			<ul class="list">
				{#each custom as type (type.name)}
					{@const count = data.countsByType[type.name] ?? 0}
					<li class="row custom">
						<span class="mono-tile stub" aria-hidden="true">{monogram(typeLabel(type.name))}</span>
						<span class="info">
							<span class="label">{typeLabel(type.name)}</span>
							<span class="meta">
								<span class="folder">{t('settings.types.folder', { folder: type.folder })}</span>
								<span class="count">{t('settings.types.entryCount', { count: formatNumber(count) })}</span>
							</span>
						</span>
						<button
							type="button"
							role="switch"
							aria-checked={type.enabled}
							class="toggle"
							class:on={type.enabled}
							onclick={() => toggle(type)}
						>
							<span class="toggle-knob"></span>
							<span class="sr-only">
								{type.enabled ? t('settings.types.enabled') : t('settings.types.disabled')}
							</span>
						</button>
					</li>
				{/each}
			</ul>
		{/if}
	</section>
</div>

<style>
	.screen {
		max-width: 640px;
		margin: 0 auto;
		padding: 22px 26px 60px;
	}
	.head h1 {
		display: flex;
		align-items: center;
		gap: 10px;
		font-size: 21px;
		letter-spacing: -0.02em;
		margin: 0;
		font-weight: 660;
	}
	.head .sub {
		color: var(--muted);
		font-size: 13px;
		margin-top: 6px;
		max-width: 62ch;
	}
	.error {
		color: var(--danger);
		font-size: 13px;
		margin: 16px 0 0;
		padding: 10px 14px;
		background: var(--danger-soft);
		border-radius: var(--radius-sm);
	}
	.group {
		margin-top: 28px;
	}
	.group-label {
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		color: var(--faint);
		margin: 0 0 10px;
	}
	.list {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 2px;
	}
	.row {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 10px 12px;
		border-radius: var(--radius);
		background: var(--surface);
		border: 1px solid var(--border);
	}
	.row:hover {
		border-color: var(--border-strong);
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
	.mono-tile.stub {
		background: var(--stub-soft);
		color: var(--stub);
	}
	.info {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: 2px;
	}
	.label {
		font-size: 14px;
		font-weight: 560;
		color: var(--text);
	}
	.meta {
		display: flex;
		align-items: center;
		gap: 12px;
		font-size: 12px;
		color: var(--muted);
	}
	.folder {
		font-family: var(--font-mono);
		font-size: 11.5px;
	}
	.empty {
		color: var(--muted);
		font-size: 13px;
		padding: 20px 0;
	}

	/* Toggle switch */
	.toggle {
		position: relative;
		width: 40px;
		height: 22px;
		padding: 0;
		border: 0;
		border-radius: 999px;
		background: var(--border-strong);
		cursor: pointer;
		transition: background 0.15s ease;
	}
	.toggle.on {
		background: var(--accent);
	}
	.toggle-knob {
		position: absolute;
		top: 2px;
		left: 2px;
		width: 18px;
		height: 18px;
		border-radius: 50%;
		background: white;
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.15);
		transition: transform 0.15s ease;
	}
	.toggle.on .toggle-knob {
		transform: translateX(18px);
	}
	.sr-only {
		position: absolute;
		width: 1px;
		height: 1px;
		padding: 0;
		margin: -1px;
		overflow: hidden;
		clip: rect(0, 0, 0, 0);
		white-space: nowrap;
		border: 0;
	}
</style>
