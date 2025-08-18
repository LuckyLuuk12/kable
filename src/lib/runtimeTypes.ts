/** Enum for different Minecraft loaders
 * ```ts
 * export enum Loader {
 *  Vanilla = "vanilla",
 *  Fabric = "fabric",
 *  Forge = "forge",
 *  Quilt = "quilt",
 *  NeoForge = "neoforge"
 * }
 * ```
 */
export enum Loader {
  Vanilla = "vanilla",
  Fabric = "fabric",
  Forge = "forge",
  Quilt = "quilt",
  NeoForge = "neoforge",
  IrisFabric = "iris_fabric" // Iris has its own installer.jar but no version manifest, so we treat it as a separate loader but implement fabric manifest
}

/** Enum for different mod providers
 * ```ts
 * export enum ProviderKind {
 *   Modrinth = 'Modrinth',
 *   CurseForge = 'CurseForge',
 * }
 * ```
 */
export enum ProviderKind {
  Modrinth = 'Modrinth',
  CurseForge = 'CurseForge',
}