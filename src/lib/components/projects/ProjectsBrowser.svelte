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
import { type KableProfile, type ProjectType } from "$lib";
import Icon from "../Icon.svelte";
import ProjectsPaginatedBrowser from "./ProjectsPaginatedBrowser.svelte";
import ProjectsScrollBrowser from "./ProjectsScrollBrowser.svelte";

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

let showPaginated = $state(false);
</script>

<!-- Absolute positioned toggle at right bottom of page to switch between the {@link ProjectsScrollBrowser} and {@link ProjectsPaginatedBrowser} -->
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

{#if showPaginated}
  <ProjectsPaginatedBrowser {profile} {projectType} {search} {smartFilter} />
{:else}
  <ProjectsScrollBrowser {profile} {projectType} {search} {smartFilter} />
{/if}

<style lang="scss">
.browser-toggle {
  position: absolute;
  bottom: $space-md;
  right: $space-md;
  z-index: 1000;

  .toggle-btn {
    background: $color-surface-3;
    border: 1px solid $color-border;
    border-radius: 0.25rem;
    padding: $space-sm;
    cursor: pointer;

    &:hover {
      background: $color-accent-hover;
    }
  }
}
</style>
