<!--
@component
This card differs from the @ProjectCard component in that it is used to display an already-installed project.
-->
<script lang="ts">
import { app, type KableProfile, type KableProject } from "$lib";
import ProjectGalleryModal from "./ProjectGalleryModal.svelte";
import ProjectModal from "./ProjectModal.svelte";
import ProjectVersionsModal from "./ProjectVersionsModal.svelte";

let {
  profile,
  project,
}: {
  profile: KableProfile | null;
  project: KableProject;
} = $props();

let isEnabled = $state(false);
let loading = $state(false);

let installedVersion = $derived(project.project.versions.find((version) => version.id === project.version_id) ?? null);

let versionLabel = $derived(installedVersion?.version_number ?? project.version_id);

let releaseDate = $derived(installedVersion?.date_published ? new Date(installedVersion.date_published).toLocaleDateString() : null);

let hasGallery = $derived((project.project.gallery?.length ?? 0) > 0);

async function refreshEnabledState() {
  if (!profile) {
    isEnabled = false;
    return;
  }

  isEnabled = await app.projectsService.isEnabled(profile, project);
}

$effect(() => {
  if (!profile) {
    isEnabled = false;
    return;
  }

  let cancelled = false;

  app.projectsService.isEnabled(profile, project).then((value) => {
    if (!cancelled) {
      isEnabled = value;
    }
  });

  return () => {
    cancelled = true;
  };
});

async function toggle() {
  if (!profile || loading) return;

  loading = true;

  try {
    await app.projectsService.toggle(profile, project);
    await refreshEnabledState();
  } finally {
    loading = false;
  }
}

async function remove() {
  if (!profile || loading) return;

  loading = true;

  try {
    await app.projectsService.remove(profile, project);
  } finally {
    loading = false;
  }
}

function showDetails() {
  app.show(ProjectModal, {
    profile,
    project: project.project,
  });
}

function showVersions() {
  app.show(ProjectVersionsModal, {
    profile,
    project,
  });
}

function showGallery() {
  if (!hasGallery) return;

  app.show(ProjectGalleryModal, {
    project: project.project,
  });
}
</script>

<article class:disabled={!isEnabled} class="installed-project-card">
  <div class="project-icon">
    {#if project.project.icon_url}
      <img src={project.project.icon_url} alt="" loading="lazy" />
    {:else}
      <div class="icon-placeholder">
        {project.project.title.charAt(0).toUpperCase()}
      </div>
    {/if}
  </div>

  <div class="project-content">
    <div class="project-header">
      <h3>{project.project.title}</h3>

      <span class:enabled={isEnabled} class="status">
        {isEnabled ? "Enabled" : "Disabled"}
      </span>
    </div>

    <div class="project-meta">
      <span>{versionLabel}</span>

      {#if releaseDate}
        <span class="separator">•</span>
        <span>{releaseDate}</span>
      {/if}
    </div>
  </div>

  <div class="project-actions">
    <button
      class:active={isEnabled}
      class="action-button toggle"
      type="button"
      disabled={!profile || loading}
      title={isEnabled ? "Disable project" : "Enable project"}
      aria-label={isEnabled ? `Disable ${project.project.title}` : `Enable ${project.project.title}`}
      onclick={toggle}>
      {isEnabled ? "Disable" : "Enable"}
    </button>

    <button class="action-button" type="button" disabled={loading} onclick={showVersions}> Versions </button>

    <button class="action-button" type="button" disabled={loading} onclick={showDetails}> Details </button>

    {#if hasGallery}
      <button class="action-button" type="button" disabled={loading} onclick={showGallery}> Gallery </button>
    {/if}

    <button class="action-button danger" type="button" disabled={!profile || loading} onclick={remove}> Uninstall </button>
  </div>
</article>

<style lang="scss">
.installed-project-card {
  display: flex;
  align-items: center;
  gap: $space-lg;
  padding: $space-lg;
  background: $color-surface-1;
  border: 1px solid $color-border-muted;
  border-radius: $radius-lg;
  transition:
    background-color 120ms ease,
    border-color 120ms ease,
    opacity 120ms ease;

  &:hover {
    background: $color-surface-2;
    border-color: $color-border;
  }

  &.disabled {
    opacity: 0.7;
  }
}

.project-icon {
  flex: 0 0 52px;
  width: 52px;
  height: 52px;
  overflow: hidden;
  border-radius: $radius-lg;
  background: $color-surface-3;

  img,
  .icon-placeholder {
    display: block;
    width: 100%;
    height: 100%;
  }

  img {
    object-fit: cover;
  }
}

.icon-placeholder {
  display: grid;
  place-items: center;
  color: $color-text-muted;
  font-size: 1.25rem;
  font-weight: 600;
}

.project-content {
  min-width: 0;
  flex: 1;
}

.project-header {
  display: flex;
  align-items: center;
  gap: $space-sm;
  min-width: 0;

  h3 {
    min-width: 0;
    margin: 0;
    overflow: hidden;
    color: $color-text;
    font-size: 1rem;
    font-weight: 600;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
}

.status {
  flex: 0 0 auto;
  padding: $space-1 $space-sm;
  border-radius: $radius-round;
  background: $color-surface-3;
  color: $color-text-muted;
  font-size: 0.7rem;
  font-weight: 600;
  line-height: 1.4;

  &.enabled {
    background: rgba(34, 197, 94, 0.12);
    color: $color-success;
  }
}

.project-meta {
  display: flex;
  align-items: center;
  gap: $space-sm;
  margin-top: $space-xs;
  color: $color-text-muted;
  font-size: 0.8rem;
}

.separator {
  color: $color-placeholder;
}

.project-actions {
  display: flex;
  align-items: center;
  gap: $space-sm;
  flex: 0 0 auto;
}

.action-button {
  padding: $space-sm $space-md;
  border: 1px solid $color-border;
  border-radius: $radius-md;
  background: $color-surface-2;
  color: $color-text-muted;
  font: inherit;
  font-size: 0.8rem;
  cursor: pointer;
  transition:
    background-color 120ms ease,
    border-color 120ms ease,
    color 120ms ease;

  &:hover:not(:disabled) {
    background: $color-surface-3;
    border-color: $color-border;
    color: $color-text;
  }

  &:focus-visible {
    outline: 2px solid $color-focus;
    outline-offset: 2px;
  }

  &:disabled {
    cursor: default;
    opacity: 0.5;
  }

  &.toggle {
    border-color: $color-accent;
    background: $color-accent;
    color: $color-text;

    &:hover:not(:disabled) {
      background: $color-accent-hover;
      border-color: $color-accent-hover;
    }

    &.active {
      background: transparent;
      color: $color-accent-muted;

      &:hover:not(:disabled) {
        background: $color-selected;
        color: $color-text;
      }
    }
  }

  &.danger {
    &:hover:not(:disabled) {
      border-color: $color-error;
      background: rgba(239, 68, 68, 0.12);
      color: $color-error;
    }
  }
}

@media (max-width: 800px) {
  .installed-project-card {
    align-items: flex-start;
    flex-wrap: wrap;
  }

  .project-actions {
    width: 100%;
    flex-wrap: wrap;
  }
}
</style>
