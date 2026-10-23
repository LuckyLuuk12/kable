
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
  error,
  log,
} from "$lib";
import { SvelteSet } from "svelte/reactivity";
import type { Service } from "./app.svelte";

export class ProjectsService implements Service {
  projects = $state<KableProject[]>([]);
  loading = $state(false);
  loadedForProfileId = $state<string | null>(null);
  loadedTypes = $state<SvelteSet<ProjectType>>(new SvelteSet());

  enabledStateRevision = $state(0);

  private pendingLoads = new Map<string, Promise<void>>();
  private loadVersions = new Map<string, number>();
  private activeLoadsByGeneration = new Map<number, number>();
  private profileGeneration = 0;
  private mutationVersions = new Map<string, number>();
  private pendingMutations = new Map<string, Promise<unknown>>();

  async init() { }

  private getProjectId(project: KableProject): string {
    return project.project.project_id;
  }

  private getProjectKey(profile: KableProfile, project: KableProject): string {
    return `${profile.id}:${project.project.project_type}:${this.getProjectId(project)}`;
  }

  private getLoadKey(profileId: string, projectType: ProjectType): string {
    return `${profileId}:${projectType}`;
  }

  private notifyEnabledStateChanged(): void {
    this.enabledStateRevision++;
    this.projects = [...this.projects];
  }

  private updateStoredProject(project: KableProject): void {
    const projectId = this.getProjectId(project);
    const index = this.projects.findIndex(
      (current) =>
        current.project.project_type === project.project.project_type &&
        this.getProjectId(current) === projectId,
    );

    if (index === -1) {
      this.projects = [...this.projects, project];
      return;
    }

    const updated = [...this.projects];
    updated[index] = project;
    this.projects = updated;
  }

  private removeStoredProject(project: KableProject): void {
    const projectId = this.getProjectId(project);
    const projectType = project.project.project_type;

    this.projects = this.projects.filter(
      (current) =>
        current.project.project_type !== projectType ||
        this.getProjectId(current) !== projectId,
    );
  }

  private replaceTypeProjects(
    projectType: ProjectType,
    incoming: KableProject[],
  ): void {
    const otherTypes = this.projects.filter(
      (project) => project.project.project_type !== projectType,
    );

    const unique = new Map<string, KableProject>();

    for (const project of incoming) {
      unique.set(this.getProjectId(project), project);
    }

    this.projects = [...otherTypes, ...unique.values()];
  }

  private beginLoad(generation: number): void {
    const active = this.activeLoadsByGeneration.get(generation) ?? 0;
    this.activeLoadsByGeneration.set(generation, active + 1);

    if (generation === this.profileGeneration) {
      this.loading = true;
    }
  }

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

  private resetProfile(profileId: string | null): void {
    this.profileGeneration++;

    this.projects = [];
    this.loadedForProfileId = profileId;
    this.loadedTypes = new SvelteSet();
    this.pendingLoads.clear();
    this.loadVersions.clear();
    this.mutationVersions.clear();
    this.pendingMutations.clear();
    this.loading = false;
    this.enabledStateRevision++;
  }

  private ensureProfile(profile: KableProfile): void {
    if (this.loadedForProfileId !== profile.id) {
      this.resetProfile(profile.id);
    }
  }

  private isCurrentProfile(profileId: string, generation: number): boolean {
    return (
      this.loadedForProfileId === profileId &&
      this.profileGeneration === generation
    );
  }

  private async serializeMutation<T>(
    key: string,
    operation: () => Promise<T>,
  ): Promise<T> {
    const previous = this.pendingMutations.get(key) ?? Promise.resolve();

    const current = previous
      .catch(() => undefined)
      .then(operation);

    this.pendingMutations.set(key, current);

    try {
      return await current;
    } finally {
      if (this.pendingMutations.get(key) === current) {
        this.pendingMutations.delete(key);
      }
    }
  }

  private bumpMutationVersion(key: string): number {
    const next = (this.mutationVersions.get(key) ?? 0) + 1;
    this.mutationVersions.set(key, next);
    return next;
  }

