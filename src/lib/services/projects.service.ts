import {
  type KableProfile,
  type KableProject_Deserialize,
  type KableProject_Serialize,
  type Project_Deserialize,
  type ProjectSearch,
  type ProjectType,
  api,
} from "$lib";
import type { Service } from "./app.service";

export class ProjectsService implements Service {
  projects = $state<KableProject_Serialize[]>([]);
  loading = $state(false);
  loadedForProfileId = $state<string | null>(null);

  /**
   * Initialize the service
   */
  async init() {
    //
  }

  /**
   * Loads installed projects for a profile(cached per profile).
   */
  async load(profile: KableProfile, projectType: ProjectType) {
    if (this.loadedForProfileId === profile.id) return;

    this.loading = true;
    try {
      this.projects = await api.listProjects(profile, projectType);
      this.loadedForProfileId = profile.id;
    } finally {
      this.loading = false;
    }
  }

  /**
   * Raw installed projects
   */
  get all() {
    return this.projects;
  }

  /**
   * Typed project views (derived state, no extra IPC)
   */
  get mods() {
    return this.projects.filter((p) => p.project.project_type === "mod");
  }

  get resourcepacks() {
    return this.projects.filter(
      (p) => p.project.project_type === "resourcepack",
    );
  }

  get shaders() {
    return this.projects.filter((p) => p.project.project_type === "shader");
  }

  get modpacks() {
    return this.projects.filter((p) => p.project.project_type === "modpack");
  }

  /**
   * Generic filter
   */
  getByType(type: ProjectType) {
    return this.projects.filter((p) => p.project.project_type === type);
  }

  /**
   * Browse remote projects
   */
  async browse(
    profile: KableProfile,
    search: ProjectSearch,
    smartFilter: boolean,
    projectType: ProjectType,
  ) {
    return await api.browse(profile, search, smartFilter, projectType);
  }

  /**
   * Convenience browse wrappers
   */
  browseMods(
    profile: KableProfile,
    search: ProjectSearch,
    smartFilter: boolean,
  ) {
    return this.browse(profile, search, smartFilter, "mod");
  }

  browseResourcepacks(
    profile: KableProfile,
    search: ProjectSearch,
    smartFilter: boolean,
  ) {
    return this.browse(profile, search, smartFilter, "resourcepack");
  }

  browseShaders(
    profile: KableProfile,
    search: ProjectSearch,
    smartFilter: boolean,
  ) {
    return this.browse(profile, search, smartFilter, "shader");
  }

  browseModpacks(
    profile: KableProfile,
    search: ProjectSearch,
    smartFilter: boolean,
  ) {
    return this.browse(profile, search, smartFilter, "modpack");
  }

  /**
   * Install/download project
   */
  async download(
    profile: KableProfile,
    project: Project_Deserialize,
    versionId: string | null,
  ) {
    const result = await api.downloadProject(profile, project, versionId);

    this.projects = [...this.projects, result];
    return result;
  }

  /**
   * Remove project
   */
  async remove(profile: KableProfile, project: KableProject_Deserialize) {
    const result = await api.removeProject(profile, project);

    this.projects = this.projects.filter(
      (p) => p.project.project_id !== result.project.project_id,
    );

    return result;
  }

  /**
   * Enable project
   */
  async enable(profile: KableProfile, project: KableProject_Deserialize) {
    const updated = await api.enableProject(profile, project);

    this.projects = this.projects.map((p) =>
      p.project.project_id === updated.project.project_id ? updated : p,
    );

    return updated;
  }

  /**
   * Disable project
   */
  async disable(profile: KableProfile, project: KableProject_Deserialize) {
    const updated = await api.disableProject(profile, project);

    this.projects = this.projects.map((p) =>
      p.project.project_id === updated.project.project_id ? updated : p,
    );

    return updated;
  }

  /**
   * Toggle project
   */
  async toggle(profile: KableProfile, project: KableProject_Deserialize) {
    const updated = await api.toggleProject(profile, project);

    this.projects = this.projects.map((p) =>
      p.project.project_id === updated.project.project_id ? updated : p,
    );

    return updated;
  }

  /**
   * Check updates for installed projects
   */
  async checkUpdates(profile: KableProfile, projectType: ProjectType) {
    return await api.checkForProjectUpdates(profile, projectType);
  }

  /**
   * Update single project
   */
  async update(profile: KableProfile, project: KableProject_Deserialize) {
    const updated = await api.updateProject(profile, project);

    this.projects = this.projects.map((p) =>
      p.project.project_id === updated.project.project_id ? updated : p,
    );

    return updated;
  }

  /**
   * Update all projects
   */
  async updateAll(profile: KableProfile, projectType: ProjectType) {
    const updated = await api.updateAllProjects(profile, projectType);

    this.projects = updated;
    return updated;
  }

  async destroy() {
    this.projects = [];
    this.loadedForProfileId = null;
  }
}
