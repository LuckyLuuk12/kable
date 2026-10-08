<!--
@component

Projects browser for discovering and installing projects.

Provides:
- Project search
- Project type filtering
- Result sorting
- Pagination
- Project cards
- Project details navigation
-->
<script lang="ts">
import { app, type KableProfile, type ModrinthResults, type ProjectSearch, type ProjectType, type SearchIndex } from "$lib";
import { untrack } from "svelte";
import ProjectCard from "./ProjectCard.svelte";

let {
  profile,
  profileId,
  projectType,
  search = "",
  smartFilter = true,
}: {
  profile: KableProfile | null;
  profileId: string | null;
  projectType: ProjectType;
  search?: string;
  smartFilter?: boolean;
} = $props();

const limit = 20;
const loadingDelay = 300;

let results = $state<ModrinthResults | null>(null);
let loading = $state(false);
let error = $state<string | null>(null);
let query = $state(search.trim());
let index = $state<SearchIndex>("relevance");
let offset = $state(0);

let browserRoot: HTMLDivElement | null = null;

let projectResults = $derived(results?.hits ?? []);

let hasNextPage = $derived(results !== null && results.offset + results.limit < results.total_hits);

let hasPreviousPage = $derived(offset > 0);

let requestId = 0;
let loadingTimer: ReturnType<typeof setTimeout> | null = null;
let prefetching = new Set<number>();
let pageCache = new Map<number, ModrinthResults>();

let visibilityObserver: ResizeObserver | null = null;
let isVisible = $state(false);
let hasInitialized = false;

function clearLoadingTimer() {
  if (loadingTimer !== null) {
    clearTimeout(loadingTimer);
    loadingTimer = null;
  }
}

function createRequest(targetOffset: number): ProjectSearch {
  return {
    query: query.trim() || null,
    facets: [],
    index,
    offset: targetOffset,
    limit,
  };
}

function checkVisibility(): boolean {
  if (!browserRoot) {
    return false;
  }

  const rect = browserRoot.getBoundingClientRect();

  return rect.width > 0 && rect.height > 0 && browserRoot.getClientRects().length > 0;
}

function updateVisibility(): void {
  const visible = checkVisibility();

  if (visible === isVisible) {
    return;
  }

  isVisible = visible;

  console.debug("[ProjectsBrowser] Visibility changed:", visible);

  if (!visible) {
    return;
  }

  /*
   * The component can be mounted while its parent is display:none.
   * Only perform the initial Modrinth request after it becomes visible.
   */
  if (!hasInitialized && profile) {
    hasInitialized = true;
    void browse(0);
  }
}

async function fetchPage(targetOffset: number): Promise<ModrinthResults | null> {
  if (!profile || !isVisible) {
    return null;
  }

  const cached = pageCache.get(targetOffset);

  if (cached) {
    return cached;
  }

  try {
    /*
     * Visibility can change while awaiting another operation, so check
     * again immediately before making the actual Modrinth request.
     */
    if (!isVisible || !profile) {
      return null;
    }

    const result = await app.projectsService.browse(profile, createRequest(targetOffset), smartFilter, projectType);

    pageCache.set(targetOffset, result);

    return result;
  } catch {
    return null;
  }
}

async function prefetchPage(targetOffset: number) {
  if (!profile || !isVisible || prefetching.has(targetOffset) || pageCache.has(targetOffset)) {
    return;
  }

  prefetching.add(targetOffset);

  try {
    await fetchPage(targetOffset);
  } finally {
    prefetching.delete(targetOffset);
  }
}

function prefetchNextPage() {
  if (!isVisible || !results || !hasNextPage) {
    return;
  }

  const nextOffset = results.offset + limit;

  void prefetchPage(nextOffset);
}

