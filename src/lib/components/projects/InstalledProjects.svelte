<!--
@component 
This component uses the @InstalledProjectCard component to display a list of projects that the user has installed. 
This is used for managing installed projects (per profile) and allows for easy uninstalling, viewing details, versions, galleries and enabling/disabling them.

-->
<script lang="ts">
import { app, type KableProfile, type ProjectType } from "$lib";
import InstalledProjectCard from "./InstalledProjectCard.svelte";

let {
  profile,
  projectType,
}: {
  profile: KableProfile | null;
  projectType: ProjectType;
} = $props();

let projects = $derived(app.projectsService.getByType(projectType));
let loading = $derived(app.projectsService.loading);

$effect(() => {
  if (profile) {
    app.projectsService.load(profile, projectType);
  }
});

let projectLabel = $derived(projectType === "mod" ? "mods" : projectType === "resourcepack" ? "resource packs" : projectType === "shader" ? "shaders" : "modpacks");
</script>

<div class="installed-projects">
  {#if !profile}
    <div class="state">
      <p>Select a profile to view installed {projectLabel}.</p>
    </div>
  {:else if loading}
    <div class="state">
      <p>Loading installed {projectLabel}...</p>
    </div>
  {:else if projects.length === 0}
    <div class="state">
      <p>No {projectLabel} installed.</p>
    </div>
  {:else}
    <div class="grid">
      {#each projects as project (project.project.project_id)}
        <InstalledProjectCard {profile} {project} />
      {/each}
    </div>
  {/if}
</div>

<style lang="scss">
.installed-projects {
  width: 100%;
  min-width: 0;
}

.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax($layout-container-1, 1fr));
  gap: $layout-gap;
  width: 100%;
}

.state {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 240px;
  padding: $space-xl;
  color: $color-text-muted;
  font-size: 0.85rem;
  text-align: center;

  p {
    margin: 0;
  }
}

@media (max-width: 640px) {
  .grid {
    grid-template-columns: 1fr;
  }
}
</style>
