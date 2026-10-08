<!--
@component

Layout and filter controls for the projects browser.

Provides:
- Project search
- Result sorting
- Smart compatibility filtering
- Pagination / scrolling browser mode
- Modrinth category/loader/game-version/side filters
- Include/exclude filtering through FacetOperator
- Collapsible right filter sidebar
- Rendered browser content through the Svelte 5 children snippet
-->

<script lang="ts">
import { api, type Facet, type FacetField, type FacetGroup, type FacetOperator, type SearchIndex } from "$lib";
import { onMount } from "svelte";
import Icon from "../Icon.svelte";

let {
  search = $bindable(""),
  index = $bindable<SearchIndex>("relevance"),
  smartFilter = $bindable(true),
  facets = $bindable<FacetGroup[]>([]),
  showPaginated = $bindable(false),
  children,
}: {
  search?: string;
  index?: SearchIndex;
  smartFilter?: boolean;
  facets?: FacetGroup[];
  showPaginated?: boolean;
  children?: import("svelte").Snippet;
} = $props();

/*
 * Keep the text currently being edited separate from the submitted
 * search value. This prevents every keystroke from triggering a browse.
 */
let query = $state(search);

let categories = $state<string[]>([]);
let loaders = $state<string[]>([]);
let gameVersions = $state<string[]>([]);
let sideTypes = $state<string[]>([]);

let loadingFilters = $state(false);
let filterError = $state<string | null>(null);
let sidebarOpen = $state(true);

/*
 * Modrinth metadata should normally contain unique values, but duplicate
 * values have been observed. Deduplicate before rendering because keyed
 * each blocks require unique keys.
 */
let uniqueCategories = $derived([...new Set(categories)]);
let uniqueLoaders = $derived([...new Set(loaders)]);
let uniqueGameVersions = $derived([...new Set(gameVersions)]);
let uniqueSideTypes = $derived([...new Set(sideTypes)]);

/*
 * The bound `facets` value is the source of truth.
 */
let selectedFilters = $derived(facets.flatMap((group) => group.facets));

function facetValue(namespace: string, value: string): string {
  return `${namespace}:${value}`;
}

function createFacet(field: FacetField, operator: FacetOperator, value: string): Facet {
  return {
    field,
    operator,
    value,
  };
}

function getFacet(field: FacetField, value: string): Facet | undefined {
  return selectedFilters.find((facet) => facet.field === field && facet.value === value);
}

function getFilterOperator(field: FacetField, value: string): FacetOperator | null {
  return getFacet(field, value)?.operator ?? null;
}

/*
 * Rebuild the grouped facet representation after changing one filter.
 *
 * Modrinth facet semantics:
 *
 *   Groups = AND
 *   Facets in a group = OR
 *
 * Equality facets with the same field and namespace are grouped together.
 * Exclusions are kept in separate groups so they behave as AND conditions.
 */
function rebuildFacets(updatedFilters: Facet[]) {
  // eslint-disable-next-line svelte/prefer-svelte-reactivity
  const includeGroups = new Map<string, Facet[]>();
  const excludeGroups: FacetGroup[] = [];

  for (const facet of updatedFilters) {
    if (facet.operator === "not_eq") {
      excludeGroups.push({
        facets: [facet],
      });

      continue;
    }

    const namespace = facet.value.split(":", 1)[0] ?? facet.value;

    const key = `${facet.field}:${namespace}`;
    const group = includeGroups.get(key);

    if (group) {
      group.push(facet);
    } else {
      includeGroups.set(key, [facet]);
    }
  }

  facets = [
    ...includeGroups.values().map(
      (group): FacetGroup => ({
        facets: group,
      }),
    ),
    ...excludeGroups,
  ];
}

function setFacet(field: FacetField, value: string, operator: FacetOperator | null) {
  const remaining = selectedFilters.filter((facet) => !(facet.field === field && facet.value === value));

  if (operator === null) {
    rebuildFacets(remaining);
    return;
  }

  rebuildFacets([...remaining, createFacet(field, operator, value)]);
}

function toggleInclude(field: FacetField, value: string) {
  const current = getFacet(field, value);

  setFacet(field, value, current?.operator === "eq" ? null : "eq");
}

function toggleExclude(field: FacetField, value: string) {
  const current = getFacet(field, value);

  setFacet(field, value, current?.operator === "not_eq" ? null : "not_eq");
}

function submitSearch() {
  search = query.trim();
}

function clearSearch() {
  query = "";
  search = "";
}

function setIndex(value: SearchIndex) {
  index = value;
}

