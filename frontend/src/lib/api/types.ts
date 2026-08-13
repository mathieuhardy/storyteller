// TypeScript mirror of the JSON the API serves (docs/api.md). Kept as a plain
// contract: it names the shapes the client returns, so screens type against the
// API, not against ad-hoc objects. Field names match the server exactly
// (`type` where the server renames `type_name`).

/** Per-entry diagnostic (non-blocking), carried inside a 200 (docs/api.md §6). */
export interface Diagnostic {
	code: string;
	message: string;
	field: string | null;
	severity: 'error' | 'warning';
}

/** Raw YAML frontmatter: unknown keys and order preserved by the backend. */
export type Frontmatter = Record<string, unknown>;

/** Incoming link — `GET /entities/{slug}/backlinks`. */
export interface Backlink {
	slug: string;
	path: string;
	type: string;
	title: string;
	/** Frontmatter field that sourced the link, `null` for a body link. */
	field: string | null;
	context: string;
}

export type Resolution = 'resolved' | 'stub' | 'ambiguous';

/** Outgoing link, resolved — `GET /entities/{slug}/links`. */
export interface OutgoingLink {
	target_raw: string;
	target_slug?: string;
	field: string | null;
	resolution: Resolution;
	/** Candidate slugs when `ambiguous`. */
	candidates?: string[];
}

/** Unresolved link target grouped by key — `GET /stubs`. */
export interface Stub {
	key: string;
	labels: string[];
	count: number;
	sources: string[];
}

/** A full entry — `GET /entities/{slug}`. */
export interface Entry {
	slug: string;
	path: string;
	type: string;
	frontmatter: Frontmatter;
	body: string;
	html?: string;
	backlinks?: Backlink[];
	errors: Diagnostic[];
}

/** Light entry for lists — items of `GET /entities`. */
export interface EntrySummary {
	slug: string;
	path: string;
	type: string;
	title: string;
	tags: string[];
	excerpt: string;
	has_errors: boolean;
	/** `updated` frontmatter value (RFC 3339), empty string when absent. */
	updated: string;
}

/** Paginated list wrapper (docs/api.md §4). */
export interface Page<T> {
	items: T[];
	page: number;
	per_page: number;
	total: number;
}

export type FieldKind =
	| 'text'
	| 'number'
	| 'boolean'
	| 'enum'
	| 'list'
	| 'link'
	| 'link-list'
	| 'image'
	| 'image-list'
	| 'number-or-text'
	| 'list-or-text';

/** One frontmatter field's schema — drives form generation (docs/api.md §3). */
export interface FieldSchema {
	/** English snake_case key; the i18n catalog keys off this (ADR 0014). */
	name: string;
	/** Server-provided French label; a fallback behind the front catalog. */
	label: string;
	kind: FieldKind;
	tier: 'mvp' | 'optional';
	required: boolean;
	enum_values?: string[];
	link_targets?: string[];
}

/** A type with its fields — `GET /types`, `GET /types/{type}`. */
export interface TypeResponse {
	name: string;
	label: string;
	folder: string;
	enabled: boolean;
	fields: FieldSchema[];
}

/** Project metadata — `GET /project` and the payload of `POST /projects/open`. */
export interface ProjectResponse {
	root: string;
	entry: Entry | null;
	enabled_types: string[];
	schema_version: number;
	stats: {
		entries: number;
		by_type: Record<string, number>;
	};
	errors: Diagnostic[];
}

/** One known project in the launcher — item of `GET /projects`. */
export interface ProjectRecord {
	path: string;
	name: string;
	entries: number;
	last_opened: string;
}

/** `GET /projects` — recent projects plus which one is active. */
export interface ProjectsResponse {
	items: ProjectRecord[];
	active: string;
}

/** `GET /version`. */
export interface VersionResponse {
	api_version: string;
	core_version: string;
	schema_version: number;
}

/** SSE event names pushed by the watcher (docs/api.md §5). */
export type ServerEventName =
	| 'entity.created'
	| 'entity.updated'
	| 'entity.deleted'
	| 'assets.changed'
	| 'index.rebuilt';

export interface ServerEvent {
	name: ServerEventName;
	data: unknown;
}
