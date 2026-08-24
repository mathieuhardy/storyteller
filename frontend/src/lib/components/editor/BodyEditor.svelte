<script lang="ts">
	// Body editor (docs/ui/screens.md §4): a markdown textarea with a small
	// insertion toolbar and an Edit/Preview toggle. Preview reuses the same
	// raw-text-with-highlighted-wikilinks rendering as the entry detail screen
	// (`BodySection.svelte`) — full HTML rendering waits on `?render=html`
	// (docs/api.md, M4 remaining work), so the editor does not get ahead of it.
	import type { OutgoingLink } from '$api/types';
	import { t } from '$i18n/index.svelte';

	let {
		body = $bindable(''),
		links = []
	}: {
		body?: string;
		links?: OutgoingLink[];
	} = $props();

	let mode = $state<'edit' | 'preview'>('edit');
	let textarea: HTMLTextAreaElement | undefined = $state();

	function wrapSelection(before: string, after: string = before) {
		if (!textarea) return;
		const { selectionStart, selectionEnd, value } = textarea;
		const selected = value.slice(selectionStart, selectionEnd);
		body = value.slice(0, selectionStart) + before + selected + after + value.slice(selectionEnd);
		queueMicrotask(() => {
			textarea?.focus();
			textarea?.setSelectionRange(selectionStart + before.length, selectionEnd + before.length);
		});
	}

	function prefixLine(prefix: string) {
		if (!textarea) return;
		const { selectionStart, value } = textarea;
		const lineStart = value.lastIndexOf('\n', selectionStart - 1) + 1;
		body = value.slice(0, lineStart) + prefix + value.slice(lineStart);
		queueMicrotask(() => {
			textarea?.focus();
			textarea?.setSelectionRange(selectionStart + prefix.length, selectionStart + prefix.length);
		});
	}

	const WIKILINK_RE = /\[\[([^\]]+)\]\]/g;

	interface Segment {
		type: 'text' | 'link';
		content: string;
		resolution?: OutgoingLink;
	}

	const linkResolutions = $derived(new Map(links.map((l) => [l.target_raw, l])));

	const segments = $derived.by(() => {
		const result: Segment[] = [];
		let lastIndex = 0;
		let match: RegExpExecArray | null;
		const regex = new RegExp(WIKILINK_RE);
		while ((match = regex.exec(body)) !== null) {
			if (match.index > lastIndex) result.push({ type: 'text', content: body.slice(lastIndex, match.index) });
			const target = match[1];
			result.push({ type: 'link', content: match[0], resolution: linkResolutions.get(target) });
			lastIndex = regex.lastIndex;
		}
		if (lastIndex < body.length) result.push({ type: 'text', content: body.slice(lastIndex) });
		return result;
	});

	function linkLabel(content: string): string {
		const inner = content.slice(2, -2);
		const pipeIndex = inner.indexOf('|');
		return pipeIndex > 0 ? inner.slice(pipeIndex + 1) : inner;
	}
</script>

<div class="body-editor">
	<div class="bodybar">
		<div class="mdtools">
			<button type="button" title={t('editor.mdHeading')} onclick={() => prefixLine('## ')}>H</button>
			<button type="button" title={t('editor.mdBold')} class="b" onclick={() => wrapSelection('**')}>B</button>
			<button type="button" title={t('editor.mdItalic')} class="i" onclick={() => wrapSelection('*')}>I</button>
			<button type="button" title={t('editor.mdList')} onclick={() => prefixLine('- ')}>•</button>
			<button type="button" title={t('editor.mdWikilink')} onclick={() => wrapSelection('[[', ']]')}
				>[[ ]]</button
			>
			<button type="button" title={t('editor.mdImage')} onclick={() => wrapSelection('![[', ']]')}
				>🖼</button
			>
		</div>
		<div class="bspacer"></div>
		<div class="modeseg">
			<button type="button" class:on={mode === 'edit'} onclick={() => (mode = 'edit')}>
				{t('editor.modeEdit')}
			</button>
			<button type="button" class:on={mode === 'preview'} onclick={() => (mode = 'preview')}>
				{t('editor.modePreview')}
			</button>
		</div>
	</div>

	<div class="editor-box">
		{#if mode === 'edit'}
			<textarea class="src" bind:this={textarea} bind:value={body} spellcheck="false"></textarea>
		{:else}
			<pre class="preview">{#each segments as seg}{#if seg.type === 'text'}{seg.content}{:else}{@const state = seg.resolution?.resolution ?? 'stub'}<span
							class="wl {state}">{linkLabel(seg.content)}</span
						>{/if}{/each}</pre>
		{/if}
	</div>
</div>

<style>
	.bodybar {
		display: flex;
		align-items: center;
		gap: 6px;
		margin-bottom: 8px;
	}
	.mdtools {
		display: flex;
		gap: 2px;
	}
	.mdtools button {
		width: 30px;
		height: 28px;
		border: 1px solid var(--border);
		background: var(--surface);
		color: var(--muted);
		border-radius: var(--radius-sm);
		cursor: pointer;
		display: grid;
		place-items: center;
		font-family: var(--font-mono);
		font-size: 12px;
	}
	.mdtools button.b {
		font-weight: 700;
	}
	.mdtools button.i {
		font-style: italic;
	}
	.mdtools button:hover {
		color: var(--text);
		border-color: var(--border-strong);
	}
	.bspacer {
		flex: 1;
	}
	.modeseg {
		display: inline-flex;
		background: var(--surface-2);
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
		padding: 2px;
		gap: 2px;
	}
	.modeseg button {
		height: 24px;
		padding: 0 12px;
		border: 0;
		background: transparent;
		color: var(--muted);
		border-radius: 4px;
		font-size: 12.5px;
		cursor: pointer;
		font-family: inherit;
	}
	.modeseg button.on {
		background: var(--surface);
		color: var(--text);
		box-shadow: var(--shadow);
		font-weight: 500;
	}
	.editor-box {
		border: 1px solid var(--border);
		border-radius: var(--radius);
		background: var(--surface);
		overflow: hidden;
	}
	.src {
		width: 100%;
		min-height: 300px;
		border: 0;
		outline: none;
		resize: vertical;
		padding: 16px 18px;
		background: transparent;
		color: var(--text);
		font-family: var(--font-mono);
		font-size: 13px;
		line-height: 1.7;
	}
	.preview {
		margin: 0;
		padding: 16px 18px;
		min-height: 300px;
		font-family: var(--font-mono);
		font-size: 13px;
		line-height: 1.7;
		white-space: pre-wrap;
		word-wrap: break-word;
		color: var(--text);
	}
	.wl {
		border-radius: 3px;
		padding: 0 1px;
	}
	.wl.resolved {
		color: var(--accent);
		box-shadow: inset 0 -1px 0 var(--accent-line);
	}
	.wl.stub {
		color: var(--stub);
		border-bottom: 1px dashed var(--stub-line);
	}
	.wl.ambiguous {
		color: var(--warning);
		border-bottom: 1px dashed var(--warning-line);
	}
</style>
