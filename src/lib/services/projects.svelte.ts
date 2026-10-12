import {
  type KableProfile,
  type KableProject,
  type Project,
  ProjectDependenciesModal,
  type ProjectSearch,
  type ProjectType,
  type ProjectVersion,
  type UpdateMap,
  api,
  app,
} from "$lib";
import { SvelteSet } from "svelte/reactivity";
import type { Service } from "./app.svelte";

export class ProjectsService implements Service {
  projects = $state<KableProject[]>([]);
  loading = $state(false);
  loadedForProfileId = $state<string | null>(null);
  loadedTypes = $state<SvelteSet<ProjectType>>(new SvelteSet());

  /**
   * Incremented whenever installed-project data or enabled state changes.
   * Consumers can depend on this revision to refresh derived state.
   */
  enabledStateRevision = $state(0);

  private pendingLoads = new Map<ProjectType, Promise<void>>();
  private loadVersions = new Map<ProjectType, number>();
  private activeLoadsByGeneration = new Map<number, number>();
  private profileGeneration = 0;

  async init() { }

  private getProjectId(project: KableProject): string {
    return project.project.project_id;
  }

  /**
   * Notify reactive consumers that installed-project data or enabled state
   * may have changed. Replacing the array also invalidates derived lists.
   */
  private notifyEnabledStateChanged(): void {
    this.enabledStateRevision++;
    this.projects = [...this.projects];
  }

  private updateStoredProject(project: KableProject): void {
    const projectId = this.getProjectId(project);
    const index = this.projects.findIndex((current) => this.getProjectId(current) === projectId);

    if (index === -1) {
      this.projects = [...this.projects, project];
      return;
    }

    const updated = [...this.projects];
    updated[index] = project;
    this.projects = updated;
  }

  private removeStoredProject(projectId: string): void {
    this.projects = this.projects.filter((current) => this.getProjectId(current) !== projectId);
  }

  private replaceTypeProjects(projectType: ProjectType, incoming: KableProject[]): void {
    const otherTypes = this.projects.filter((project) => project.project.project_type !== projectType);

    this.projects = [...otherTypes, ...incoming];
  }

  /**
   * Starts tracking a load against the profile generation that owns it.
   */
  private beginLoad(generation: number): void {
    const active = this.activeLoadsByGeneration.get(generation) ?? 0;

    this.activeLoadsByGeneration.set(generation, active + 1);

    if (generation === this.profileGeneration) {
      this.loading = true;
    }
  }

  /**
   * Ends a load without allowing stale requests to change the current
   * profile's loading state.
   */
  private endLoad(generation: number): void {
    const active = this.activeLoadsByGeneration.get(generation) ?? 0;
    const remaining = Math.max(0, active - 1);

    if (remaining === 0) {
      this.activeLoadsByGeneration.delete(generation);
    } else {
      this.activeLoadsByGeneration.set(generation, remaining);
    }

    if (generation === this.profileGeneration) {
      this.loading = remaining > 0;
    }
  }

  /**
   * Invalidates outstanding requests and resets the selected profile's
   * project cache.
   */
  private resetProfile(profileId: string | null): void {
    this.profileGeneration++;
    this.projects = [];
    this.loadedForProfileId = profileId;
    this.loadedTypes = new SvelteSet();
    this.pendingLoads.clear();
    this.loadVersions.clear();
    this.loading = false;
    this.enabledStateRevision++;
  }

  private ensureProfile(profile: KableProfile): void {
    if (this.loadedForProfileId === profile.id) {
      return;
    }

    this.resetProfile(profile.id);
  }

  async load(profile: KableProfile, projectType: ProjectType, force = false): Promise<void> {
    this.ensureProfile(profile);

    if (!force && this.loadedTypes.has(projectType)) {
      return;
    }

    const pending = this.pendingLoads.get(projectType);

    if (pending && !force) {
      return pending;
    }

    const generation = this.profileGeneration;
    const version = (this.loadVersions.get(projectType) ?? 0) + 1;

    this.loadVersions.set(projectType, version);
    this.beginLoad(generation);

    const request = Promise.resolve().then(async () => {
      try {
        const [enabled, disabled] = await Promise.all([api.listProfileProjects(profile, true), api.listProfileProjects(profile, false)]);

        if (generation !== this.profileGeneration || this.loadedForProfileId !== profile.id || this.loadVersions.get(projectType) !== version) {
          return;
        }

        const loadedProjects = [...(enabled[projectType] ?? []), ...(disabled[projectType] ?? [])];

        this.replaceTypeProjects(projectType, loadedProjects);

        this.loadedTypes = new SvelteSet([...this.loadedTypes, projectType]);

        this.notifyEnabledStateChanged();
      } catch (error) {
        console.error(`[ProjectsService] Failed to load ${projectType} projects`, error);

        throw error;
      } finally {
        if (this.pendingLoads.get(projectType) === request) {
          this.pendingLoads.delete(projectType);
        }

        this.endLoad(generation);
      }
    });

    this.pendingLoads.set(projectType, request);

    return request;
  }

