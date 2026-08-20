<!--
@component
This card differs from the @ProjectCard component in that it is used to display an already-installed project.
-->
<script lang="ts">
import { app, Icon, type KableProfile, type KableProject } from "$lib";
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
      <Icon name={isEnabled ? "error" : "success"} forceType="svg" />
    </button>

    <button class="action-button" type="button" disabled={loading} title="Versions" aria-label={`Versions for ${project.project.title}`} onclick={showVersions}>
      <Icon name="list" forceType="svg" />
    </button>

    <button class="action-button" type="button" disabled={loading} title="Details" aria-label={`Details for ${project.project.title}`} onclick={showDetails}>
      <Icon name="info" forceType="svg" />
    </button>

    {#if hasGallery}
      <button class="action-button" type="button" disabled={loading} title="Gallery" aria-label={`Gallery for ${project.project.title}`} onclick={showGallery}>
        <Icon name="image" forceType="svg" />
      </button>
    {/if}

    <button
      class="action-button danger"
      type="button"
      disabled={!profile || loading}
      title="Uninstall"
      aria-label={`Uninstall ${project.project.title}`}
      onclick={remove}>
      <Icon name="trash" forceType="svg" />
    </button>
  </div>
</article>

<style lang="scss">
.installed-project-card {
  display: grid;
  grid-template-columns: 48px minmax(0, 1fr) 44px;
  grid-template-areas: "icon content actions";
  align-items: center;
  gap: $space-md;

  width: 100%;
  min-width: 0;
  box-sizing: border-box;
  padding: $space-md;

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
    opacity: 0.65;
  }
}

.project-icon {
  grid-area: icon;

  width: 48px;
  height: 48px;
  overflow: hidden;

  border-radius: $radius-md;
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
  font-size: 1.1rem;
  font-weight: 600;
}

.project-content {
  grid-area: content;
  min-width: 0;
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
    font-size: 0.95rem;
    font-weight: 600;
    line-height: 1.3;

    text-overflow: ellipsis;
    white-space: nowrap;
  }
}

.status {
  flex: 0 0 auto;

  padding: 2px $space-sm;

  border: 1px solid $color-border-muted;
  border-radius: $radius-round;

  background: $color-surface-3;
  color: $color-text-muted;

  font-size: 0.65rem;
  font-weight: 600;
  line-height: 1.4;

  &.enabled {
    border-color: rgba(34, 197, 94, 0.2);
    background: rgba(34, 197, 94, 0.08);
    color: $color-success;
  }
}

.project-meta {
  display: flex;
  align-items: center;
  gap: $space-xs;

  margin-top: 3px;

  overflow: hidden;

  color: $color-text-muted;
  font-size: 0.72rem;
  line-height: 1.4;

  span {
    min-width: 0;
  }
}

.separator {
  flex: 0 0 auto;
  color: $color-placeholder;
}

.project-actions {
  grid-area: actions;

  display: grid;
  grid-template-columns: repeat(2, 20px);
  grid-auto-rows: 20px;
  justify-content: end;
  gap: $space-md;

  width: 43px;
}

.action-button {
  display: grid;
  place-items: center;

  width: 20px;
  height: 20px;
  padding: 0;

  border: 1px solid transparent;
  border-radius: $radius-sm;

  background: transparent;
  color: $color-text-muted;

  cursor: pointer;

  transition:
    background-color 100ms ease,
    border-color 100ms ease,
    color 100ms ease;

  :global(svg) {
    width: 12px;
    height: 12px;
  }

  &:hover:not(:disabled) {
    background: $color-surface-3;
    border-color: $color-border-muted;
    color: $color-text;
  }

  &:focus-visible {
    outline: 2px solid $color-focus;
    outline-offset: 1px;
  }

  &:disabled {
    opacity: 0.35;
    cursor: default;
  }

  &.toggle {
    color: $color-accent;

    &:hover:not(:disabled) {
      background: $color-selected;
      color: $color-accent-hover;
    }

    &.active {
      color: $color-text-muted;
    }
  }

  &.danger {
    &:hover:not(:disabled) {
      background: rgba(239, 68, 68, 0.08);
      border-color: rgba(239, 68, 68, 0.2);
      color: $color-error;
    }
  }
}
</style>
