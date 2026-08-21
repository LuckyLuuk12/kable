<!--
@component

Projects browser with infinite scrolling for discovering and installing projects.

Provides:
- Project search
- Project type filtering
- Result sorting
- Infinite scrolling
- Debounced search
- Background prefetching
- Local page caching
- Previous/next page prefetching
- Project cards
- Project details navigation
-->
<script lang="ts">
import { app, type KableProfile, type ModrinthResults, type ProjectSearch, type ProjectType, type SearchIndex } from "$lib";
import { tick } from "svelte";
import ProjectCard from "./ProjectCard.svelte";

let {
  profile,
  projectType,
  search = "",
  smartFilter = true,
}: {
  profile: KableProfile | null;
  projectType: ProjectType;
  search?: string;
  smartFilter?: boolean;
} = $props();

const limit = 20;
const searchDebounce = 450;
const scrollThreshold = 800;
const prefetchPages = 2;

let pages = $state<ModrinthResults[]>([]);
let loading = $state(false);
let loadingMore = $state(false);
let error = $state<string | null>(null);

let query = $state(search.trim());
let index = $state<SearchIndex>("relevance");

let scrollContainer: HTMLDivElement | null = null;
let bottomSentinel: HTMLDivElement | null = null;

let pageCache = new Map<number, ModrinthResults>();
let inFlight = new Map<number, Promise<ModrinthResults | null>>();

let generation = 0;

let searchTimer: ReturnType<typeof setTimeout> | null = null;
let loadMorePromise: Promise<void> | null = null;

let intersectionObserver: IntersectionObserver | null = null;
let resizeObserver: ResizeObserver | null = null;

let projectResults = $derived(pages.flatMap((page) => page.hits));

let totalHits = $derived(pages.length > 0 ? pages[0].total_hits : 0);

let loadedHits = $derived(projectResults.length);

let hasNextPage = $derived(pages.length > 0 && pages[pages.length - 1].offset + pages[pages.length - 1].hits.length < pages[pages.length - 1].total_hits);

function createRequest(offset: number): ProjectSearch {
  return {
    query: query.trim() || null,
    facets: [],
    index,
    offset,
    limit,
  };
}

function getNextOffset(): number {
  if (pages.length === 0) {
    return 0;
  }

  const lastPage = pages[pages.length - 1];

  return lastPage.offset + lastPage.hits.length;
}

function getPreviousOffset(): number | null {
  if (pages.length === 0) {
    return null;
  }

  const firstPage = pages[0];

  if (firstPage.offset <= 0) {
    return null;
  }

  return Math.max(0, firstPage.offset - limit);
}

function isNearBottom(): boolean {
  if (!scrollContainer) {
    return false;
  }

  const remaining = scrollContainer.scrollHeight - scrollContainer.scrollTop - scrollContainer.clientHeight;

  return remaining <= scrollThreshold;
}

function canLoadNextPage(): boolean {
  return profile !== null && pages.length > 0 && hasNextPage;
}

async function fetchPage(targetOffset: number, requestGeneration: number): Promise<ModrinthResults | null> {
  if (!profile) {
    return null;
  }

  if (requestGeneration !== generation) {
    return null;
  }

  const cached = pageCache.get(targetOffset);

  if (cached) {
    console.debug("[ProjectsScrollBrowser] Cache hit:", targetOffset);

    return cached;
  }

  const existingRequest = inFlight.get(targetOffset);

  if (existingRequest) {
    console.debug("[ProjectsScrollBrowser] Awaiting in-flight page:", targetOffset);

    return existingRequest;
  }

  console.debug("[ProjectsScrollBrowser] Fetching page:", targetOffset);

  const request = app.projectsService
    .browse(profile, createRequest(targetOffset), smartFilter, projectType)
    .then((result) => {
      if (requestGeneration !== generation) {
        console.debug("[ProjectsScrollBrowser] Ignoring stale page:", targetOffset);

        return null;
      }

      pageCache.set(targetOffset, result);

      console.debug("[ProjectsScrollBrowser] Fetched page:", targetOffset, result.hits.length, "results");

      return result;
    })
    .catch((e) => {
      console.debug("[ProjectsScrollBrowser] Failed page:", targetOffset, e);

      return null;
    });

  inFlight.set(targetOffset, request);

  try {
    return await request;
  } finally {
    if (inFlight.get(targetOffset) === request) {
      inFlight.delete(targetOffset);
    }
  }
}

