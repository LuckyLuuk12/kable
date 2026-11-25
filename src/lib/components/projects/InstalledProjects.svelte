<script lang="ts">
import { app, error, Icon, type KableProfile, type KableProject, type ProjectType } from "#lib";
import { untrack } from "svelte";
import InstalledProjectCard from "./InstalledProjectCard.svelte";

type SortOrder = "name-asc" | "name-desc" | "downloads-desc" | "downloads-asc" | "followers-desc" | "followers-asc";

type ProjectMetadata = {
  title?: string;
  name?: string;
  description?: string;
  downloads?: number;
  followers?: number;
  likes?: number;
};

let {
  profile,
  profileId,
  projectType,
  updateKeys,
}: {
  profile: KableProfile | null;
  profileId: string | null;
  projectType: ProjectType;
  updateKeys: ReadonlySet<string>;
} = $props();

let projects = $derived(app.projectsService.getByType(projectType));
let loading = $derived(app.projectsService.loading);

let searchInput = $state("");
let debouncedQuery = $state("");
let sortOrder = $state<SortOrder>("name-asc");

let enabledOpen = $state(true);
let disabledOpen = $state(true);

let pendingById = $state<Record<string, boolean>>({});
let optimisticEnabledById = $state<Record<string, boolean>>({});

const projectId = (project: KableProject): string => project.project.project_id;

const projectKey = (selectedProfile: KableProfile, project: KableProject): string => `${selectedProfile.id}:${project.project.project_type}:${projectId(project)}`;

const updateKey = (project: KableProject): string => `${project.project.project_type}:${project.project.project_id}:${project.filename}`;

const metadataOf = (project: KableProject): ProjectMetadata => project.project as ProjectMetadata;

function projectName(project: KableProject): string {
  const metadata = metadataOf(project);
  return metadata.title ?? metadata.name ?? projectId(project);
}

function projectDescription(project: KableProject): string {
  return metadataOf(project).description ?? "";
}

function normalize(value: string): string {
  return value
    .toLocaleLowerCase()
    .normalize("NFKD")
    .replace(/[\u0300-\u036f]/g, "")
    .replace(/[^a-z0-9]+/g, " ")
    .trim();
}

function fuzzyScore(query: string, value: string): number {
  const q = normalize(query);
  const text = normalize(value);

  if (!q || !text) return 0;
  if (text.includes(q)) return 1000 - text.indexOf(q);

  const words = text.split(/\s+/);

  if (words.some((word) => word.startsWith(q))) return 800;

  let qi = 0;
  let first = -1;
  let last = -1;

  for (let i = 0; i < text.length && qi < q.length; i++) {
    if (text[i] === q[qi]) {
      if (first === -1) first = i;
      last = i;
      qi++;
    }
  }

  if (qi === q.length) {
    return 500 - (last - first + 1 - q.length);
  }

  const maxDistance = q.length >= 5 ? 2 : 1;
  let best = Infinity;

  for (const word of words) {
    if (Math.abs(word.length - q.length) > maxDistance) continue;

    let previous = Array.from({ length: word.length + 1 }, (_, i) => i);

    for (let i = 1; i <= q.length; i++) {
      const current = [i];

      for (let j = 1; j <= word.length; j++) {
        current[j] = Math.min(current[j - 1] + 1, previous[j] + 1, previous[j - 1] + (q[i - 1] === word[j - 1] ? 0 : 1));
      }

      previous = current;
    }

    best = Math.min(best, previous[word.length]);
  }

  return best <= maxDistance ? 300 - best * 10 : 0;
}

function compareNames(a: KableProject, b: KableProject): number {
  return projectName(a).localeCompare(projectName(b), undefined, {
    sensitivity: "base",
    numeric: true,
  });
}

function downloads(project: KableProject): number {
  return metadataOf(project).downloads ?? 0;
}

function followers(project: KableProject): number {
  const metadata = metadataOf(project);
  return metadata.followers ?? metadata.likes ?? 0;
}

function isProjectEnabled(project: KableProject): boolean {
  if (!profile || profile.id !== profileId) {
    return false;
  }

  const key = projectKey(profile, project);
  const optimistic = optimisticEnabledById[key];

  if (optimistic !== undefined) {
    return optimistic;
  }

  return app.projectsService.enabledProjectKeys?.has(key) ?? false;
}

function isTogglePending(project: KableProject): boolean {
  if (!profile) return false;

  return pendingById[projectKey(profile, project)] === true;
}

