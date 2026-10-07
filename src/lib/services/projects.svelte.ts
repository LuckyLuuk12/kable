import {
  type KableProfile,
  type KableProject,
  type Project,
  type ProjectSearch,
  type ProjectType,
  type ProjectVersion,
  api,
  app
} from "$lib";
import { SvelteSet } from "svelte/reactivity";
import type { Service } from "./app.svelte";

export class ProjectsService implements Service {
  projects = $state<KableProject[]>([]);
  loading = $state(false);
  loadedForProfileId = $state<string | null>(null);
  loadedTypes = $state<SvelteSet<ProjectType>>(new SvelteSet());

  async init() {
    //
  }

  private async refreshProfiles(): Promise<void> {
    await app.profilesService.refreshProfiles();
  }

  private async refreshProfile(profile: KableProfile): Promise<KableProfile> {
    await app.profilesService.refreshProfiles();

    return (
      app.profilesService.profiles.find((p) => p.id === profile.id) ??
      profile
    );
  }

  private updateStoredProject(project: KableProject) {
    const exists = this.projects.some(
      (current) =>
        current.project.project_id === project.project.project_id
    );

    if (exists) {
      this.projects = this.projects.map((current) =>
        current.project.project_id === project.project.project_id
          ? project
          : current
      );
    } else {
      this.projects = [...this.projects, project];
    }
  }