async function browse(targetOffset: number) {
  if (!profile || !isVisible) {
    return;
  }

  const cached = pageCache.get(targetOffset);

  if (cached) {
    results = cached;
    offset = targetOffset;
    error = null;

    prefetchNextPage();
    return;
  }

  const id = ++requestId;

  clearLoadingTimer();

  loading = false;
  loadingTimer = setTimeout(() => {
    if (id === requestId && isVisible) {
      loading = true;
    }
  }, loadingDelay);

  error = null;

  try {
    const nextResults = await fetchPage(targetOffset);

    if (id !== requestId || !isVisible) {
      return;
    }

    if (!nextResults) {
      throw new Error("Failed to load projects.");
    }

    results = nextResults;
    offset = targetOffset;

    prefetchNextPage();
  } catch (e) {
    if (id !== requestId || !isVisible) {
      return;
    }

    error = e instanceof Error ? e.message : String(e);
  } finally {
    if (id === requestId) {
      clearLoadingTimer();
      loading = false;
    }
  }
}

function resetPages() {
  requestId++;

  clearLoadingTimer();

  pageCache.clear();
  prefetching.clear();
  offset = 0;
  results = null;
  loading = false;
  error = null;
}

function submitSearch() {
  if (!isVisible || !profile) {
    return;
  }

  resetPages();
  void browse(0);
}

function setIndex(value: SearchIndex) {
  if (value === index) {
    return;
  }

  index = value;

  if (!isVisible || !profile) {
    return;
  }

  resetPages();
  void browse(0);
}

function nextPage() {
  if (!isVisible || !results || !hasNextPage) {
    return;
  }

  void browse(results.offset + limit);
}

function previousPage() {
  if (!isVisible || !results || !hasPreviousPage) {
    return;
  }

  void browse(Math.max(0, results.offset - limit));
}

function handleScroll() {
  if (!isVisible || !results || !hasNextPage) {
    return;
  }

  const scrollPosition = window.scrollY + window.innerHeight;

  const threshold = document.documentElement.scrollHeight - 600;

  if (scrollPosition >= threshold) {
    const nextOffset = results.offset + limit;

    if (!pageCache.has(nextOffset) && !prefetching.has(nextOffset)) {
      void prefetchPage(nextOffset);
    }
  }
}

/*
 * Search state can change while this browser is hidden, but changing it
 * must not cause a Modrinth request. When the browser becomes visible,
 * the current state is used for the initial request.
 */
$effect(() => {
  if (!profileId) {
    results = null;
    error = null;
    resetPages();
    hasInitialized = false;
    return;
  }

  query;
  index;
  projectType;
  smartFilter;

  const { visible, initialized } = untrack(() => ({
    visible: isVisible,
    initialized: hasInitialized,
  }));

  if (!visible || !initialized) {
    return;
  }

  resetPages();
  void browse(0);
});

/*
 * Observe the actual rendered size of this browser.
 *
 * When its parent is `display:none`, this element has zero dimensions.
 * When the parent becomes visible again, ResizeObserver fires and we
 * initialize the browser.
 */
$effect(() => {
  const root = browserRoot;

  if (!root) {
    return;
  }

  visibilityObserver?.disconnect();

  const observer = new ResizeObserver(() => {
    updateVisibility();
  });

  visibilityObserver = observer;
  observer.observe(root);

  /*
   * ResizeObserver callbacks are asynchronous, so also check immediately.
   */
  updateVisibility();

  return () => {
    observer.disconnect();

    if (visibilityObserver === observer) {
      visibilityObserver = null;
    }
  };
});

$effect(() => {
  if (!isVisible) {
    return;
  }

  window.addEventListener("scroll", handleScroll, { passive: true });

  return () => {
    window.removeEventListener("scroll", handleScroll);
  };
});

$effect(() => {
  return () => {
    clearLoadingTimer();

    requestId++;

    visibilityObserver?.disconnect();

    prefetching.clear();
  };
});
</script>

