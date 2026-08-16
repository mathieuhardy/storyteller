<script lang="ts">
	// Body section (docs/ui/screens.md §2): displays the markdown body. For now
	// it shows raw markdown; later it will render HTML with clickable wikilinks
	// once the ?render=html endpoint is implemented.
	import type { OutgoingLink } from '$api/types';
	import { t } from '$i18n/index.svelte';

	let {
		body,
		links
	}: {
		body: string;
		links: OutgoingLink[];
	} = $props();

	// Simple wikilink regex to highlight them in raw view
	const WIKILINK_RE = /\[\[([^\]]+)\]\]/g;

	// Build resolution map for wikilinks
	const linkResolutions = $derived(
		new Map(links.map((l) => [l.target_raw, l]))
	);

	// Parse body into segments for highlighting wikilinks
	interface Segment {
		type: 'text' | 'link';
		content: string;
		resolution?: OutgoingLink;
	}

	const segments = $derived.by(() => {
		const result: Segment[] = [];
		let lastIndex = 0;
		let match: RegExpExecArray | null;

		const regex = new RegExp(WIKILINK_RE);
		while ((match = regex.exec(body)) !== null) {
			if (match.index > lastIndex) {
				result.push({ type: 'text', content: body.slice(lastIndex, match.index) });
			}
			const target = match[1];
			result.push({
				type: 'link',
				content: match[0],
				resolution: linkResolutions.get(target)
			});
			lastIndex = regex.lastIndex;
		}
		if (lastIndex < body.length) {
			result.push({ type: 'text', content: body.slice(lastIndex) });
		}
		return result;
	});

	function getLinkHref(resolution?: OutgoingLink): string | undefined {
		if (resolution?.resolution === 'resolved' && resolution.target_slug) {
			return `/entry/${resolution.target_slug}`;
		}
		return undefined;
	}

	function getLinkLabel(content: string): string {
		// Extract from [[target|label]] or [[target]]
		const inner = content.slice(2, -2);
		const pipeIndex = inner.indexOf('|');
		return pipeIndex > 0 ? inner.slice(pipeIndex + 1) : inner;
	}
</script>

{#if body.trim()}
	<section class="body-section">
		<h2 class="section-title">{t('entry.content')}</h2>
		<div class="body-content">
			<pre class="markdown">{#each segments as seg}{#if seg.type === 'text'}{seg.content}{:else}{@const href = getLinkHref(seg.resolution)}{@const state = seg.resolution?.resolution ?? 'stub'}{#if href}<a class="wikilink resolved" {href}>{getLinkLabel(seg.content)}</a>{:else}<span class="wikilink {state}">{getLinkLabel(seg.content)}{#if state === 'stub'}<sup>+</sup>{:else if state === 'ambiguous'}<sup>?</sup>{/if}</span>{/if}{/if}{/each}</pre>
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

	.markdown {
		margin: 0;
		font-family: inherit;
		font-size: 14px;
		line-height: 1.7;
		white-space: pre-wrap;
		word-wrap: break-word;
		color: var(--text);
	}

	.wikilink {
		text-decoration: underline;
		text-decoration-thickness: 1px;
		text-underline-offset: 2px;
		cursor: pointer;
	}

	.wikilink.resolved {
		color: var(--accent);
		text-decoration-color: var(--accent-line);
	}
	.wikilink.resolved:hover {
		background: var(--accent-soft);
		border-radius: 2px;
	}

	.wikilink.stub {
		color: var(--stub);
		text-decoration-style: dashed;
		text-decoration-color: var(--stub-line);
	}

	.wikilink.ambiguous {
		color: var(--warning);
		text-decoration-style: dashed;
		text-decoration-color: var(--warning-line);
	}

	.wikilink sup {
		font-size: 9px;
		font-weight: 700;
		margin-left: 1px;
	}
</style>
