<!--
@component
Displays the details of a project and allows it to be installed or, when already
installed, opens the project's installed versions.
-->
<script lang="ts">
import { app, ProjectVersionsModal, type KableProfile, type Project } from "$lib";

let {
  profile = null,
  project,
}: {
  profile?: KableProfile | null;
  project: Project;
} = $props();

let installing = $state(false);

let installedProject = $derived(profile ? (app.projectsService.all.find((p) => p.project.project_id === project.project_id) ?? null) : null);

let installedVersion = $derived(installedProject?.project.versions.find((version) => version.id === installedProject.version_id) ?? null);

let latestVersion = $derived([...project.versions].sort((a, b) => new Date(b.date_published).getTime() - new Date(a.date_published).getTime())[0] ?? null);

let isUpdateAvailable = $derived(!!installedProject && !!latestVersion && installedProject.version_id !== latestVersion.id);

async function install() {
  if (!profile || installing || !latestVersion) return;

  installing = true;

  try {
    if (installedProject) {
      if (!isUpdateAvailable) return;

      await app.projectsService.update(profile, {
        ...installedProject,
        project: {
          ...installedProject.project,
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

function showVersions() {
  if (!installedProject) return;

  app.show(ProjectVersionsModal, {
    profile,
    project: installedProject,
  });
}
</script>

<div class="project-modal">
  <div class="hero">
    <div class="icon">
      {#if project.icon_url}
        <img src={project.icon_url} alt="" />
      {:else}
        <div class="icon-placeholder">
          {project.title.charAt(0).toUpperCase()}
        </div>
      {/if}
    </div>

    <div class="hero-content">
      <div class="title-row">
        <h2>{project.title}</h2>

        {#if installedProject}
          <span class="installed-badge">Installed</span>
        {/if}
      </div>

      <p class="description">{project.description}</p>

      <div class="meta">
        <span>by {project.author}</span>
        <span>•</span>
        <span>{project.project_type}</span>
        <span>•</span>
        <span>{project.downloads.toLocaleString()} downloads</span>
      </div>
    </div>
  </div>

  <div class="content">
    {#if project.categories?.length}
      <section>
        <h3>Categories</h3>

        <div class="tags">
          {#each project.categories as category (category)}
            <span class="tag">{category}</span>
          {/each}
        </div>
      </section>
    {/if}

    <section class="compatibility">
      <h3>Compatibility</h3>

      <div class="compatibility-grid">
        <div>
          <span class="label">Client</span>
          <span>{project.client_side}</span>
        </div>

        <div>
          <span class="label">Server</span>
          <span>{project.server_side}</span>
        </div>

        <div>
          <span class="label">Minecraft</span>
          <span>{project.latest_version ?? "Unknown"}</span>
        </div>

        <div>
          <span class="label">License</span>
          <span>{project.license}</span>
        </div>
      </div>
    </section>

    {#if project.gallery?.length}
      <section>
        <h3>Gallery</h3>

        <div class="gallery-preview">
          {#each project.gallery.slice(0, 4) as image (image)}
            <img src={image} alt="" loading="lazy" />
          {/each}
        </div>
      </section>
    {/if}

    <section class="version">
      <h3>Version</h3>

      {#if installedVersion}
        <div class="version-info">
          <div>
            <span class="label">Installed</span>
            <strong>{installedVersion.version_number}</strong>
          </div>

          {#if isUpdateAvailable && latestVersion}
            <div>
              <span class="label">Latest</span>
              <strong>{latestVersion.version_number}</strong>
            </div>
          {/if}
        </div>
      {:else if latestVersion}
        <div class="version-info">
          <div>
            <span class="label">Latest</span>
            <strong>{latestVersion.version_number}</strong>
          </div>

          <div>
            <span class="label">Released</span>
            <span>
              {new Date(latestVersion.date_published).toLocaleDateString()}
            </span>
          </div>
        </div>
      {:else}
        <p class="muted">No versions available.</p>
      {/if}
    </section>
  </div>

  <footer class="actions">
    {#if installedProject}
      <button type="button" class="secondary" onclick={showVersions}>
        {isUpdateAvailable ? "View updates" : "View versions"}
      </button>

      {#if isUpdateAvailable}
        <button type="button" class="primary" disabled={!profile || installing} onclick={install}>
          {installing ? "Updating..." : "Update"}
        </button>
      {/if}
    {:else}
      <button type="button" class="primary" disabled={!profile || !latestVersion || installing} onclick={install}>
        {installing ? "Installing..." : "Install"}
      </button>
    {/if}
  </footer>
</div>

<style lang="scss">
.project-modal {
  display: flex;
  flex-direction: column;
  width: min($layout-container-5, 90vw);
  max-height: 85vh;
  overflow: hidden;
  background: $color-surface-1;
  color: $color-text;
  border: 1px solid $color-border;
  border-radius: $radius-xl;
}

.hero {
  display: flex;
  gap: $space-lg;
  padding: $space-xl;
  border-bottom: 1px solid $color-border-muted;
  background: $color-surface-2;
}

.icon {
  flex: 0 0 auto;
  width: 72px;
  height: 72px;
  overflow: hidden;
  border-radius: $radius-lg;
  background: $color-surface-3;

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
  color: $color-text-muted;
  font-size: 1.8rem;
  font-weight: 600;
}

.hero-content {
  display: flex;
  flex-direction: column;
  justify-content: center;
  min-width: 0;
}

.title-row {
  display: flex;
  align-items: center;
  gap: $space-sm;
  flex-wrap: wrap;

  h2 {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 600;
  }
}

.installed-badge {
  padding: $space-1 $space-sm;
  border-radius: $radius-round;
  background: rgba(34, 197, 94, 0.12);
  color: $color-success;
  font-size: 0.65rem;
  font-weight: 600;
}

.description {
  margin: $space-sm 0 0;
  color: $color-text-muted;
  font-size: 0.85rem;
  line-height: 1.45;
}

.meta {
  display: flex;
  flex-wrap: wrap;
  gap: $space-xs;
  margin-top: $space-sm;
  color: $color-text-muted;
  font-size: 0.7rem;
}

.content {
  display: flex;
  flex-direction: column;
  gap: $space-xl;
  overflow-y: auto;
  padding: $space-xl;

  section {
    display: flex;
    flex-direction: column;
    gap: $space-md;
  }

  h3 {
    margin: 0;
    font-size: 0.8rem;
    font-weight: 600;
  }
}

.tags {
  display: flex;
  flex-wrap: wrap;
  gap: $space-sm;
}

.tag {
  padding: $space-1 $space-sm;
  border: 1px solid $color-border-muted;
  border-radius: $radius-round;
  background: $color-surface-2;
  color: $color-text-muted;
  font-size: 0.7rem;
}

.compatibility-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: $space-md;
}

.compatibility-grid > div,
.version-info > div {
  display: flex;
  flex-direction: column;
  gap: $space-xs;
  padding: $space-md;
  border-radius: $radius-md;
  background: $color-surface-2;
}

.label {
  color: $color-text-muted;
  font-size: 0.65rem;
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.compatibility-grid span:last-child,
.version-info span:last-child {
  font-size: 0.78rem;
}

.gallery-preview {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: $space-sm;

  img {
    display: block;
    width: 100%;
    aspect-ratio: 16 / 9;
    border-radius: $radius-md;
    object-fit: cover;
    background: $color-surface-2;
  }
}

.version-info {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: $space-md;

  strong {
    font-size: 0.9rem;
  }
}

.muted {
  margin: 0;
  color: $color-text-muted;
  font-size: 0.8rem;
}

.actions {
  display: flex;
  justify-content: flex-end;
  gap: $space-sm;
  padding: $space-lg $space-xl;
  border-top: 1px solid $color-border-muted;
}

.actions button {
  padding: $space-sm $space-lg;
  border: 1px solid transparent;
  border-radius: $radius-md;
  font: inherit;
  font-size: 0.8rem;
  cursor: pointer;

  &:focus-visible {
    outline: 2px solid $color-focus;
    outline-offset: 2px;
  }

  &:disabled {
    cursor: default;
    opacity: 0.5;
  }
}

.primary {
  border-color: $color-accent !important;
  background: $color-accent;
  color: $color-text;

  &:hover:not(:disabled) {
    background: $color-accent-hover;
    border-color: $color-accent-hover !important;
  }
}

.secondary {
  border-color: $color-border !important;
  background: $color-surface-2;
  color: $color-text;

  &:hover {
    background: $color-surface-3;
  }
}

@media (max-width: 600px) {
  .hero {
    padding: $space-lg;
  }

  .content {
    padding: $space-lg;
  }

  .compatibility-grid,
  .version-info {
    grid-template-columns: 1fr;
  }

  .gallery-preview {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}
</style>
