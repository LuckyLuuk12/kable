import {
  type KableProfile,
  type KableProject,
  type Project,
  type ProjectSearch,
  type ProjectType,
  api
} from "$lib";
import type { Service } from "./app.svelte";

export class ProjectsService implements Service {
  projects = $state<KableProject[]>([]);
  loading = $state(false);
  loadedForProfileId = $state<string | null>(null);

  async init() {
    //
  }

  private updateStoredProject(project: KableProject) {
    this.projects = this.projects.map((current) =>
      current.project.project_id === project.project.project_id
        ? project
        : current
    );
  }

  async load(
    profile: KableProfile,
    projectType: ProjectType
  ) {
    if (
      this.loadedForProfileId === profile.id &&
      this.projects.some(
        (project) =>
          project.project.project_type === projectType
      )
    ) {
      return;
    }

    this.loading = true;

    try {
      const [enabled, disabled] = await Promise.all([
        api.listProfileProjects(profile, true),
        api.listProfileProjects(profile, false)
      ]);

      this.projects = [
        ...(enabled[projectType] ?? []),
        ...(disabled[projectType] ?? [])
      ];

      this.loadedForProfileId = profile.id;
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
    return this.projects.filter(
      (project) =>
        project.project.project_type === type
    );
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

      await this.load(
        profile,
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

      await this.load(
        profile,
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

      await this.load(
        profile,
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
    this.loadedForProfileId = profile?.id ?? null;
    if (profile) {
      // re-fetch projects:
      for (const type of ["mod", "resourcepack", "shader", "modpack"] as ProjectType[]) {
        await this.load(profile, type);

      }
    } else {
      this.projects = [];
    }
    console.log("[ProjectsService] selected profile changed to", profile?.metadata.name ?? "null", "with", this.projects.length, "projects loaded");
  }
}