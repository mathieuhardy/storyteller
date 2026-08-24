<script lang="ts">
	// Body section (docs/ui/screens.md §2): renders the entry's HTML
	// (`?render=html`, ADR 0015) — wikilinks/embeds are already resolved to
	// `<a>`/`<span>`/`<img>` server-side, so this just injects it. Falls back to
	// raw text only if `html` wasn't requested/returned (defensive, not the
	// normal path: the loader always asks for it).
	import { t } from '$i18n/index.svelte';

	let {
		body,
		html
	}: {
		body: string;
		html?: string;
	} = $props();
</script>

{#if html || body.trim()}
	<section class="body-section">
		<h2 class="section-title">{t('entry.content')}</h2>
		<div class="body-content">
			{#if html}
				<div class="rendered">{@html html}</div>
			{:else}
				<pre class="raw">{body}</pre>
			{/if}
		</div>
	</section>
{/if}

<style>
	.body-section {
		margin-bottom: 28px;
	}

	.section-title {
		font-size: 11px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.08em;
		color: var(--muted);
		margin: 0 0 14px;
		padding-bottom: 8px;
		border-bottom: 1px solid var(--border);
	}

	.body-content {
		background: var(--surface);
		border: 1px solid var(--border);
		border-radius: var(--radius);
		padding: 16px 18px;
	}

	.raw {
		margin: 0;
		font-family: inherit;
		font-size: 14px;
		line-height: 1.7;
		white-space: pre-wrap;
		word-wrap: break-word;
		color: var(--text);
	}

	.rendered {
		font-size: 14px;
		line-height: 1.7;
		color: var(--text);
	}
	.rendered :global(p) {
		margin: 0 0 14px;
	}
	.rendered :global(p:last-child) {
		margin-bottom: 0;
	}
	.rendered :global(h1),
	.rendered :global(h2),
	.rendered :global(h3) {
		margin: 22px 0 10px;
		font-weight: 650;
		line-height: 1.3;
	}
	.rendered :global(h1) {
		font-size: 19px;
	}
	.rendered :global(h2) {
		font-size: 16px;
	}
	.rendered :global(h3) {
		font-size: 14.5px;
	}
	.rendered :global(ul),
	.rendered :global(ol) {
		margin: 0 0 14px;
		padding-left: 22px;
	}
	.rendered :global(li) {
		margin: 4px 0;
	}
	.rendered :global(blockquote) {
		margin: 0 0 14px;
		padding: 2px 14px;
		border-left: 3px solid var(--border-strong);
		color: var(--muted);
	}
	.rendered :global(code) {
		font-family: var(--font-mono);
		font-size: 0.9em;
		background: var(--surface-2);
		padding: 1px 5px;
		border-radius: 4px;
	}
	.rendered :global(pre) {
		background: var(--surface-2);
		border-radius: var(--radius-sm);
		padding: 12px 14px;
		overflow-x: auto;
		margin: 0 0 14px;
	}
	.rendered :global(pre code) {
		background: none;
		padding: 0;
	}
	.rendered :global(img) {
		max-width: 100%;
		border-radius: var(--radius-sm);
	}
	.rendered :global(table) {
		border-collapse: collapse;
		margin: 0 0 14px;
	}
	.rendered :global(th),
	.rendered :global(td) {
		border: 1px solid var(--border);
		padding: 6px 10px;
		text-align: left;
	}
	.rendered :global(hr) {
		border: none;
		border-top: 1px solid var(--border);
		margin: 20px 0;
	}

	.rendered :global(.wikilink) {
		text-decoration: underline;
		text-decoration-thickness: 1px;
		text-underline-offset: 2px;
		border-radius: 2px;
	}
	.rendered :global(a.wikilink.resolved) {
		color: var(--accent);
		text-decoration-color: var(--accent-line);
		cursor: pointer;
	}
	.rendered :global(a.wikilink.resolved:hover) {
		background: var(--accent-soft);
	}
	.rendered :global(.wikilink.stub) {
		color: var(--stub);
		text-decoration-style: dashed;
		text-decoration-color: var(--stub-line);
	}
	.rendered :global(.wikilink.ambiguous) {
		color: var(--warning);
		text-decoration-style: dashed;
		text-decoration-color: var(--warning-line);
	}
	.rendered :global(.asset-embed.missing) {
		margin: 6px 0 16px;
		border: 1px dashed var(--stub-line);
		background: var(--stub-soft);
		border-radius: var(--radius);
		padding: 12px;
		color: var(--warn);
		font-size: 12.5px;
	}
	.rendered :global(.asset-embed.missing code) {
		background: transparent;
		padding: 0;
	}
</style>
