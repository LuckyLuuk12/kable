<script lang="ts">
import { app, ProjectGalleryModal, ProjectModal, ProjectVersionsModal, type KableProfile, type Project } from "$lib";

let {
  profile = null,
  project,
}: {
  profile?: KableProfile | null;
  project: Project;
} = $props();

let installing = $state(false);
let liking = $state(false);

let installed = $derived(profile ? (app.projectsService.all.find((p) => p.project.project_id === project.project_id) ?? null) : null);

let installedVersion = $derived(installed?.project.versions.find((version) => version.id === installed.version_id) ?? null);

let isInstalled = $derived(installed !== null);

let latestVersion = $derived([...project.versions].sort((a, b) => new Date(b.date_published).getTime() - new Date(a.date_published).getTime())[0] ?? null);

let isUpdateAvailable = $derived(!!installed && !!latestVersion && installed.version_id !== latestVersion.id);

let hasGallery = $derived((project.gallery?.length ?? 0) > 0);

async function install() {
  if (!profile || installing || !latestVersion) return;

  installing = true;

  try {
    if (installed) {
      if (!isUpdateAvailable) return;

      await app.projectsService.update(profile, {
        ...installed,
        project: {
          ...installed.project,
          versions: project.versions,
        },
        version_id: latestVersion.id,
      });
    } else {
      await app.projectsService.download(profile, project, latestVersion.id);
    }
  } finally {
    installing = false;
  }
}

async function toggleLike() {
  if (!profile || liking) return;

  liking = true;

  try {
    // await app.projectsService.toggleLike(profile, project);
  } finally {
    liking = false;
  }
}

function showDetails() {
  app.show(ProjectModal, {
    profile,
    project,
  });
}

function showVersions() {
  if (!installed) return;

  app.show(ProjectVersionsModal, {
    profile,
    project: installed,
  });
}

function showGallery() {
  if (!hasGallery) return;

  app.show(ProjectGalleryModal, {
    project,
  });
}
</script>

