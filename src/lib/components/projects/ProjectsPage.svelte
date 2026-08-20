<!--
@component

Project management page for a specific project type.

Provides:
- Profile selection
- Installed/browse navigation
- Project browsing
- Installed project management
- Profile launching
-->
<script lang="ts">
import { type ProjectType, app, Icon } from "$lib";

import InstalledProjects from "./InstalledProjects.svelte";
import ProfilePicker from "./ProfilePicker.svelte";
import ProjectsBrowser from "./ProjectsBrowser.svelte";

let {
  projectType = "mod",
}: {
  projectType: ProjectType;
} = $props();

let currentTab: "installed" | "browse" = $state("installed");
let profile = $state(app.profilesService.profiles.find((p) => p.id === app.projectsService.loadedForProfileId) ?? null);
let pickerCollapsed = $state(false);

let projectLabel = $derived(projectType === "mod" ? "Mods" : projectType === "resourcepack" ? "Resource Packs" : projectType === "shader" ? "Shaders" : "Modpacks");

function launch() {
  if (!profile) return;

  app.launcherService.launch(profile);
}
</script>

<div class={`${projectType}-page page`}>
  <aside class:collapsed={pickerCollapsed} class="profile-panel">
    <header class="profile-panel-header">
      {#if !pickerCollapsed}
        <div class="profile-heading">
          <h2>Profile</h2>

          {#if profile}
            <span>{profile.metadata.name}</span>
          {:else}
            <span>Select a profile</span>
          {/if}
        </div>
      {/if}

      <button
        class="collapse-button"
        type="button"
        onclick={() => (pickerCollapsed = !pickerCollapsed)}
        aria-label={pickerCollapsed ? "Expand profile picker" : "Collapse profile picker"}
        title={pickerCollapsed ? "Expand profile picker" : "Collapse profile picker"}>
        <Icon name={pickerCollapsed ? "chevron-right" : "chevron-left"} forceType="svg" />
      </button>
    </header>

    <div class="profile-picker">
      <ProfilePicker bind:profile />
    </div>
  </aside>

  <main class="content">
    <header class="page-header">
      <nav class="navigation" aria-label="Project view">
        <button class="tab-btn" class:active={currentTab === "installed"} type="button" onclick={() => (currentTab = "installed")}>
          Installed {projectLabel}
        </button>

        <button class="tab-btn" class:active={currentTab === "browse"} type="button" onclick={() => (currentTab = "browse")}>
          Browse {projectLabel}
        </button>
      </nav>

      {#if profile}
        <div class="profile-actions">
          <span class="selected-profile">
            {profile.metadata.name}
          </span>

          <button class="launch-btn" type="button" onclick={launch}>
            <Icon name="play" forceType="svg" />
            <span>Launch</span>
          </button>
        </div>
      {/if}
    </header>

    <div class="tab-content">
      {#if currentTab === "installed"}
        <InstalledProjects {profile} {projectType} />
      {:else}
        <ProjectsBrowser {profile} {projectType} />
      {/if}
    </div>
  </main>
</div>

<style lang="scss">
.page {
  display: flex;
  width: 100%;
  height: 100%;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
  color: $color-text;
}

.profile-panel {
  display: flex;
  flex: 0 0 240px;
  flex-direction: column;
  width: 240px;
  min-width: 240px;
  height: 100%;
  overflow: hidden;
  border-right: 1px solid $color-border;
  background: $color-surface-1;
  transition:
    width 180ms ease,
    min-width 180ms ease,
    flex-basis 180ms ease;

  &.collapsed {
    flex-basis: 72px;
    width: 72px;
    min-width: 72px;
  }
}

.profile-panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: $space-sm;
  flex: 0 0 auto;
  min-height: $layout-tabbar-height;
  padding: $space-sm $space-md;
  border-bottom: 1px solid $color-border-muted;
}

.profile-heading {
  display: flex;
  flex-direction: column;
  min-width: 0;

  h2 {
    margin: 0;
    font-size: 0.85rem;
    font-weight: 600;
  }

  span {
    margin-top: $space-1;
    overflow: hidden;
    color: $color-text-muted;
    font-size: 0.7rem;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
}

.collapse-button {
  display: flex;
  align-items: center;
  justify-content: center;
  flex: 0 0 auto;
  width: 32px;
  height: 32px;
  padding: 0;
  border: 1px solid transparent;
  border-radius: $radius-md;
  background: transparent;
  color: $color-text-muted;
  cursor: pointer;

  &:hover {
    border-color: $color-border;
    background: $color-hover;
    color: $color-text;
  }

  &:focus-visible {
    outline: 2px solid $color-focus;
    outline-offset: 2px;
  }
}

.profile-picker {
  flex: 1 1 auto;
  min-height: 0;
  overflow: hidden;
}

.content {
  display: flex;
  flex: 1 1 auto;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
}

.page-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: $space-lg;
  flex: 0 0 auto;
  min-height: $layout-tabbar-height;
  padding: $space-sm $space-md;
  border-bottom: 1px solid $color-border;
}

.navigation {
  display: flex;
  align-items: center;
  gap: $space-xs;
}

.tab-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: $space-sm $space-md;
  border: 1px solid transparent;
  border-radius: $radius-md;
  background: transparent;
  color: $color-text-muted;
  font: inherit;
  font-size: 0.85rem;
  cursor: pointer;
  transition:
    background 120ms ease,
    color 120ms ease,
    border-color 120ms ease;

  &:hover {
    background: $color-hover;
    color: $color-text;
  }

  &.active {
    border-color: $color-border;
    background: $color-surface-2;
    color: $color-text;
  }

  &:focus-visible {
    outline: 2px solid $color-focus;
    outline-offset: 2px;
  }
}

.profile-actions {
  display: flex;
  align-items: center;
  gap: $space-md;
}

.selected-profile {
  max-width: 220px;
  overflow: hidden;
  color: $color-text-muted;
  font-size: 0.8rem;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.launch-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: $space-sm;
  padding: $space-sm $space-md;
  border: 1px solid transparent;
  border-radius: $radius-md;
  background: transparent;
  color: $color-success;
  font: inherit;
  font-size: 0.8rem;
  cursor: pointer;

  &:hover {
    border-color: $color-border;
    background: $color-hover;
  }

  &:focus-visible {
    outline: 2px solid $color-focus;
    outline-offset: 2px;
  }
}

.tab-content {
  flex: 1 1 auto;
  min-width: 0;
  min-height: 0;
  overflow: auto;
  padding: $space-lg;
}

@media (max-width: 720px) {
  .profile-panel {
    flex-basis: 200px;
    width: 200px;
    min-width: 200px;

    &.collapsed {
      flex-basis: 60px;
      width: 60px;
      min-width: 60px;
    }
  }

  .page-header {
    align-items: stretch;
    flex-direction: column;
  }

  .navigation {
    width: 100%;
  }

  .tab-btn {
    flex: 1;
  }

  .profile-actions {
    justify-content: space-between;
  }
}
</style>
