<!--
@component

Displays project information and installation status for a Modrinth project.

The modal deliberately separates:

- the selected profile
- the exact installed Modrinth version
- the project's available compatibility
- the backend-authoritative update status

Update availability is determined through ProjectsService.checkUpdate(),
which invokes the backend compatibility/update logic. The frontend does not
independently decide whether a newer compatible version exists.
-->
<script lang="ts">
import { app, ProjectVersionsModal, type KableProfile, type Project, type ProjectVersion } from "$lib";

let {
  profile = null,
  project,
}: {
  profile?: KableProfile | null;
  project: Project;
} = $props();

let installing = $state(false);
let checkingUpdate = $state(false);
let updateAvailable = $state(false);

/**
 * Installed project metadata for the currently selected profile.
 *
 * The project ID identifies the project. `version_id` identifies the exact
 * Modrinth version that is installed.
 */
let installedProject = $derived(profile ? (app.projectsService.all.find((installed) => installed.project.project_id === project.project_id) ?? null) : null);

/**
 * Resolve the exact installed ProjectVersion.
 *
 * Never infer the installed version from the profile, filename, version
 * number, latest release, or loader.
 */
let installedVersion = $derived.by<ProjectVersion | null>(() => {
  if (!installedProject) {
    return null;
  }

  return (
    project.versions.find((version) => version.id === installedProject.version_id) ??
    installedProject.project.versions.find((version) => version.id === installedProject.version_id) ??
    null
  );
});

let profileMinecraftVersion = $derived(profile?.version.minecraft_version ?? null);

let profileLoader = $derived(profile ? normalizeLoader(profile.version.loader.toString()) : null);

/**
 * Compatibility of the exact installed version with the selected profile.
 *
 * This is display information. Update availability itself is determined by
 * the backend through ProjectsService.checkUpdate().
 */
let installedVersionCompatible = $derived(profile && installedVersion ? app.projectsService.isVersionCompatible(profile, installedVersion, project.project_type) : false);

let installedLoaderMatchesProfile = $derived.by(() => {
  if (!profileLoader || !installedVersion) {
    return false;
  }

  return installedVersion.loaders.some((loader) => normalizeLoader(loader) === profileLoader);
});

let installedMinecraftMatchesProfile = $derived.by(() => {
  if (!profileMinecraftVersion || !installedVersion) {
    return false;
  }

  return installedVersion.game_versions.includes(profileMinecraftVersion);
});

/**
 * Latest compatible version used for display/install targeting.
 *
 * This is currently still resolved by the service because the existing
 * frontend service API exposes getLatestCompatibleVersion(). The actual
 * update decision below is NOT based on this value.
 *
 * Ideally the backend should eventually return the exact resolved update
 * version from checkUpdate(), allowing this frontend-side resolution to be
 * removed as well.
 */
let latestCompatibleVersion = $derived(profile ? app.projectsService.getLatestCompatibleVersion(profile, project) : null);

/**
 * Latest release of the project, irrespective of profile compatibility.
 */
let latestVersion = $derived([...project.versions].sort((a, b) => new Date(b.date_published).getTime() - new Date(a.date_published).getTime())[0] ?? null);

/**
 * Whether the exact installed version itself works with the selected profile.
 */
let installedMatchesProfile = $derived(Boolean(installedVersion && profile && installedVersionCompatible));

/**
 * Whether the project has any version that supports the profile's loader and
 * Minecraft version.
 *
 * This remains display information. The backend is authoritative for actual
 * installation/update decisions.
 */
let projectSupportsProfile = $derived(latestCompatibleVersion !== null);

/**
 * Check update availability through the backend.
 *
 * ProjectsService.checkUpdate() invokes the backend's check_for_update()
 * command. The backend returns Ok(Project) when an update exists and Err when
 * no update is available or no compatible version exists.
 *
 * No version comparison is performed here.
 */