  async load(
    profile: KableProfile,
    projectType: ProjectType,
    force = false
  ) {
    if (this.loadedForProfileId !== profile.id) {
      this.projects = [];
      this.loadedForProfileId = profile.id;
      this.loadedTypes = new SvelteSet();
    }

    if (!force && this.loadedTypes.has(projectType)) {
      return;
    }

    this.loading = true;

    try {
      const [enabled, disabled] = await Promise.all([
        api.listProfileProjects(profile, true),
        api.listProfileProjects(profile, false)
      ]);

      console.log("[ProjectsService] load", projectType,
        {
          enabled,
          disabled,
          enabledForType: enabled[projectType],
          disabledForType: disabled[projectType]
        });

      const loadedProjects = [
        ...(enabled[projectType] ?? []),
        ...(disabled[projectType] ?? [])
      ];

      console.log("[ProjectsService] loaded projects", projectType, loadedProjects);

      this.projects = [
        ...this.projects.filter(
          (project) => project.project.project_type !== projectType
        ),
        ...loadedProjects
      ];

      this.loadedTypes = new SvelteSet([
        ...this.loadedTypes,
        projectType
      ]);
    } finally {
      this.loading = false;
    }
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

  async browse(
    profile: KableProfile,
    search: ProjectSearch,
    smartFilter: boolean,
    projectType: ProjectType
  ) {
    try {
      return await api.browse(
        profile,
        search,
        smartFilter,
        projectType
      );
    } catch (e) {
      console.error("Failed to browse projects", e);
      throw e;
    }
  }

  async browseMods(
    profile: KableProfile,
    search: ProjectSearch,
    smartFilter: boolean
  ) {
    return this.browse(profile, search, smartFilter, "mod");
  }

  async browseResourcepacks(
    profile: KableProfile,
    search: ProjectSearch,
    smartFilter: boolean
  ) {
    return this.browse(
      profile,
      search,
      smartFilter,
      "resourcepack"
    );
  }

  async browseShaders(
    profile: KableProfile,
    search: ProjectSearch,
    smartFilter: boolean
  ) {
    return this.browse(profile, search, smartFilter, "shader");
  }

  async browseModpacks(
    profile: KableProfile,
    search: ProjectSearch,
    smartFilter: boolean
  ) {
    return this.browse(
      profile,
      search,
      smartFilter,
      "modpack"
    );
  }

  async download(
    profile: KableProfile,
    project: Project,
    versionId: string | null
  ) {
    try {
      const result = await api.addProjectToProfile(
        profile,
        project,
        versionId
      );

      this.updateStoredProject(result);
      await this.refreshProfiles();

      return result;
    } catch (e) {
      console.error("Failed to download project", e);
      throw e;
    }
  }

  async remove(
    profile: KableProfile,
    project: KableProject
  ) {
    try {
      const result = await api.removeProject(
        profile,
        project
      );

      this.projects = this.projects.filter(
        (current) =>
          current.project.project_id !==
          result.project.project_id
      );

      await this.refreshProfiles();

      return result;
    } catch (e) {
      console.error("Failed to remove project", e);
      throw e;
    }
  }

  async isEnabled(
    profile: KableProfile,
    project: KableProject
  ) {
    try {
      return await api.isProjectEnabled(
        profile,
        project
      );
    } catch (e) {
      console.error("Failed to check project state", e);
      throw e;
    }
  }

  async enable(
    profile: KableProfile,
    project: KableProject
  ) {
    try {
      if (await api.isProjectEnabled(profile, project)) {
        return project;
      }

      await api.toggleProject(profile, project);

      const updatedProfile = await this.refreshProfile(profile);

      await this.load(
        updatedProfile,
        project.project.project_type
      );

      return project;
    } catch (e) {
      console.error("Failed to enable project", e);
      throw e;
    }
  }

  async disable(
    profile: KableProfile,
    project: KableProject
  ) {
    try {
      if (!(await api.isProjectEnabled(profile, project))) {
        return project;
      }

      await api.toggleProject(profile, project);

      const updatedProfile = await this.refreshProfile(profile);

      await this.load(
        updatedProfile,
        project.project.project_type
      );

      return project;
    } catch (e) {
      console.error("Failed to disable project", e);
      throw e;
    }
  }

  async toggle(
    profile: KableProfile,
    project: KableProject
  ) {
    try {
      await api.toggleProject(profile, project);

      const updatedProfile = await this.refreshProfile(profile);

      await this.load(
        updatedProfile,
        project.project.project_type
      );

      return project;
    } catch (e) {
      console.error("Failed to toggle project", e);
      throw e;
    }
  }

  async checkUpdates(
    profile: KableProfile,
    projectType: ProjectType
  ) {
    try {
      return await api.checkForUpdates(
        profile,
        projectType
      );
    } catch (e) {
      console.error(
        "Failed to check project updates",
        e
      );
      throw e;
    }
  }

  async update(
    profile: KableProfile,
    project: KableProject
  ) {
    try {
      const updated = await api.updateProject(
        profile,
        project
      );

      this.updateStoredProject(updated);

      await this.refreshProfiles();

      return updated;
    } catch (e) {
      console.error("Failed to update project", e);
      throw e;
    }
  }

  async updateAll(
    profile: KableProfile,
    projectType: ProjectType
  ) {
    try {
      const updated = await api.updateAllProjects(
        profile,
        projectType
      );

      this.projects = updated;
      await this.refreshProfiles();

      return updated;
    } catch (e) {
      console.error("Failed to update all projects", e);
      throw e;
    }
  }

  async destroy() {
    this.projects = [];
    this.loadedForProfileId = null;
  }

  async select(profile: KableProfile | null) {
    this.projects = [];
    this.loadedTypes = new SvelteSet();
    this.loadedForProfileId = profile?.id ?? null;

    if (!profile) {
      console.log(
        "[ProjectsService] selected profile changed to null with 0 projects loaded"
      );
      return;
    }

    for (const type of [
      "mod",
      "resourcepack",
      "shader"
    ] as ProjectType[]) {
      await this.load(profile, type);
    }

    console.log(
      "[ProjectsService] selected profile changed to",
      profile.metadata.name,
      "with",
      this.projects.length,
      "projects loaded"
    );
  }

  isVersionCompatible(
    profile: KableProfile,
    version: ProjectVersion,
    projectType: ProjectType
  ): boolean {
    const minecraftVersion = profile.version.minecraft_version;

    if (
      minecraftVersion &&
      !version.game_versions.includes(minecraftVersion)
    ) {
      return false;
    }

    const loaderRelevant =
      projectType === "mod" ||
      projectType === "modpack";

    if (!loaderRelevant) {
      return true;
    }

    const profileLoader = profile.version.loader
      .toString()
      .toLowerCase()
      .replace(/^iris_/, "");

    return version.loaders.some(
      (loader) =>
        loader.toLowerCase().replace(/^iris_/, "") === profileLoader
    );
  }
}