import { type KableProfile, type KableProject_Deserialize, type KableProject_Serialize, type Project_Deserialize, type ProjectSearch, type ProjectType, api } from "$lib";
import type { Service } from "./app.service";

export class ProjectsService implements Service {
  // TODO: rethink if we should hold the "global share" of projects here and have a map-state keyed by profiles
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
      const enabledProfileProjects = await api.listProfileProjects(profile, true);
      const disabledProfileProjects = await api.listProfileProjects(profile, false);
      this.projects = [...(enabledProfileProjects[projectType] ?? []), ...(disabledProfileProjects[projectType] ?? [])];
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
    return this.projects.filter((p) => p.project.project_type === "resourcepack");
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
  async browse(profile: KableProfile, search: ProjectSearch, smartFilter: boolean, projectType: ProjectType) {
    try {
      return await api.browse(profile, search, smartFilter, projectType);
    } catch (e) {
      console.error("API call failed: `return await api.browse(profile, search, smartFilter, projectType);`", e);
      throw e;
    }
  }

  /**
   * Convenience browse wrappers
   */
  async browseMods(profile: KableProfile, search: ProjectSearch, smartFilter: boolean) {
    return await this.browse(profile, search, smartFilter, "mod");
  }

  async browseResourcepacks(profile: KableProfile, search: ProjectSearch, smartFilter: boolean) {
    return await this.browse(profile, search, smartFilter, "resourcepack");
  }

  async browseShaders(profile: KableProfile, search: ProjectSearch, smartFilter: boolean) {
    return await this.browse(profile, search, smartFilter, "shader");
  }

  async browseModpacks(profile: KableProfile, search: ProjectSearch, smartFilter: boolean) {
    return await this.browse(profile, search, smartFilter, "modpack");
  }

  /**
   * Install/download project
   */
  async download(profile: KableProfile, project: Project_Deserialize, versionId: string | null) {
    try {
      const result = await api.addProjectToProfile(profile, project, versionId);

      this.projects = [...this.projects, result];
      return result;
    } catch (e) {
      console.error("Failed to download project", e);
      throw e;
    }
  }

  /**
   * Remove project
   */
  async remove(profile: KableProfile, project: KableProject_Deserialize) {
    try {
      const result = await api.removeProject(profile, project);

      this.projects = this.projects.filter((p) => p.project.project_id !== result.project.project_id);
      return result;
    } catch (e) {
      console.error("Failed to remove project", e);
      throw e;
    }
  }

  async isEnabled(profile: KableProfile, project: KableProject_Deserialize) {
    try {
      return await api.isProjectEnabled(profile, project);
    } catch (e) {
      console.error("API call failed: `return await api.isProjectEnabled(profile, project);`", e);
      throw e;
    }
  }

  /**
   * Enable project
   */
  async enable(profile: KableProfile, project: KableProject_Deserialize) {
    try {
      const isEnabled = await api.isProjectEnabled(profile, project);
      if (isEnabled) {
        console.warn("Project is already enabled", project);
        return project;
      }

      const updatedProfile = await api.toggleProject(profile, project);

      // Reload the projects list
      await this.load(profile, project.project.project_type);

      return updatedProfile;
    } catch (e) {
      console.error("Failed to enable project", e);
      throw e;
    }
  }
  /**
   * Disable project
   */
  async disable(profile: KableProfile, project: KableProject_Deserialize) {
    try {
      const isEnabled = await api.isProjectEnabled(profile, project);
      if (!isEnabled) {
        console.warn("Project is already disabled", project);
        return project;
      }

      const updatedProfile = await api.toggleProject(profile, project);

      // Reload the projects list
      await this.load(profile, project.project.project_type);

      return updatedProfile;
    } catch (e) {
      console.error("Failed to disable project", e);
      throw e;
    }
  }

  /**
   * Toggle project
   */
  async toggle(profile: KableProfile, project: KableProject_Deserialize) {
    try {
      const updatedProfile = await api.toggleProject(profile, project);

      // Reload the projects list
      await this.load(profile, project.project.project_type);

      return updatedProfile;
    } catch (e) {
      console.error("Failed to toggle project", e);
      throw e;
    }
  }

  /**
   * Check updates for installed projects
   */
  async checkUpdates(profile: KableProfile, projectType: ProjectType) {
    try {
      return await api.checkForUpdates(profile, projectType);
    } catch (e) {
      console.error("API call failed: `return await api.checkForProjectUpdates(profile, projectType);`", e);
      throw e;
    }
  }

  /**
   * Update single project
   */
  async update(profile: KableProfile, project: KableProject_Deserialize) {
    try {
      const updated = await api.updateProject(profile, project);

      this.projects = this.projects.map((p) => (p.project.project_id === updated.project.project_id ? updated : p));
      return updated;
    } catch (e) {
      console.error("Failed to update project", e);
      throw e;
    }
  }

  /**
   * Update all projects
   */
  async updateAll(profile: KableProfile, projectType: ProjectType) {
    try {
      const updated = await api.updateAllProjects(profile, projectType);

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
}