$effect(() => {
  const currentProfile = profile;
  const currentInstalledProject = installedProject;

  updateAvailable = false;

  if (!currentProfile || !currentInstalledProject) {
    return;
  }

  let cancelled = false;

  checkingUpdate = true;

  void app.projectsService
    .checkUpdate(currentProfile, currentInstalledProject)
    .then(() => {
      if (cancelled) {
        return;
      }

      updateAvailable = true;
    })
    .catch(() => {
      if (cancelled) {
        return;
      }

      updateAvailable = false;
    })
    .finally(() => {
      if (!cancelled) {
        checkingUpdate = false;
      }
    });

  return () => {
    cancelled = true;
    checkingUpdate = false;
  };
});

/**
 * Human-readable loader names.
 */
function normalizeLoader(loader: string): string {
  return loader
    .trim()
    .toLowerCase()
    .replace(/^iris_/, "")
    .replace(/^quilted_fabric$/, "fabric");
}

function formatLoader(loader: string): string {
  return loader
    .replace(/^iris_/i, "")
    .replace(/_/g, " ")
    .replace(/\b\w/g, (character) => character.toUpperCase());
}

function formatDate(date: string | null | undefined): string {
  if (!date) {
    return "Unknown";
  }

  const parsed = new Date(date);

  if (Number.isNaN(parsed.getTime())) {
    return "Unknown";
  }

  return parsed.toLocaleDateString();
}

function formatDateTime(date: string | null | undefined): string {
  if (!date) {
    return "Unknown";
  }

  const parsed = new Date(date);

  if (Number.isNaN(parsed.getTime())) {
    return "Unknown";
  }

  return parsed.toLocaleString();
}

function formatNumber(value: number): string {
  return value.toLocaleString();
}

function formatProjectType(type: string): string {
  return type.charAt(0).toUpperCase() + type.slice(1);
}

function formatSupport(value: string): string {
  return value.charAt(0).toUpperCase() + value.slice(1);
}

/**
 * Format the Minecraft versions of a ProjectVersion without implying that
 * only one of them is relevant.
 */
function formatMinecraftVersions(version: ProjectVersion | null): string {
  if (!version?.game_versions.length) {
    return "Unknown";
  }

  return version.game_versions.join(", ");
}

function formatLoaders(version: ProjectVersion | null): string {
  if (!version?.loaders.length) {
    return "Unknown";
  }

  return version.loaders.map(formatLoader).join(", ");
}

/**
 * Returns the relationship between a project loader and the active profile.
 *
 * Green:
 *   The loader has at least one project version that also supports the
 *   profile's Minecraft version.
 *
 * Orange:
 *   This is the profile loader, but the project does not currently expose a
 *   version for the profile's Minecraft version.
 *
 * Red:
 *   This is another loader.
 */
function getLoaderState(loader: string): "success" | "warning" | "error" | "neutral" {
  if (!profile) {
    return "neutral";
  }

  const normalizedLoader = normalizeLoader(loader);

  const isProfileLoader = normalizedLoader === profileLoader;

  const supportsProfileVersion = project.versions.some(
    (version) =>
      version.game_versions.includes(profileMinecraftVersion ?? "") && version.loaders.some((versionLoader) => normalizeLoader(versionLoader) === normalizedLoader),
  );

  if (isProfileLoader && supportsProfileVersion) {
    return "success";
  }

  if (isProfileLoader) {
    return "warning";
  }

  return "error";
}

/**
 * All loaders represented by project versions.
 */
let supportedLoaders = $derived(Array.from(new Set(project.versions.flatMap((version) => version.loaders))));

/**
 * All Minecraft versions represented by project versions.
 */
let supportedMinecraftVersions = $derived(
  Array.from(new Set(project.versions.flatMap((version) => version.game_versions))).sort((a, b) =>
    b.localeCompare(a, undefined, {
      numeric: true,
      sensitivity: "base",
    }),
  ),
);

/**
 * For the profile's Minecraft version, show which loaders actually support
 * that exact Minecraft version.
 */
