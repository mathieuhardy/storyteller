<script lang="ts">
	// Entry editor (docs/ui/screens.md §4): creation and edition, continuous with
	// the entry detail screen. The frontmatter form is generated from the type
	// schema (`GET /types/{type}`); saving only ever sends the fields that
	// actually changed, so untouched keys/order/body stay byte-for-byte
	// (`docs/api.md` §2 — non-destructive writing owns the rest server-side).
	//
	// Asset fields (`cover`, `portrait`, and any `image`/`image-list` field) are
	// not editable here: `/assets` upload is still unimplemented (docs/roadmap.md
	// M4 remaining work), so faking a dropzone would promise something the
	// backend can't do yet — they stay untouched, preserved by omission.
	import { goto } from '$app/navigation';
	import type { Entry, FieldSchema, Frontmatter, OutgoingLink, TypeResponse } from '$api/types';
	import { createEntity, updateEntity, ApiError } from '$api/client';
	import { t, typeLabel, fieldLabel } from '$i18n/index.svelte';
	import TokenInput from './TokenInput.svelte';
	import AttributeField from './AttributeField.svelte';
	import BodyEditor from './BodyEditor.svelte';
	import SaveBar from './SaveBar.svelte';

	let {
		mode,
		type,
		schema,
		entry = null,
		links = []
	}: {
		mode: 'create' | 'edit';
		type: string;
		schema: TypeResponse;
		entry?: Entry | null;
		links?: OutgoingLink[];
	} = $props();

	// `created`/`updated` are managed by the backend on every write (`docs/api.md`
	// §2 — `created` immutable, `updated` refreshed) and never user-editable.
	const HIDDEN_FIELDS = new Set([
		'type',
		'title',
		'aliases',
		'tags',
		'cover',
		'portrait',
		'created',
		'updated'
	]);
	const ASSET_KINDS = new Set(['image', 'image-list']);

	const editableFields = schema.fields.filter(
		(f) => !HIDDEN_FIELDS.has(f.name) && !ASSET_KINDS.has(f.kind)
	);

	const initial: Frontmatter = entry?.frontmatter ?? {};

	function isEmptyValue(v: unknown): boolean {
		if (v === undefined || v === null) return true;
		if (Array.isArray(v)) return v.length === 0;
		if (typeof v === 'string') return v === '';
		return false;
	}

	function stripWikilink(raw: unknown): string | null {
		if (typeof raw !== 'string') return null;
		const match = raw.match(/^\[\[(.+)\]\]$/);
		return match ? match[1] : raw;
	}

	function initFieldValue(kind: FieldSchema['kind'], raw: unknown): unknown {
		switch (kind) {
			case 'text':
				return typeof raw === 'string' ? raw : '';
			case 'number':
				return typeof raw === 'number' ? raw : null;
			case 'number-or-text':
				return raw === undefined || raw === null ? '' : String(raw);
			case 'boolean':
				return raw === true;
			case 'enum':
				return typeof raw === 'string' ? raw : '';
			case 'list':
			case 'list-or-text':
				return Array.isArray(raw) ? raw.map(String) : raw != null ? [String(raw)] : [];
			case 'link': {
				const stripped = stripWikilink(raw);
				return stripped ? [stripped] : [];
			}
			case 'link-list':
				return Array.isArray(raw)
					? (raw.map(stripWikilink).filter(Boolean) as string[])
					: [];
			default:
				return typeof raw === 'string' ? raw : '';
		}
	}

	function serializeFieldValue(kind: FieldSchema['kind'], value: unknown): unknown {
		switch (kind) {
			case 'text':
				return value;
			case 'number':
				return value;
			case 'number-or-text': {
				const text = String(value ?? '').trim();
				if (text === '') return null;
				return /^-?\d+(\.\d+)?$/.test(text) ? Number(text) : text;
			}
			case 'boolean':
				return value === true;
			case 'enum':
				return value === '' ? null : value;
			case 'list':
			case 'list-or-text':
				return value;
			case 'link': {
				const tokens = (value as string[]) ?? [];
				return tokens.length > 0 ? `[[${tokens[0]}]]` : null;
			}
			case 'link-list':
				return ((value as string[]) ?? []).map((v) => `[[${v}]]`);
			default:
				return value;
		}
	}

	function arraysEqual(a: string[], b: string[]): boolean {
		return a.length === b.length && a.every((v, i) => v === b[i]);
	}

	const initialTitle = typeof initial.title === 'string' ? initial.title : '';
	const initialAliases = Array.isArray(initial.aliases) ? initial.aliases.map(String) : [];
	const initialTags = Array.isArray(initial.tags) ? initial.tags.map(String) : [];

	let title = $state(initialTitle);
	let aliases = $state<string[]>([...initialAliases]);
	let tags = $state<string[]>([...initialTags]);
	let fieldValues = $state<Record<string, unknown>>(
		Object.fromEntries(editableFields.map((f) => [f.name, initFieldValue(f.kind, initial[f.name])]))
	);
	let bodyText = $state(entry?.body ?? '');

	const knownNames = new Set(schema.fields.map((f) => f.name));
	const preservedFields =
		mode === 'edit'
			? Object.entries(initial).filter(([key]) => !knownNames.has(key) && !HIDDEN_FIELDS.has(key))
			: [];

	// Compares against the initial value round-tripped through the same
	// init→serialize pipeline used to seed the form, not the raw frontmatter
	// value directly: a `list-or-text` field stored as a bare string (or a link
	// stored as `[[slug|Display]]`) would otherwise look "changed" on an
	// untouched field purely because of representation, and get needlessly
	// rewritten on save.
	function fieldChanged(field: FieldSchema): boolean {
		const current = serializeFieldValue(field.kind, fieldValues[field.name]);
		const initialRoundTrip = serializeFieldValue(field.kind, initFieldValue(field.kind, initial[field.name]));
		if (isEmptyValue(current) && isEmptyValue(initialRoundTrip)) return false;
		return JSON.stringify(current) !== JSON.stringify(initialRoundTrip);
	}

	const dirty = $derived(
		title !== initialTitle ||
			!arraysEqual(aliases, initialAliases) ||
			!arraysEqual(tags, initialTags) ||
			bodyText !== (entry?.body ?? '') ||
			editableFields.some(fieldChanged)
	);

	const missingFields = $derived(
		new Set(
			editableFields.filter((f) => f.required && isEmptyValue(fieldValues[f.name])).map((f) => f.name)
		)
	);

	let saving = $state(false);
	let error = $state<string | null>(null);

	function validate(): string | null {
		if (!title.trim()) return t('editor.errorTitleRequired');
		if (missingFields.size > 0) return t('editor.errorMissingRequired');
		return null;
	}

	async function handleSave() {
		const validationError = validate();
		if (validationError) {
			error = validationError;
			return;
		}
		saving = true;
		error = null;
		try {
			if (mode === 'create') {
				const frontmatter: Frontmatter = {};
				if (aliases.length > 0) frontmatter.aliases = aliases;
				if (tags.length > 0) frontmatter.tags = tags;
				for (const field of editableFields) {
					const value = serializeFieldValue(field.kind, fieldValues[field.name]);
					if (!isEmptyValue(value)) frontmatter[field.name] = value;
				}
				const created = await createEntity({ type, title: title.trim(), frontmatter, body: bodyText });
				await goto(`/entry/${created.slug}`);
			} else if (entry) {
				const frontmatter: Frontmatter = {};
				if (title.trim() !== initialTitle) frontmatter.title = title.trim();
				if (!arraysEqual(aliases, initialAliases)) frontmatter.aliases = aliases;
				if (!arraysEqual(tags, initialTags)) frontmatter.tags = tags;
				for (const field of editableFields) {
					if (fieldChanged(field)) frontmatter[field.name] = serializeFieldValue(field.kind, fieldValues[field.name]);
				}
				const payload: { frontmatter?: Frontmatter; body?: string } = {};
				if (Object.keys(frontmatter).length > 0) payload.frontmatter = frontmatter;
				if (bodyText !== entry.body) payload.body = bodyText;
				await updateEntity(entry.slug, payload);
				await goto(`/entry/${entry.slug}`);
			}
		} catch (e) {
			error = e instanceof ApiError ? e.message : String(e);
		} finally {
			saving = false;
		}
	}

	function handleCancel() {
		goto(mode === 'create' || !entry ? `/type/${type}` : `/entry/${entry.slug}`);
	}

	function onWindowKeydown(event: KeyboardEvent) {
		if ((event.metaKey || event.ctrlKey) && event.key === 's') {
			event.preventDefault();
			handleSave();
		}
	}