  async refresh(profile: KableProfile, projectType?: ProjectType): Promise<void> {
    this.ensureProfile(profile);

    if (projectType) {
      await this.load(profile, projectType, true);
      return;
    }

    const types = [...this.loadedTypes];

    await Promise.all(types.map((type) => this.load(profile, type, true)));
  }

  get all(): KableProject[] {
    return this.projects;
  }

  get mods(): KableProject[] {
    return this.getByType("mod");
  }

  get resourcepacks(): KableProject[] {
    return this.getByType("resourcepack");
  }

  get shaders(): KableProject[] {
    return this.getByType("shader");
  }

  get modpacks(): KableProject[] {
    return this.getByType("modpack");
  }

  getByType(type: ProjectType): KableProject[] {
    return this.projects.filter((project) => project.project.project_type === type);
  }

  async browse(profile: KableProfile, search: ProjectSearch, smartFilter: boolean, projectType: ProjectType) {
    try {
      return await api.browse(profile, search, smartFilter, projectType);
    } catch (error) {
      console.error("Failed to browse projects", error);
      throw error;
    }
  }

  async browseMods(profile: KableProfile, search: ProjectSearch, smartFilter: boolean) {
    return this.browse(profile, search, smartFilter, "mod");
  }

  async browseResourcepacks(profile: KableProfile, search: ProjectSearch, smartFilter: boolean) {
    return this.browse(profile, search, smartFilter, "resourcepack");
  }

  async browseShaders(profile: KableProfile, search: ProjectSearch, smartFilter: boolean) {
    return this.browse(profile, search, smartFilter, "shader");
  }

  async browseModpacks(profile: KableProfile, search: ProjectSearch, smartFilter: boolean) {
    return this.browse(profile, search, smartFilter, "modpack");
  }

  async install(profile: KableProfile, project: Project, versionId: string): Promise<KableProject | void> {
    try {
      const dependencies = await api.resolveProjectDependencies(profile, project, versionId);

      if (dependencies.length === 0) {
        return await this.download(profile, project, versionId);
      }

      app.show(ProjectDependenciesModal, {
        profile,
        project,
        versionId,
        dependencies,
        onConfirm: async (selectedDependencies: KableProject[]) => {
          for (const dependency of selectedDependencies) {
            await this.download(profile, dependency.project, dependency.version_id);
          }

          await this.download(profile, project, versionId);
        },
      });
    } catch (error) {
      console.error("Failed to install project", error);
      throw error;
    }
  }

  async download(profile: KableProfile, project: Project, versionId: string | null): Promise<KableProject> {
    try {
      const result = await api.addProjectToProfile(profile, project, versionId);

      this.updateStoredProject(result);

      this.loadedTypes = new SvelteSet([...this.loadedTypes, result.project.project_type]);

      // Installation can change enabledById in InstalledProjects.svelte.
      this.notifyEnabledStateChanged();

      return result;
    } catch (error) {
      console.error("Failed to download project", error);
      throw error;
    }
  }

  async remove(profile: KableProfile, project: KableProject) {
    try {
      const result = await api.removeProject(profile, project);

      this.removeStoredProject(result.project.project_id);
      this.notifyEnabledStateChanged();

      return result;
    } catch (error) {
      console.error("Failed to remove project", error);
      throw error;
    }
  }

  async isEnabled(profile: KableProfile, project: KableProject): Promise<boolean> {
    try {
      return await api.isProjectEnabled(profile, project);
    } catch (error) {
      console.error("Failed to check project state", error);
      throw error;
    }
  }

  async enable(profile: KableProfile, project: KableProject) {
    try {
      if (!(await api.isProjectEnabled(profile, project))) {
        await api.toggleProject(profile, project);
        await this.isEnabled(profile, project);
        this.notifyEnabledStateChanged();
      }

      return project;
    } catch (error) {
      console.error("Failed to enable project", error);
      throw error;
    }
  }

  async disable(profile: KableProfile, project: KableProject) {
    try {
      if (await api.isProjectEnabled(profile, project)) {
        await api.toggleProject(profile, project);
        await this.isEnabled(profile, project);
        this.notifyEnabledStateChanged();
      }

      return project;
    } catch (error) {
      console.error("Failed to disable project", error);
      throw error;
    }
  }