<article class:installed class="project-card">
  <button class="project-main" type="button" onclick={showDetails}>
    <div class="project-icon">
      {#if project.icon_url}
        <img src={project.icon_url} alt="" loading="lazy" />
      {:else}
        <div class="icon-placeholder">
          {project.title.charAt(0).toUpperCase()}
        </div>
      {/if}
    </div>

    <div class="project-content">
      <div class="project-header">
        <h3>{project.title}</h3>

        {#if isInstalled}
          <span class="installed-badge">Installed</span>
        {/if}
      </div>

      <p class="description">
        {project.description}
      </p>

      <div class="metadata">
        <span>{project.author}</span>

        <span class="separator">•</span>

        <span>
          {project.downloads.toLocaleString()} downloads
        </span>

        {#if project.latest_version}
          <span class="separator">•</span>
          <span>{project.latest_version}</span>
        {/if}
      </div>

      {#if project.display_categories?.length}
        <div class="categories">
          {#each project.display_categories.slice(0, 3) as category (category)}
            <span>{category}</span>
          {/each}
        </div>
      {/if}
    </div>
  </button>

  <div class="actions">
    {#if installed}
      <span class="installed-version">
        {installedVersion?.version_number ?? installed.version_id}
      </span>
    {/if}

    <button class="action-button" type="button" disabled={!profile || liking} onclick={toggleLike} title="Like project"> Like </button>

    <button class="action-button" type="button" disabled={!installed} onclick={showVersions}> Versions </button>

    {#if hasGallery}
      <button class="action-button" type="button" onclick={showGallery}> Gallery </button>
    {/if}

    <button class="action-button" type="button" onclick={showDetails}> Details </button>

    {#if profile}
      <button class="action-button primary" type="button" disabled={!latestVersion || installing} onclick={install}>
        {#if installing}
          {isInstalled ? "Updating..." : "Installing..."}
        {:else if isUpdateAvailable}
          Update
        {:else if isInstalled}
          Installed
        {:else}
          Install
        {/if}
      </button>
    {/if}
  </div>
</article>

<style lang="scss">
.project-card {
  display: flex;
  flex-direction: column;
  gap: 12px;

  padding: 14px;
  border: 1px solid var(--border-color, #2a2a2a);
  border-radius: 12px;

  background: var(--surface-color, #181818);
  color: var(--text-color, #f2f2f2);

  transition:
    border-color 0.15s ease,
    background 0.15s ease,
    box-shadow 0.15s ease;

  &:hover {
    border-color: var(--border-hover-color, #3a3a3a);
    background: var(--surface-hover-color, #1c1c1c);
  }

  &.installed {
    border-color: var(--accent-color, #5865f2);
  }
}

.project-main {
  display: flex;
  width: 100%;
  min-width: 0;

  gap: 14px;
  padding: 0;

  border: 0;
  background: transparent;
  color: inherit;

  text-align: left;
  cursor: pointer;

  &:focus-visible {
    outline: 2px solid var(--accent-color, #5865f2);
    outline-offset: 4px;
    border-radius: 6px;
  }
}

.project-icon {
  flex: 0 0 64px;
  width: 64px;
  height: 64px;

  overflow: hidden;
  border-radius: 10px;

  background: var(--icon-background, #242424);

  img {
    display: block;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
}

.icon-placeholder {
  display: flex;
  align-items: center;
  justify-content: center;

  width: 100%;
  height: 100%;

  font-size: 26px;
  font-weight: 700;

  color: var(--text-color, #f2f2f2);
}

.project-content {
  flex: 1;
  min-width: 0;
}

.project-header {
  display: flex;
  align-items: center;
  gap: 8px;

  margin-bottom: 4px;

  h3 {
    min-width: 0;
    margin: 0;

    overflow: hidden;

    font-size: 15px;
    font-weight: 600;
    line-height: 1.3;

    white-space: nowrap;
    text-overflow: ellipsis;
  }
}

.installed-badge {
  flex: 0 0 auto;

  padding: 2px 7px;
  border-radius: 999px;

  background: color-mix(in srgb, var(--accent-color, #5865f2) 15%, transparent);

  color: var(--accent-color, #7c83ff);

  font-size: 10px;
  font-weight: 600;
  line-height: 1.4;
  text-transform: uppercase;
}

.description {
  margin: 0;

  overflow: hidden;

  color: var(--text-secondary, #999);

  font-size: 12px;
  line-height: 1.45;

  display: -webkit-box;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
}

.metadata {
  display: flex;
  align-items: center;
  gap: 7px;

  margin-top: 7px;

  overflow: hidden;

  color: var(--text-muted, #666);

  font-size: 11px;
  line-height: 1.3;

  span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .separator {
    flex: 0 0 auto;
    color: var(--text-muted, #555);
  }
}

.categories {
  display: flex;
  gap: 5px;

  margin-top: 8px;

  overflow: hidden;

  span {
    flex: 0 0 auto;

    padding: 3px 7px;
    border-radius: 5px;

    background: var(--tag-background, #242424);
    color: var(--text-secondary, #999);

    font-size: 10px;
    line-height: 1.2;
  }
}

.actions {
  display: flex;
  align-items: center;
  gap: 6px;

  min-height: 30px;
  padding-top: 2px;
}

.installed-version {
  margin-right: auto;

  max-width: 140px;

  overflow: hidden;

  color: var(--text-muted, #777);

  font-family: monospace;
  font-size: 10px;

  white-space: nowrap;
  text-overflow: ellipsis;
}

.action-button {
  flex: 0 0 auto;

  min-height: 28px;
  padding: 0 9px;

  border: 1px solid var(--button-border, #303030);
  border-radius: 6px;

  background: var(--button-background, #202020);
  color: var(--text-secondary, #aaa);

  font: inherit;
  font-size: 11px;
  font-weight: 500;

  cursor: pointer;

  transition:
    background 0.12s ease,
    border-color 0.12s ease,
    color 0.12s ease,
    opacity 0.12s ease;

  &:hover:not(:disabled) {
    border-color: var(--button-border-hover, #444);
    background: var(--button-background-hover, #292929);
    color: var(--text-color, #f2f2f2);
  }

  &:active:not(:disabled) {
    transform: translateY(1px);
  }

  &:focus-visible {
    outline: 2px solid var(--accent-color, #5865f2);
    outline-offset: 2px;
  }

  &:disabled {
    cursor: default;
    opacity: 0.45;
  }

  &.primary {
    border-color: var(--accent-color, #5865f2);
    background: var(--accent-color, #5865f2);
    color: white;

    &:hover:not(:disabled) {
      filter: brightness(1.1);
    }

    &:disabled {
      opacity: 0.5;
    }
  }
}

@media (max-width: 600px) {
  .project-card {
    padding: 12px;
  }

  .project-icon {
    flex-basis: 52px;
    width: 52px;
    height: 52px;
  }

  .actions {
    flex-wrap: wrap;
  }

  .installed-version {
    width: 100%;
    margin-right: 0;
    margin-bottom: 2px;
  }
}
</style>
