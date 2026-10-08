<!--
@component

Reusable filter controls for the projects browser.

Provides:
- Project search
- Result sorting
- Smart compatibility filtering
- Modrinth category/loader/game-version/project-type/side filters
- Include/exclude filtering through FacetOperator
-->

<script lang="ts">
import { api, type Facet, type FacetField, type FacetGroup, type FacetOperator, type SearchIndex } from "$lib";
import { onMount } from "svelte";

let {
  search = $bindable(""),
  index = $bindable<SearchIndex>("relevance"),
  smartFilter = $bindable(true),
  facets = $bindable<FacetGroup[]>([]),
}: {
  search?: string;
  index?: SearchIndex;
  smartFilter?: boolean;
  facets?: FacetGroup[];
} = $props();

/*
 * Keep the text currently being edited separate from the submitted
 * search value. This prevents every keystroke from triggering a browse.
 */
let query = $state(search);

let categories = $state<string[]>([]);
let loaders = $state<string[]>([]);
let gameVersions = $state<string[]>([]);
let projectTypes = $state<string[]>([]);
let sideTypes = $state<string[]>([]);

let loadingFilters = $state(false);
let filterError = $state<string | null>(null);

/*
 * Modrinth metadata should normally contain unique values, but duplicate
 * values have been observed. Deduplicate before rendering because all
 * keyed each blocks require unique keys.
 */
let uniqueCategories = $derived([...new Set(categories)]);
let uniqueLoaders = $derived([...new Set(loaders)]);
let uniqueGameVersions = $derived([...new Set(gameVersions)]);
let uniqueProjectTypes = $derived([...new Set(projectTypes)]);
let uniqueSideTypes = $derived([...new Set(sideTypes)]);

/*
 * The bound `facets` value is the source of truth.
 *
 * Flattening the groups makes lookup and mutation easier while keeping
 * the grouped representation for the backend.
 */
let selectedFilters = $derived(facets.flatMap((group) => group.facets));

/*
 * Modrinth uses namespaced facet values such as:
 *
 *   categories:adventure
 *   categories:fabric
 *   versions:1.21.1
 *   project_type:mod
 *   client_side:required
 *   server_side:required
 */
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
 * Modrinth facet semantics are:
 *
 *   Groups     = AND
 *   Facets in a group = OR
 *
 * Therefore:
 *
 *   category A + category B
 *
 * becomes:
 *
 *   (A OR B)
 *
 * while:
 *
 *   exclude A + exclude B
 *
 * becomes:
 *
 *   NOT A AND NOT B
 *
 * because exclusions must be in separate groups.
 *
 * Equality facets with the same field and namespace are grouped together.
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

function facetClass(field: FacetField, value: string, operator: FacetOperator): string {
  return getFilterOperator(field, value) === operator ? "active" : "";
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
    const [loadedCategories, loadedLoaders, loadedGameVersions, loadedProjectTypes, loadedSideTypes] = await Promise.all([
      api.getModrinthCategories(),
      api.getModrinthLoaders(),
      api.getModrinthGameVersions(),
      api.getModrinthProjectTypes(),
      api.getModrinthSideTypes(),
    ]);

    categories = loadedCategories;
    loaders = loadedLoaders;
    gameVersions = loadedGameVersions;
    projectTypes = loadedProjectTypes;
    sideTypes = loadedSideTypes;
  } catch (error) {
    filterError = error instanceof Error ? error.message : String(error);
  } finally {
    loadingFilters = false;
  }
}

/*
 * Filter metadata is component initialization work, not reactive work.
 *
 * Using $effect here would make loadingFilters a dependency if it is read
 * by loadFilters(), which can cause the effect to retrigger when the
 * loading state changes.
 */
onMount(() => {
  void loadFilters();
});

/*
 * Keep the local search input synchronized when the parent changes the
 * submitted search value externally.
 */
$effect(() => {
  if (query !== search) {
    query = search;
  }
});
</script>