<div bind:this={browserRoot} class="projects-browser">
  <form
    class="toolbar"
    onsubmit={(event) => {
      event.preventDefault();
      submitSearch();
    }}>
    <div class="search">
      <input type="search" bind:value={query} placeholder="Search projects..." aria-label="Search projects" />

      <button type="submit" disabled={loading || !profile || !isVisible}> Search </button>
    </div>

    <select
      value={index}
      onchange={(event) => {
        setIndex(event.currentTarget.value as SearchIndex);
      }}
      aria-label="Sort projects"
      disabled={loading || !profile || !isVisible}>
      <option value="relevance">Relevance</option>
      <option value="downloads">Downloads</option>
      <option value="follows">Follows</option>
      <option value="newest">Newest</option>
      <option value="updated">Updated</option>
    </select>
  </form>

  {#if error}
    <div class="error">
      <span>{error}</span>

      <button type="button" onclick={() => browse(offset)} disabled={loading || !profile || !isVisible}> Retry </button>
    </div>
  {:else if loading && !results}
    <div class="state">
      <span>Loading projects...</span>
    </div>
  {:else if projectResults.length === 0}
    <div class="state">
      <span>No projects found.</span>
    </div>
  {:else}
    <div class:loading class="results">
      {#each projectResults as project (project.project_id)}
        <ProjectCard {profile} {project} />
      {/each}
    </div>

    <footer class="pagination">
      <button type="button" disabled={!hasPreviousPage || loading} onclick={previousPage}> Previous </button>

      {#if results}
        <span>
          {results.offset + 1}
          -
          {Math.min(results.offset + results.limit, results.total_hits)}
          of {results.total_hits.toLocaleString()}
        </span>
      {/if}

      <button type="button" disabled={!hasNextPage || loading} onclick={nextPage}> Next </button>
    </footer>
  {/if}
</div>

<style lang="scss">
.projects-browser {
  display: flex;
  flex-direction: column;
  gap: $space-lg;
  width: 100%;
  min-width: 0;
}

.toolbar {
  display: flex;
  align-items: center;
  gap: $space-md;
}

.search {
  display: flex;
  flex: 1;
  min-width: 0;
  gap: $space-sm;

  input {
    flex: 1;
    min-width: 0;
    padding: $space-sm $space-md;
    border: 1px solid $color-border;
    border-radius: $radius-md;
    background: $color-surface-2;
    color: $color-text;
    font: inherit;
    font-size: 0.85rem;

    &::placeholder {
      color: $color-placeholder;
    }

    &:focus {
      border-color: $color-focus;
      outline: none;
    }
  }
}

select {
  padding: $space-sm $space-md;
  border: 1px solid $color-border;
  border-radius: $radius-md;
  background: $color-surface-2;
  color: $color-text;
  font: inherit;
  font-size: 0.8rem;

  &:focus {
    border-color: $color-focus;
    outline: none;
  }
}

.toolbar button,
.pagination button,
.error button {
  padding: $space-sm $space-md;
  border: 1px solid $color-border;
  border-radius: $radius-md;
  background: $color-surface-2;
  color: $color-text;
  font: inherit;
  font-size: 0.8rem;
  cursor: pointer;

  &:hover:not(:disabled) {
    background: $color-surface-3;
  }

  &:focus-visible {
    outline: 2px solid $color-focus;
    outline-offset: 2px;
  }

  &:disabled {
    cursor: default;
    opacity: 0.5;
  }
}

.results {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax($layout-container-1, 1fr));
  gap: $layout-gap;
  transition: opacity 120ms ease;

  &.loading {
    opacity: 0.6;
    pointer-events: none;
  }
}

.state {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 240px;
  color: $color-text-muted;
  font-size: 0.85rem;
}

.error {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: $space-md;
  padding: $space-md;
  border: 1px solid rgba(239, 68, 68, 0.35);
  border-radius: $radius-md;
  background: rgba(239, 68, 68, 0.08);
  color: $color-error;
  font-size: 0.8rem;
}

.pagination {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: $space-md;
  padding-top: $space-sm;
  color: $color-text-muted;
  font-size: 0.75rem;
}

@media (max-width: 640px) {
  .toolbar {
    align-items: stretch;
    flex-direction: column;
  }

  .search {
    flex-direction: column;
  }

  select {
    width: 100%;
  }
}
</style>
