<!--
@component 
This component uses the @InstalledProjectCard component to display a list of projects that the user has installed. 
This is used for managing installed projects (per profile) and allows for easy uninstalling, viewing details, versions, galleries and enabling/disabling them.

-->
<script lang="ts">
import { type KableProfile, type KableProject_Deserialize } from "$lib";
import InstalledProjectCard from "./InstalledProjectCard.svelte";

let { profile }: { profile: KableProfile | null } = $props();

let installedProjects: KableProject_Deserialize[] = [];
</script>

<div class="installed-projects">
  {#each installedProjects as project (project.project.project_id)}
    <InstalledProjectCard {profile} {project} />
  {/each}
</div>

<style lang="scss">
</style>
