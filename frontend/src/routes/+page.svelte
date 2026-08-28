<script lang="ts">
	// Launcher (docs/ui/layout.md §5): full-screen, no shell. Opens a project
	// folder or picks a recent one, then enters the dashboard. Local-first — no
	// account, just a folder.
	import { goto } from '$app/navigation';
	import Button from '$components/Button.svelte';
	import Icon from '$components/Icon.svelte';
	import { openProject, ApiError } from '$api/client';
	import { isTauri, pickFolder } from '$lib/tauri';
	import { t, formatNumber, formatRelative } from '$i18n/index.svelte';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	const nativePicker = isTauri();

	let showPathForm = $state(false);
	let pathInput = $state('');
	let opening = $state(false);
	let errorMessage = $state('');

	async function browse() {
		const path = await pickFolder();
		if (path) pathInput = path;
	}

	async function open(path: string) {
		if (!path.trim() || opening) return;
		opening = true;
		errorMessage = '';
		try {
			await openProject(path.trim());
			await goto('/dashboard');
		} catch (error) {
			// Surfaced globally below, whatever the entry point (recent item or path
			// form). Logged too, so the exact failure is visible in devtools.
			console.error('open project failed', error);
			errorMessage = error instanceof ApiError ? error.message : t('launcher.openError');
			opening = false;
		}
	}
</script>

