<!--
@component

Paginated project browser for discovering and installing projects.

Provides:
- Paginated project results
- Project cards
- Page prefetching
- Project details navigation
- Visibility-aware loading
-->
<script lang="ts">
import { app, type FacetGroup, type KableProfile, type ModrinthResults, type ProjectSearch, type ProjectType, type SearchIndex } from "$lib";
import { untrack } from "svelte";
import Icon from "../Icon.svelte";
import ProjectCard from "./ProjectCard.svelte";

let {
  profile,
  profileId,
  projectType,
  search = "",
  smartFilter = true,
  facets = [],
  index = "relevance",
}: {
  profile: KableProfile | null;
  profileId: string | null;
  projectType: ProjectType;
  search?: string;
  smartFilter?: boolean;
  facets?: FacetGroup[];
  index?: SearchIndex;
} = $props();

const loadingDelay = 300;
const minLimit = 1;
const maxLimit = 100;

const pageButtonWidth = 32;
const pageButtonGap = 2;
const ellipsisWidth = 20;

let limit = $state(16);

let results = $state<ModrinthResults | null>(null);
let loading = $state(false);
let error = $state<string | null>(null);
let offset = $state(0);

let browserRoot: HTMLDivElement | null = null;
let resultsRoot: HTMLDivElement | null = null;
let pageNumbersRoot: HTMLElement | null = null;

let projectResults = $derived(results?.hits ?? []);

let totalPages = $derived(results ? Math.max(1, Math.ceil(results.total_hits / limit)) : 1);

let currentPage = $derived(results ? Math.floor(results.offset / limit) + 1 : 1);

let hasNextPage = $derived(results !== null && currentPage < totalPages);

let hasPreviousPage = $derived(currentPage > 1);

let pageNumbersWidth = $state(0);

function getPageButtonWidth(page: number): number {
  if (page >= 100) {
    return 38;
  }

  if (page >= 10) {
    return 32;
  }

  return pageButtonWidth;
}

function getPaginationWidth(pages: number[]): number {
  if (pages.length === 0) {
    return 0;
  }

  let width = 0;

  for (let i = 0; i < pages.length; i++) {
    const page = pages[i];

    width += getPageButtonWidth(page);

    if (i > 0) {
      width += pageButtonGap;

      if (page - pages[i - 1] > 1) {
        width += ellipsisWidth;
      }
    }
  }

  return width;
}

function canFitPages(pages: number[]): boolean {
  return getPaginationWidth(pages) <= pageNumbersWidth;
}

let pageNumbers = $derived.by(() => {
  if (!results) {
    return [];
  }

  if (totalPages <= 1) {
    return [1];
  }

  const allPages = Array.from({ length: totalPages }, (_, index) => index + 1);

  if (canFitPages(allPages)) {
    return allPages;
  }

  /*
   * These pages are always preferred:
   * - first three pages
   * - final page
   * - current page and its immediate neighbours
   */
  const pages = new Set<number>();

  pages.add(1);

  if (totalPages >= 2) {
    pages.add(2);
  }

  if (totalPages >= 3) {
    pages.add(3);
  }

  pages.add(totalPages);

  const preferredPages = [currentPage - 2, currentPage - 1, currentPage, currentPage + 1, currentPage + 2].filter((page) => page >= 1 && page <= totalPages);

  for (const page of preferredPages) {
    if (pages.has(page)) {
      continue;
    }

    const candidate = [...pages, page].sort((a, b) => a - b);

    if (canFitPages(candidate)) {
      pages.add(page);
    }
  }

  /*
   * Expand around the current page as long as the resulting
   * pagination still fits inside the available width.
   */
  let distance = 3;

  while (distance <= totalPages) {
    const candidates = [currentPage - distance, currentPage + distance];

    let added = false;

    for (const page of candidates) {
      if (page < 1 || page > totalPages || pages.has(page)) {
        continue;
      }

      const candidate = [...pages, page].sort((a, b) => a - b);

      if (canFitPages(candidate)) {
        pages.add(page);
        added = true;
      }
    }

    if (!added) {
      break;
    }

    distance++;
  }

  /*
   * Finally, use any remaining space for pages closest to the
   * current page. This makes the pagination aggressively use
   * available horizontal space instead of leaving large gaps.
   */
  const remainingPages = allPages
    .filter((page) => !pages.has(page))
    .sort((a, b) => {
      const distanceA = Math.abs(a - currentPage);
      const distanceB = Math.abs(b - currentPage);

      if (distanceA !== distanceB) {
        return distanceA - distanceB;
      }

      return a - b;
    });

  for (const page of remainingPages) {
    const candidate = [...pages, page].sort((a, b) => a - b);

    if (canFitPages(candidate)) {
      pages.add(page);
    }
  }

  return [...pages].sort((a, b) => a - b);
});

