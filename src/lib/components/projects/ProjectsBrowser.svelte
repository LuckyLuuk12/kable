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
  <ProjectsBrowserFilters bind:search bind:index bind:facets bind:smartFilter bind:showPaginated>
    {#if showPaginated}
      <ProjectsPaginatedBrowser {profile} {profileId} {projectType} {search} {smartFilter} {facets} {index} />
    {:else}
      <ProjectsScrollBrowser {profile} {profileId} {projectType} {search} {smartFilter} {facets} {index} />
    {/if}
  </ProjectsBrowserFilters>
</div>

<style lang="scss">
.projects-browser {
  display: flex;

  width: 100%;
  height: 100%;
  min-width: 0;
  min-height: 0;
}

.browser-toggle {
  position: absolute;
  right: $space-md;
  bottom: $space-md;
  z-index: 1000;
}
</style>
