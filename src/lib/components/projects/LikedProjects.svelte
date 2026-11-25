<!--
@component

Displays globally liked projects for the selected project type.

Provides:
- Loading and error handling
- Debounced fuzzy search by project name and description
- Sorting by installation status, name, downloads, and followers
- Project details and installation through ProjectCard
- Installed project identification
- Removing projects from the liked collection
-->

<script lang="ts">
import { type KableProfile, type KableProject, type Project, type ProjectType, app, debug, error, Icon } from "#lib";
import { SvelteSet } from "svelte/reactivity";

import ProjectCard from "./ProjectCard.svelte";

type SortMode = "installation" | "name-asc" | "name-desc" | "downloads" | "followers";

let {
  profile,
  profileId,
  projectType,
  installedProjects,
}: {
  profile: KableProfile | null;
  profileId: string | null;
  projectType: ProjectType;
  installedProjects: KableProject[];
} = $props();

let likedProjects = $state<Project[]>([]);
let loading = $state(true);
let loadError = $state<string | null>(null);
// If we don't use state the unliking projects don't get updated.
// eslint-disable-next-line svelte/no-unnecessary-state-wrap
let unlikingProjects = $state<SvelteSet<string>>(new SvelteSet());

let searchQuery = $state("");
let debouncedSearchQuery = $state("");
let sortMode = $state<SortMode>("installation");

let installedProjectIds = $derived(new Set(installedProjects.map((project) => `${project.project.project_type}:${project.project.project_id}`)));

let visibleProjects = $derived.by(() => {
  const query = debouncedSearchQuery.trim().toLowerCase();

  let projects = likedProjects.filter((project) => project.project_type === projectType);

  if (query) {
    const terms = query.split(/\s+/).filter(Boolean);

    projects = projects.filter((project) => {
      const title = project.title.toLowerCase();
      const description = (project.description ?? "").toLowerCase();

      return terms.every((term) => fuzzyMatch(title, term) || fuzzyMatch(description, term));
    });
  }

  projects = [...projects];

  switch (sortMode) {
    case "installation":
      projects.sort((a, b) => {
        const aInstalled = isInstalled(a);
        const bInstalled = isInstalled(b);

        if (aInstalled !== bInstalled) {
          return Number(aInstalled) - Number(bInstalled);
        }

        return compareNames(a, b);
      });
      break;

    case "name-asc":
      projects.sort(compareNames);
      break;

    case "name-desc":
      projects.sort((a, b) => compareNames(b, a));
      break;

    case "downloads":
      projects.sort((a, b) => b.downloads - a.downloads);
      break;

    case "followers":
      projects.sort((a, b) => b.follows - a.follows);
      break;
  }

  return projects;
});

let loadGeneration = 0;

function projectKey(project: Project): string {
  return `${project.project_type}:${project.project_id}`;
}

function isInstalled(project: Project): boolean {
  return installedProjectIds.has(projectKey(project));
}

function compareNames(a: Project, b: Project): number {
  return a.title.localeCompare(b.title, undefined, {
    sensitivity: "base",
    numeric: true,
  });
}

function fuzzyMatch(text: string, query: string): boolean {
  if (text.includes(query)) {
    return true;
  }

  let queryIndex = 0;

  for (const character of text) {
    if (character === query[queryIndex]) {
      queryIndex++;

      if (queryIndex === query.length) {
        return true;
      }
    }
  }

  return false;
}

async function loadLikedProjects(): Promise<void> {
  const generation = ++loadGeneration;

  loading = true;
  loadError = null;

  try {
    const projects = await app.projectsService.listLikedProjects();

    if (generation !== loadGeneration) {
      return;
    }

    likedProjects = projects;

    debug("[LikedProjects] Loaded liked projects:", projects.length);
  } catch (err) {
    if (generation !== loadGeneration) {
      return;
    }

    loadError = err instanceof Error ? err.message : String(err);

    error("[LikedProjects] Failed to load liked projects:", err);
  } finally {
    if (generation === loadGeneration) {
      loading = false;
    }
  }
}