let paginationItems = $derived.by(() => {
  const items: Array<number | "ellipsis"> = [];

  for (let i = 0; i < pageNumbers.length; i++) {
    const page = pageNumbers[i];

    if (i > 0) {
      const previousPage = pageNumbers[i - 1];

      if (page - previousPage > 1) {
        items.push("ellipsis");
      }
    }

    items.push(page);
  }

  return items;
});

let requestId = 0;
let loadingTimer: ReturnType<typeof setTimeout> | null = null;

// eslint-disable-next-line svelte/prefer-svelte-reactivity
let prefetching = new Set<number>();

// eslint-disable-next-line svelte/prefer-svelte-reactivity
let pageCache = new Map<number, ModrinthResults>();

let visibilityObserver: ResizeObserver | null = null;
let pageNumbersObserver: ResizeObserver | null = null;

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
    query: search.trim() || null,
    facets,
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

  console.debug("[ProjectsPaginatedBrowser] Visibility changed:", visible);

  if (!visible) {
    return;
  }

  if (!hasInitialized && profile) {
    hasInitialized = true;
    void browse(0);
  }
}

function updatePageNumbersWidth(): void {
  if (!pageNumbersRoot) {
    return;
  }

  pageNumbersWidth = pageNumbersRoot.getBoundingClientRect().width;
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

    requestAnimationFrame(() => {
      resultsRoot?.scrollTo({
        top: 0,
        behavior: "auto",
      });
    });

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

    requestAnimationFrame(() => {
      resultsRoot?.scrollTo({
        top: 0,
        behavior: "auto",
      });
    });
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

  resultsRoot?.scrollTo({
    top: 0,
    behavior: "auto",
  });
}

function setLimit(value: number) {
  const nextLimit = Math.min(maxLimit, Math.max(minLimit, Math.floor(value) || minLimit));

  if (nextLimit === limit) {
    return;
  }

  limit = nextLimit;

  resetPages();

  if (isVisible && hasInitialized) {
    void browse(0);
  }
}

