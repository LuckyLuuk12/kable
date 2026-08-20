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

let results = $state<ModrinthResults | null>(null);
let loading = $state(false);
let error = $state<string | null>(null);

let query = $derived(search.trim());
let index = $state<SearchIndex>("relevance");
let offset = $state(0);

let searchRequest = $derived<ProjectSearch>({
  query: query.trim() || null,
  facets: [],
  index,
  offset,
  limit,
});

let projectResults = $derived(results?.hits ?? []);

let hasNextPage = $derived(results !== null && results.offset + results.limit < results.total_hits);

let hasPreviousPage = $derived(offset > 0);

let requestId = 0;

async function browse(targetOffset = offset) {
  if (!profile) return;

  const id = ++requestId;

  loading = true;
  error = null;

  try {
    const request: ProjectSearch = {
      ...searchRequest,
      offset: targetOffset,
    };

    const nextResults = await app.projectsService.browse(profile, request, smartFilter, projectType);

    if (id !== requestId) return;

    results = nextResults;
  } catch (e) {
    if (id !== requestId) return;

    error = e instanceof Error ? e.message : String(e);
  } finally {
    if (id === requestId) {
      loading = false;
    }
  }
}

function submitSearch() {
  offset = 0;
  browse();
}

function setIndex(value: SearchIndex) {
  index = value;
  offset = 0;
  browse();
}

function nextPage() {
  if (!hasNextPage || loading) return;

  offset += limit;
  browse();
}

function previousPage() {
  if (!hasPreviousPage || loading) return;

  offset = Math.max(0, offset - limit);
  browse();
}

$effect(() => {
  if (!profile) {
    results = null;
    error = null;
    return;
  }

  query;
  index;
  projectType;
  smartFilter;

  offset = 0;
  void browse(0);
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

  {#if error}
    <div class="error">
      <span>{error}</span>

      <button type="button" onclick={() => browse(offset)} disabled={loading || !profile}> Retry </button>
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
