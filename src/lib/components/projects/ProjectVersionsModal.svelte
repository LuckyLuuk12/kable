<!--
@component

Project version browser and installer.

Supports both:
- Project: a Modrinth project that can be installed into a profile.
- KableProject: an already-installed project that can be updated.

The modal always treats the selected version ID as authoritative and refreshes
the profile/project state after an install or update.
-->

<script lang="ts">
import { app, type KableProfile, type KableProject, type Project, type ProjectVersion } from "$lib";

let {
  profile = null,
  project,
}: {
  profile?: KableProfile | null;
  project: KableProject | Project;
} = $props();

let loading = $state(false);
let search = $state("");
let showFilters = $state(false);
let filterCompatibility = $state(true);
let expandedChangelog = $state<string | null>(null);
let selectedVersionId = $state<string | null>(null);

function isKableProject(value: KableProject | Project): value is KableProject {
  return "version_id" in value;
}

let projectData = $derived(isKableProject(project) ? project.project : project);

let installedVersionId = $derived(isKableProject(project) ? project.version_id : null);

let installedVersion = $derived(installedVersionId ? (projectData.versions.find((version) => version.id === installedVersionId) ?? null) : null);

let selectedVersion = $derived(selectedVersionId ? (projectData.versions.find((version) => version.id === selectedVersionId) ?? null) : null);

function normalize(value: string | null | undefined): string {
  return (value ?? "").trim().toLowerCase();
}

function fuzzyMatch(query: string, value: string): boolean {
  const normalizedQuery = normalize(query);
  const normalizedValue = normalize(value);

  if (!normalizedQuery) {
    return true;
  }

  if (!normalizedValue) {
    return false;
  }

  if (normalizedValue.includes(normalizedQuery)) {
    return true;
  }

  let queryIndex = 0;

  for (const character of normalizedValue) {
    if (character === normalizedQuery[queryIndex]) {
      queryIndex++;

      if (queryIndex >= normalizedQuery.length) {
        return true;
      }
    }
  }

  return false;
}

function versionMatchesSearch(version: ProjectVersion): boolean {
  if (!search.trim()) {
    return true;
  }

  return fuzzyMatch(search, version.name) || fuzzyMatch(search, version.version_number) || fuzzyMatch(search, version.id) || fuzzyMatch(search, version.changelog ?? "");
}

function versionMatchesCompatibility(version: ProjectVersion): boolean {
  if (!filterCompatibility || !profile) {
    return true;
  }

  return app.projectsService.isVersionCompatible(profile, version, projectData.project_type);
}

function versionMatchesFilters(version: ProjectVersion): boolean {
  return versionMatchesSearch(version) && versionMatchesCompatibility(version);
}

let sortedVersions = $derived(
  [...projectData.versions].filter(versionMatchesFilters).sort((a, b) => new Date(b.date_published).getTime() - new Date(a.date_published).getTime()),
);

let filterDescription = $derived.by(() => {
  if (!filterCompatibility) {
    return "All versions";
  }

  if (!profile) {
    return "No profile selected";
  }

  return "Compatible with profile";
});

function formatDate(date: string): string {
  return new Date(date).toLocaleDateString();
}

function formatDownloads(downloads: number): string {
  return downloads.toLocaleString();
}

function selectVersion(version: ProjectVersion): void {
  selectedVersionId = version.id;

  if (expandedChangelog !== version.id) {
    expandedChangelog = null;
  }
}

function toggleChangelog(version: ProjectVersion): void {
  expandedChangelog = expandedChangelog === version.id ? null : version.id;
}

function clearSearch(): void {
  search = "";
}

function isVersionCompatible(version: ProjectVersion): boolean {
  if (!profile) {
    return false;
  }

  return app.projectsService.isVersionCompatible(profile, version, projectData.project_type);
}

/**
 * Returns the relationship between the installed version and the selected
 * version based on publication date.
 *
 * This is only used for UI wording. The backend receives the exact version
 * ID and decides whether the requested version is valid for the profile.
 */
