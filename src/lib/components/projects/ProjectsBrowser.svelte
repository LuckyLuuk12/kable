<!--
@component 
This projects browser behaves and looks similar to the [Modrinth Browser](https://modrinth.com/discover/mods) in which it provides 
- a searchbar 
- filters 
- and a grid of project cards. 
It is used in various routes/pages as we make different pages per project type (e.g. mods, resource packs, etc.)


-->
<script lang="ts">
import { type KableProfile, type Project } from "$lib";
import ProjectCard from "./ProjectCard.svelte";

let { selectedProfile = null }: { selectedProfile: KableProfile | null } = $props();

//
let projects: Project[] = [];
</script>

<div class="projects-browser">
  <!-- Header with searchbar and view controls -->
  <div class="projects-browser__header"></div>
  <!-- Filters and content -->
  <div class="projects-browser__filters"></div>
  <!-- Project cards -->
  <div class="projects-browser__content">
    {#each projects as project (project.project_id)}
      <ProjectCard {project} />
    {/each}
  </div>
</div>

<style lang="scss">
</style>
