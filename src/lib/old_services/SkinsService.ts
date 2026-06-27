import * as skinsApi from "../api/skins";
import type {
  PlayerProfile,
  AccountSkin,
  AccountCape,
  CurrentSkin,
  SkinUploadConfig,
  SkinUploadResponse,
} from "../types";
import { NotificationService } from "./NotificationService";

/**
 * Service for managing Minecraft skins and capes
 * Follows the architecture pattern used in other services
 */
export class SkinsService {
  /**
   * Get the full player profile including skins and capes
   */
  static async getPlayerProfile(): Promise<PlayerProfile> {
    try {
      return await skinsApi.getPlayerProfile();
    } catch (error) {
      console.error("Failed to get player profile:", error);
      throw error;
    }
  }

  /**
   * Get all skins (remote + local)
   */
  static async getAllSkins(): Promise<AccountSkin[]> {
    try {
      const [remoteSkins, localSkins] = await Promise.allSettled([
        this.getRemoteSkins(),
        this.getLocalSkins(),
      ]);

      const skins: AccountSkin[] = [];

      if (remoteSkins.status === "fulfilled") {
        skins.push(...remoteSkins.value);
      }

      if (localSkins.status === "fulfilled") {
        skins.push(...localSkins.value);
      }

      return skins;
    } catch (error) {
      console.error("Failed to get all skins:", error);
      throw error;
    }
  }

  /**
   * Get remote skins from Microsoft/Mojang account
   */
  static async getRemoteSkins(): Promise<AccountSkin[]> {
    try {
      const profile = await skinsApi.getPlayerProfile();
      return profile.skins || [];
    } catch (error) {
      console.error("Failed to get remote skins:", error);
      return [];
    }
  }

  /**
   * Get local skins stored in launcher_custom_skins.json
   */
  static async getLocalSkins(): Promise<AccountSkin[]> {
    try {
      return await skinsApi.getLocalSkins();
    } catch (error) {
      console.error("Failed to get local skins:", error);
      return [];
    }
  }

  /**
   * Get current skin information
   */
  static async getCurrentSkin(): Promise<CurrentSkin> {
    try {
      return await skinsApi.getCurrentSkinInfo();
    } catch (error) {
      console.error("Failed to get current skin:", error);
      throw error;
    }
  }

  /**
   * Get all capes available to the user
   */
  static async getCapes(): Promise<AccountCape[]> {
    try {
      const profile = await skinsApi.getPlayerProfile();
      return profile.capes || [];
    } catch (error) {
      console.error("Failed to get capes:", error);
      return [];
    }
  }

  /**
   * Get the currently active cape
   */
  static async getActiveCape(): Promise<AccountCape | null> {
    try {
      return await skinsApi.getActiveCape();
    } catch (error) {
      console.error("Failed to get active cape:", error);
      return null;
    }
  }

  /**
   * Apply a skin by ID
   */
  static async applySkin(skinId: string): Promise<SkinUploadResponse> {
    try {
      const result = await skinsApi.applyAccountSkin(skinId);
      NotificationService.success(`Skin applied successfully`);
      return result;
    } catch (error) {
      console.error("Failed to apply skin:", error);
      NotificationService.error(`Failed to apply skin: ${error}`);
      throw error;
    }
  }

  /**
   * Apply a cape by ID (or remove if null)
   */
  static async applyCape(capeId: string | null): Promise<string> {
    try {
      const result = await skinsApi.applyCape(capeId);
      NotificationService.success(
        capeId ? `Cape applied successfully` : `Cape removed`,
      );
      return result;
    } catch (error) {
      console.error("Failed to apply cape:", error);
      NotificationService.error(
        `Failed to ${capeId ? "apply" : "remove"} cape: ${error}`,
      );
      throw error;
    }
  }

  /**
   * Upload a new skin
   */
  static async uploadSkin(
    config: SkinUploadConfig,
  ): Promise<SkinUploadResponse> {
    try {
      const result = await skinsApi.uploadSkinToAccount(config);
      NotificationService.success(`Skin uploaded successfully`);
      return result;
    } catch (error) {
      console.error("Failed to upload skin:", error);
      NotificationService.error(`Failed to upload skin: ${error}`);
      throw error;
    }
  }

  /**
   * Select a skin file using file dialog
   */
  static async selectSkinFile(): Promise<string | null> {
    try {
      return await skinsApi.selectSkinFile();
    } catch (error) {
      console.error("Failed to select skin file:", error);
      return null;
    }
  }

  /**
   * Modify a skin entry
   */
  static async modifySkin(
    skinId: string,
    newName?: string,
    newCapeId?: string,
    newSlim?: boolean,
  ): Promise<void> {
    try {
      return await skinsApi.modifySkinById(skinId, newName, newCapeId, newSlim);
    } catch (error) {
      console.error("Failed to modify skin:", error);
      throw error;
    }
  }

  /**
   * Remove a skin entry
   */
  static async removeSkin(skinId: string): Promise<void> {
    try {
      await skinsApi.removeSkinById(skinId);
      NotificationService.success(`Skin removed successfully`);
    } catch (error) {
      console.error("Failed to remove skin:", error);
      NotificationService.error(`Failed to remove skin: ${error}`);
      throw error;
    }
  }

  /**
   * Get skin display name
   */
  static getSkinDisplayName(skin: AccountSkin): string {
    if (
      skin.name &&
      skin.name !== "Current Skin" &&
      skin.name !== "Account Skin"
    ) {
      return skin.name;
    }
    return `${skin.model} Skin`;
  }

  /**
   * Get cape display name
   */
  static getCapeDisplayName(cape: AccountCape): string {
    return cape.alias || cape.id;
  }

  /**
   * Check if a skin is currently active
   */
  static isSkinActive(skin: AccountSkin): boolean {
    return skin.is_current === true;
  }

  /**
   * Check if a cape is currently active
   */
  static isCapeActive(cape: AccountCape): boolean {
    return cape.state === "ACTIVE";
  }
}