let profileMinecraftLoaders = $derived.by(() => {
  if (!profileMinecraftVersion) {
    return [];
  }

  return Array.from(new Set(project.versions.filter((version) => version.game_versions.includes(profileMinecraftVersion)).flatMap((version) => version.loaders)));
});

let categories = $derived(project.display_categories?.length ? project.display_categories : (project.categories ?? []));

let gallery = $derived(
  project.featured_gallery && project.gallery?.includes(project.featured_gallery)
    ? [project.featured_gallery, ...(project.gallery?.filter((image) => image !== project.featured_gallery) ?? [])]
    : (project.gallery ?? []),
);

let modrinthUrl = $derived(`https://modrinth.com/${project.project_type}/${project.slug}?utm_source=kable&utm_medium=launcher`);

async function install() {
  if (!profile || installing || checkingUpdate || !latestCompatibleVersion) {
    return;
  }

  installing = true;

  try {
    if (installedProject) {
      if (!updateAvailable) {
        return;
      }

      await app.projectsService.update(profile, installedProject, latestCompatibleVersion.id);
    } else {
      await app.projectsService.download(profile, project, latestCompatibleVersion.id);
    }
  } finally {
    installing = false;
  }
}

function showVersions() {
  app.show(ProjectVersionsModal, {
    profile,
    project: installedProject ?? project,
  });
}

function openModrinth() {
  app.openUrl(modrinthUrl);
}
</script>