  async load(
    profile: KableProfile,
    projectType: ProjectType,
    force = false,
  ): Promise<void> {
    this.ensureProfile(profile);

    if (!force && this.loadedTypes.has(projectType)) {
      return;
    }

    const generation = this.profileGeneration;
    const key = this.getLoadKey(profile.id, projectType);
    const pending = this.pendingLoads.get(key);

    if (pending && !force) {
      return pending;
    }

    const version = (this.loadVersions.get(key) ?? 0) + 1;
    this.loadVersions.set(key, version);
    this.beginLoad(generation);

    let request!: Promise<void>;

    request = (async () => {
      try {
        const [enabled, disabled] = await Promise.all([
          api.listProfileProjects(profile, true),
          api.listProfileProjects(profile, false),
        ]);

        if (
          !this.isCurrentProfile(profile.id, generation) ||
          this.loadVersions.get(key) !== version
        ) {
          return;
        }

        const loadedProjects = [
          ...(enabled[projectType] ?? []),
          ...(disabled[projectType] ?? []),
        ];

        this.replaceTypeProjects(projectType, loadedProjects);
        this.loadedTypes = new SvelteSet([
          ...this.loadedTypes,
          projectType,
        ]);
        this.notifyEnabledStateChanged();
      } catch (err) {
        error(
          `[ProjectsService] Failed to load ${projectType} projects`,
          err,
        );
        throw err;
      } finally {
        if (this.pendingLoads.get(key) === request) {
          this.pendingLoads.delete(key);
        }

        this.endLoad(generation);
      }
    })();

    this.pendingLoads.set(key, request);
    return request;
  }

  async refresh(
    profile: KableProfile,
    projectType?: ProjectType,
  ): Promise<void> {
    this.ensureProfile(profile);

    if (projectType) {
      await this.load(profile, projectType, true);
      return;
    }

    const types = [...this.loadedTypes];

    await Promise.all(
      types.map((type) => this.load(profile, type, true)),
    );
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
    return this.projects.filter(
      (project) => project.project.project_type === type,
    );
  }

  async browse(
    profile: KableProfile,
    search: ProjectSearch,
    smartFilter: boolean,
    projectType: ProjectType,
  ) {
    try {
      return await api.browse(profile, search, smartFilter, projectType);
    } catch (err) {
      error("Failed to browse projects", err);
      throw err;
    }
  }

  async browseMods(
    profile: KableProfile,
    search: ProjectSearch,
    smartFilter: boolean,
  ) {
    return this.browse(profile, search, smartFilter, "mod");
  }

  async browseResourcepacks(
    profile: KableProfile,
    search: ProjectSearch,
    smartFilter: boolean,
  ) {
    return this.browse(profile, search, smartFilter, "resourcepack");
  }

  async browseShaders(
    profile: KableProfile,
    search: ProjectSearch,
    smartFilter: boolean,
  ) {
    return this.browse(profile, search, smartFilter, "shader");
  }

  async browseModpacks(
    profile: KableProfile,
    search: ProjectSearch,
    smartFilter: boolean,
  ) {
    return this.browse(profile, search, smartFilter, "modpack");
  }

  async install(
    profile: KableProfile,
    project: Project,
    versionId: string,
  ): Promise<KableProject | void> {
    try {
      const dependencies = await api.resolveProjectDependencies(
        profile,
        project,
        versionId,
      );

      if (dependencies.length === 0) {
        return await this.download(profile, project, versionId);
      }

      app.show(ProjectDependenciesModal, {
        profile,
        project,
        versionId,
        dependencies,
        onConfirm: async (selectedDependencies: KableProject[]) => {
          try {
            for (const dependency of selectedDependencies) {
              await this.download(
                profile,
                dependency.project,
                dependency.version_id,
              );
            }

            await this.download(profile, project, versionId);
          } catch (err) {
            error(
              "Failed to install project and dependencies",
              err,
            );
            app.notificationService.error(
              "Failed to install project or one of its dependencies.",
            );
            throw err;
          }
        },
      });
    } catch (err) {
      error("Failed to install project", err);
      throw err;
    }
  }

  async hasUpdate(
    profile: KableProfile,
    project: KableProject,
  ): Promise<boolean> {
    try {
      await this.checkUpdate(profile, project);
      return true;
    } catch (err) {
      return false;
    }
  }


  async download(
    profile: KableProfile,
    project: Project,
    versionId: string | null,
  ): Promise<KableProject> {
    const generation = this.profileGeneration;
    const profileId = profile.id;

    try {
      const result = await api.addProjectToProfile(
        profile,
        project,
        versionId,
      );

      if (this.isCurrentProfile(profileId, generation)) {
        this.updateStoredProject(result);
        this.loadedTypes = new SvelteSet([
          ...this.loadedTypes,
          result.project.project_type,
        ]);
        this.notifyEnabledStateChanged();
      }

      app.notificationService.success(
        `${project.title} installed successfully.`,
      );

      return result;
    } catch (err) {
      error("Failed to download project", err);
      app.notificationService.error(`Failed to install ${project.title}.`);
      throw err;
    }
  }

