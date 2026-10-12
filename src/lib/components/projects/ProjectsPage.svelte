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
import { type KableProfile, type ProjectType, app, Icon } from "$lib";
import { untrack } from "svelte";

import InstalledProjects from "./InstalledProjects.svelte";
import ProfilePicker from "./ProfilePicker.svelte";
import ProjectsBrowser from "./ProjectsBrowser.svelte";

let {
  projectType = "mod",
}: {
  projectType: ProjectType;
} = $props();

let currentTab: "installed" | "browse" = $state("installed");

let profileId = $state<string | null>(app.projectsService.loadedForProfileId);

let profile = $derived(app.profilesService.profiles.find((p) => p.id === profileId) ?? null);

let pickerCollapsed = $state(false);

let projectLabel = $derived(projectType === "mod" ? "Mods" : projectType === "resourcepack" ? "Resource Packs" : projectType === "shader" ? "Shaders" : "Modpacks");

let refreshing = $state(false);
let updatingAll = $state(false);
let checkingUpdates = $state(false);
let availableUpdates = $state(0);

/**
 * Tracks concurrent update checks so the automatic effect and explicit
 * actions can share the same request for a given profile and project type.
 *
 * This is intentionally a regular Map: its contents are not rendered.
 */
const updateChecksInFlight = new Map<string, Promise<number>>();

/**
 * Invalidates results from automatic update checks when the selected profile
 * or project type changes.
 */
let updateEffectGeneration = 0;

/**
 * Checks for available updates and updates the displayed count only when
 * the result still belongs to the currently selected profile and type.
 */
async function refreshUpdateCount(currentProfile: KableProfile | null, requestedType: ProjectType = projectType): Promise<void> {
  if (!currentProfile) {
    if (profileId === null) {
      availableUpdates = 0;
    }

    return;
  }

  const key = `${currentProfile.id}:${requestedType}`;

  let request = updateChecksInFlight.get(key);

  if (!request) {
    request = app.projectsService.checkUpdates(currentProfile, requestedType).then((updates) => updates.length);

    updateChecksInFlight.set(key, request);
  }

  try {
    const count = await request;

    // A completed request must not overwrite the count for a different
    // profile or project type.
    if (profileId === currentProfile.id && projectType === requestedType) {
      availableUpdates = count;
    }
  } finally {
    if (updateChecksInFlight.get(key) === request) {
      updateChecksInFlight.delete(key);
    }
  }
}

async function checkForUpdates(): Promise<void> {
  const currentProfile = profile;
  const requestedType = projectType;

  if (!currentProfile || checkingUpdates || updatingAll || refreshing) {
    return;
  }

  checkingUpdates = true;

  try {
    await refreshUpdateCount(currentProfile, requestedType);

    console.debug("[ProjectsPage] Checked for updates for profile:", currentProfile.id);
  } catch (error) {
    console.error("[ProjectsPage] Failed to check for updates for profile:", currentProfile.id, error);
  } finally {
    checkingUpdates = false;
  }
}

async function refresh(): Promise<void> {
  const currentProfile = profile;
  const requestedType = projectType;

  if (!currentProfile || refreshing || updatingAll || checkingUpdates) {
    return;
  }

  const requestedProfileId = currentProfile.id;

  refreshing = true;

  try {
    await app.profilesService.refreshProfiles(true);

    // Do not load an old profile if the user changed selection while
    // the profile list was being refreshed.
    if (profileId !== requestedProfileId || projectType !== requestedType) {
      return;
    }

    const refreshedProfile = app.profilesService.profiles.find((candidate) => candidate.id === requestedProfileId) ?? null;

    if (!refreshedProfile) {
      availableUpdates = 0;

      console.warn("[ProjectsPage] Selected profile no longer exists:", requestedProfileId);

      return;
    }

    await app.projectsService.load(refreshedProfile, requestedType, true);

    // The selection may have changed while the project list was loading.
    if (profileId !== requestedProfileId || projectType !== requestedType) {
      return;
    }

    await refreshUpdateCount(refreshedProfile, requestedType);

    console.debug("[ProjectsPage] Refreshed projects for profile:", requestedProfileId);
  } catch (error) {
    console.error("[ProjectsPage] Failed to refresh projects for profile:", requestedProfileId, error);
  } finally {
    refreshing = false;
  }
}

