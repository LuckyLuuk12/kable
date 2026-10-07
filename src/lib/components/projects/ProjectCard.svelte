<script lang="ts">
import { app, ProjectGalleryModal, ProjectModal, ProjectVersionsModal, type KableProfile, type Project, type ProjectVersion } from "$lib";

let {
  profile = null,
  project,
}: {
  profile?: KableProfile | null;
  project: Project;
} = $props();

let installing = $state(false);
let liking = $state(false);

/*
 * The installed project comes from ProjectsService, which is the
 * authoritative frontend representation of what is installed for
 * the selected profile.
 */
let installed = $derived(profile ? (app.projectsService.all.find((p) => p.project.project_id === project.project_id) ?? null) : null);

let isInstalled = $derived(installed !== null);

/*
 * Find the installed version from the currently browsed project first.
 *
 * The project returned by the browser contains the current Modrinth
 * version data. Fall back to the installed project's embedded version
 * data if necessary.
 */
let installedVersion = $derived.by(() => {
  if (!installed) return null;

  return (
    project.versions.find((version) => version.id === installed.version_id) ?? installed.project.versions.find((version) => version.id === installed.version_id) ?? null
  );
});

/*
 * IMPORTANT:
 *
 * Do not use project.latest_version here.
 *
 * That field represents Modrinth's globally latest version and can be
 * for another loader, such as NeoForge, while this profile uses Fabric.
 *
 * ProjectsService.isVersionCompatible() mirrors the backend's
 * compatibility rules:
 *   - Minecraft version must match
 *   - mods/modpacks must use the profile's loader
 *   - resourcepacks/shaders do not require loader matching
 */
let compatibleVersions = $derived.by(() => {
  if (!profile) return [];

  return project.versions.filter((version) => app.projectsService.isVersionCompatible(profile, version, project.project_type));
});

/*
 * Match the backend's latest_compatible_version() behavior:
 * newest compatible version by publication date.
 */
let latestCompatibleVersion = $derived.by<ProjectVersion | null>(() => {
  return [...compatibleVersions].sort((a, b) => new Date(b.date_published).getTime() - new Date(a.date_published).getTime())[0] ?? null;
});

/*
 * The backend considers a project updated when the newest compatible
 * version has a different ID from the installed version.
 *
 * We deliberately do not compare against project.latest_version,
 * because that can be a version for another loader.
 */
let isUpdateAvailable = $derived.by(() => {
  if (!installed || !latestCompatibleVersion) return false;

  return installed.version_id !== latestCompatibleVersion.id;
});

let hasGallery = $derived((project.gallery?.length ?? 0) > 0);

/*
 * There is something installable only when we have a compatible
 * version for the current profile.
 */
let canInstall = $derived(!isInstalled && latestCompatibleVersion !== null);

/*
 * Only show the update action when the backend-equivalent logic says
 * there is an actual compatible update.
 */
let canUpdate = $derived(isInstalled && isUpdateAvailable && latestCompatibleVersion !== null);

async function install() {
  if (!profile || installing) return;

  /*
   * Always resolve the version from the same compatible-version
   * calculation used by the UI.
   *
   * Never use project.latest_version here.
   */
  const version = latestCompatibleVersion;

  if (!version) return;

  if (installed && !isUpdateAvailable) return;

  installing = true;

  try {
    if (installed) {
      /*
       * Pass the selected Modrinth version ID explicitly.
       *
       * The backend then validates the ID against the profile and
       * downloads exactly that version.
       */
      await app.projectsService.update(profile, installed, version.id);
    } else {
      /*
       * Explicitly install the newest compatible version rather than
       * allowing the backend/download layer to fall back to
       * project.latest_version.
       */
      await app.projectsService.download(profile, project, version.id);
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
  app.show(ProjectVersionsModal, {
    profile,
    project,
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

    <button class="action-button" type="button" onclick={showVersions}> Versions </button>

    {#if hasGallery}
      <button class="action-button" type="button" onclick={showGallery}> Gallery </button>
    {/if}

    <button class="action-button" type="button" onclick={showDetails}> Details </button>

    {#if profile && (canInstall || canUpdate)}
      <button class="action-button primary" type="button" disabled={installing} onclick={install}>
        {#if installing}
          {canUpdate ? "Updating..." : "Installing..."}
        {:else if canUpdate}
          Update
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

  border: 1px solid $color-border-muted;
  border-radius: 12px;

  background: $color-surface-1;
  color: $color-text;

  transition:
    border-color 0.15s ease,
    background 0.15s ease,
    box-shadow 0.15s ease,
    opacity 0.15s ease;

  &:hover {
    border-color: $color-border;
    background: $color-surface-2;
  }

  &.installed {
    border-color: $color-accent-muted;
    opacity: 0.475;

    &:hover {
      border-color: $color-accent;
      background: $color-surface-2;
      opacity: 0.88;
    }
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
    outline: 2px solid $color-focus;
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

  background: $color-surface-2;

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

  color: $color-text;

  font-size: 26px;
  font-weight: 700;
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

  background: $color-selected;
  color: $color-accent-muted;

  font-size: 10px;
  font-weight: 600;
  line-height: 1.4;
  text-transform: uppercase;
}

.description {
  display: -webkit-box;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
  line-clamp: 2;

  margin: 0;

  overflow: hidden;

  color: $color-text-muted;

  font-size: 12px;
  line-height: 1.45;
}

.metadata {
  display: flex;
  align-items: center;
  gap: 7px;

  margin-top: 7px;

  overflow: hidden;

  color: $color-placeholder;

  font-size: 11px;
  line-height: 1.3;

  span {
    overflow: hidden;

    white-space: nowrap;
    text-overflow: ellipsis;
  }

  .separator {
    flex: 0 0 auto;
    color: $color-border;
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

    background: $color-surface-2;
    color: $color-text-muted;

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
  max-width: 140px;

  margin-right: auto;

  overflow: hidden;

  color: $color-placeholder;

  font-family: monospace;
  font-size: 10px;

  white-space: nowrap;
  text-overflow: ellipsis;
}

.action-button {
  flex: 0 0 auto;

  min-height: 28px;
  padding: 0 9px;

  border: 1px solid $color-border-muted;
  border-radius: 6px;

  background: $color-surface-2;
  color: $color-text-muted;

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
    border-color: $color-border;
    background: $color-surface-3;
    color: $color-text;
  }

  &:active:not(:disabled) {
    transform: translateY(1px);
  }

  &:focus-visible {
    outline: 2px solid $color-focus;
    outline-offset: 2px;
  }

  &:disabled {
    cursor: default;
    opacity: 0.45;
  }

  &.primary {
    border-color: $color-accent;
    background: $color-accent;
    color: $color-text;

    &:hover:not(:disabled) {
      border-color: $color-accent-hover;
      background: $color-accent-hover;
    }

    &:active:not(:disabled) {
      background: $color-accent-active;
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