  async remove(
    profile: KableProfile,
    project: KableProject,
  ): Promise<KableProject> {
    const key = this.getProjectKey(profile, project);
    const generation = this.profileGeneration;

    try {
      const result = await this.serializeMutation(key, async () => {
        const removed = await api.removeProject(profile, project);

        if (
          this.isCurrentProfile(profile.id, generation)
        ) {
          this.removeStoredProject(project);
          this.notifyEnabledStateChanged();
        }

        return removed;
      });

      app.notificationService.success(
        `${project.project.title} removed successfully.`,
      );

      return result;
    } catch (err) {
      error("Failed to remove project", err);
      app.notificationService.error(
        `Failed to remove ${project.project.title}.`,
      );
      throw err;
    }
  }

  async isEnabled(
    profile: KableProfile,
    project: KableProject,
  ): Promise<boolean> {
    try {
      return await api.isProjectEnabled(profile, project);
    } catch (err) {
      error("Failed to check project state", err);
      throw err;
    }
  }

  async enable(
    profile: KableProfile,
    project: KableProject,
  ): Promise<KableProject> {
    const key = this.getProjectKey(profile, project);

    try {
      return await this.serializeMutation(key, async () => {
        if (await api.isProjectEnabled(profile, project)) {
          return project;
        }

        await api.toggleProject(profile, project);

        const enabled = await api.isProjectEnabled(profile, project);

        if (!enabled) {
          throw new Error(
            `Project ${project.project.title} remained disabled after enabling.`,
          );
        }

        if (this.loadedForProfileId === profile.id) {
          this.bumpMutationVersion(key);
          this.notifyEnabledStateChanged();
        }

        app.notificationService.success(
          `${project.project.title} enabled.`,
        );

        return project;
      });
    } catch (err) {
      error("Failed to enable project", err);
      app.notificationService.error(
        `Failed to enable ${project.project.title}.`,
      );
      throw err;
    }
  }

  async disable(
    profile: KableProfile,
    project: KableProject,
  ): Promise<KableProject> {
    const key = this.getProjectKey(profile, project);

    try {
      return await this.serializeMutation(key, async () => {
        if (!(await api.isProjectEnabled(profile, project))) {
          return project;
        }

        await api.toggleProject(profile, project);

        const enabled = await api.isProjectEnabled(profile, project);

        if (enabled) {
          throw new Error(
            `Project ${project.project.title} remained enabled after disabling.`,
          );
        }

        if (this.loadedForProfileId === profile.id) {
          this.bumpMutationVersion(key);
          this.notifyEnabledStateChanged();
        }

        app.notificationService.success(
          `${project.project.title} disabled.`,
        );

        return project;
      });
    } catch (err) {
      error("Failed to disable project", err);
      app.notificationService.error(
        `Failed to disable ${project.project.title}.`,
      );
      throw err;
    }
  }

  async toggle(
    profile: KableProfile,
    project: KableProject,
  ): Promise<KableProject> {
    const key = this.getProjectKey(profile, project);

    try {
      return await this.serializeMutation(key, async () => {
        await api.toggleProject(profile, project);

        const enabled = await api.isProjectEnabled(profile, project);

        if (this.loadedForProfileId === profile.id) {
          this.bumpMutationVersion(key);
          this.notifyEnabledStateChanged();
        }

        app.notificationService.success(
          `${project.project.title} ${enabled ? "enabled" : "disabled"}.`,
        );

        return project;
      });
    } catch (err) {
      error("Failed to toggle project", err);
      app.notificationService.error(
        `Failed to change the enabled state of ${project.project.title}.`,
      );
      throw err;
    }
  }

  async checkUpdate(
    profile: KableProfile,
    project: KableProject,
  ): Promise<Project | null> {
    try {
      return await api.checkForUpdate(profile, project);
    } catch (err) {
      if (
        typeof err === "string" &&
        err.includes("No update available")
      ) {
        return null;
      }

      error("Failed to check project update", err);
      throw err;
    }
  }

  async checkUpdates(
    profile: KableProfile,
    projectType: ProjectType,
  ): Promise<UpdateMap[]> {
    try {
      return await api.checkForUpdates(profile, projectType);
    } catch (err) {
      error("Failed to check project updates", err);
      throw err;
    }
  }