function hasUpdate(project: KableProject): boolean {
  return updateKeys.has(updateKey(project));
}

$effect(() => {
  const value = searchInput;

  const timer = setTimeout(() => {
    debouncedQuery = value;
  }, 250);

  return () => clearTimeout(timer);
});

$effect(() => {
  const selectedProfile = profile;
  const selectedProfileId = profileId;
  const selectedType = projectType;

  if (!selectedProfile || selectedProfile.id !== selectedProfileId) {
    return;
  }

  untrack(() => {
    void app.projectsService.load(selectedProfile, selectedType).catch((err) => {
      error("[InstalledProjects] Failed to load installed projects", err);
    });
  });
});

async function toggleProject(project: KableProject, nextEnabled: boolean): Promise<void> {
  const selectedProfile = profile;

  if (!selectedProfile || selectedProfile.id !== profileId) {
    throw new Error("Cannot toggle a project without the currently selected profile.");
  }

  const key = projectKey(selectedProfile, project);

  if (pendingById[key]) {
    return;
  }

  pendingById = {
    ...pendingById,
    [key]: true,
  };

  optimisticEnabledById = {
    ...optimisticEnabledById,
    [key]: nextEnabled,
  };

  try {
    await app.projectsService.toggle(selectedProfile, project);
  } catch (err) {
    error(`[InstalledProjects] Failed to toggle project ${projectId(project)}`, err);

    throw err;
  } finally {
    const nextOptimistic = { ...optimisticEnabledById };
    delete nextOptimistic[key];
    optimisticEnabledById = nextOptimistic;

    const nextPending = { ...pendingById };
    delete nextPending[key];
    pendingById = nextPending;
  }
}

let matchingProjects = $derived.by(() => {
  const query = debouncedQuery.trim();
  const all = [...projects];

  if (!query) return all;

  const nameMatches = all
    .map((project) => ({
      project,
      score: fuzzyScore(query, projectName(project)),
    }))
    .filter((item) => item.score > 0)
    .sort((a, b) => b.score - a.score)
    .map((item) => item.project);

  if (nameMatches.length > 0) return nameMatches;

  return all
    .map((project) => ({
      project,
      score: fuzzyScore(query, projectDescription(project)),
    }))
    .filter((item) => item.score > 0)
    .sort((a, b) => b.score - a.score)
    .map((item) => item.project);
});

function sortProjects(list: KableProject[]): KableProject[] {
  const sorted = [...list];

  switch (sortOrder) {
    case "name-asc":
      sorted.sort(compareNames);
      break;
    case "name-desc":
      sorted.sort((a, b) => compareNames(b, a));
      break;
    case "downloads-desc":
      sorted.sort((a, b) => downloads(b) - downloads(a));
      break;
    case "downloads-asc":
      sorted.sort((a, b) => downloads(a) - downloads(b));
      break;
    case "followers-desc":
      sorted.sort((a, b) => followers(b) - followers(a));
      break;
    case "followers-asc":
      sorted.sort((a, b) => followers(a) - followers(b));
      break;
  }

  return sorted;
}

let enabledProjects = $derived(sortProjects(matchingProjects.filter((project) => isProjectEnabled(project))));

let disabledProjects = $derived(sortProjects(matchingProjects.filter((project) => !isProjectEnabled(project))));

let projectLabel = $derived(projectType === "mod" ? "mods" : projectType === "resourcepack" ? "resource packs" : projectType === "shader" ? "shaders" : "modpacks");
</script>