<div class="launch">
	<section class="hero">
		<div class="brand">
			<Icon name="feather" size={22} />
			<span>{t('app.name')}</span>
		</div>
		<h1>{t('launcher.tagline')}</h1>
		<p class="subtitle">{t('launcher.subtitle')}</p>

		<div class="actions">
			<Button variant="primary" onclick={() => (showPathForm = !showPathForm)}>
				<Icon name="folder" size={16} />
				{t('action.openFolder')}
			</Button>
			<Button disabled title={t('screen.comingSoonBody')}>
				<Icon name="plus" size={15} />
				{t('action.newProject')}
			</Button>
		</div>

		{#if showPathForm}
			<form
				class="path-form"
				onsubmit={(e) => {
					e.preventDefault();
					open(pathInput);
				}}
			>
				<label for="project-path">{t('launcher.pathLabel')}</label>
				<div class="row">
					<input
						id="project-path"
						class="mono"
						bind:value={pathInput}
						placeholder="/home/…/my-novel"
						autocomplete="off"
						spellcheck="false"
					/>
					{#if nativePicker}
						<Button type="button" onclick={browse} disabled={opening}>
							<Icon name="folder" size={15} />
							{t('action.browseFolder')}
						</Button>
					{/if}
					<Button variant="primary" type="submit" disabled={opening}>
						{opening ? t('launcher.opening') : t('action.open')}
					</Button>
				</div>
				<p class="hint">{nativePicker ? t('launcher.pathHintDesktop') : t('launcher.pathHint')}</p>
			</form>
		{/if}

		{#if errorMessage}
			<p class="error" role="alert">{errorMessage}</p>
		{/if}

		<p class="local-first">{t('launcher.localFirst')}</p>
	</section>

	<section class="recent">
		<p class="rt">
			{t('launcher.recent')}
			{#if data.projects.items.length}<span class="cnt mono"
					>· {data.projects.items.length}</span
				>{/if}
		</p>

		{#if data.projects.items.length === 0}
			<p class="empty">{t('launcher.noRecent')}</p>
		{:else}
			<ul>
				{#each data.projects.items as project (project.path)}
					<li>
						<button class="recent-item" onclick={() => open(project.path)} disabled={opening}>
							<span class="cov" aria-hidden="true">{project.name.charAt(0).toUpperCase()}</span>
							<span class="rb">
								<span class="rn">{project.name}</span>
								<span class="rp mono">{project.path}</span>
							</span>
							<span class="rmeta">
								<span class="m1">{t('launcher.entries', { count: formatNumber(project.entries) })}</span
								>
								<span class="m2">{formatRelative(project.last_opened)}</span>
							</span>
							<span class="go" aria-hidden="true"><Icon name="chevron-right" size={16} /></span>
						</button>
					</li>
				{/each}
			</ul>
		{/if}
	</section>
</div>

<style>
	.launch {
		min-height: 100vh;
		display: grid;
		grid-template-columns: 1.05fr 1fr;
	}
	.hero {
		display: flex;
		flex-direction: column;
		justify-content: center;
		gap: 14px;
		padding: 64px clamp(32px, 6vw, 88px);
	}
	.brand {
		display: flex;
		align-items: center;
		gap: 9px;
		color: var(--accent);
		font-weight: 700;
		font-size: 15px;
	}
	.brand span {
		color: var(--text);
	}
	h1 {
		margin: 6px 0 0;
		font-size: 40px;
		font-weight: 700;
		line-height: 1.1;
		letter-spacing: -0.01em;
		white-space: pre-line;
		text-wrap: balance;
	}
	.subtitle {
		margin: 0;
		max-width: 46ch;
		color: var(--muted);
		font-size: 15px;
		line-height: 1.6;
	}
	.actions {
		display: flex;
		gap: 10px;
		margin-top: 10px;
	}
	.path-form {
		display: flex;
		flex-direction: column;
		gap: 6px;
		margin-top: 4px;
		max-width: 520px;
	}
	.path-form label {
		font-size: 12px;
		font-weight: 600;
		color: var(--muted);
	}
	.path-form .row {
		display: flex;
		gap: 8px;
	}
	.path-form input {
		flex: 1;
		height: 34px;
		padding: 0 11px;
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
		background: var(--surface);
		color: var(--text);
		font-size: 13px;
	}
	.path-form input:hover {
		border-color: var(--border-strong);
	}
	.hint {
		margin: 0;
		font-size: 11.5px;
		color: var(--faint);
		line-height: 1.5;
	}
	.error {
		margin: 0;
		font-size: 12.5px;
		color: var(--danger);
	}
	.local-first {
		margin-top: 18px;
		font-size: 12.5px;
		color: var(--faint);
	}
	.recent {
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 64px clamp(28px, 4vw, 56px);
		background: var(--surface);
		border-left: 1px solid var(--border);
	}
	.rt {
		margin: 0 0 6px;
		font-size: 12px;
		font-weight: 700;
		letter-spacing: 0.04em;
		text-transform: uppercase;
		color: var(--faint);
	}
	.cnt {
		color: var(--faint);
	}
	.empty {
		color: var(--faint);
		font-size: 13.5px;
	}
	ul {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.recent-item {
		display: flex;
		align-items: center;
		gap: 14px;
		width: 100%;
		padding: 13px 14px;
		border: 1px solid transparent;
		border-radius: var(--radius);
		background: transparent;
		text-align: left;
		cursor: pointer;
		color: var(--text);
		font: inherit;
	}
	.recent-item:hover {
		background: var(--bg);
		border-color: var(--border);
		box-shadow: var(--shadow);
	}
	.recent-item:disabled {
		opacity: 0.6;
		pointer-events: none;
	}
	.cov {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 42px;
		height: 42px;
		flex: none;
		border-radius: 10px;
		background: radial-gradient(120% 120% at 20% 10%, #cdd3fb, #8b8ce8 55%, #5b5bd6);
		color: #fff;
		font-weight: 700;
		font-size: 17px;
	}
	.rb {
		flex: 1;
		min-width: 0;
	}
	.rn {
		display: block;
		font-weight: 600;
		font-size: 14.5px;
	}
	.rp {
		display: block;
		font-size: 11.5px;
		color: var(--faint);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.rmeta {
		text-align: right;
		font-size: 11.5px;
		color: var(--muted);
		white-space: nowrap;
	}
	.rmeta .m2 {
		display: block;
		color: var(--faint);
	}
	.go {
		color: var(--faint);
		display: inline-flex;
	}
	.recent-item:hover .go {
		color: var(--accent);
	}
	@media (max-width: 820px) {
		.launch {
			grid-template-columns: 1fr;
		}
		.recent {
			border-left: 0;
			border-top: 1px solid var(--border);
		}
	}
</style>