<div class="project-modal">
  <!-- Header -->
  <header class="hero">
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

        <span class="project-type">
          {formatProjectType(project.project_type)}
        </span>
      </div>

      <div class="author">
        by <strong>{project.author}</strong>
      </div>

      <div class="reputation">
        <span>
          <strong>{formatNumber(project.downloads)}</strong>
          downloads
        </span>

        <span>
          <strong>{formatNumber(project.follows)}</strong>
          followers
        </span>

        <span>
          updated {formatDate(project.date_modified)}
        </span>
      </div>
    </div>
  </header>

  <div class="content">
    <!-- Installation / profile status -->
    {#if profile}
      <section class="status-section">
        <div class="section-heading">
          <h3>Installation</h3>

          <span class="profile-label">
            {profile.metadata.name}
          </span>
        </div>

        {#if installedProject && installedVersion}
          <div class:compatible={installedMatchesProfile} class:incompatible={!installedMatchesProfile} class="status-card">
            <div class="status-main">
              <div class="status-icon">
                {#if installedMatchesProfile}
                  ✓
                {:else}
                  !
                {/if}
              </div>

              <div class="status-copy">
                <strong>
                  {#if installedMatchesProfile}
                    Installed and compatible
                  {:else}
                    Installed version does not match this profile
                  {/if}
                </strong>

                <span>
                  {installedVersion.version_number}
                </span>
              </div>
            </div>

            <div class="status-details">
              <div>
                <span class="label">Minecraft</span>

                <strong class:match={installedMinecraftMatchesProfile} class:mismatch={!installedMinecraftMatchesProfile}>
                  {formatMinecraftVersions(installedVersion)}
                </strong>

                <small>
                  Profile:
                  {profileMinecraftVersion ?? "Unknown"}
                </small>
              </div>

              {#if project.project_type === "mod" || project.project_type === "modpack"}
                <div>
                  <span class="label">Loader</span>

                  <strong class:match={installedLoaderMatchesProfile} class:mismatch={!installedLoaderMatchesProfile}>
                    {formatLoaders(installedVersion)}
                  </strong>

                  <small>
                    Profile:
                    {formatLoader(profile.version.loader.toString())}
                  </small>
                </div>
              {/if}

              <div>
                <span class="label">Released</span>

                <strong>
                  {formatDate(installedVersion.date_published)}
                </strong>

                <small>
                  Version ID:
                  {installedVersion.id}
                </small>
              </div>
            </div>
          </div>

          {#if checkingUpdate}
            <div class="next-version checking">
              <div>
                <span class="label">Update status</span>

                <strong>Checking for updates...</strong>
              </div>
            </div>
          {:else if updateAvailable && latestCompatibleVersion}
            <div class="next-version">
              <div>
                <span class="label">Profile-compatible update</span>

                <strong>
                  {latestCompatibleVersion.version_number}
                </strong>

                <span>
                  {formatLoaders(latestCompatibleVersion)}
                  ·
                  {formatMinecraftVersions(latestCompatibleVersion)}
                </span>
              </div>

              <span class="update-label">Update available</span>
            </div>
          {:else if updateAvailable}
            <div class="next-version">
              <div>
                <span class="label">Update status</span>

                <strong>Update available</strong>

                <span> The backend found a newer compatible version. </span>
              </div>

              <span class="update-label">Update available</span>
            </div>
          {:else if installedMatchesProfile && !checkingUpdate}
            <div class="up-to-date">✓ This is the latest compatible version.</div>
          {/if}
        {:else if installedProject}
          <div class="status-card incompatible">
            <div class="status-main">
              <div class="status-icon">!</div>

              <div class="status-copy">
                <strong>Installed version information unavailable</strong>

                <span>
                  {installedProject.version_id}
                </span>
              </div>
            </div>

            <p>The installed version is recorded, but its ProjectVersion data is not present in the currently loaded project data.</p>
          </div>

          {#if latestCompatibleVersion}
            <div class="next-version">
              <div>
                <span class="label">Profile-compatible version</span>

                <strong>
                  {latestCompatibleVersion.version_number}
                </strong>

                <span>
                  {formatLoaders(latestCompatibleVersion)}
                  ·
                  {formatMinecraftVersions(latestCompatibleVersion)}
                </span>
              </div>
            </div>
          {/if}
        {:else if latestCompatibleVersion}
          <div class="status-card not-installed">
            <div class="status-main">
              <div class="status-icon">+</div>

              <div class="status-copy">
                <strong>Not installed</strong>

                <span>Compatible with this profile</span>
              </div>
            </div>

            <div class="status-details">
              <div>
                <span class="label">Recommended version</span>

                <strong>
                  {latestCompatibleVersion.version_number}
                </strong>
              </div>

              <div>
                <span class="label">Minecraft</span>

                <strong>
                  {formatMinecraftVersions(latestCompatibleVersion)}
                </strong>
              </div>

              {#if project.project_type === "mod" || project.project_type === "modpack"}
                <div>
                  <span class="label">Loader</span>

                  <strong>
                    {formatLoaders(latestCompatibleVersion)}
                  </strong>
                </div>
              {/if}
            </div>
          </div>
        {:else}
          <div class="status-card incompatible">
            <div class="status-main">
              <div class="status-icon">!</div>

              <div class="status-copy">
                <strong>Not compatible with this profile</strong>

                <span>No matching project version was found.</span>
              </div>
            </div>
          </div>
        {/if}
      </section>
    {/if}

    <!-- Compatibility -->
    <section>
      <div class="section-heading">
        <h3>Compatibility</h3>

        {#if projectSupportsProfile}
          <span class="section-state success"> Compatible version available </span>
        {:else if profile}
          <span class="section-state error"> No compatible version </span>
        {/if}
      </div>

      {#if profile}
        <div class="profile-target">
          <div>
            <span class="label">Your profile</span>

            <strong>
              {profileMinecraftVersion ?? "Unknown"}
            </strong>
          </div>

          {#if project.project_type === "mod" || project.project_type === "modpack"}
            <div>
              <span class="label">Loader</span>

              <strong>
                {formatLoader(profile.version.loader.toString())}
              </strong>
            </div>
          {/if}

          {#if project.project_type === "mod" || project.project_type === "modpack"}
            <div>
              <span class="label"> Available for this Minecraft version </span>

              <strong class:success={profileMinecraftLoaders.length > 0} class:error={profileMinecraftLoaders.length === 0}>
                {profileMinecraftLoaders.length ? profileMinecraftLoaders.map(formatLoader).join(", ") : "None"}
              </strong>
            </div>
          {/if}
        </div>
      {/if}

      {#if project.project_type === "mod" || project.project_type === "modpack"}
        <div class="compatibility-row">
          <span class="label">Loaders</span>

          <div class="tags">
            {#each supportedLoaders as loader (loader)}
              {@const state = getLoaderState(loader)}

              <span class:success={state === "success"} class:warning={state === "warning"} class:error={state === "error"} class="tag">
                {formatLoader(loader)}

                {#if profile && normalizeLoader(loader) === profileLoader}
                  <span class="tag-note">Profile</span>
                {/if}
              </span>
            {/each}
          </div>
        </div>
      {/if}

      <div class="compatibility-row">
        <span class="label">Minecraft versions</span>

        <div class="tags">
          {#each supportedMinecraftVersions as version (version)}
            <span class:profile-version={profile && version === profileMinecraftVersion} class="tag version-tag">
              {version}

              {#if profile && version === profileMinecraftVersion}
                <span class="tag-note">Profile</span>
              {/if}
            </span>
          {/each}
        </div>
      </div>

      <div class="support-grid">
        <div>
          <span class="label">Client</span>
          <strong>
            {formatSupport(project.client_side)}
          </strong>
        </div>

        <div>
          <span class="label">Server</span>
          <strong>
            {formatSupport(project.server_side)}
          </strong>
        </div>
      </div>
    </section>

    <!-- About -->
    <section>
      <div class="section-heading">
        <h3>About</h3>
      </div>

      <p class="description">
        {project.description}
      </p>

      {#if categories.length}
        <div class="category-area">
          <span class="label">Categories</span>

          <div class="tags">
            {#each categories as category (category)}
              <span class="tag">
                {category}
              </span>
            {/each}
          </div>
        </div>
      {/if}
    </section>

    <!-- Versions -->
    <section>
      <div class="section-heading">
        <h3>Versions</h3>

        <span class="version-count">
          {project.versions.length}
          {project.versions.length === 1 ? "release" : "releases"}
        </span>
      </div>

      {#if installedVersion}
        <div class="version-card installed">
          <div class="version-card-header">
            <div>
              <span class="label">Installed</span>

              <strong>
                {installedVersion.version_number}
              </strong>
            </div>

            <span class:success={installedMatchesProfile} class:error={!installedMatchesProfile} class="version-status">
              {installedMatchesProfile ? "Profile compatible" : "Profile mismatch"}
            </span>
          </div>

          <div class="version-card-meta">
            <span>
              {formatLoaders(installedVersion)}
            </span>

            <span>
              {formatMinecraftVersions(installedVersion)}
            </span>

            <span>
              {formatDate(installedVersion.date_published)}
            </span>
          </div>
        </div>
      {/if}

      {#if updateAvailable && latestCompatibleVersion}
        <div class="version-card update">
          <div class="version-card-header">
            <div>
              <span class="label">Latest compatible</span>

              <strong>
                {latestCompatibleVersion.version_number}
              </strong>
            </div>

            <span class="version-status warning"> Update available </span>
          </div>

          <div class="version-card-meta">
            <span>
              {formatLoaders(latestCompatibleVersion)}
            </span>

            <span>
              {formatMinecraftVersions(latestCompatibleVersion)}
            </span>

            <span>
              {formatDate(latestCompatibleVersion.date_published)}
            </span>
          </div>
        </div>
      {/if}

      {#if !installedVersion && latestCompatibleVersion}
        <div class="version-card recommended">
          <div class="version-card-header">
            <div>
              <span class="label">Recommended for this profile</span>

              <strong>
                {latestCompatibleVersion.version_number}
              </strong>
            </div>

            <span class="version-status success"> Compatible </span>
          </div>

          <div class="version-card-meta">
            <span>
              {formatLoaders(latestCompatibleVersion)}
            </span>

            <span>
              {formatMinecraftVersions(latestCompatibleVersion)}
            </span>

            <span>
              {formatDate(latestCompatibleVersion.date_published)}
            </span>
          </div>
        </div>
      {/if}

      {#if installedMatchesProfile && !updateAvailable && !checkingUpdate}
        <div class="up-to-date">
          <span>✓</span>
          <span> Installed version is the latest compatible release for this profile. </span>
        </div>
      {/if}

      {#if !installedProject && latestVersion && latestCompatibleVersion && latestVersion.id !== latestCompatibleVersion.id}
        <div class="version-note">The newest project release is not compatible with this profile.</div>
      {/if}
    </section>

    <!-- Gallery -->
    {#if gallery.length}
      <section>
        <div class="section-heading">
          <h3>Gallery</h3>

          <span class="gallery-count">
            {gallery.length}
            {gallery.length === 1 ? "image" : "images"}
          </span>
        </div>

        <div class="gallery-preview">
          {#each gallery.slice(0, 6) as image, index (image)}
            <img src={image} alt={`${project.title} screenshot ${index + 1}`} loading="lazy" />
          {/each}
        </div>
      </section>
    {/if}

    <!-- Additional information -->
    <section>
      <div class="section-heading">
        <h3>Additional information</h3>
      </div>

      <div class="information-grid">
        <div>
          <span class="label">License</span>
          <strong>
            {project.license || "Unknown"}
          </strong>
        </div>

        <div>
          <span class="label">Project type</span>
          <strong>
            {formatProjectType(project.project_type)}
          </strong>
        </div>

        <div>
          <span class="label">Created</span>
          <strong>
            {formatDateTime(project.date_created)}
          </strong>
        </div>

        <div>
          <span class="label">Last updated</span>
          <strong>
            {formatDateTime(project.date_modified)}
          </strong>
        </div>
      </div>
    </section>
  </div>

  <footer class="actions">
    <button type="button" class="secondary" onclick={openModrinth}> Open Modrinth </button>

    {#if installedProject}
      <button type="button" class="secondary" onclick={showVersions}> View versions </button>

      {#if updateAvailable}
        <button type="button" class="primary" disabled={!profile || installing || checkingUpdate || !latestCompatibleVersion} onclick={install}>
          {installing ? "Updating..." : "Update"}
        </button>
      {/if}
    {:else}
      <button type="button" class="primary" disabled={!profile || !latestCompatibleVersion || installing || checkingUpdate} onclick={install}>
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
  border: 1px solid $color-border;
  border-radius: $radius-xl;
  background: $color-surface-1;
  color: $color-text;
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
  flex-wrap: wrap;
  gap: $space-sm;

  h2 {
    margin: 0;
    font-size: 1.35rem;
    font-weight: 650;
  }
}

.author {
  margin-top: $space-xs;
  color: $color-text-muted;
  font-size: 0.8rem;

  strong {
    color: $color-accent;
    font-weight: 600;
  }
}

.project-type {
  padding: $space-1 $space-sm;
  border-radius: $radius-round;
  background: $color-surface-3;
  color: $color-accent;
  font-size: 0.62rem;
  font-weight: 600;
}

.reputation {
  display: flex;
  flex-wrap: wrap;
  gap: $space-lg;
  margin-top: $space-md;
  color: $color-text-muted;
  font-size: 0.68rem;

  strong {
    color: $color-text;
    font-weight: 600;
  }
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
    font-size: 0.82rem;
    font-weight: 650;
  }
}

.section-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: $space-md;
}

.profile-label,
.version-count,
.gallery-count {
  color: $color-text-muted;
  font-size: 0.65rem;
}

.section-state {
  font-size: 0.65rem;
  font-weight: 600;

  &.success {
    color: $color-success;
  }

  &.error {
    color: $color-error;
  }
}

.label {
  color: $color-text-muted;
  font-size: 0.6rem;
  letter-spacing: 0.04em;
  text-transform: uppercase;
}

.description {
  margin: 0;
  color: $color-text-muted;
  font-size: 0.82rem;
  line-height: 1.5;
}

.status-card {
  display: flex;
  flex-direction: column;
  gap: $space-lg;
  padding: $space-lg;
  border: 1px solid $color-border;
  border-radius: $radius-lg;
  background: $color-surface-2;

  &.compatible {
    border-color: rgba(34, 197, 94, 0.45);
  }

  &.incompatible {
    border-color: rgba(239, 68, 68, 0.45);
  }

  &.not-installed {
    border-color: rgba(59, 130, 246, 0.35);
  }

  p {
    margin: 0;
    color: $color-text-muted;
    font-size: 0.72rem;
    line-height: 1.45;
  }
}

.status-main {
  display: flex;
  align-items: center;
  gap: $space-md;
}

.status-icon {
  display: flex;
  flex: 0 0 auto;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border-radius: 50%;
  background: $color-surface-3;
  font-size: 0.85rem;
  font-weight: 700;
}

.compatible .status-icon {
  background: rgba(34, 197, 94, 0.12);
  color: $color-success;
}

.incompatible .status-icon {
  background: rgba(239, 68, 68, 0.12);
  color: $color-error;
}

.not-installed .status-icon {
  background: rgba(59, 130, 246, 0.12);
  color: $color-accent;
}

.status-copy {
  display: flex;
  flex-direction: column;
  gap: $space-1;
  min-width: 0;

  strong {
    font-size: 0.85rem;
    font-weight: 650;
  }

  span {
    color: $color-text-muted;
    font-size: 0.7rem;
  }
}

.status-details {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: $space-md;

  > div {
    display: flex;
    flex-direction: column;
    gap: $space-xs;
    min-width: 0;
  }

  strong {
    overflow-wrap: anywhere;
    font-size: 0.75rem;
    font-weight: 600;
  }

  small {
    color: $color-text-muted;
    font-size: 0.62rem;
    line-height: 1.35;
  }

  .match {
    color: $color-success;
  }

  .mismatch {
    color: $color-error;
  }
}

.next-version {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: $space-lg;
  padding: $space-md $space-lg;
  border-left: 2px solid $color-warning;
  border-radius: $radius-md;
  background: $color-surface-2;

  > div {
    display: flex;
    flex-direction: column;
    gap: $space-xs;
    min-width: 0;
  }

  strong {
    font-size: 0.82rem;
  }

  > div > span:last-child {
    color: $color-text-muted;
    font-size: 0.65rem;
  }

  &.checking {
    border-left-color: $color-accent;

    strong {
      color: $color-text-muted;
    }
  }
}

.update-label {
  flex: 0 0 auto;
  color: $color-warning;
  font-size: 0.65rem;
  font-weight: 600;
}

.up-to-date {
  display: flex;
  align-items: center;
  gap: $space-sm;
  color: $color-success;
  font-size: 0.7rem;
}

.profile-target {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: $space-md;
  padding: $space-md;
  border: 1px solid $color-border;
  border-radius: $radius-md;
  background: $color-surface-2;

  > div {
    display: flex;
    flex-direction: column;
    gap: $space-xs;
    min-width: 0;
  }

  strong {
    overflow-wrap: anywhere;
    font-size: 0.8rem;
  }

  strong.success {
    color: $color-success;
  }

  strong.error {
    color: $color-error;
  }
}

.compatibility-row {
  display: flex;
  flex-direction: column;
  gap: $space-sm;
}

.tags {
  display: flex;
  flex-wrap: wrap;
  gap: $space-xs;
}

.tag {
  display: inline-flex;
  align-items: center;
  gap: $space-xs;
  padding: $space-1 $space-sm;
  border: 1px solid $color-border-muted;
  border-radius: $radius-round;
  background: $color-surface-2;
  color: $color-text-muted;
  font-size: 0.68rem;

  &.success {
    border-color: rgba(34, 197, 94, 0.5);
    background: rgba(34, 197, 94, 0.1);
    color: $color-success;
  }

  &.warning {
    border-color: rgba(245, 158, 11, 0.5);
    background: rgba(245, 158, 11, 0.1);
    color: $color-warning;
  }

  &.error {
    border-color: rgba(239, 68, 68, 0.5);
    background: rgba(239, 68, 68, 0.1);
    color: $color-error;
  }

  &.profile-version {
    border-color: rgba(34, 197, 94, 0.45);
    background: rgba(34, 197, 94, 0.08);
    color: $color-success;
    font-weight: 600;
  }
}

.tag-note {
  font-size: 0.52rem;
  font-weight: 600;
  text-transform: uppercase;
}

.support-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: $space-md;

  > div {
    display: flex;
    flex-direction: column;
    gap: $space-xs;
    padding: $space-md;
    border-radius: $radius-md;
    background: $color-surface-2;
  }

  strong {
    font-size: 0.75rem;
    font-weight: 600;
  }
}

.version-card {
  display: flex;
  flex-direction: column;
  gap: $space-md;
  padding: $space-md $space-lg;
  border: 1px solid $color-border;
  border-radius: $radius-md;
  background: $color-surface-2;

  &.installed {
    border-left: 2px solid $color-accent;
  }

  &.update {
    border-left: 2px solid $color-warning;
  }

  &.recommended {
    border-left: 2px solid $color-success;
  }
}

.version-card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: $space-md;

  > div {
    display: flex;
    flex-direction: column;
    gap: $space-xs;
  }

  strong {
    font-size: 0.9rem;
    font-weight: 650;
  }
}

.version-status {
  flex: 0 0 auto;
  font-size: 0.62rem;
  font-weight: 600;

  &.success {
    color: $color-success;
  }

  &.warning {
    color: $color-warning;
  }

  &.error {
    color: $color-error;
  }
}

.version-card-meta {
  display: flex;
  flex-wrap: wrap;
  gap: $space-md;
  color: $color-text-muted;
  font-size: 0.65rem;
}

.version-note {
  color: $color-text-muted;
  font-size: 0.68rem;
  line-height: 1.4;
}

.category-area {
  display: flex;
  flex-direction: column;
  gap: $space-sm;
}

.gallery-preview {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: $space-sm;

  img {
    display: block;
    width: 100%;
    aspect-ratio: 16 / 9;
    border-radius: $radius-md;
    background: $color-surface-2;
    object-fit: cover;
  }
}

.information-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: $space-md;

  > div {
    display: flex;
    flex-direction: column;
    gap: $space-xs;
    padding: $space-md;
    border-radius: $radius-md;
    background: $color-surface-2;
  }

  strong {
    font-size: 0.75rem;
    font-weight: 600;
  }
}

.actions {
  display: flex;
  justify-content: flex-end;
  flex-wrap: wrap;
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
    border-color: $color-accent-hover !important;
    background: $color-accent-hover;
  }
}

.secondary {
  border-color: $color-border !important;
  background: $color-surface-2;
  color: $color-text;

  &:hover:not(:disabled) {
    background: $color-surface-3;
  }
}

@media (max-width: 600px) {
  .project-modal {
    width: 95vw;
    max-height: 90vh;
  }

  .hero {
    padding: $space-lg;
  }

  .content {
    padding: $space-lg;
  }

  .status-details,
  .profile-target,
  .support-grid,
  .information-grid {
    grid-template-columns: 1fr;
  }

  .gallery-preview {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .next-version {
    align-items: flex-start;
    flex-direction: column;
  }

  .actions {
    padding: $space-md $space-lg;
  }

  .actions button {
    flex: 1 1 auto;
  }
}
</style>