function getVersionChangeType(installed: ProjectVersion | null, selected: ProjectVersion | null): "same" | "update" | "downgrade" | "change" {
  if (!installed || !selected) {
    return "change";
  }

  if (installed.id === selected.id) {
    return "same";
  }

  const installedDate = new Date(installed.date_published).getTime();
  const selectedDate = new Date(selected.date_published).getTime();

  if (Number.isFinite(installedDate) && Number.isFinite(selectedDate)) {
    if (selectedDate > installedDate) {
      return "update";
    }

    if (selectedDate < installedDate) {
      return "downgrade";
    }
  }

  return "change";
}

let selectedVersionChange = $derived(isKableProject(project) ? getVersionChangeType(installedVersion, selectedVersion) : null);

let selectedVersionActionLabel = $derived.by(() => {
  if (!isKableProject(project)) {
    return "Install this version";
  }

  switch (selectedVersionChange) {
    case "same":
      return "Installed";

    case "update":
      return "Update to this version";

    case "downgrade":
      return "Downgrade to this version";

    default:
      return "Switch to this version";
  }
});

async function installVersion(version: ProjectVersion): Promise<void> {
  if (!profile || loading) {
    return;
  }

  if (!isVersionCompatible(version)) {
    console.warn("[ProjectVersionsModal] Refusing to install incompatible version", {
      project: projectData.project_id,
      version: version.id,
      profile: profile.id,
    });

    return;
  }

  if (isKableProject(project) && version.id === project.version_id) {
    return;
  }

  loading = true;

  try {
    /*
     * A KableProject already exists in the profile, so changing its version
     * must always go through update_project.
     *
     * Passing Some(version.id) makes this an explicit version change.
     * The backend can therefore perform both upgrades and downgrades.
     */
    if (isKableProject(project)) {
      await app.projectsService.update(profile, project, version.id);
    } else {
      await app.projectsService.download(profile, project, version.id);
    }

    selectedVersionId = version.id;
  } finally {
    loading = false;
  }
}

/*
 * Keep the selected version valid when the project/profile/filter state
 * changes.
 *
 * Priority:
 * 1. Keep the explicit user selection if it is visible.
 * 2. Keep the installed version if it is visible.
 * 3. Select the newest visible version.
 */
$effect(() => {
  const visibleVersionIds = new Set(sortedVersions.map((version) => version.id));

  if (selectedVersionId !== null && visibleVersionIds.has(selectedVersionId)) {
    return;
  }

  if (installedVersionId !== null && visibleVersionIds.has(installedVersionId)) {
    selectedVersionId = installedVersionId;
    return;
  }

  selectedVersionId = sortedVersions[0]?.id ?? null;
});
</script>