async function updateAll(): Promise<void> {
  const currentProfile = profile;
  const requestedType = projectType;

  if (!currentProfile || updatingAll || checkingUpdates || refreshing) {
    return;
  }

  if (availableUpdates === 0) {
    await checkForUpdates();
    return;
  }

  const requestedProfileId = currentProfile.id;

  updatingAll = true;

  try {
    await app.projectsService.updateAll(currentProfile, requestedType);

    // Avoid switching the service back to an old profile if the user
    // changed selection while the update operation was running.
    if (profileId !== requestedProfileId || projectType !== requestedType) {
      return;
    }

    await app.projectsService.load(currentProfile, requestedType, true);

    if (profileId !== requestedProfileId || projectType !== requestedType) {
      return;
    }

    await refreshUpdateCount(currentProfile, requestedType);

    console.debug("[ProjectsPage] Updated all projects for profile:", requestedProfileId);
  } catch (error) {
    console.error("[ProjectsPage] Failed to update all projects for profile:", requestedProfileId, error);
  } finally {
    updatingAll = false;
  }
}

function launch(): void {
  const currentProfile = profile;

  if (!currentProfile) {
    return;
  }

  app.launcherService.launch(currentProfile);
}

/**
 * Synchronize the service's selected profile and maintain the update count.
 *
 * Only profileId and projectType are tracked as dependencies. Reads from
 * the profile list and project service are untracked, preventing changes
 * to service state during selection/loading from repeatedly triggering
 * this effect.
 */
