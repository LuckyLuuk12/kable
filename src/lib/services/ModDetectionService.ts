import type { MinecraftInstallation, ModDetectionResult } from '../types';
import { loadSettings } from '../api/settings';


/**
 * Enhanced Mod Detection Service
 * Provides additional mod detection capabilities beyond basic installation scanning
 */

export class ModDetectionService {
  // /**
  //  * Analyze an installation to determine its actual modding status
  //  */
  // static async analyzeInstallation(installation: MinecraftInstallation): Promise<ModDetectionResult> {
  //   const result: ModDetectionResult = {
  //     hasActiveMods: false,
  //     modCount: 0,
  //     detectedLoaders: [],
  //     modLoaderType: 'vanilla',
  //     modsList: []
  //   };

  //   try {
  //     // 1. Use the new backend API to detect the actual mod loader
  //     const detectionResult = await detectInstallationModLoader(installation.id);
      
  //     if (detectionResult.modLoader && detectionResult.modLoader !== 'vanilla') {
  //       result.detectedLoaders.push(detectionResult.modLoader);
  //       result.modLoaderType = detectionResult.modLoader as any;
  //       result.loaderVersion = detectionResult.loaderVersion || undefined;
  //       result.hasActiveMods = true;
  //     }

  //     // 2. Try to get actual mod count
  //     try {
  //       const settings = await loadSettings();
  //       const minecraftPath = settings.general.game_directory || '';
  //       if (minecraftPath) {
  //         const modsResult = await getInstalledMods(minecraftPath, installation.id);
  //         result.modCount = modsResult?.length || 0;
          
  //         // Map to our format
  //         if (modsResult && modsResult.length > 0) {
  //           result.modsList = modsResult.map((mod: any) => ({
  //             name: mod.name || mod.file_name || 'Unknown',
  //             fileName: mod.file_name || mod.name || 'unknown.jar',
  //             enabled: mod.enabled !== false // Default to enabled if not specified
  //           }));
  //         }
  //       }
  //     } catch (modError) {
  //       console.warn('Failed to get mod count:', modError);
  //       // Not critical - mod count is just extra info
  //     }
      
  //     return result;
  //   } catch (error) {
  //     console.error('Failed to analyze installation:', error);
      
  //     // Fallback to original configuration-based detection
  //     if (installation.mod_loader && installation.mod_loader !== 'vanilla') {
  //       result.detectedLoaders.push(installation.mod_loader);
  //       result.modLoaderType = installation.mod_loader as any;
  //       if (installation.loader_version) {
  //         result.loaderVersion = installation.loader_version;
  //       }
  //       result.hasActiveMods = true;
  //     }
      
  //     return result;
  //   }
  // }

  /**
   * Get a user-friendly description of the modding status
   */
  static getModdingStatusDescription(detection: ModDetectionResult): string {
    if (!detection.hasActiveMods) {
      return 'Vanilla Minecraft';
    }

    let description = detection.modLoaderType.charAt(0).toUpperCase() + detection.modLoaderType.slice(1);
    
    if (detection.loaderVersion) {
      description += ` ${detection.loaderVersion}`;
    }

    if (detection.modCount > 0) {
      description += ` (${detection.modCount} mods)`;
    }

    return description;
  }

  /**
   * Get appropriate icon for the mod loader
   */
  static getModLoaderIcon(modLoaderType: string): string {
    switch (modLoaderType) {
      case 'fabric':
        return 'fabric'; // Assuming you have a fabric icon
      case 'forge':
        return 'hammer'; // Forge uses hammer icon
      case 'quilt':
        return 'quilt'; // Assuming you have a quilt icon
      case 'neoforge':
        return 'neo'; // Assuming you have a neoforge icon
      default:
        return 'cube'; // Vanilla Minecraft
    }
  }

  /**
   * Get appropriate color for the mod loader
   */
  static getModLoaderColor(modLoaderType: string): string {
    switch (modLoaderType) {
      case 'fabric':        return '#dbb866'; // Fabric's golden color
      case 'forge':         return '#1e2328'; // Forge's dark color
      case 'quilt':         return '#9c5aa0'; // Quilt's purple color
      case 'neoforge':      return '#f16436'; // NeoForge's orange color
      case 'iris_fabric':   return '#4c8cff'; // Iris Fabric's blue color
      default:              return '#28a745'; // Green for vanilla
    }
  }
}