function handleLimitInput(event: Event) {
  const input = event.currentTarget as HTMLInputElement;
  const value = Number.parseInt(input.value, 10);

  if (!Number.isNaN(value)) {
    setLimit(value);
  }
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

function goToPage(page: number) {
  if (!isVisible || page < 1 || page > totalPages || page === currentPage) {
    return;
  }

  void browse((page - 1) * limit);
}

function handleResultsScroll() {
  if (!isVisible || !results || !hasNextPage || !resultsRoot) {
    return;
  }

  const scrollPosition = resultsRoot.scrollTop + resultsRoot.clientHeight;

  const threshold = resultsRoot.scrollHeight - 600;

  if (scrollPosition >= threshold) {
    const nextOffset = results.offset + limit;

    if (!pageCache.has(nextOffset) && !prefetching.has(nextOffset)) {
      void prefetchPage(nextOffset);
    }
  }
}

$effect(() => {
  if (!profileId) {
    results = null;
    error = null;
    resetPages();
    hasInitialized = false;
    return;
  }

  search;
  index;
  projectType;
  smartFilter;
  facets;

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

  updateVisibility();

  return () => {
    observer.disconnect();

    if (visibilityObserver === observer) {
      visibilityObserver = null;
    }
  };
});

$effect(() => {
  const root = pageNumbersRoot;

  if (!root) {
    return;
  }

  pageNumbersObserver?.disconnect();

  const observer = new ResizeObserver(() => {
    updatePageNumbersWidth();
  });

  pageNumbersObserver = observer;
  observer.observe(root);

  updatePageNumbersWidth();

  return () => {
    observer.disconnect();

    if (pageNumbersObserver === observer) {
      pageNumbersObserver = null;
    }
  };
});

$effect(() => {
  const root = resultsRoot;

  if (!root || !isVisible) {
    return;
  }

  root.addEventListener("scroll", handleResultsScroll, {
    passive: true,
  });

  return () => {
    root.removeEventListener("scroll", handleResultsScroll);
  };
});

$effect(() => {
  return () => {
    clearLoadingTimer();

    requestId++;

    visibilityObserver?.disconnect();
    pageNumbersObserver?.disconnect();

    prefetching.clear();
  };
});
</script>

<div bind:this={browserRoot} class="projects-browser">
  <header class="pagination">
    <div class="result-range">
      <span>1 -</span>

      <input
        class="limit-input"
        type="number"
        min={minLimit}
        max={maxLimit}
        step="1"
        value={limit}
        aria-label="Projects per page"
        title="Projects per page"
        onchange={handleLimitInput} />

      <span>
        of {results?.total_hits.toLocaleString() ?? "0"}
      </span>
    </div>

    <div class="pagination-controls">
      <button type="button" class="navigation-button" disabled={!hasPreviousPage || loading} onclick={previousPage} aria-label="Previous page" title="Previous page">
        <Icon name="chevron-left" forceType="svg" />
      </button>

      <nav bind:this={pageNumbersRoot} class="page-numbers" aria-label="Project pages">
        {#each paginationItems as item, i (item + "+" + i)}
          {#if item === "ellipsis"}
            <span class="ellipsis" aria-hidden="true" data-index={i}> ... </span>
          {:else}
            <button
              type="button"
              class:current={item === currentPage}
              class="page-button"
              disabled={loading}
              aria-label={`Page ${item}`}
              aria-current={item === currentPage ? "page" : undefined}
              onclick={() => goToPage(item)}>
              {item}
            </button>
          {/if}
        {/each}
      </nav>

      <button type="button" class="navigation-button" disabled={!hasNextPage || loading} onclick={nextPage} aria-label="Next page" title="Next page">
        <Icon name="chevron-right" forceType="svg" />
      </button>
    </div>
  </header>

  <div bind:this={resultsRoot} class="results-container">
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
    {/if}
  </div>
</div>

<style lang="scss">
.projects-browser {
  display: flex;
  flex-direction: column;

  width: 100%;
  height: 100%;
  min-width: 0;
  min-height: 0;

  overflow: hidden;
}

.pagination {
  display: flex;
  align-items: center;
  justify-content: space-between;

  gap: $space-lg;

  padding-bottom: $space-md;

  color: $color-text-muted;
  font-size: 0.75rem;
}

.result-range {
  display: flex;
  align-items: center;
  gap: $space-xs;

  flex-shrink: 0;

  white-space: nowrap;
}

.limit-input {
  width: 42px;
  height: 28px;
  padding: 0 $space-xs;

  border: 1px solid $color-border;
  border-radius: $radius-sm;

  background: $color-surface-2;
  color: $color-text;

  font: inherit;
  font-size: 0.75rem;
  font-weight: 500;
  text-align: center;

  appearance: textfield;

  &:hover {
    border-color: $color-accent;
    background: $color-surface-3;
  }

  &:focus {
    border-color: $color-accent;
    outline: none;
    background: $color-surface-3;
  }

  &::-webkit-inner-spin-button,
  &::-webkit-outer-spin-button {
    margin: 0;
    appearance: none;
  }
}

.pagination-controls {
  display: flex;
  align-items: center;

  flex: 1;
  min-width: 0;

  gap: $space-xs;
}

.page-numbers {
  display: flex;
  align-items: center;
  justify-content: center;

  flex: 1;
  min-width: 0;

  gap: 2px;

  overflow: hidden;
}

.navigation-button,
.page-button {
  display: grid;
  place-items: center;

  min-width: $space-2xl;
  padding: $space-xs;

  border: 1px solid transparent;
  border-radius: $radius-sm;

  background: transparent;
  color: $color-text-muted;

  font: inherit;
  font-size: 0.75rem;

  cursor: pointer;

  &:hover:not(:disabled) {
    border-color: $color-border;
    background: $color-surface-3;
    color: $color-text;
  }

  &:focus-visible {
    outline: 2px solid $color-focus;
    outline-offset: 1px;
  }

  &:disabled {
    cursor: default;
    opacity: 0.4;
  }
}

.navigation-button {
  flex-shrink: 0;

  border-color: $color-border;
  background: $color-surface-2;

  &:hover:not(:disabled) {
    background: $color-surface-3;
  }
}

.page-button {
  flex-shrink: 0;
}

.page-button.current {
  border-color: $color-accent;
  background: $color-accent;
  color: $color-text;

  cursor: default;
}

.ellipsis {
  display: grid;
  place-items: center;

  flex-shrink: 0;

  min-width: $space-xl;
  height: 28px;

  color: $color-text-muted;

  user-select: none;
}

.results-container {
  flex: 1;
  min-height: 0;

  overflow-y: auto;
  overflow-x: hidden;

  scrollbar-width: none;
  -ms-overflow-style: none;

  &::-webkit-scrollbar {
    display: none;
  }
}

.results {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax($layout-container-1, 1fr));
  gap: $layout-gap;

  padding-bottom: $space-md;

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
</style>