$effect(() => {
  const selectedProfileId = profileId;
  const selectedType = projectType;
  const generation = ++updateEffectGeneration;

  if (!selectedProfileId) {
    availableUpdates = 0;
    return;
  }

  untrack(() => {
    void (async () => {
      try {
        let selectedProfile = app.profilesService.profiles.find((candidate) => candidate.id === selectedProfileId) ?? null;

        if (!selectedProfile) {
          if (generation === updateEffectGeneration) {
            availableUpdates = 0;
          }

          return;
        }

        if (app.projectsService.loadedForProfileId !== selectedProfileId) {
          availableUpdates = 0;

          await app.projectsService.select(selectedProfile);

          if (generation !== updateEffectGeneration) {
            return;
          }

          // Re-read the profile in case selecting it refreshed the profile
          // list or replaced the profile object.
          selectedProfile = app.profilesService.profiles.find((candidate) => candidate.id === selectedProfileId) ?? null;
        }

        if (!selectedProfile || generation !== updateEffectGeneration) {
          return;
        }

        // Share the same in-flight request as manual checks and refreshes.
        await refreshUpdateCount(selectedProfile, selectedType);
      } catch (error) {
        if (generation !== updateEffectGeneration) {
          return;
        }

        console.error("[ProjectsPage] Failed to synchronize project state:", selectedProfileId, error);

        availableUpdates = 0;
      }
    })();
  });
});
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
        title={pickerCollapsed ? "Expand profile picker" : "Collapse profile picker"}
      >
        <Icon name={pickerCollapsed ? "chevron-right" : "chevron-left"} forceType="svg" />
      </button>
    </header>

    {#if !pickerCollapsed}
      <div class="profile-picker">
        <ProfilePicker bind:profileId />
      </div>
    {/if}
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
          <button
            class="update-all-btn"
            class:updating={updatingAll || checkingUpdates}
            type="button"
            onclick={updateAll}
            disabled={updatingAll || checkingUpdates || refreshing}
            aria-label={availableUpdates > 0 ? `Update all ${projectLabel.toLowerCase()} (${availableUpdates})` : `Check for ${projectLabel.toLowerCase()} updates`}
            title={availableUpdates > 0 ? `Update all ${availableUpdates} ${projectLabel.toLowerCase()}` : `Check for ${projectLabel.toLowerCase()} updates`}
          >
            <span class="update-all-icon">
              <Icon name="refresh" forceType="svg" size="sm" />
            </span>

            {#if availableUpdates > 0}
              <span>Update all</span>

              <span class="update-count">
                ({availableUpdates})
              </span>
            {:else}
              <span>Check for updates</span>
            {/if}

            {#if updatingAll || checkingUpdates}
              <span class="update-progress" aria-hidden="true"></span>
            {/if}
          </button>

          <span class="projects-count">
            {#if app.projectsService.projects.length > 0}
              {app.projectsService.projects.length}
              {projectLabel.toLowerCase()} installed
            {:else}
              No {projectLabel.toLowerCase()} installed
            {/if}
          </span>

          <span class="selected-profile">
            {profile.metadata.name}
          </span>

          <button
            class="refresh-btn"
            type="button"
            onclick={refresh}
            disabled={refreshing || updatingAll || checkingUpdates}
            class:loading={refreshing}
            aria-label="Refresh"
            title="Refresh"
          >
            <Icon name="refresh" forceType="svg" size="sm" />
          </button>

          <button class="launch-btn" type="button" onclick={launch}>
            <Icon name="play" forceType="svg" size="sm" />
            <span>Launch</span>
          </button>
        </div>
      {/if}
    </header>

    <div class="tab-content">
      <div class="tab-panel" class:active={currentTab === "installed"}>
        <InstalledProjects {profile} {profileId} {projectType} />
      </div>

      <div class="tab-panel" class:active={currentTab === "browse"}>
        <ProjectsBrowser {profile} {profileId} {projectType} />
      </div>
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
  flex: 0 0 300px;
  flex-direction: column;
  width: 300px;
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
    flex-basis: fit-content;
    width: fit-content;
    min-width: fit-content;
  }
}

.profile-panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: $space-sm;
  flex: 0 0 auto;
  height: $layout-tabbar-height;
  padding: $space-md;
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
  height: $layout-tabbar-height;
  border-bottom: 1px solid $color-border;
}

.navigation {
  display: flex;
  align-items: center;
}

.tab-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: $space-sm $space-md;
  border-right: 1px solid transparent;
  border-radius: unset !important;
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

.selected-profile,
.projects-count {
  max-width: 220px;
  overflow: hidden;
  color: $color-text-muted;
  font-size: 0.8rem;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.projects-count {
  padding-right: $space-md;
  border-right: 1px solid $color-border-muted;
}

.update-all-btn {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: $space-sm;
  overflow: hidden;
  padding: $space-sm $space-md;
  border: 1px solid transparent;
  border-radius: $radius-md;
  background: transparent;
  color: $color-text-muted;
  font: inherit;
  font-size: 0.8rem;
  cursor: pointer;

  &:hover:not(:disabled) {
    border-color: $color-border;
    background: $color-hover;
    color: $color-text;
  }

  &:focus-visible {
    outline: 2px solid $color-focus;
    outline-offset: 2px;
  }

  &:disabled {
    opacity: 0.6;
    cursor: wait;
  }

  &.updating {
    border-color: $color-success;
    color: $color-success;
    cursor: wait;
  }
}

.update-all-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;

  .updating & {
    animation: update-icon-spin 1s linear infinite;
  }
}

.update-count {
  color: $color-warning;
  font-weight: 600;
}

.update-progress {
  position: absolute;
  right: 0;
  bottom: 0;
  left: 0;
  height: 2px;
  overflow: hidden;
  pointer-events: none;
  background: transparent;

  &::before {
    position: absolute;
    top: 0;
    left: -40%;
    width: 40%;
    height: 100%;
    background: $color-success;
    content: "";
    animation: update-progress 1.4s ease-in-out infinite;
  }
}

.refresh-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: $space-sm;
  border: 1px solid transparent;
  border-radius: $radius-md;
  background: transparent;
  color: $color-text-muted;
  font: inherit;
  font-size: 0.85rem;
  cursor: pointer;

  &:hover {
    border-color: $color-border;
    background: $color-hover;
  }

  &:focus-visible {
    outline: 2px solid $color-focus;
    outline-offset: 2px;
  }

  &:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  &.loading {
    color: $color-accent;
    animation: spin 1s linear infinite;
  }
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }

  to {
    transform: rotate(360deg);
  }
}

@keyframes update-icon-spin {
  from {
    transform: rotate(0deg);
  }

  to {
    transform: rotate(360deg);
  }
}

@keyframes update-progress {
  0% {
    left: -40%;
  }

  50% {
    left: 60%;
  }

  100% {
    left: 100%;
  }
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
  font-size: 0.85rem;
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

  .tab-panel {
    display: none;
    width: 100%;
    height: 100%;
    min-width: 0;
    min-height: 0;

    &.active {
      display: flex;
    }
  }
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