async function unlikeProject(project: Project): Promise<void> {
  const key = projectKey(project);

  if (unlikingProjects.has(key)) {
    return;
  }

  unlikingProjects.add(key);

  try {
    await app.projectsService.unlikeProject(project.project_id, project.project_type);

    likedProjects = likedProjects.filter((liked) => projectKey(liked) !== key);

    debug("[LikedProjects] Removed liked project:", project.project_id);
  } catch (err) {
    error("[LikedProjects] Failed to remove liked project:", project.project_id, err);
  } finally {
    unlikingProjects.delete(key);
  }
}

$effect(() => {
  projectType;
  profileId;

  void loadLikedProjects();
});

$effect(() => {
  const query = searchQuery.trim();

  const timeout = setTimeout(() => {
    debouncedSearchQuery = query;
  }, 250);

  return () => clearTimeout(timeout);
});
</script>

<div class="liked-projects">
  <div class="toolbar">
    <div class="search-container">
      <Icon name="search" forceType="svg" size="sm" />

      <input type="text" bind:value={searchQuery} placeholder="Search liked projects..." aria-label="Search liked projects" />

      {#if searchQuery}
        <button class="clear-search" type="button" onclick={() => (searchQuery = "")} aria-label="Clear search" title="Clear search">
          <Icon name="x" forceType="svg" size="sm" />
        </button>
      {/if}
    </div>

    <div class="sort-container">
      <label for="liked-sort">Sort by</label>

      <select id="liked-sort" bind:value={sortMode}>
        <option value="installation">Installation status</option>
        <option value="name-asc">Name (A-Z)</option>
        <option value="name-desc">Name (Z-A)</option>
        <option value="downloads">Most downloads</option>
        <option value="followers">Most followers</option>
      </select>
    </div>

    <button
      class="refresh-btn"
      type="button"
      onclick={loadLikedProjects}
      disabled={loading}
      class:loading
      aria-label="Refresh liked projects"
      title="Refresh liked projects"
    >
      <Icon name="refresh" forceType="svg" size="sm" />
    </button>
  </div>

  <div class="results-info">
    {#if !loading && !loadError}
      <span>
        {visibleProjects.length} of {likedProjects.filter((project) => project.project_type === projectType).length} liked projects
      </span>
    {:else}
      <span>Your saved projects</span>
    {/if}
  </div>

  {#if loading}
    <div class="state-message">
      <Icon name="loader-circle" forceType="svg" />
      <span>Loading liked projects...</span>
    </div>
  {:else if loadError}
    <div class="state-message error">
      <span>Failed to load liked projects.</span>
      <span class="state-description">{loadError}</span>

      <button class="retry-btn" type="button" onclick={loadLikedProjects}> Retry </button>
    </div>
  {:else if visibleProjects.length === 0}
    <div class="state-message">
      <Icon name="heart" forceType="svg" />

      {#if debouncedSearchQuery}
        <span>No matching projects</span>
        <span class="state-description"> Try changing your search query. </span>
      {:else}
        <span>No liked projects</span>
        <span class="state-description"> Projects you like while browsing will appear here. </span>
      {/if}
    </div>
  {:else}
    <div class="projects-grid">
      {#each visibleProjects as project (projectKey(project))}
        <div class="liked-project">
          <ProjectCard {profile} {project} />

          <div class="project-actions">
            {#if isInstalled(project)}
              <span class="installed-label">
                <Icon name="check" forceType="svg" size="sm" />
                Installed
              </span>
            {:else if profile}
              <span class="not-installed-label"> Not installed </span>
            {:else}
              <span class="not-installed-label"> Select a profile to install </span>
            {/if}

            <button
              class="unlike-btn"
              type="button"
              onclick={() => unlikeProject(project)}
              disabled={unlikingProjects.has(projectKey(project))}
              title="Remove from liked projects"
              aria-label={`Remove ${project.title} from liked projects`}
            >
              <Icon name="delete" forceType="svg" size="sm" />
              <span>Remove</span>
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style lang="scss">
.liked-projects {
  display: flex;
  flex-direction: column;
  width: 100%;
  min-width: 0;
  min-height: 0;
  gap: $space-md;
}

.toolbar {
  display: grid;
  grid-template-columns: minmax(180px, 1fr) auto auto;
  align-items: center;
  gap: $space-md;
  width: 100%;
}

.search-container {
  display: flex;
  align-items: center;
  min-width: 0;
  gap: $space-sm;
  padding: 0 $space-md;
  border: 1px solid $color-border;
  border-radius: $radius-md;
  background: $color-surface-2;
  color: $color-text-muted;

  &:focus-within {
    border-color: $color-focus;
  }

  input {
    width: 100%;
    min-width: 0;
    padding: $space-sm 0;
    border: none;
    outline: none;
    background: transparent;
    color: $color-text;
    font: inherit;
    font-size: 0.85rem;

    &::placeholder {
      color: $color-text-muted;
    }
  }
}

.clear-search {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex: 0 0 auto;
  padding: $space-xs;
  border: none;
  border-radius: $radius-sm;
  background: transparent;
  color: $color-text-muted;
  cursor: pointer;

  &:hover {
    background: $color-hover;
    color: $color-text;
  }
}

.sort-container {
  display: flex;
  align-items: center;
  gap: $space-sm;
  white-space: nowrap;

  label {
    color: $color-text-muted;
    font-size: 0.8rem;
  }

  select {
    min-width: 160px;
    padding: $space-sm $space-md;
    border: 1px solid $color-border;
    border-radius: $radius-md;
    background: $color-surface-2;
    color: $color-text;
    font: inherit;
    font-size: 0.8rem;
    cursor: pointer;

    &:focus-visible {
      outline: 2px solid $color-focus;
      outline-offset: 2px;
    }
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
  cursor: pointer;

  &:hover:not(:disabled) {
    border-color: $color-border;
    background: $color-hover;
    color: $color-text;
  }

  &:disabled {
    opacity: 0.6;
    cursor: wait;
  }

  &.loading {
    color: $color-accent;
    animation: spin 1s linear infinite;
  }

  &:focus-visible {
    outline: 2px solid $color-focus;
    outline-offset: 2px;
  }
}

.results-info {
  display: flex;
  align-items: center;
  color: $color-text-muted;
  font-size: 0.75rem;
}

.projects-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  grid-auto-rows: 1fr;
  align-items: stretch;
  gap: $space-lg;
}

.liked-project {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-width: 0;
  gap: $space-sm;

  :global(.project-card) {
    flex: 1 1 auto;
    min-height: 0;
  }
}

.project-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex: 0 0 auto;
  min-height: 32px;
  gap: $space-sm;
  padding: 0 $space-xs;
}

.installed-label,
.not-installed-label {
  display: inline-flex;
  align-items: center;
  gap: $space-xs;
  color: $color-text-muted;
  font-size: 0.75rem;
}

.installed-label {
  color: $color-success;
}

.unlike-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex: 0 0 auto;
  gap: $space-xs;
  padding: $space-xs $space-sm;
  border: 1px solid transparent;
  border-radius: $radius-md;
  background: transparent;
  color: $color-text-muted;
  font: inherit;
  font-size: 0.75rem;
  cursor: pointer;

  &:hover:not(:disabled) {
    border-color: $color-border;
    background: $color-hover;
    color: $color-error;
  }

  &:disabled {
    opacity: 0.6;
    cursor: wait;
  }

  &:focus-visible {
    outline: 2px solid $color-focus;
    outline-offset: 2px;
  }
}

.state-message {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: $space-sm;
  min-height: 240px;
  padding: $space-xl;
  color: $color-text-muted;
  text-align: center;

  > :first-child {
    color: $color-text-muted;
  }

  &.error {
    color: $color-error;
  }
}

.state-description {
  max-width: 420px;
  color: $color-text-muted;
  font-size: 0.8rem;
  overflow-wrap: anywhere;
}

.retry-btn {
  margin-top: $space-sm;
  padding: $space-sm $space-md;
  border: 1px solid $color-border;
  border-radius: $radius-md;
  background: $color-surface-2;
  color: $color-text;
  font: inherit;
  font-size: 0.8rem;
  cursor: pointer;

  &:hover {
    background: $color-hover;
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

@media (max-width: 640px) {
  .toolbar {
    grid-template-columns: minmax(0, 1fr) auto;
  }

  .search-container {
    grid-column: 1 / -1;
  }

  .sort-container {
    min-width: 0;

    select {
      min-width: 0;
      max-width: 100%;
    }
  }
}
</style>