<div class="project-versions-modal">
  <header class="header">
    <div class="header-main">
      <div class="header-title">
        <h2>Versions</h2>
        <p>{projectData.title}</p>
      </div>

      {#if isKableProject(project)}
        <span class="installed">
          Installed:
          {projectData.versions.find((version) => version.id === project.version_id)?.version_number ?? project.version_id}
        </span>
      {:else}
        <span class="available">
          {projectData.versions.length.toLocaleString()} versions
        </span>
      {/if}
    </div>

    <div class="toolbar">
      <div class="search-wrapper">
        <input class="search" type="search" bind:value={search} placeholder="Search versions..." aria-label="Search versions" />

        {#if search}
          <button class="clear-search" type="button" aria-label="Clear search" onclick={clearSearch}> × </button>
        {/if}
      </div>

      <button class:active={showFilters} class="filter-button" type="button" onclick={() => (showFilters = !showFilters)}> Filters </button>
    </div>
  </header>

  {#if showFilters}
    <section class="filters">
      <div class="filter-header">
        <div><strong>Version filters</strong> <span>{filterDescription}</span></div>

        <span class="filter-count">
          {sortedVersions.length} / {projectData.versions.length}
        </span>
      </div>

      <div class="filter-options">
        <label class="filter-option">
          <input type="checkbox" bind:checked={filterCompatibility} disabled={!profile} />

          <span>
            <strong>Compatibility</strong>

            <small>
              {#if profile}
                {projectData.project_type === "mod" || projectData.project_type === "modpack"
                  ? `${profile.version.minecraft_version ?? "Any Minecraft version"} · ${profile.version.loader}`
                  : (profile.version.minecraft_version ?? "Any Minecraft version")}
              {:else}
                No profile selected
              {/if}
            </small>
          </span>
        </label>
      </div>
    </section>
  {/if}

  <div class="versions">
    {#if sortedVersions.length === 0}
      <div class="empty">
        {#if projectData.versions.length === 0}
          <strong>No versions available.</strong>
          <span>This project does not contain version data.</span>
        {:else if search}
          <strong>No matching versions.</strong>
          <span>Try a different search query or adjust the filters.</span>
        {:else}
          <strong>No compatible versions.</strong>
          <span>Try disabling one or more filters.</span>
        {/if}
      </div>
    {:else}
      {#each sortedVersions as version (version.id)}
        {@const isInstalled = isKableProject(project) && project.version_id === version.id}

        {@const isSelected = selectedVersionId === version.id}

        {@const changelogOpen = expandedChangelog === version.id}

        {@const isCompatible = !profile || isVersionCompatible(version)}

        <article class:installed={isInstalled} class:selected={isSelected} class="version-card">
          <button class="version" type="button" onclick={() => selectVersion(version)}>
            <div class="version-main">
              <div class="version-title">
                <strong>{version.name}</strong>

                {#if version.featured}
                  <span class="featured">Featured</span>
                {/if}

                {#if isInstalled}
                  <span class="current">Installed</span>
                {/if}

                {#if profile && !isCompatible}
                  <span class="incompatible">Incompatible</span>
                {/if}
              </div>

              <span class="version-number">
                {version.version_number}
              </span>
            </div>

            <div class="version-meta">
              <span>{formatDate(version.date_published)}</span>
              <span>•</span>
              <span>
                {formatDownloads(version.downloads)} downloads
              </span>

              {#if version.loaders.length}
                <span>•</span>
                <span>{version.loaders.join(", ")}</span>
              {/if}

              {#if version.game_versions.length}
                <span>•</span>
                <span>
                  {version.game_versions.length === 1 ? version.game_versions[0] : `${version.game_versions.length} Minecraft versions`}
                </span>
              {/if}
            </div>
          </button>

          {#if version.changelog}
            <button class:open={changelogOpen} class="changelog-toggle" type="button" onclick={() => toggleChangelog(version)}>
              <span>Changelog</span>
              <span class="chevron">
                {changelogOpen ? "▴" : "▾"}
              </span>
            </button>

            {#if changelogOpen}
              <div class="changelog">
                <p>{version.changelog}</p>
              </div>
            {/if}
          {/if}
        </article>
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

        <div class="details-badges">
          <span class="release-badge">
            {selectedVersion.version_type}
          </span>

          {#if isKableProject(project) && selectedVersion.id === project.version_id}
            <span class="current-badge"> Current version </span>
          {/if}

          {#if profile && !isVersionCompatible(selectedVersion)}
            <span class="incompatible-badge"> Incompatible </span>
          {/if}
        </div>
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

        <div>
          <span class="label">Downloads</span>
          <span>{formatDownloads(selectedVersion.downloads)}</span>
        </div>

        <div>
          <span class="label">Files</span>
          <span>{selectedVersion.files.length}</span>
        </div>
      </div>

      {#if selectedVersion.changelog}
        <div class="details-changelog">
          <h4>Changelog</h4>
          <p>{selectedVersion.changelog}</p>
        </div>
      {/if}

      {#if profile}
        <footer class="actions">
          <button
            class="install"
            type="button"
            disabled={loading || !isVersionCompatible(selectedVersion) || (isKableProject(project) && selectedVersion.id === project.version_id)}
            onclick={() => installVersion(selectedVersion)}>
            {#if loading}
              {isKableProject(project) ? "Updating..." : "Installing..."}
            {:else if isKableProject(project) && selectedVersion.id === project.version_id}
              Installed
            {:else if !isVersionCompatible(selectedVersion)}
              Incompatible with profile
            {:else}
              {selectedVersionActionLabel}
            {/if}
          </button>
        </footer>
      {:else}
        <footer class="no-profile">Select a profile to install this version.</footer>
      {/if}
    </section>
  {/if}
</div>

<style lang="scss">
.project-versions-modal {
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

.header {
  display: flex;
  flex-direction: column;
  gap: $space-md;

  flex: 0 0 auto;

  padding: $space-lg;

  border-bottom: 2px solid $color-border;

  background: $color-surface-1;
}

.header-main {
  display: flex;
  align-items: center;
  justify-content: space-between;

  gap: $space-lg;
}

.header-title {
  min-width: 0;

  h2 {
    margin: 0;

    font-size: 1.15rem;
    font-weight: 600;
  }

  p {
    margin: $space-xs 0 0;

    overflow: hidden;

    color: $color-text-muted;
    font-size: 0.8rem;

    text-overflow: ellipsis;
    white-space: nowrap;
  }
}

.installed,
.available {
  flex: 0 0 auto;

  padding: $space-1 $space-sm;

  border: 1px solid $color-border-muted;
  border-radius: $radius-round;

  background: $color-surface-2;
  color: $color-text-muted;

  font-size: 0.68rem;
  font-weight: 500;
}

/* Search */

.toolbar {
  display: flex;
  gap: $space-sm;
}

.search-wrapper {
  position: relative;

  flex: 1;
  min-width: 0;
}

.search {
  box-sizing: border-box;

  width: 100%;

  padding: $space-sm $space-md;
  padding-right: 2.25rem;

  border: 1px solid $color-border;
  border-radius: $radius-md;

  outline: none;

  background: $color-surface-2;
  color: $color-text;

  font: inherit;
  font-size: 0.78rem;

  transition:
    border-color 120ms ease,
    background-color 120ms ease;

  &:hover {
    background: $color-surface-3;
  }

  &:focus {
    border-color: $color-accent;
    background: $color-surface-2;
  }
}

.clear-search {
  position: absolute;

  top: 50%;
  right: $space-xs;

  display: flex;
  align-items: center;
  justify-content: center;

  width: 24px;
  height: 24px;

  transform: translateY(-50%);

  border: 0;
  border-radius: $radius-round;

  background: transparent;
  color: $color-text-muted;

  cursor: pointer;

  &:hover {
    background: $color-hover;
    color: $color-text;
  }
}

.filter-button {
  flex: 0 0 auto;

  padding: $space-sm $space-md;

  border: 1px solid $color-border;
  border-radius: $radius-md;

  background: $color-surface-2;
  color: $color-text-muted;

  font: inherit;
  font-size: 0.75rem;

  cursor: pointer;

  transition:
    background-color 120ms ease,
    border-color 120ms ease,
    color 120ms ease;

  &:hover,
  &.active {
    border-color: $color-accent;

    background: $color-selected;
    color: $color-text;
  }
}

/* Compact filters */

.filters {
  display: flex;
  flex-direction: column;
  gap: $space-sm;

  flex: 0 0 auto;

  padding: $space-md $space-lg;

  border-bottom: 2px solid $color-border;

  background: $color-surface-0;
}

.filter-header {
  display: flex;
  align-items: center;
  justify-content: space-between;

  gap: $space-md;

  > div {
    display: flex;
    align-items: baseline;
    gap: $space-sm;

    min-width: 0;

    strong {
      font-size: 0.72rem;
      font-weight: 600;
    }

    span {
      overflow: hidden;

      color: $color-text-muted;
      font-size: 0.65rem;

      text-overflow: ellipsis;
      white-space: nowrap;
    }
  }
}

.filter-count {
  flex: 0 0 auto;

  color: $color-text-muted;
  font-size: 0.65rem;
}

.filter-options {
  display: flex;
  flex-wrap: wrap;
  gap: $space-xs;
}

.filter-option {
  display: inline-flex;
  align-items: center;

  gap: $space-xs;

  padding: $space-xs $space-sm;

  border: 1px solid $color-border-muted;
  border-radius: $radius-md;

  background: $color-surface-1;

  cursor: pointer;

  input {
    width: 13px;
    height: 13px;

    margin: 0;

    accent-color: $color-accent;

    cursor: pointer;
  }

  > span {
    display: flex;
    align-items: center;

    gap: $space-xs;

    min-width: 0;
  }

  strong {
    font-size: 0.68rem;
    font-weight: 600;
    white-space: nowrap;
  }

  small {
    max-width: 180px;

    overflow: hidden;

    color: $color-text-muted;
    font-size: 0.6rem;

    text-overflow: ellipsis;
    white-space: nowrap;
  }

  &:has(input:checked) {
    border-color: color-mix(in srgb, $color-accent 45%, $color-border-muted);

    background: color-mix(in srgb, $color-accent 6%, $color-surface-1);

    strong {
      color: $color-text;
    }
  }

  &:hover {
    border-color: $color-border;
    background: $color-surface-2;
  }
}

/* Version list */

.versions {
  display: flex;
  flex-direction: column;

  gap: $space-sm;

  flex: 1 1 auto;
  min-height: 0;

  overflow-y: auto;

  padding: $space-md $space-lg;

  border-bottom: 2px solid $color-border;

  background: $color-surface-0;
}

.version-card {
  flex: 0 0 auto;

  overflow: hidden;

  border: 1px solid $color-border-muted;
  border-radius: $radius-md;

  background: $color-surface-2;

  transition:
    background-color 120ms ease,
    border-color 120ms ease,
    box-shadow 120ms ease;

  &.selected {
    border-color: $color-accent;

    background: $color-selected;
  }

  &.installed {
    border-color: rgba(34, 197, 94, 0.35);
  }

  &:hover {
    border-color: $color-border;

    background: $color-surface-3;
  }
}

.version {
  display: flex;
  flex-direction: column;

  gap: $space-sm;

  box-sizing: border-box;

  width: 100%;
  min-height: 76px;

  padding: $space-md;

  border: 0;

  background: transparent;
  color: $color-text;

  font: inherit;
  text-align: left;

  cursor: pointer;

  &:focus-visible {
    outline: 2px solid $color-focus;
    outline-offset: -2px;
  }
}

.version-main {
  display: flex;
  align-items: center;
  justify-content: space-between;

  gap: $space-md;

  min-width: 0;
}

.version-title {
  display: flex;
  align-items: center;

  flex-wrap: wrap;

  gap: $space-xs;

  min-width: 0;

  strong {
    overflow: hidden;

    font-size: 0.82rem;
    font-weight: 600;

    text-overflow: ellipsis;
    white-space: nowrap;
  }
}

.version-number {
  flex: 0 0 auto;

  color: $color-text-muted;
  font-size: 0.7rem;
}

.featured,
.current,
.incompatible,
.current-badge,
.incompatible-badge,
.release-badge {
  display: inline-flex;
  align-items: center;

  padding: 2px $space-xs;

  border-radius: $radius-round;

  font-size: 0.58rem;
  font-weight: 600;
  line-height: 1.3;
}

.featured {
  background: rgba(234, 179, 8, 0.12);
  color: $color-warning;
}

.current {
  background: rgba(34, 197, 94, 0.12);
  color: $color-success;
}

.incompatible,
.incompatible-badge {
  background: rgba(239, 68, 68, 0.12);
  color: $color-error;
}

.release-badge {
  background: $color-surface-2;
  color: $color-text-muted;
}

.current-badge {
  background: rgba(34, 197, 94, 0.12);
  color: $color-success;
}

.version-meta {
  display: flex;
  align-items: center;

  flex-wrap: wrap;

  gap: $space-xs;

  color: $color-text-muted;

  font-size: 0.65rem;
  line-height: 1.3;
}

/* Changelog */

.changelog-toggle {
  display: flex;
  align-items: center;
  justify-content: space-between;

  box-sizing: border-box;

  width: 100%;

  padding: $space-xs $space-md;

  border: 0;
  border-top: 1px solid $color-border-muted;

  background: transparent;
  color: $color-text-muted;

  font: inherit;
  font-size: 0.65rem;
  text-align: left;

  cursor: pointer;

  &:hover {
    background: $color-hover;
    color: $color-text;
  }

  &.open {
    color: $color-text;
  }
}

.chevron {
  font-size: 0.6rem;
}

.changelog {
  padding: $space-sm $space-md;

  border-top: 1px solid $color-border-muted;

  background: $color-surface-1;

  p {
    max-height: 160px;

    overflow-y: auto;

    margin: 0;

    color: $color-text-muted;

    font-size: 0.68rem;
    line-height: 1.45;

    white-space: pre-wrap;
  }
}

/* Empty */

.empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;

  flex: 1;

  gap: $space-xs;

  padding: $space-2xl;

  color: $color-text-muted;

  text-align: center;

  strong {
    color: $color-text;
    font-size: 0.8rem;
  }

  span {
    font-size: 0.7rem;
  }
}

/* Details */

.details {
  display: flex;
  flex-direction: column;

  gap: $space-md;

  flex: 0 0 auto;

  max-height: 35vh;

  overflow-y: auto;

  padding: $space-lg;

  background: $color-surface-1;
}

.details-header {
  display: flex;
  align-items: center;
  justify-content: space-between;

  gap: $space-md;

  h3 {
    margin: 0;

    font-size: 0.95rem;
    font-weight: 600;
  }

  > div:first-child > span {
    display: block;

    margin-top: $space-xs;

    color: $color-text-muted;

    font-size: 0.72rem;
  }
}

.details-badges {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;

  gap: $space-xs;
}

.details-grid {
  display: grid;

  grid-template-columns: repeat(3, minmax(0, 1fr));

  gap: $space-sm;

  padding: $space-md;

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

    font-size: 0.62rem;

    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  span:last-child {
    color: $color-text;

    font-size: 0.72rem;

    word-break: break-word;
  }
}

.details-changelog {
  max-height: 140px;

  overflow-y: auto;

  padding: $space-md;

  border-radius: $radius-lg;

  background: $color-surface-2;

  h4 {
    margin: 0 0 $space-sm;

    font-size: 0.72rem;
    font-weight: 600;
  }

  p {
    margin: 0;

    color: $color-text-muted;

    font-size: 0.68rem;
    line-height: 1.45;

    white-space: pre-wrap;
  }
}

/* Actions */

.actions {
  display: flex;
  justify-content: flex-end;

  gap: $space-sm;
}

.install {
  padding: $space-sm $space-lg;

  border: 1px solid $color-accent;
  border-radius: $radius-md;

  background: $color-accent;
  color: $color-text;

  font: inherit;
  font-size: 0.75rem;

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

.no-profile {
  padding: $space-sm;

  border-radius: $radius-md;

  background: $color-surface-2;
  color: $color-text-muted;

  font-size: 0.68rem;
  text-align: center;
}

/* Responsive */

@media (max-width: 720px) {
  .header-main {
    align-items: flex-start;
    flex-direction: column;
  }

  .filter-options {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .details-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 600px) {
  .toolbar {
    flex-direction: column;
  }

  .filter-options {
    grid-template-columns: 1fr;
  }

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
