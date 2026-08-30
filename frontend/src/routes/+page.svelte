<script lang="ts">
	// Launcher (docs/ui/layout.md §5): full-screen, no shell. Opens a project
	// folder or picks a recent one, then enters the dashboard. Local-first — no
	// account, just a folder.
	import { goto } from '$app/navigation';
	import Button from '$components/Button.svelte';
	import Icon from '$components/Icon.svelte';
	import { openProject, openBook, removeRecent, ApiError } from '$api/client';
	import { isTauri, pickFolder } from '$lib/tauri';
	import { t, formatNumber, formatRelative } from '$i18n/index.svelte';
	import type { PageData } from './$types';
	import type { ProjectRecord } from '$api/types';

	let { data }: { data: PageData } = $props();

	const nativePicker = isTauri();

	// Local copies of the lists (for optimistic removal)
	let projects = $state<ProjectRecord[]>([...data.projects.items]);
	let books = $state<ProjectRecord[]>([...data.projects.books]);

	// Project state
	let showProjectForm = $state(false);
	let projectPathInput = $state('');
	let openingProject = $state(false);
	let projectError = $state('');

	// Book state
	let showBookForm = $state(false);
	let bookPathInput = $state('');
	let openingBook = $state(false);
	let bookError = $state('');

	async function browseProject() {
		const path = await pickFolder();
		if (path) projectPathInput = path;
	}

	async function browseBook() {
		const path = await pickFolder();
		if (path) bookPathInput = path;
	}

	async function openProjectFolder(path: string) {
		if (!path.trim() || openingProject) return;
		openingProject = true;
		projectError = '';
		try {
			await openProject(path.trim());
			await goto('/dashboard');
		} catch (error) {
			console.error('open project failed', error);
			projectError = error instanceof ApiError ? error.message : t('launcher.openError');
			openingProject = false;
		}
	}

	async function openBookFolder(path: string) {
		if (!path.trim() || openingBook) return;
		openingBook = true;
		bookError = '';
		try {
			await openBook(path.trim());
			await goto('/book');
		} catch (error) {
			console.error('open book failed', error);
			bookError = error instanceof ApiError ? error.message : t('launcher.openError');
			openingBook = false;
		}
	}

	async function removeItem(path: string, name: string, isBook: boolean) {
		if (!confirm(t('launcher.removeConfirm', { name }))) return;
		try {
			await removeRecent(path);
			// Optimistic update
			if (isBook) {
				books = books.filter((b) => b.path !== path);
			} else {
				projects = projects.filter((p) => p.path !== path);
			}
		} catch (error) {
			console.error('remove failed', error);
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
		<p class="local-first">{t('launcher.localFirst')}</p>
	</section>

	<section class="panels">
		<!-- Projects panel -->
		<div class="panel">
			<div class="panel-header">
				<p class="rt">
					{t('launcher.recent')}
					{#if projects.length}<span class="cnt mono">· {projects.length}</span>{/if}
				</p>
				<Button variant="primary" onclick={() => (showProjectForm = !showProjectForm)}>
					<Icon name="folder" size={16} />
					{t('action.openFolder')}
				</Button>
			</div>

			{#if showProjectForm}
				<form
					class="path-form"
					onsubmit={(e) => {
						e.preventDefault();
						openProjectFolder(projectPathInput);
					}}
				>
					<label for="project-path">{t('launcher.pathLabel')}</label>
					<div class="row">
						<input
							id="project-path"
							class="mono"
							bind:value={projectPathInput}
							placeholder="/home/…/my-novel"
							autocomplete="off"
							spellcheck="false"
						/>
						{#if nativePicker}
							<Button type="button" onclick={browseProject} disabled={openingProject}>
								<Icon name="folder" size={15} />
								{t('action.browseFolder')}
							</Button>
						{/if}
						<Button variant="primary" type="submit" disabled={openingProject}>
							{openingProject ? t('launcher.opening') : t('action.open')}
						</Button>
					</div>
					<p class="hint">{nativePicker ? t('launcher.pathHintDesktop') : t('launcher.pathHint')}</p>
				</form>
			{/if}

			{#if projectError}
				<p class="error" role="alert">{projectError}</p>
			{/if}

			{#if projects.length === 0}
				<p class="empty">{t('launcher.noRecent')}</p>
			{:else}
				<ul>
					{#each projects as project (project.path)}
						<li class="recent-row">
							<button class="recent-item" onclick={() => openProjectFolder(project.path)} disabled={openingProject}>
								<span class="cov project-cov" aria-hidden="true">{project.name.charAt(0).toUpperCase()}</span>
								<span class="rb">
									<span class="rn">{project.name}</span>
									<span class="rp mono">{project.path}</span>
								</span>
								<span class="rmeta">
									<span class="m1">{t('launcher.entries', { count: formatNumber(project.entries) })}</span>
									<span class="m2">{formatRelative(project.last_opened)}</span>
								</span>
								<span class="go" aria-hidden="true"><Icon name="chevron-right" size={16} /></span>
							</button>
							<button
								class="remove-btn"
								title={t('launcher.removeRecent')}
								onclick={() => removeItem(project.path, project.name, false)}
							>
								<Icon name="x" size={14} />
							</button>
						</li>
					{/each}
				</ul>
			{/if}
		</div>

		<!-- Books panel -->
		<div class="panel">
			<div class="panel-header">
				<p class="rt">
					{t('launcher.recentBooks')}
					{#if books.length}<span class="cnt mono">· {books.length}</span>{/if}
				</p>
				<Button onclick={() => (showBookForm = !showBookForm)}>
					<Icon name="book-open" size={16} />
					{t('launcher.openBook')}
				</Button>
			</div>

			{#if showBookForm}
				<form
					class="path-form"
					onsubmit={(e) => {
						e.preventDefault();
						openBookFolder(bookPathInput);
					}}
				>
					<label for="book-path">{t('launcher.pathLabel')}</label>
					<div class="row">
						<input
							id="book-path"
							class="mono"
							bind:value={bookPathInput}
							placeholder="/home/…/my-book"
							autocomplete="off"
							spellcheck="false"
						/>
						{#if nativePicker}
							<Button type="button" onclick={browseBook} disabled={openingBook}>
								<Icon name="folder" size={15} />
								{t('action.browseFolder')}
							</Button>
						{/if}
						<Button variant="primary" type="submit" disabled={openingBook}>
							{openingBook ? t('launcher.opening') : t('action.open')}
						</Button>
					</div>
					<p class="hint">{t('launcher.bookSubtitle')}</p>
				</form>
			{/if}

			{#if bookError}
				<p class="error" role="alert">{bookError}</p>
			{/if}

			{#if books.length === 0}
				<p class="empty">{t('launcher.noRecentBooks')}</p>
			{:else}
				<ul>
					{#each books as book (book.path)}
						<li class="recent-row">
							<button class="recent-item" onclick={() => openBookFolder(book.path)} disabled={openingBook}>
								<span class="cov book-cov" aria-hidden="true"><Icon name="book-open" size={18} /></span>
								<span class="rb">
									<span class="rn">{book.name}</span>
									<span class="rp mono">{book.path}</span>
								</span>
								<span class="rmeta">
									<span class="m1">{t('launcher.files', { count: formatNumber(book.entries) })}</span>
									<span class="m2">{formatRelative(book.last_opened)}</span>
								</span>
								<span class="go" aria-hidden="true"><Icon name="chevron-right" size={16} /></span>
							</button>
							<button
								class="remove-btn"
								title={t('launcher.removeRecent')}
								onclick={() => removeItem(book.path, book.name, true)}
							>
								<Icon name="x" size={14} />
							</button>
						</li>
					{/each}
				</ul>
			{/if}
		</div>
	</section>
</div>

<style>
	.launch {
		min-height: 100vh;
		display: flex;
		flex-direction: column;
	}
	.hero {
		display: flex;
		flex-direction: column;
		gap: 14px;
		padding: 48px clamp(32px, 6vw, 88px) 32px;
		background: var(--surface);
		border-bottom: 1px solid var(--border);
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
		font-size: 36px;
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
	.local-first {
		margin: 0;
		font-size: 12.5px;
		color: var(--faint);
	}
	.panels {
		display: grid;
		grid-template-columns: 1fr 1fr;
		flex: 1;
	}
	.panel {
		display: flex;
		flex-direction: column;
		gap: 12px;
		padding: 32px clamp(28px, 4vw, 56px);
	}
	.panel:first-child {
		border-right: 1px solid var(--border);
	}
	.panel-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
	}
	.path-form {
		display: flex;
		flex-direction: column;
		gap: 6px;
		margin-top: 4px;
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
	.rt {
		margin: 0;
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
	.recent-row {
		position: relative;
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
		background: var(--surface);
		border-color: var(--border);
		box-shadow: var(--shadow);
	}
	.recent-item:disabled {
		opacity: 0.6;
		pointer-events: none;
	}
	.remove-btn {
		position: absolute;
		top: 4px;
		right: 4px;
		display: none;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border: 0;
		border-radius: var(--radius-sm);
		background: var(--surface);
		color: var(--muted);
		cursor: pointer;
	}
	.remove-btn:hover {
		background: var(--danger);
		color: #fff;
	}
	.recent-row:hover .remove-btn {
		display: flex;
	}
	.cov {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 42px;
		height: 42px;
		flex: none;
		border-radius: 10px;
		color: #fff;
		font-weight: 700;
		font-size: 17px;
	}
	.project-cov {
		background: radial-gradient(120% 120% at 20% 10%, #cdd3fb, #8b8ce8 55%, #5b5bd6);
	}
	.book-cov {
		background: radial-gradient(120% 120% at 20% 10%, #d4edda, #6bc67e 55%, #28a745);
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
		.panels {
			grid-template-columns: 1fr;
		}
		.panel:first-child {
			border-right: 0;
			border-bottom: 1px solid var(--border);
		}
	}
</style>
