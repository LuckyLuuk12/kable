import type { MinecraftInstallation, MinecraftVersion } from '../types';
import * as installationsApi from '../api/installations';

/**
 * Installation Service
 * High-level business logic for Minecraft installation management
 */

export class InstallationService {
  static async getInstallations(): Promise<MinecraftInstallation[]> {
    return await installationsApi.getInstallations();
  }

  static async createInstallation(
    name: string,
    version: string,
    modLoader: string,
    gameDirectory?: string,
    javaPath?: string,
    jvmArgs?: string,
    memory?: number,
    description?: string
  ): Promise<MinecraftInstallation> {
    return await installationsApi.createInstallation(
      name,
      version,
      modLoader,
      gameDirectory,
      javaPath,
      jvmArgs,
      memory,
      description
    );
  }

  static async updateInstallation(
    installationId: string,
    name: string,
    version: string,
    modLoader: string,
    gameDirectory?: string,
    javaPath?: string,
    jvmArgs?: string,
    memory?: number,
    description?: string
  ): Promise<MinecraftInstallation> {
    return await installationsApi.updateInstallation(
      installationId,
      name,
      version,
      modLoader,
      gameDirectory,
      javaPath,
      jvmArgs,
      memory,
      description
    );
  }

  static async deleteInstallation(installationId: string): Promise<void> {
    return await installationsApi.deleteInstallation(installationId);
  }

  static async launchInstallation(installationId: string): Promise<void> {
    return await installationsApi.launchMinecraftInstallation(installationId);
  }

  static async openInstallationFolder(installationId: string): Promise<void> {
    return await installationsApi.openInstallationFolder(installationId);
  }

  static async getMinecraftVersions(): Promise<MinecraftVersion[]> {
    return await installationsApi.getMinecraftVersions();
  }
}