async function prefetchPage(targetOffset: number, requestGeneration: number = generation): Promise<void> {
  if (!profile) {
    return;
  }

  if (requestGeneration !== generation) {
    return;
  }

  if (targetOffset < 0 || targetOffset >= totalHits) {
    return;
  }

  if (pageCache.has(targetOffset)) {
    return;
  }

  console.debug("[ProjectsScrollBrowser] Prefetching:", targetOffset);

  await fetchPage(targetOffset, requestGeneration);
}

function prefetchAdjacentPages(): void {
  if (!profile || pages.length === 0) {
    return;
  }

  const currentGeneration = generation;

  const nextOffset = getNextOffset();

  for (let i = 0; i < prefetchPages; i++) {
    const targetOffset = nextOffset + i * limit;

    if (targetOffset >= totalHits) {
      break;
    }

    void prefetchPage(targetOffset, currentGeneration);
  }

  const previousOffset = getPreviousOffset();

  if (previousOffset !== null) {
    void prefetchPage(previousOffset, currentGeneration);
  }
}

async function appendNextPage(): Promise<void> {
  if (!canLoadNextPage()) {
    return;
  }

  if (loadMorePromise) {
    return loadMorePromise;
  }

  const currentGeneration = generation;
  const targetOffset = getNextOffset();

  if (pages.some((page) => page.offset === targetOffset)) {
    return;
  }

  const cachedPage = pageCache.get(targetOffset);

  loadMorePromise = (async () => {
    if (currentGeneration !== generation) {
      return;
    }

    loadingMore = true;
    error = null;

    try {
      const nextPage = cachedPage ?? (await fetchPage(targetOffset, currentGeneration));

      if (currentGeneration !== generation) {
        return;
      }

      if (!nextPage) {
        throw new Error("Failed to load projects.");
      }

      if (pages.some((page) => page.offset === nextPage.offset)) {
        return;
      }

      pages = [...pages, nextPage];

      console.debug("[ProjectsScrollBrowser] Appended page:", nextPage.offset);

      prefetchAdjacentPages();

      await tick();

      if (currentGeneration === generation && canLoadNextPage() && isNearBottom()) {
        void appendNextPage();
      }
    } catch (e) {
      if (currentGeneration !== generation) {
        return;
      }

      error = e instanceof Error ? e.message : String(e);
    } finally {
      if (currentGeneration === generation) {
        loadingMore = false;
      }
    }
  })();

  try {
    await loadMorePromise;
  } finally {
    loadMorePromise = null;
  }
}

async function ensureScrollable(): Promise<void> {
  if (!profile || pages.length === 0) {
    return;
  }

  const currentGeneration = generation;

  await tick();

  while (currentGeneration === generation && canLoadNextPage() && scrollContainer && scrollContainer.scrollHeight <= scrollContainer.clientHeight) {
    console.debug("[ProjectsScrollBrowser] Content does not fill container, loading next page");

    await appendNextPage();
    await tick();
  }
}

async function loadInitialPage(): Promise<void> {
  if (!profile) {
    return;
  }

  const currentGeneration = generation;

  loading = true;
  loadingMore = false;
  error = null;

  console.debug("[ProjectsScrollBrowser] Loading initial page");

  try {
    const firstPage = await fetchPage(0, currentGeneration);

    if (currentGeneration !== generation) {
      return;
    }

    if (!firstPage) {
      throw new Error("Failed to load projects.");
    }

    pages = [firstPage];

    console.debug("[ProjectsScrollBrowser] Initial page loaded");

    prefetchAdjacentPages();

    await tick();

    await ensureScrollable();
  } catch (e) {
    if (currentGeneration !== generation) {
      return;
    }

    error = e instanceof Error ? e.message : String(e);
  } finally {
    if (currentGeneration === generation) {
      loading = false;
    }
  }
}

