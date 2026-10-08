<!--
@component

Projects browser for discovering and installing projects.

Provides:
- Project search
- Project type filtering
- Result sorting
- Smart compatibility filtering
- Facet filtering
- Pagination / scrolling browser modes
- Project cards
- Project details navigation
-->
<script lang="ts">
import { type FacetGroup, type KableProfile, type ProjectType, type SearchIndex } from "$lib";
import Icon from "../Icon.svelte";
import ProjectsBrowserFilters from "./ProjectsBrowserFilters.svelte";
import ProjectsPaginatedBrowser from "./ProjectsPaginatedBrowser.svelte";
import ProjectsScrollBrowser from "./ProjectsScrollBrowser.svelte";

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

let showPaginated = $state(false);

let index = $state<SearchIndex>("relevance");
let facets = $state<FacetGroup[]>([]);
</script>

<div class="projects-browser">
  <ProjectsBrowserFilters bind:search bind:index bind:facets bind:smartFilter />

  <div class="browser-content">
    {#if showPaginated}
      <ProjectsPaginatedBrowser {profile} {profileId} {projectType} {search} {smartFilter} {facets} {index} />
    {:else}
      <ProjectsScrollBrowser {profile} {profileId} {projectType} {search} {smartFilter} {facets} {index} />
    {/if}
  </div>

  <!-- Absolute positioned toggle at right bottom of page to switch between
       the scroll and paginated project browsers. -->
  <div class="browser-toggle">
    <button
      type="button"
      class="toggle-btn"
      onclick={() => (showPaginated = !showPaginated)}
      aria-label={showPaginated ? "Switch to scroll browser" : "Switch to paginated browser"}
      title={showPaginated ? "Switch to scroll browser" : "Switch to paginated browser"}>
      <Icon name={showPaginated ? "list" : "search"} />
    </button>
  </div>
</div>

<style lang="scss">
.projects-browser {
  position: relative;
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  min-width: 0;
  min-height: 0;
}

.browser-content {
  flex: 1;
  min-width: 0;
  min-height: 0;
}

.browser-toggle {
  position: absolute;
  right: $space-md;
  bottom: $space-md;
  z-index: 1000;

  .toggle-btn {
    padding: $space-sm;
    border: 1px solid $color-border;
    border-radius: 0.25rem;
    background: $color-surface-3;
    cursor: pointer;

    &:hover {
      background: $color-accent-hover;
    }
  }
}
</style>