  async toggle(profile: KableProfile, project: KableProject): Promise<KableProject> {
    try {
      await api.toggleProject(profile, project);

      // Confirm the API can read the state after the toggle completes.
      await this.isEnabled(profile, project);

      this.notifyEnabledStateChanged();

      return project;
    } catch (error) {
      console.error("Failed to toggle project", error);
      throw error;
    }
  }

  async checkUpdate(profile: KableProfile, project: KableProject): Promise<Project | null> {
    try {
      return await api.checkForUpdate(profile, project);
    } catch (error) {
      if (!(error as string).includes("No update available")) {
        console.error("Failed to check project update", error);
        throw error;
      }

      return null;
    }
  }

  async checkUpdates(profile: KableProfile, projectType: ProjectType): Promise<UpdateMap[]> {
    try {
      return await api.checkForUpdates(profile, projectType);
    } catch (error) {
      console.error("Failed to check project updates", error);
      throw error;
    }
  }

  async update(profile: KableProfile, project: KableProject, versionId: string | null = null) {
    try {
      const updated = await api.updateProject(profile, project, versionId);

      this.updateStoredProject(updated);
      this.notifyEnabledStateChanged();

      return updated;
    } catch (error) {
      console.error("Failed to update project", error);
      throw error;
    }
  }

  async updateAll(profile: KableProfile, projectType: ProjectType) {
    try {
      const updated = await api.updateAllProjects(profile, projectType);

      this.replaceTypeProjects(projectType, updated);
      this.notifyEnabledStateChanged();

      return updated;
    } catch (error) {
      console.error("Failed to update all projects", error);
      throw error;
    }
  }

  getCompatibleVersions(profile: KableProfile, project: Project): ProjectVersion[] {
    return project.versions.filter((version) => this.isVersionCompatible(profile, version, project.project_type));
  }

  getLatestCompatibleVersion(profile: KableProfile, project: Project): ProjectVersion | null {
    const compatibleVersions = this.getCompatibleVersions(profile, project);

    return [...compatibleVersions].sort((a, b) => new Date(b.date_published).getTime() - new Date(a.date_published).getTime())[0] ?? null;
  }

  isVersionCompatible(profile: KableProfile, version: ProjectVersion, projectType: ProjectType): boolean {
    const minecraftVersion = profile.version.minecraft_version;

    if (minecraftVersion && !version.game_versions.includes(minecraftVersion)) {
      return false;
    }

    const loaderRelevant = projectType === "mod" || projectType === "modpack";

    if (!loaderRelevant) {
      return true;
    }

    const profileLoader = profile.version.loader
      .toString()
      .toLowerCase()
      .replace(/^iris_/, "");

    return version.loaders.some((loader) => loader.toLowerCase().replace(/^iris_/, "") === profileLoader);
  }

  async destroy() {
    this.resetProfile(null);
  }

  async select(profile: KableProfile | null) {
    this.resetProfile(profile?.id ?? null);

    if (!profile) {
      console.log("[ProjectsService] selected profile changed to null with 0 projects loaded");
      return;
    }

    for (const type of ["mod", "resourcepack", "shader"] as ProjectType[]) {
      await this.load(profile, type);
    }

    if (this.loadedForProfileId !== profile.id) {
      return;
    }

    console.log("[ProjectsService] selected profile changed to", profile.metadata.name, "with", this.projects.length, "projects loaded");
  }

  async getModrinthCategories(): Promise<string[]> {
    try {
      return await api.getModrinthCategories();
    } catch (error) {
      console.error("Failed to get Modrinth categories", error);
      throw error;
    }
  }

  async getModrinthLoaders(): Promise<string[]> {
    try {
      return await api.getModrinthLoaders();
    } catch (error) {
      console.error("Failed to get Modrinth loaders", error);
      throw error;
    }
  }

  async getModrinthGameVersions(): Promise<string[]> {
    try {
      return await api.getModrinthGameVersions();
    } catch (error) {
      console.error("Failed to get Modrinth game versions", error);
      throw error;
    }
  }

  async getModrinthProjectTypes(): Promise<string[]> {
    try {
      return await api.getModrinthProjectTypes();
    } catch (error) {
      console.error("Failed to get Modrinth project types", error);
      throw error;
    }
  }

  async getModrinthSideTypes(): Promise<string[]> {
    try {
      return await api.getModrinthSideTypes();
    } catch (error) {
      console.error("Failed to get Modrinth side types", error);
      throw error;
    }
  }
}