function resetState(): void {
  generation++;

  pages = [];
  pageCache = new Map();
  inFlight = new Map();

  loading = false;
  loadingMore = false;
  error = null;
  loadMorePromise = null;

  if (scrollContainer) {
    scrollContainer.scrollTop = 0;
  }

  console.debug("[ProjectsScrollBrowser] State reset. Generation:", generation);
}

function scheduleReload(): void {
  if (searchTimer !== null) {
    clearTimeout(searchTimer);
    searchTimer = null;
  }

  searchTimer = setTimeout(() => {
    searchTimer = null;

    resetState();
    void loadInitialPage();
  }, searchDebounce);
}

function submitSearch(): void {
  if (searchTimer !== null) {
    clearTimeout(searchTimer);
    searchTimer = null;
  }

  resetState();
  void loadInitialPage();
}

function setIndex(value: SearchIndex): void {
  if (value === index) {
    return;
  }

  index = value;

  if (searchTimer !== null) {
    clearTimeout(searchTimer);
    searchTimer = null;
  }

  resetState();
  void loadInitialPage();
}

function retry(): void {
  if (!profile) {
    return;
  }

  error = null;

  if (pages.length === 0) {
    void loadInitialPage();
  } else {
    void appendNextPage();
  }
}

function handleScroll(event: Event): void {
  const target = event.currentTarget;

  if (!(target instanceof HTMLDivElement)) {
    return;
  }

  if (loading || loadingMore || !hasNextPage) {
    return;
  }

  const remaining = target.scrollHeight - target.scrollTop - target.clientHeight;

  console.debug("[ProjectsScrollBrowser] Scroll:", {
    scrollTop: target.scrollTop,
    scrollHeight: target.scrollHeight,
    clientHeight: target.clientHeight,
    remaining,
  });

  if (remaining > scrollThreshold) {
    return;
  }

  console.debug("[ProjectsScrollBrowser] Near bottom, loading next page");

  void appendNextPage();
}

function setupIntersectionObserver(node: HTMLDivElement): void {
  intersectionObserver?.disconnect();

  intersectionObserver = new IntersectionObserver(
    (entries) => {
      const entry = entries[0];

      if (!entry?.isIntersecting) {
        return;
      }

      console.debug("[ProjectsScrollBrowser] Bottom sentinel intersected");

      if (!loading && !loadingMore && hasNextPage) {
        void appendNextPage();
      }
    },
    {
      root: scrollContainer,
      rootMargin: `${scrollThreshold}px 0px`,
      threshold: 0,
    },
  );

  intersectionObserver.observe(node);
}

function destroyIntersectionObserver(): void {
  intersectionObserver?.disconnect();
  intersectionObserver = null;
}

$effect(() => {
  const currentProfile = profile;

  projectType;
  smartFilter;
  index;
  query;

  if (!currentProfile) {
    resetState();
    return;
  }

  scheduleReload();
});

$effect(() => {
  const container = scrollContainer;

  if (!container) {
    return;
  }

  resizeObserver?.disconnect();

  resizeObserver = new ResizeObserver(() => {
    if (!loading && !loadingMore && hasNextPage && container.scrollHeight <= container.clientHeight) {
      console.debug("[ProjectsScrollBrowser] Resize observer detected non-scrollable content");

      void appendNextPage();
    }
  });

  resizeObserver.observe(container);

  return () => {
    resizeObserver?.disconnect();
    resizeObserver = null;
  };
});

$effect(() => {
  const sentinel = bottomSentinel;

  if (!sentinel) {
    return;
  }

  setupIntersectionObserver(sentinel);

  return () => {
    destroyIntersectionObserver();
  };
});