async function loadFilters() {
  loadingFilters = true;
  filterError = null;

  try {
    const [loadedCategories, loadedLoaders, loadedGameVersions, loadedSideTypes] = await Promise.all([
      api.getModrinthCategories(),
      api.getModrinthLoaders(),
      api.getModrinthGameVersions(),
      api.getModrinthSideTypes(),
    ]);

    categories = loadedCategories;
    loaders = loadedLoaders;
    gameVersions = loadedGameVersions;
    sideTypes = loadedSideTypes;
  } catch (error) {
    filterError = error instanceof Error ? error.message : String(error);
  } finally {
    loadingFilters = false;
  }
}

onMount(() => {
  void loadFilters();
});

/*
 * Keep the local search input synchronized when the parent changes
 * the submitted search value externally.
 */
$effect(() => {
  if (query !== search) {
    query = search;
  }
});
</script>

<div class="projects-browser-filters">
  <div class="topbar">
    <form
      class="search"
      onsubmit={(event) => {
        event.preventDefault();
        submitSearch();
      }}>
      <input type="search" bind:value={query} placeholder="Search projects..." aria-label="Search projects" />

      <button type="submit" class="search-button" aria-label="Search projects">
        <Icon name="search" forceType="svg" />
      </button>

      {#if query}
        <button type="button" class="clear-search" onclick={clearSearch} aria-label="Clear search">
          <Icon name="close" forceType="svg" />
        </button>
      {/if}
    </form>

    <select
      value={index}
      onchange={(event) => {
        setIndex(event.currentTarget.value as SearchIndex);
      }}
      aria-label="Sort projects">
      <option value="relevance">Relevance</option>
      <option value="downloads">Downloads</option>
      <option value="follows">Follows</option>
      <option value="newest">Newest</option>
      <option value="updated">Updated</option>
    </select>

    <button
      type="button"
      class="toolbar-button"
      class:active={smartFilter}
      onclick={() => {
        smartFilter = !smartFilter;
      }}
      aria-pressed={smartFilter}>
      <Icon name="filter" forceType="svg" />
      Smart filter
    </button>

    <button
      type="button"
      class="toolbar-button"
      onclick={() => {
        showPaginated = !showPaginated;
      }}
      aria-label={showPaginated ? "Switch to scrolling browser" : "Switch to paginated browser"}
      title={showPaginated ? "Switch to scrolling browser" : "Switch to paginated browser"}>
      <Icon name={showPaginated ? "search" : "list"} forceType="svg" />
      {showPaginated ? "Scroll" : "Pages"}
    </button>

    <button
      type="button"
      class="toolbar-button"
      class:active={sidebarOpen}
      onclick={() => {
        sidebarOpen = !sidebarOpen;
      }}
      aria-expanded={sidebarOpen}
      aria-controls="project-filters-sidebar">
      <Icon name="filter" forceType="svg" />
      Filters
    </button>
  </div>

  <div class="main-layout">
    <main class="browser-content">
      {@render children?.()}
    </main>

    {#if sidebarOpen}
      <aside id="project-filters-sidebar" class="filter-sidebar">
        <div class="sidebar-header">
          <h2>Filters</h2>

          <button
            type="button"
            class="close-button"
            onclick={() => {
              sidebarOpen = false;
            }}
            aria-label="Close filters">
            <Icon name="close" forceType="svg" />
          </button>
        </div>

        {#if loadingFilters}
          <div class="state">Loading filters...</div>
        {:else if filterError}
          <div class="error">
            {filterError}
          </div>
        {:else}
          <div class="filter-list">
            <!-- Categories -->
            <details class="filter-group">
              <summary>Categories</summary>

              <div class="filter-options">
                {#each uniqueCategories as category (category)}
                  {@const value = facetValue("categories", category)}
                  {@const operator = getFilterOperator("categories", value)}

                  <div class="filter-row" class:filter-selected={operator !== null}>
                    <span>{category}</span>

                    <div class="filter-actions">
                      <button
                        type="button"
                        class="filter-action include"
                        class:include-active={operator === "eq"}
                        onclick={() => toggleInclude("categories", value)}
                        aria-label={operator === "eq" ? `Remove ${category} include filter` : `Include ${category}`}
                        aria-pressed={operator === "eq"}
                        title={operator === "eq" ? `Stop including ${category}` : `Include ${category}`}>
                        <Icon name="success" forceType="svg" />
                      </button>

                      <button
                        type="button"
                        class="filter-action exclude"
                        class:exclude-active={operator === "not_eq"}
                        onclick={() => toggleExclude("categories", value)}
                        aria-label={operator === "not_eq" ? `Remove ${category} exclude filter` : `Exclude ${category}`}
                        aria-pressed={operator === "not_eq"}
                        title={operator === "not_eq" ? `Stop excluding ${category}` : `Exclude ${category}`}>
                        <Icon name="error" forceType="svg" />
                      </button>
                    </div>
                  </div>
                {/each}
              </div>
            </details>

            <!-- Client-side support -->
            <details class="filter-group">
              <summary>Client-side support</summary>

              <div class="filter-options">
                {#each uniqueSideTypes as side (side)}
                  {@const value = facetValue("client_side", side)}
                  {@const operator = getFilterOperator("client_side", value)}

                  <div class="filter-row" class:filter-selected={operator !== null}>
                    <span>{side}</span>

                    <div class="filter-actions">
                      <button
                        type="button"
                        class="filter-action include"
                        class:include-active={operator === "eq"}
                        onclick={() => toggleInclude("client_side", value)}
                        aria-label={operator === "eq" ? `Remove client-side ${side} include filter` : `Require client-side ${side}`}
                        aria-pressed={operator === "eq"}
                        title={operator === "eq" ? `Stop requiring client-side ${side}` : `Require client-side ${side}`}>
                        <Icon name="success" forceType="svg" />
                      </button>

                      <button
                        type="button"
                        class="filter-action exclude"
                        class:exclude-active={operator === "not_eq"}
                        onclick={() => toggleExclude("client_side", value)}
                        aria-label={operator === "not_eq" ? `Remove client-side ${side} exclude filter` : `Exclude client-side ${side}`}
                        aria-pressed={operator === "not_eq"}
                        title={operator === "not_eq" ? `Stop excluding client-side ${side}` : `Exclude client-side ${side}`}>
                        <Icon name="error" forceType="svg" />
                      </button>
                    </div>
                  </div>
                {/each}
              </div>
            </details>

            <!-- Server-side support -->
            <details class="filter-group">
              <summary>Server-side support</summary>

              <div class="filter-options">
                {#each uniqueSideTypes as side (side)}
                  {@const value = facetValue("server_side", side)}
                  {@const operator = getFilterOperator("server_side", value)}

                  <div class="filter-row" class:filter-selected={operator !== null}>
                    <span>{side}</span>

                    <div class="filter-actions">
                      <button
                        type="button"
                        class="filter-action include"
                        class:include-active={operator === "eq"}
                        onclick={() => toggleInclude("server_side", value)}
                        aria-label={operator === "eq" ? `Remove server-side ${side} include filter` : `Require server-side ${side}`}
                        aria-pressed={operator === "eq"}
                        title={operator === "eq" ? `Stop requiring server-side ${side}` : `Require server-side ${side}`}>
                        <Icon name="success" forceType="svg" />
                      </button>

                      <button
                        type="button"
                        class="filter-action exclude"
                        class:exclude-active={operator === "not_eq"}
                        onclick={() => toggleExclude("server_side", value)}
                        aria-label={operator === "not_eq" ? `Remove server-side ${side} exclude filter` : `Exclude server-side ${side}`}
                        aria-pressed={operator === "not_eq"}
                        title={operator === "not_eq" ? `Stop excluding server-side ${side}` : `Exclude server-side ${side}`}>
                        <Icon name="error" forceType="svg" />
                      </button>
                    </div>
                  </div>
                {/each}
              </div>
            </details>

            <!-- Loaders -->
            <details class="filter-group">
              <summary>Loaders</summary>

              <div class="filter-options">
                {#each uniqueLoaders as loader (loader)}
                  {@const value = facetValue("categories", loader)}
                  {@const operator = getFilterOperator("categories", value)}

                  <div class="filter-row" class:filter-selected={operator !== null}>
                    <span>{loader}</span>

                    <div class="filter-actions">
                      <button
                        type="button"
                        class="filter-action include"
                        class:include-active={operator === "eq"}
                        onclick={() => toggleInclude("categories", value)}
                        aria-label={operator === "eq" ? `Remove ${loader} include filter` : `Include ${loader}`}
                        aria-pressed={operator === "eq"}
                        title={operator === "eq" ? `Stop including ${loader}` : `Include ${loader}`}>
                        <Icon name="success" forceType="svg" />
                      </button>

                      <button
                        type="button"
                        class="filter-action exclude"
                        class:exclude-active={operator === "not_eq"}
                        onclick={() => toggleExclude("categories", value)}
                        aria-label={operator === "not_eq" ? `Remove ${loader} exclude filter` : `Exclude ${loader}`}
                        aria-pressed={operator === "not_eq"}
                        title={operator === "not_eq" ? `Stop excluding ${loader}` : `Exclude ${loader}`}>
                        <Icon name="error" forceType="svg" />
                      </button>
                    </div>
                  </div>
                {/each}
              </div>
            </details>

            <!-- Minecraft versions -->
            <details class="filter-group">
              <summary>Minecraft versions</summary>

              <div class="filter-options">
                {#each uniqueGameVersions as version (version)}
                  {@const value = facetValue("versions", version)}
                  {@const operator = getFilterOperator("version", value)}

                  <div class="filter-row" class:filter-selected={operator !== null}>
                    <span>{version}</span>

                    <div class="filter-actions">
                      <button
                        type="button"
                        class="filter-action include"
                        class:include-active={operator === "eq"}
                        onclick={() => toggleInclude("version", value)}
                        aria-label={operator === "eq" ? `Remove Minecraft ${version} include filter` : `Include Minecraft ${version}`}
                        aria-pressed={operator === "eq"}
                        title={operator === "eq" ? `Stop including Minecraft ${version}` : `Include Minecraft ${version}`}>
                        <Icon name="success" forceType="svg" />
                      </button>

                      <button
                        type="button"
                        class="filter-action exclude"
                        class:exclude-active={operator === "not_eq"}
                        onclick={() => toggleExclude("version", value)}
                        aria-label={operator === "not_eq" ? `Remove Minecraft ${version} exclude filter` : `Exclude Minecraft ${version}`}
                        aria-pressed={operator === "not_eq"}
                        title={operator === "not_eq" ? `Stop excluding Minecraft ${version}` : `Exclude Minecraft ${version}`}>
                        <Icon name="error" forceType="svg" />
                      </button>
                    </div>
                  </div>
                {/each}
              </div>
            </details>
          </div>
        {/if}
      </aside>
    {/if}
  </div>
</div>

<style lang="scss">
.projects-browser-filters {
  display: flex;
  flex-direction: column;

  width: 100%;
  height: 100%;
  min-width: 0;
  min-height: 0;

  overflow: hidden;
}

/*
 * Top-level controls.
 */
.topbar {
  display: flex;
  align-items: center;
  flex-shrink: 0;

  gap: $space-sm;
  padding-bottom: $space-md;
}

.search {
  display: flex;
  flex: 1;
  min-width: 0;
  gap: $space-xs;

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

.search-button,
.clear-search {
  display: grid;
  place-items: center;

  flex-shrink: 0;

  width: 34px;
  min-width: $space-3xl;
  padding: $space-sm;

  border: 1px solid $color-border;
  border-radius: $radius-md;

  background: $color-surface-2;
  color: $color-text;

  cursor: pointer;

  &:hover {
    background: $color-surface-3;
  }

  &:focus-visible {
    outline: 2px solid $color-focus;
    outline-offset: 2px;
  }
}

.clear-search {
  color: $color-text-muted;

  &:hover {
    color: $color-text;
  }
}

.topbar > select,
.toolbar-button {
  flex-shrink: 0;

  min-height: 100%;
  padding: $space-sm $space-md;

  border: 1px solid $color-border;
  border-radius: $radius-md;

  background: $color-surface-2;
  color: $color-text;

  font: inherit;
  font-size: 0.8rem;

  cursor: pointer;

  &:hover {
    background: $color-surface-3;
  }

  &:focus-visible {
    outline: 2px solid $color-focus;
    outline-offset: 2px;
  }

  &.active {
    border-color: $color-focus;
    background: $color-surface-3;
  }
}

.toolbar-button {
  display: flex;
  align-items: center;
  justify-content: center;

  gap: $space-xs;
}

/*
 * Browser + sidebar.
 */
.main-layout {
  display: flex;
  flex: 1;

  width: 100%;
  min-width: 0;
  min-height: 0;

  gap: $space-md;
}

.browser-content {
  flex: 1;
  min-width: 0;
  min-height: 0;

  overflow: hidden;
}

/*
 * Right filter sidebar.
 */
.filter-sidebar {
  display: flex;
  flex-direction: column;
  flex: 0 0 280px;

  width: 280px;
  min-height: 0;

  border: 1px solid $color-border;
  border-radius: $radius-lg;

  background: $color-surface-1;

  overflow: hidden;
}

.sidebar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;

  flex-shrink: 0;

  padding: $space-md;

  border-bottom: 1px solid $color-border;

  h2 {
    margin: 0;

    color: $color-text;
    font-size: 0.85rem;
    font-weight: 600;
  }
}

.close-button {
  display: grid;
  place-items: center;

  width: 26px;
  height: 26px;
  padding: 0;

  border: 1px solid $color-border;
  border-radius: $radius-md;

  background: $color-surface-2;
  color: $color-text-muted;

  cursor: pointer;

  &:hover {
    background: $color-surface-3;
    color: $color-text;
  }

  &:focus-visible {
    outline: 2px solid $color-focus;
    outline-offset: 2px;
  }
}

.filter-list {
  flex: 1;
  min-height: 0;

  padding: $space-md;

  overflow-y: auto;
  overflow-x: hidden;

  scrollbar-gutter: stable;
}

/*
 * Native collapsible filter groups.
 *
 * <details> is intentionally closed by default.
 */
.filter-group {
  padding: 0;
  margin: 0 0 $space-sm;

  border: 1px solid $color-border-muted;
  border-radius: $radius-md;

  background: $color-surface-1;

  overflow: hidden;

  &:last-child {
    margin-bottom: 0;
  }

  summary {
    display: flex;
    align-items: center;

    min-height: 38px;
    padding: $space-sm $space-md;

    color: $color-text;

    font-size: 0.75rem;
    font-weight: 600;

    text-transform: uppercase;
    letter-spacing: 0.04em;

    cursor: pointer;
    user-select: none;

    &:hover {
      background: $color-surface-2;
    }

    &:focus-visible {
      outline: 2px solid $color-focus;
      outline-offset: -2px;
    }

    &::marker {
      color: $color-text-muted;
    }
  }

  &[open] {
    summary {
      border-bottom: 1px solid $color-border-muted;
      background: $color-surface-2;
    }
  }
}

.filter-options {
  display: flex;
  flex-direction: column;

  gap: 2px;

  padding: $space-sm;
}

/*
 * Individual filter.
 */
.filter-row {
  display: flex;
  align-items: center;
  justify-content: space-between;

  gap: $space-md;
  min-height: 32px;

  padding: 2px $space-xs;

  border-radius: $radius-sm;

  transition:
    background 100ms ease,
    padding 100ms ease;

  > span {
    min-width: 0;

    overflow: hidden;

    color: $color-text;
    font-size: 0.8rem;

    text-overflow: ellipsis;
    white-space: nowrap;
  }

  &:hover {
    background: $color-surface-2;
  }

  &.filter-selected {
    background: $color-surface-2;
  }
}

/*
 * Include / exclude controls.
 */
.filter-actions {
  display: flex;
  flex-shrink: 0;
  gap: 3px;
}

.filter-action {
  display: grid;
  place-items: center;

  padding: $space-xs;

  color: $color-text-muted;

  cursor: pointer;

  transition:
    border-color 100ms ease,
    background 100ms ease,
    color 100ms ease,
    transform 100ms ease;

  &:hover {
    transform: translateY(-1px);
  }

  &:focus-visible {
    outline: 2px solid $color-focus;
    outline-offset: 1px;
  }

  /*
   * The include button previews a positive/inclusion action.
   */
  &.include {
    &:hover {
      border-color: $color-success;
      background: rgba(34, 197, 94, 0.14);
      color: $color-success;
    }

    &.include-active {
      border-color: $color-success;
      background: rgba(34, 197, 94, 0.2);
      color: $color-success;
    }
  }

  /*
   * The exclude button previews a negative/exclusion action.
   */
  &.exclude {
    &:hover {
      border-color: $color-error;
      background: rgba(239, 68, 68, 0.14);
      color: $color-error;
    }

    &.exclude-active {
      border-color: $color-error;
      background: rgba(239, 68, 68, 0.2);
      color: $color-error;
    }
  }
}

.state {
  padding: $space-md;

  color: $color-text-muted;
  font-size: 0.8rem;
}

.error {
  margin: $space-md;
  padding: $space-md;

  border: 1px solid rgba(239, 68, 68, 0.35);
  border-radius: $radius-md;

  background: rgba(239, 68, 68, 0.08);
  color: $color-error;

  font-size: 0.8rem;
}

@media (max-width: 900px) {
  .filter-sidebar {
    flex-basis: 250px;
    width: 250px;
  }

  .topbar {
    gap: $space-xs;
  }

  .toolbar-button {
    padding-inline: $space-sm;
  }
}

@media (max-width: 640px) {
  .topbar {
    align-items: stretch;
    flex-wrap: wrap;
  }

  .search {
    flex-basis: 100%;
  }

  .topbar > select,
  .toolbar-button {
    flex: 1;
  }

  .main-layout {
    flex-direction: column;
  }

  .filter-sidebar {
    width: 100%;
    flex: 1;
    min-height: 250px;
  }

  .browser-content {
    min-height: 300px;
  }
}
</style>