<div class="installed-projects">
  {#if !profile}
    <div class="state">
      <p>Select a profile to view installed {projectLabel}.</p>
    </div>
  {:else}
    <div class="toolbar">
      <div class="search">
        <label for="installed-project-search">
          Search {projectLabel}
        </label>

        <input
          id="installed-project-search"
          type="search"
          placeholder="Search by name or description..."
          bind:value={searchInput}
          autocomplete="off"
          spellcheck="false"
        />
      </div>

      <div class="sort">
        <label for="installed-project-sort">Sort by</label>

        <select id="installed-project-sort" bind:value={sortOrder}>
          <option value="name-asc">Name (A–Z)</option>
          <option value="name-desc">Name (Z–A)</option>
          <option value="downloads-desc">Downloads (most first)</option>
          <option value="downloads-asc">Downloads (least first)</option>
          <option value="followers-desc">Followers (most first)</option>
          <option value="followers-asc">Followers (least first)</option>
        </select>
      </div>
    </div>

    {#if loading}
      <p class="refresh-status" role="status">
        Loading installed {projectLabel}...
      </p>
    {/if}

    {#if projects.length === 0 && loading}
      <div class="state">
        <p>Loading installed {projectLabel}...</p>
      </div>
    {:else if projects.length === 0}
      <div class="state">
        <p>No {projectLabel} installed.</p>
      </div>
    {:else if matchingProjects.length === 0}
      <div class="state">
        <p>No {projectLabel} match "{debouncedQuery}".</p>
      </div>
    {:else}
      {#if enabledProjects.length > 0}
        <section class="project-group">
          <button class="group-heading" type="button" aria-expanded={enabledOpen} onclick={() => (enabledOpen = !enabledOpen)}>
            <span class="group-title">Enabled</span>

            <span class="group-count">
              {enabledProjects.length}
            </span>

            {#if enabledOpen}
              <Icon name="chevron-up" size="sm" forceType="svg" />
            {:else}
              <Icon name="chevron-down" size="sm" forceType="svg" />
            {/if}
          </button>

          <div class="grid" class:collapsed={!enabledOpen} aria-hidden={!enabledOpen}>
            {#each enabledProjects as project (projectId(project))}
              <InstalledProjectCard
                {profile}
                {project}
                enabled={true}
                updateAvailable={hasUpdate(project)}
                togglePending={isTogglePending(project)}
                onToggled={(enabled) => toggleProject(project, enabled)}
              />
            {/each}
          </div>
        </section>
      {/if}

      {#if disabledProjects.length > 0}
        <section class="project-group">
          <button class="group-heading" type="button" aria-expanded={disabledOpen} onclick={() => (disabledOpen = !disabledOpen)}>
            <span class="group-title">Disabled</span>

            <span class="group-count">
              {disabledProjects.length}
            </span>

            {#if disabledOpen}
              <Icon name="chevron-up" size="sm" forceType="svg" />
            {:else}
              <Icon name="chevron-down" size="sm" forceType="svg" />
            {/if}
          </button>

          <div class="grid" class:collapsed={!disabledOpen} aria-hidden={!disabledOpen}>
            {#each disabledProjects as project (projectId(project))}
              <InstalledProjectCard
                {profile}
                {project}
                enabled={false}
                updateAvailable={hasUpdate(project)}
                togglePending={isTogglePending(project)}
                onToggled={(enabled) => toggleProject(project, enabled)}
              />
            {/each}
          </div>
        </section>
      {/if}
    {/if}
  {/if}
</div>

<style lang="scss">
.installed-projects {
  width: 100%;
  min-width: 0;
}

.toolbar {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(200px, 260px);
  gap: $layout-gap;
  margin-bottom: $space-md;
}

.search,
.sort {
  display: flex;
  flex-direction: column;
  gap: $space-xs;
  min-width: 0;

  label {
    color: $color-text-muted;
    font-size: 0.8rem;
  }
}

select,
input {
  box-sizing: border-box;
  width: 100%;
  min-width: 0;
  padding: $space-sm $space-md;
  color: inherit;
  border: 1px solid $color-border;
  border-radius: $radius-md;
  font: inherit;
}

.refresh-status {
  margin: 0 0 $space-sm;
  color: $color-text-muted;
  font-size: 0.8rem;
}

.project-group {
  margin-bottom: $space-lg;
}

.group-heading {
  display: flex;
  align-items: center;
  gap: $space-sm;
  width: 100%;
  margin-bottom: $space-sm;
  padding: $space-sm 0;
  color: inherit;
  background: transparent;
  border: 0;
  border-bottom: 1px solid $color-border;
  cursor: pointer;
  text-align: left;
}

.group-title {
  font-size: 1rem;
  font-weight: 600;
}

.group-count {
  color: $color-text-muted;
  font-size: 0.8rem;
}

.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax($layout-container-1, 1fr));
  gap: $layout-gap;
  width: 100%;

  &.collapsed {
    display: none;
  }
}

.state {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 240px;
  padding: $space-xl;
  color: $color-text-muted;
  font-size: 0.85rem;
  text-align: center;

  p {
    margin: 0;
  }
}

@media (max-width: 640px) {
  .toolbar {
    grid-template-columns: 1fr;
  }

  .grid {
    grid-template-columns: 1fr;
  }
}
</style>
