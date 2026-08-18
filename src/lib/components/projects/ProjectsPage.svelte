<!--
@component 
This projects page wrapper takes as prop(s) the type of the project (e.g. mods, resource packs, etc.) 
and renders a profile picker, top-navigation (just 2 buttons) to switch between installed and browse, 
the [ProjectsBrowser.svelte](./ProjectsBrowser.svelte) component and the [InstalledProjects.svelte](./InstalledProjects.svelte) component.

This should allow the mods, resourcepacks, shaders pages to only contain this component with the correct type.
-->
<script lang="ts">
import { type KableProfile, type ProjectType, app } from "$lib";
import Icon from "../Icon.svelte";
import InstalledProjects from "./InstalledProjects.svelte";
import ProfilePicker from "./ProfilePicker.svelte";
import ProjectsBrowser from "./ProjectsBrowser.svelte";

let { projectType = "mod" }: { projectType: ProjectType } = $props();

let currentTab: "installed" | "browse" = $state("installed");

let profile = $state<KableProfile | null>(null);
</script>

<div class={projectType + "-page page"}>
  <nav class="tab-navigation">
    <button class="tab-btn" class:active={currentTab === "installed"} onclick={() => (currentTab = "installed")}> 📦 Installed {projectType} </button>
    <button class="tab-btn" class:active={currentTab === "browse"} onclick={() => (currentTab = "browse")}> 🔍 Browse {projectType} </button>
    {#if profile}
      <div class="current-profile">
        Selected: <strong>{profile.metadata.name}</strong>
        <button class="tab-btn" onclick={(e) => app.launcherService.launch(profile!)}><Icon name="play" /> Launch </button>
      </div>
    {/if}
  </nav>
  <ProfilePicker bind:profile />
  <div class="tab-content">
    {#if currentTab === "installed"}
      <InstalledProjects {profile} />
    {:else if currentTab === "browse"}
      <ProjectsBrowser {profile} {projectType} />
    {/if}
  </div>
</div>
