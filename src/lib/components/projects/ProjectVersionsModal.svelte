<!--
@component 

-->
<script lang="ts">
import { app, type KableProfile, type KableProject, type ProjectVersion } from "$lib";

let {
  profile = null,
  project,
}: {
  profile?: KableProfile | null;
  project: KableProject;
} = $props();

let loading = $state(false);

let projectData = $derived(project.project);

let selectedVersionId = $derived(project.version_id);

let selectedVersion = $derived(projectData.versions.find((version) => version.id === selectedVersionId) ?? null);

let sortedVersions = $derived([...projectData.versions].sort((a, b) => new Date(b.date_published).getTime() - new Date(a.date_published).getTime()));

function formatDate(date: string) {
  return new Date(date).toLocaleDateString();
}

function formatDownloads(downloads: number) {
  return downloads.toLocaleString();
}

async function installVersion(version: ProjectVersion) {
  if (!profile || loading || version.id === project.version_id) {
    return;
  }

  loading = true;

  try {
    await app.projectsService.update(profile, {
      ...project,
      version_id: version.id,
    });

    selectedVersionId = version.id;
  } finally {
    loading = false;
  }
}
</script>

<div class="project-versions-modal">
  <header class="header">
    <div>
      <h2>Versions</h2>
      <p>{projectData.title}</p>
    </div>

    <span class="installed">
      Installed:
      {projectData.versions.find((version) => version.id === project.version_id)?.version_number ?? project.version_id}
    </span>
  </header>

  <div class="versions">
    {#if sortedVersions.length === 0}
      <div class="empty">No versions available.</div>
    {:else}
      {#each sortedVersions as version (version.id)}
        {@const isInstalled = project.version_id === version.id}
        {@const isSelected = selectedVersionId === version.id}

        <button class:selected={isSelected} class:installed={isInstalled} class="version" type="button" onclick={() => (selectedVersionId = version.id)}>
          <div class="version-main">
            <div class="version-title">
              <strong>{version.name}</strong>

              {#if version.featured}
                <span class="featured">Featured</span>
              {/if}

              {#if isInstalled}
                <span class="current">Installed</span>
              {/if}
            </div>

            <span class="version-number">
              {version.version_number}
            </span>
          </div>

          <div class="version-meta">
            <span>{formatDate(version.date_published)}</span>
            <span>•</span>
            <span>{formatDownloads(version.downloads)} downloads</span>

            {#if version.loaders.length}
              <span>•</span>
              <span>{version.loaders.join(", ")}</span>
            {/if}
          </div>
        </button>
      {/each}
    {/if}
  </div>

  {#if selectedVersion}
    <section class="details">
      <div class="details-header">
        <div>
          <h3>{selectedVersion.name}</h3>
          <span>{selectedVersion.version_number}</span>
        </div>

        {#if selectedVersion.id === project.version_id}
          <span class="current-badge">Current version</span>
        {/if}
      </div>

      <div class="details-grid">
        <div>
          <span class="label">Minecraft</span>
          <span>
            {selectedVersion.game_versions.join(", ") || "Unknown"}
          </span>
        </div>

        <div>
          <span class="label">Loaders</span>
          <span>
            {selectedVersion.loaders.join(", ") || "Unknown"}
          </span>
        </div>

        <div>
          <span class="label">Release</span>
          <span>{selectedVersion.version_type}</span>
        </div>

        <div>
          <span class="label">Published</span>
          <span>{formatDate(selectedVersion.date_published)}</span>
        </div>
      </div>

      {#if selectedVersion.changelog}
        <div class="changelog">
          <h4>Changelog</h4>
          <p>{selectedVersion.changelog}</p>
        </div>
      {/if}

      <footer class="actions">
        <button class="install" type="button" disabled={loading || !profile || selectedVersion.id === project.version_id} onclick={() => installVersion(selectedVersion)}>
          {#if loading}
            Updating...
          {:else if selectedVersion.id === project.version_id}
            Installed
          {:else}
            Update to this version
          {/if}
        </button>
      </footer>
    </section>
  {/if}
</div>

<style lang="scss">
.project-versions-modal {
  display: flex;
  flex-direction: column;
  width: min($layout-container-5, 90vw);
  max-height: 80vh;
  overflow: hidden;
  background: $color-surface-1;
  color: $color-text;
  border: 1px solid $color-border;
  border-radius: $radius-xl;
}

.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: $space-lg;
  padding: $space-xl;
  border-bottom: 1px solid $color-border-muted;

  h2 {
    margin: 0;
    font-size: 1.2rem;
    font-weight: 600;
  }

  p {
    margin: $space-xs 0 0;
    color: $color-text-muted;
    font-size: 0.85rem;
  }
}

.installed {
  flex: 0 0 auto;
  padding: $space-sm $space-md;
  border-radius: $radius-round;
  background: $color-selected;
  color: $color-accent-muted;
  font-size: 0.75rem;
}

.versions {
  display: flex;
  flex-direction: column;
  gap: $space-sm;
  overflow-y: auto;
  padding: $space-lg;
}

.version {
  display: flex;
  flex-direction: column;
  gap: $space-sm;
  width: 100%;
  padding: $space-md;
  border: 1px solid $color-border-muted;
  border-radius: $radius-md;
  background: $color-surface-2;
  color: $color-text;
  font: inherit;
  text-align: left;
  cursor: pointer;
  transition:
    background-color 120ms ease,
    border-color 120ms ease;

  &:hover {
    background: $color-surface-3;
    border-color: $color-border;
  }

  &:focus-visible {
    outline: 2px solid $color-focus;
    outline-offset: 2px;
  }

  &.selected {
    border-color: $color-accent;
    background: $color-selected;
  }

  &.installed {
    border-color: rgba(34, 197, 94, 0.35);
  }
}

.version-main {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: $space-md;
}

.version-title {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: $space-sm;

  strong {
    font-size: 0.9rem;
    font-weight: 600;
  }
}

.version-number {
  color: $color-text-muted;
  font-size: 0.8rem;
}

.featured,
.current,
.current-badge {
  padding: $space-1 $space-sm;
  border-radius: $radius-round;
  font-size: 0.65rem;
  font-weight: 600;
}

.featured {
  background: rgba(234, 179, 8, 0.12);
  color: $color-warning;
}

.current {
  background: rgba(34, 197, 94, 0.12);
  color: $color-success;
}

.version-meta {
  display: flex;
  flex-wrap: wrap;
  gap: $space-xs;
  color: $color-text-muted;
  font-size: 0.72rem;
}

.empty {
  padding: $space-2xl;
  color: $color-text-muted;
  text-align: center;
}

.details {
  display: flex;
  flex-direction: column;
  gap: $space-lg;
  padding: $space-xl;
  border-top: 1px solid $color-border-muted;
  background: $color-surface-1;
}

.details-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: $space-md;

  h3 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
  }

  > div > span {
    display: block;
    margin-top: $space-xs;
    color: $color-text-muted;
    font-size: 0.8rem;
  }
}

.current-badge {
  background: rgba(34, 197, 94, 0.12);
  color: $color-success;
}

.details-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: $space-md;
  padding: $space-lg;
  border-radius: $radius-lg;
  background: $color-surface-2;

  div {
    display: flex;
    flex-direction: column;
    gap: $space-xs;
    min-width: 0;
  }

  .label {
    color: $color-text-muted;
    font-size: 0.7rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  span:last-child {
    color: $color-text;
    font-size: 0.8rem;
    word-break: break-word;
  }
}

.changelog {
  max-height: 180px;
  overflow-y: auto;
  padding: $space-lg;
  border-radius: $radius-lg;
  background: $color-surface-2;

  h4 {
    margin: 0 0 $space-sm;
    font-size: 0.8rem;
    font-weight: 600;
  }

  p {
    margin: 0;
    color: $color-text-muted;
    font-size: 0.8rem;
    line-height: 1.5;
    white-space: pre-wrap;
  }
}

.actions {
  display: flex;
  justify-content: flex-end;

  .install {
    padding: $space-sm $space-lg;
    border: 1px solid $color-accent;
    border-radius: $radius-md;
    background: $color-accent;
    color: $color-text;
    font: inherit;
    font-size: 0.8rem;
    cursor: pointer;
    transition:
      background-color 120ms ease,
      border-color 120ms ease;

    &:hover:not(:disabled) {
      background: $color-accent-hover;
      border-color: $color-accent-hover;
    }

    &:focus-visible {
      outline: 2px solid $color-focus;
      outline-offset: 2px;
    }

    &:disabled {
      cursor: default;
      opacity: 0.5;
    }
  }
}

@media (max-width: 600px) {
  .version-main {
    align-items: flex-start;
    flex-direction: column;
    gap: $space-xs;
  }

  .details-grid {
    grid-template-columns: 1fr;
  }
}
</style>