<div class="projects-browser-filters">
  <form
    class="toolbar"
    onsubmit={(event) => {
      event.preventDefault();
      submitSearch();
    }}>
    <div class="search">
      <input type="search" bind:value={query} placeholder="Search projects..." aria-label="Search projects" />

      <button type="submit"> Search </button>

      {#if query}
        <button type="button" class="clear-search" onclick={clearSearch} aria-label="Clear search"> × </button>
      {/if}
    </div>

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
      class:active={smartFilter}
      onclick={() => {
        smartFilter = !smartFilter;
      }}
      aria-pressed={smartFilter}>
      Smart filter
    </button>
  </form>

  {#if loadingFilters}
    <div class="state">Loading filters...</div>
  {:else if filterError}
    <div class="error">
      {filterError}
    </div>
  {:else}
    <div class="filters">
      <section class="filter-group">
        <h3>Categories</h3>

        {#each uniqueCategories as category (category)}
          {@const value = facetValue("categories", category)}

          <div class="filter-row">
            <span>{category}</span>

            <div class="filter-actions">
              <button
                type="button"
                class={facetClass("categories", value, "eq")}
                class:include-active={getFilterOperator("categories", value) === "eq"}
                onclick={() => toggleInclude("categories", value)}
                aria-label={`Include ${category}`}
                aria-pressed={getFilterOperator("categories", value) === "eq"}>
                +
              </button>

              <button
                type="button"
                class={facetClass("categories", value, "not_eq")}
                class:exclude-active={getFilterOperator("categories", value) === "not_eq"}
                onclick={() => toggleExclude("categories", value)}
                aria-label={`Exclude ${category}`}
                aria-pressed={getFilterOperator("categories", value) === "not_eq"}>
                −
              </button>
            </div>
          </div>
        {/each}
      </section>

      <section class="filter-group">
        <h3>Loaders</h3>

        {#each uniqueLoaders as loader (loader)}
          {@const value = facetValue("categories", loader)}

          <div class="filter-row">
            <span>{loader}</span>

            <div class="filter-actions">
              <button
                type="button"
                class:include-active={getFilterOperator("categories", value) === "eq"}
                onclick={() => toggleInclude("categories", value)}
                aria-label={`Include ${loader}`}
                aria-pressed={getFilterOperator("categories", value) === "eq"}>
                +
              </button>

              <button
                type="button"
                class:exclude-active={getFilterOperator("categories", value) === "not_eq"}
                onclick={() => toggleExclude("categories", value)}
                aria-label={`Exclude ${loader}`}
                aria-pressed={getFilterOperator("categories", value) === "not_eq"}>
                −
              </button>
            </div>
          </div>
        {/each}
      </section>

      <section class="filter-group">
        <h3>Minecraft versions</h3>

        {#each uniqueGameVersions as version (version)}
          {@const value = facetValue("versions", version)}

          <div class="filter-row">
            <span>{version}</span>

            <div class="filter-actions">
              <button
                type="button"
                class:include-active={getFilterOperator("version", value) === "eq"}
                onclick={() => toggleInclude("version", value)}
                aria-label={`Include Minecraft ${version}`}
                aria-pressed={getFilterOperator("version", value) === "eq"}>
                +
              </button>

              <button
                type="button"
                class:exclude-active={getFilterOperator("version", value) === "not_eq"}
                onclick={() => toggleExclude("version", value)}
                aria-label={`Exclude Minecraft ${version}`}
                aria-pressed={getFilterOperator("version", value) === "not_eq"}>
                −
              </button>
            </div>
          </div>
        {/each}
      </section>

      <section class="filter-group">
        <h3>Project types</h3>

        {#each uniqueProjectTypes as type (type)}
          {@const value = facetValue("project_type", type)}

          <div class="filter-row">
            <span>{type}</span>

            <div class="filter-actions">
              <button
                type="button"
                class:include-active={getFilterOperator("project_type", value) === "eq"}
                onclick={() => toggleInclude("project_type", value)}
                aria-label={`Include ${type}`}
                aria-pressed={getFilterOperator("project_type", value) === "eq"}>
                +
              </button>

              <button
                type="button"
                class:exclude-active={getFilterOperator("project_type", value) === "not_eq"}
                onclick={() => toggleExclude("project_type", value)}
                aria-label={`Exclude ${type}`}
                aria-pressed={getFilterOperator("project_type", value) === "not_eq"}>
                −
              </button>
            </div>
          </div>
        {/each}
      </section>

      <section class="filter-group">
        <h3>Side support</h3>

        {#each uniqueSideTypes as side (side)}
          {@const clientValue = facetValue("client_side", side)}
          {@const serverValue = facetValue("server_side", side)}

          <div class="filter-row">
            <span>{side}</span>

            <div class="filter-actions">
              <button
                type="button"
                class:include-active={getFilterOperator("client_side", clientValue) === "eq"}
                onclick={() => toggleInclude("client_side", clientValue)}
                aria-label={`Require client side ${side}`}
                aria-pressed={getFilterOperator("client_side", clientValue) === "eq"}>
                C+
              </button>

              <button
                type="button"
                class:exclude-active={getFilterOperator("client_side", clientValue) === "not_eq"}
                onclick={() => toggleExclude("client_side", clientValue)}
                aria-label={`Exclude client side ${side}`}
                aria-pressed={getFilterOperator("client_side", clientValue) === "not_eq"}>
                C−
              </button>

              <button
                type="button"
                class:include-active={getFilterOperator("server_side", serverValue) === "eq"}
                onclick={() => toggleInclude("server_side", serverValue)}
                aria-label={`Require server side ${side}`}
                aria-pressed={getFilterOperator("server_side", serverValue) === "eq"}>
                S+
              </button>

              <button
                type="button"
                class:exclude-active={getFilterOperator("server_side", serverValue) === "not_eq"}
                onclick={() => toggleExclude("server_side", serverValue)}
                aria-label={`Exclude server side ${side}`}
                aria-pressed={getFilterOperator("server_side", serverValue) === "not_eq"}>
                S−
              </button>
            </div>
          </div>
        {/each}
      </section>
    </div>
  {/if}
</div>

<style lang="scss">
.projects-browser-filters {
  display: flex;
  flex-direction: column;
  gap: $space-md;
  width: 100%;
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

.toolbar > select,
.toolbar > button,
.filter-actions button {
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
}

.toolbar > button.active,
.filter-actions button.include-active {
  border-color: $color-focus;
  background: $color-surface-3;
}

.filter-actions button.exclude-active {
  border-color: $color-error;
  color: $color-error;
}

.clear-search {
  flex-shrink: 0;
}

.filters {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: $space-md;
}

.filter-group {
  display: flex;
  flex-direction: column;
  gap: $space-xs;
  padding: $space-md;
  border: 1px solid $color-border;
  border-radius: $radius-md;
  background: $color-surface-2;

  h3 {
    margin: 0 0 $space-sm;
    font-size: 0.8rem;
    font-weight: 600;
  }
}

.filter-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: $space-md;
  min-height: 30px;

  > span {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 0.8rem;
  }
}

.filter-actions {
  display: flex;
  flex-shrink: 0;
  gap: 2px;

  button {
    min-width: 30px;
    padding: $space-xs;
  }
}

.state {
  color: $color-text-muted;
  font-size: 0.8rem;
}

.error {
  padding: $space-md;
  border: 1px solid rgba(239, 68, 68, 0.35);
  border-radius: $radius-md;
  background: rgba(239, 68, 68, 0.08);
  color: $color-error;
  font-size: 0.8rem;
}

@media (max-width: 640px) {
  .toolbar {
    align-items: stretch;
    flex-direction: column;
  }

  .search {
    flex-direction: column;
  }

  .toolbar > select,
  .toolbar > button {
    width: 100%;
  }
}
</style>