  async update(
    profile: KableProfile,
    project: KableProject,
    versionId: string | null = null,
  ): Promise<KableProject> {
    const key = this.getProjectKey(profile, project);
    const generation = this.profileGeneration;

    try {
      const updated = await this.serializeMutation(key, async () => {
        const result = await api.updateProject(
          profile,
          project,
          versionId,
        );

        if (this.isCurrentProfile(profile.id, generation)) {
          this.updateStoredProject(result);
          this.notifyEnabledStateChanged();
        }

        return result;
      });

      app.notificationService.success(
        `${project.project.title} updated successfully.`,
      );

      return updated;
    } catch (err) {
      error("Failed to update project", err);
      app.notificationService.error(
        `Failed to update ${project.project.title}.`,
      );
      throw err;
    }
  }

  async updateAll(
    profile: KableProfile,
    projectType: ProjectType,
  ): Promise<KableProject[]> {
    const generation = this.profileGeneration;

    try {
      const updated = await api.updateAllProjects(profile, projectType);

      if (this.isCurrentProfile(profile.id, generation)) {
        this.replaceTypeProjects(projectType, updated);
        this.notifyEnabledStateChanged();
      }

      app.notificationService.success(
        `All ${projectType === "resourcepack"
          ? "resource packs"
          : `${projectType}s`
        } updated successfully.`,
      );

      return updated;
    } catch (err) {
      error("Failed to update all projects", err);
      app.notificationService.error(
        `Failed to update ${projectType} projects.`,
      );
      throw err;
    }
  }

  getCompatibleVersions(
    profile: KableProfile,
    project: Project,
  ): ProjectVersion[] {
    return project.versions.filter((version) =>
      this.isVersionCompatible(profile, version, project.project_type),
    );
  }

  getLatestCompatibleVersion(
    profile: KableProfile,
    project: Project,
  ): ProjectVersion | null {
    const compatibleVersions = this.getCompatibleVersions(profile, project);

    return (
      [...compatibleVersions].sort(
        (a, b) =>
          new Date(b.date_published).getTime() -
          new Date(a.date_published).getTime(),
      )[0] ?? null
    );
  }

  isVersionCompatible(
    profile: KableProfile,
    version: ProjectVersion,
    projectType: ProjectType,
  ): boolean {
    const minecraftVersion = profile.version.minecraft_version;

    if (
      minecraftVersion &&
      !version.game_versions.includes(minecraftVersion)
    ) {
      return false;
    }

    const loaderRelevant =
      projectType === "mod" || projectType === "modpack";

    if (!loaderRelevant) {
      return true;
    }

    const profileLoader = String(profile.version.loader)
      .toLowerCase()
      .replace(/^iris_/, "");

    return version.loaders.some(
      (loader) =>
        loader.toLowerCase().replace(/^iris_/, "") === profileLoader,
    );
  }

  async destroy(): Promise<void> {
    this.resetProfile(null);
  }

  async select(profile: KableProfile | null): Promise<void> {
    this.resetProfile(profile?.id ?? null);

    if (!profile) {
      log("[ProjectsService] selected profile changed to null with 0 projects loaded");
      return;
    }

    const generation = this.profileGeneration;

    await Promise.all(
      (["mod", "resourcepack", "shader"] as ProjectType[]).map(
        (type) => this.load(profile, type),
      ),
    );

    if (!this.isCurrentProfile(profile.id, generation)) {
      return;
    }

    log(
      "[ProjectsService] selected profile changed to",
      profile.metadata.name,
      "with",
      this.projects.length,
      "projects loaded",
    );
  }

  async getModrinthCategories(): Promise<string[]> {
    try {
      return await api.getModrinthCategories();
    } catch (err) {
      error("Failed to get Modrinth categories", err);
      throw err;
    }
  }

  async getModrinthLoaders(): Promise<string[]> {
    try {
      return await api.getModrinthLoaders();
    } catch (err) {
      error("Failed to get Modrinth loaders", err);
      throw err;
    }
  }

  async getModrinthGameVersions(): Promise<string[]> {
    try {
      return await api.getModrinthGameVersions();
    } catch (err) {
      error("Failed to get Modrinth game versions", err);
      throw err;
    }
  }

  async getModrinthProjectTypes(): Promise<string[]> {
    try {
      return await api.getModrinthProjectTypes();
    } catch (err) {
      error("Failed to get Modrinth project types", err);
      throw err;
    }
  }

  async getModrinthSideTypes(): Promise<string[]> {
    try {
      return await api.getModrinthSideTypes();
    } catch (err) {
      error("Failed to get Modrinth side types", err);
      throw err;
    }
  }
}