$effect(() => {
  return () => {
    if (searchTimer !== null) {
      clearTimeout(searchTimer);
      searchTimer = null;
    }

    intersectionObserver?.disconnect();
    resizeObserver?.disconnect();

    generation++;

    inFlight.clear();
  };
});
</script>

<div class="projects-browser">
  <form
    class="toolbar"
    onsubmit={(event) => {
      event.preventDefault();
      submitSearch();
    }}>
    <div class="search">
      <input type="search" bind:value={query} placeholder="Search projects..." aria-label="Search projects" />

      <button type="submit" disabled={loading || !profile}> Search </button>
    </div>

    <select
      value={index}
      onchange={(event) => {
        setIndex(event.currentTarget.value as SearchIndex);
      }}
      aria-label="Sort projects"
      disabled={loading || !profile}>
      <option value="relevance">Relevance</option>
      <option value="downloads">Downloads</option>
      <option value="follows">Follows</option>
      <option value="newest">Newest</option>
      <option value="updated">Updated</option>
    </select>
  </form>

  {#if error && pages.length === 0}
    <div class="error">
      <span>{error}</span>

      <button type="button" onclick={retry} disabled={loading || !profile}> Retry </button>
    </div>
  {:else if loading && pages.length === 0}
    <div class="state">
      <div class="spinner" aria-hidden="true"></div>

      <span>Loading projects...</span>
    </div>
  {:else if projectResults.length === 0}
    <div class="state">
      <span>No projects found.</span>
    </div>
  {:else}
    <div bind:this={scrollContainer} class="results-container" onscroll={handleScroll}>
      <div class="results">
        {#each projectResults as project (project.project_id)}
          <ProjectCard {profile} {project} />
        {/each}
      </div>

      <div bind:this={bottomSentinel} class="bottom-sentinel" aria-hidden="true"></div>

      <div class="scroll-status">
        {#if error}
          <div class="error">
            <span>{error}</span>

            <button type="button" onclick={retry} disabled={loadingMore || !profile}> Retry </button>
          </div>
        {:else if loadingMore}
          <div class="loading-indicator">
            <div class="spinner small" aria-hidden="true"></div>

            <span> Loading more projects... </span>
          </div>
        {:else if !hasNextPage}
          <span>
            Showing all
            {totalHits.toLocaleString()}
            projects
          </span>
        {:else}
          <span>
            Showing
            {loadedHits.toLocaleString()}
            of
            {totalHits.toLocaleString()}
          </span>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style lang="scss">
.projects-browser {
  display: flex;
  flex-direction: column;
  gap: $space-lg;
  width: 100%;
  min-width: 0;
  min-height: 0;
}

.toolbar {
  display: flex;
  align-items: center;
  gap: $space-md;
  flex-shrink: 0;
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

.results-container {
  min-height: 0;
  height: calc(100vh - 220px);
  max-height: calc(100vh - 220px);
  overflow-y: auto;
  overflow-x: hidden;
  padding-right: $space-xs;
  scrollbar-width: thin;
}

.results {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax($layout-container-1, 1fr));
  gap: $layout-gap;
}

.bottom-sentinel {
  width: 100%;
  height: 1px;
  pointer-events: none;
}

.state {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: $space-sm;
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

.scroll-status {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 52px;
  padding: $space-md 0;
  color: $color-text-muted;
  font-size: 0.75rem;
}

.loading-indicator {
  display: flex;
  align-items: center;
  gap: $space-sm;
}

.spinner {
  width: 18px;
  height: 18px;
  border: 2px solid $color-border;
  border-top-color: $color-text-muted;
  border-radius: 50%;
  animation: spin 700ms linear infinite;

  &.small {
    width: 14px;
    height: 14px;
    border-width: 2px;
  }
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
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

  .results-container {
    height: calc(100vh - 280px);
    max-height: calc(100vh - 280px);
  }
}
</style>