</script>

<svelte:window onkeydown={onWindowKeydown} />

<div class="entry-editor">
	<div class="header">
		<span class="type-lock">🔒 {typeLabel(type)}</span>
		<input class="title-input" type="text" bind:value={title} placeholder={t('editor.titlePlaceholder')} spellcheck="false" />
		{#if mode === 'edit' && entry}
			<p class="slug-hint">
				{t('editor.identityLabel')} <b>{entry.path}</b> — {t('editor.identityHint')}
			</p>
		{:else}
			<p class="slug-hint">{t('editor.newIdentityHint')}</p>
		{/if}
	</div>

	<div class="tokrow">
		<span class="tl">aliases</span>
		<TokenInput bind:tokens={aliases} placeholder={t('editor.addAlias')} />
	</div>
	<div class="tokrow">
		<span class="tl">tags</span>
		<TokenInput bind:tokens={tags} placeholder={t('editor.addTag')} />
	</div>

	{#if editableFields.length > 0}
		<div class="section-h">
			<h2>{t('entry.attributes')}</h2>
			<span class="line"></span>
			<span class="hint">{t('editor.generatedFrom', { type: typeLabel(type) })}</span>
		</div>
		<div class="fields">
			{#each editableFields as field (field.name)}
				<div class="field" class:invalid={missingFields.has(field.name)}>
					<div class="fl">
						{fieldLabel(field)}
						{#if field.required}<span class="req" title={t('editor.required')}>✳</span>{/if}
					</div>
					<div class="ctrl">
						<AttributeField {field} bind:value={fieldValues[field.name]} />
					</div>
				</div>
			{/each}
		</div>
	{/if}

	{#if preservedFields.length > 0}
		<div class="section-h">
			<h2>{t('entry.preservedFields')}</h2>
			<span class="line"></span>
		</div>
		<div class="preserved-box">
			<div class="ph">🔒 {t('editor.preservedHint')}</div>
			{#each preservedFields as [key, value] (key)}
				<div class="pr">
					<span class="pk">{key}</span>
					<span class="pv">{JSON.stringify(value)}</span>
				</div>
			{/each}
		</div>
	{/if}

	<div class="section-h">
		<h2>{t('entry.content')}</h2>
		<span class="line"></span>
	</div>
	<BodyEditor bind:body={bodyText} {links} />

	<SaveBar {dirty} {saving} {error} oncancel={handleCancel} onsave={handleSave} />
</div>

<style>
	.entry-editor {
		max-width: 780px;
		margin: 0 auto;
		padding: 24px 26px 60px;
	}

	.header {
		margin-bottom: 8px;
	}
	.type-lock {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		height: 22px;
		padding: 0 9px;
		border-radius: 20px;
		font-size: 12px;
		font-weight: 600;
		background: var(--accent-soft);
		color: var(--accent);
	}
	.title-input {
		display: block;
		width: 100%;
		border: 0;
		background: transparent;
		color: var(--text);
		font-size: 27px;
		font-weight: 680;
		letter-spacing: -0.02em;
		padding: 6px 0 4px;
		margin: 6px 0 2px;
		border-bottom: 2px solid transparent;
		outline: none;
		font-family: inherit;
	}
	.title-input:focus {
		border-bottom-color: var(--accent);
	}
	.slug-hint {
		margin: 0;
		font-family: var(--font-mono);
		font-size: 11.5px;
		color: var(--faint);
	}
	.slug-hint b {
		color: var(--muted);
		font-weight: 500;
	}

	.tokrow {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-top: 12px;
	}
	.tl {
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--faint);
		width: 62px;
		flex: none;
	}

	.section-h {
		display: flex;
		align-items: center;
		gap: 8px;
		margin: 28px 0 14px;
	}
	.section-h h2 {
		font-size: 12px;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		color: var(--muted);
		margin: 0;
		font-weight: 600;
	}
	.section-h .line {
		flex: 1;
		height: 1px;
		background: var(--border);
	}
	.section-h .hint {
		font-size: 11.5px;
		color: var(--faint);
	}

	.fields {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}
	.field {
		display: grid;
		grid-template-columns: 150px 1fr;
		gap: 14px;
		align-items: start;
		padding: 8px 0;
	}
	.fl {
		padding-top: 7px;
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--muted);
		display: flex;
		align-items: center;
		gap: 6px;
	}
	.req {
		color: var(--danger);
		font-size: 13px;
		line-height: 1;
	}
	.field.invalid :global(.ctrl-input),
	.field.invalid :global(.ctrl-textarea) {
		border-color: var(--danger);
	}

	.preserved-box {
		border: 1px solid var(--border);
		border-radius: var(--radius);
		background: var(--surface-2);
		overflow: hidden;
	}
	.preserved-box .ph {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 12px;
		font-size: 11.5px;
		color: var(--muted);
		border-bottom: 1px solid var(--border);
	}
	.preserved-box .pr {
		display: grid;
		grid-template-columns: 150px 1fr;
		padding: 8px 12px;
	}
	.preserved-box .pk {
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--faint);
	}
	.preserved-box .pv {
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--muted);
	}

	@media (max-width: 700px) {
		.field {
			grid-template-columns: 1fr;
			gap: 4px;
		}
		.field .fl {
			padding-top: 0;
		}
	}
</style>
