//
// Here we wrap all commands which are generated with Specta into api.ts such that we can use them more easily with the Result type and unwrap() function.
// the api constant should look something like:
// ```ts
// export const api = {
//   getSettings: () => unwrapAsync(commands.getSettings()),
//   getProfiles: () => unwrapAsync(commands.getProfiles()),
// };
// ```
// We copy the commands manually each time here to make IDE tab completion do most heavy lifting...
// ```ts
// export const commands = {
// 	startAuthentication: () => typedError<DeviceCodeResponse, string>(__TAURI_INVOKE("start_authentication")),
// 	pollAuthentication: (deviceCode: string) => typedError<MicrosoftToken, string>(__TAURI_INVOKE("poll_authentication", { deviceCode })),
// 	listAccounts: () => typedError<KableAccount[], string>(__TAURI_INVOKE("list_accounts")),
// 	addAccount: (account: KableAccount) => typedError<null, string>(__TAURI_INVOKE("add_account", { account })),
// 	removeAccount: (account: KableAccount) => typedError<KableAccount[], string>(__TAURI_INVOKE("remove_account", { account })),
// 	setActiveAccount: (account: KableAccount) => typedError<null, string>(__TAURI_INVOKE("set_active_account", { account })),
// 	getActiveAccount: () => typedError<KableAccount, string>(__TAURI_INVOKE("get_active_account")),
// 	/**  Initialize Discord Rich Presence */
// 	initializeDiscordRpc: () => typedError<null, string>(__TAURI_INVOKE("initialize_discord_rpc")),
// 	/**  Enable or disable Discord Rich Presence globally */
// 	setDiscordEnabled: (enabled: boolean) => typedError<null, string>(__TAURI_INVOKE("set_discord_enabled", { enabled })),
// 	/**  Set presence to "playing Minecraft" */
// 	setDiscordPlaying: (profile: KableProfile) => typedError<null, string>(__TAURI_INVOKE("set_discord_playing", { profile })),
// 	/**  Set presence to browsing a launcher section */
// 	setDiscordBrowsing: (section: string) => typedError<null, string>(__TAURI_INVOKE("set_discord_browsing", { section })),
// 	/**  Clear "playing" state (revert to idle / launcher state) */
// 	clearDiscordPlaying: () => typedError<null, string>(__TAURI_INVOKE("clear_discord_playing")),
// 	/**  Clear all Discord presence (hard reset) */
// 	clearDiscordPresence: () => typedError<null, string>(__TAURI_INVOKE("clear_discord_presence")),
// 	/**  Disconnect completely from Discord IPC */
// 	disconnectDiscord: () => typedError<null, string>(__TAURI_INVOKE("disconnect_discord")),
// 	getCustomIconTemplates: () => typedError<CustomIconTemplate[], string>(__TAURI_INVOKE("get_custom_icon_templates")),
// 	saveCustomIconTemplate: (template: CustomIconTemplate) => typedError<string, string>(__TAURI_INVOKE("save_custom_icon_template", { template })),
// 	deleteCustomIconTemplate: (templateName: string) => typedError<null, string>(__TAURI_INVOKE("delete_custom_icon_template", { templateName })),
// 	openIconsDirectory: () => typedError<null, string>(__TAURI_INVOKE("open_icons_directory")),
// 	resolveImagePath: (key: string) => typedError<string, string>(__TAURI_INVOKE("resolve_image_path", { key })),
// 	launchGame: (profile: KableProfile) => typedError<LaunchResult, string>(__TAURI_INVOKE("launch_game", { profile })),
// 	autoDetectJava: () => typedError<string, string>(__TAURI_INVOKE("auto_detect_java")),
// 	getJavaPath: (javaPath: string | null) => typedError<string, string>(__TAURI_INVOKE("get_java_path", { javaPath })),
// 	browse: (profile: KableProfile, search: ProjectSearch, smartFilter: boolean, projectType: ProjectType) => typedError<ModrinthResults_Serialize, string>(__TAURI_INVOKE("browse", { profile, search, smartFilter, projectType })),
// 	listProjects: (profile: KableProfile, projectType: ProjectType) => typedError<KableProject_Serialize[], string>(__TAURI_INVOKE("list_projects", { profile, projectType })),
// 	removeProject: (profile: KableProfile, kableProject: KableProject_Deserialize) => typedError<KableProject_Serialize, string>(__TAURI_INVOKE("remove_project", { profile, kableProject })),
// 	downloadProject: (profile: KableProfile, project: Project_Deserialize, versionId: string | null) => typedError<KableProject_Serialize, string>(__TAURI_INVOKE("download_project", { profile, project, versionId })),
// 	enableProject: (profile: KableProfile, kableProject: KableProject_Deserialize) => typedError<KableProject_Serialize, string>(__TAURI_INVOKE("enable_project", { profile, kableProject })),
// 	disableProject: (profile: KableProfile, kableProject: KableProject_Deserialize) => typedError<KableProject_Serialize, string>(__TAURI_INVOKE("disable_project", { profile, kableProject })),
// 	toggleProject: (profile: KableProfile, kableProject: KableProject_Deserialize) => typedError<KableProject_Serialize, string>(__TAURI_INVOKE("toggle_project", { profile, kableProject })),
// 	checkForProjectUpdate: (kableProfile: KableProfile, kableProject: KableProject_Deserialize) => typedError<Project_Serialize, string>(__TAURI_INVOKE("check_for_project_update", { kableProfile, kableProject })),
// 	checkForProjectUpdates: (profile: KableProfile, projectType: ProjectType) => typedError<UpdateMap_Serialize[], string>(__TAURI_INVOKE("check_for_project_updates", { profile, projectType })),
// 	updateProject: (profile: KableProfile, kableProject: KableProject_Deserialize) => typedError<KableProject_Serialize, string>(__TAURI_INVOKE("update_project", { profile, kableProject })),
// 	updateAllProjects: (profile: KableProfile, projectType: ProjectType) => typedError<KableProject_Serialize[], string>(__TAURI_INVOKE("update_all_projects", { profile, projectType })),
// 	getProfiles: () => typedError<KableProfile[], string>(__TAURI_INVOKE("get_profiles")),
// 	getProfile: (id: string) => typedError<KableProfile, string>(__TAURI_INVOKE("get_profile", { id })),
// 	modifyProfile: (oldProfile: KableProfile, newProfile: KableProfile) => typedError<KableProfile, string>(__TAURI_INVOKE("modify_profile", { oldProfile, newProfile })),
// 	deleteProfile: (id: string) => typedError<null, string>(__TAURI_INVOKE("delete_profile", { id })),
// 	getVersions: () => typedError<Versions, string>(__TAURI_INVOKE("get_versions")),
// 	getSettings: () => typedError<CategorizedLauncherSettings, string>(__TAURI_INVOKE("get_settings")),
// 	setSettings: (settings: CategorizedLauncherSettings) => typedError<null, string>(__TAURI_INVOKE("set_settings", { settings })),
// 	listSoundpacks: () => typedError<string[], string>(__TAURI_INVOKE("list_soundpacks")),
// 	getSoundpackMetadata: (pack: string) => typedError<SoundpackMetadata, string>(__TAURI_INVOKE("get_soundpack_metadata", { pack })),
// 	loadSoundpackFile: (pack: string, file: string) => typedError<number[], string>(__TAURI_INVOKE("load_soundpack_file", { pack, file })),
// 	importSoundpackZip: (path: string) => typedError<string, string>(__TAURI_INVOKE("import_soundpack_zip", { path })),
// 	getSoundsDirectoryPath: () => typedError<string, string>(__TAURI_INVOKE("get_sounds_directory_path")),
// 	openSoundsDirectory: () => typedError<null, string>(__TAURI_INVOKE("open_sounds_directory")),
// 	getSymlinks: () => typedError<SymlinkView[], string>(__TAURI_INVOKE("get_symlinks")),
// 	createSymlink: (request: CreateSymlinkRequest) => typedError<null, string>(__TAURI_INVOKE("create_symlink", { request })),
// 	deleteSymlink: (id: string) => typedError<null, string>(__TAURI_INVOKE("delete_symlink", { id })),
// 	enableSymlink: (id: string) => typedError<null, string>(__TAURI_INVOKE("enable_symlink", { id })),
// 	disableSymlink: (id: string) => typedError<null, string>(__TAURI_INVOKE("disable_symlink", { id })),
// 	checkForUpdates: (includePrerelease: boolean) => typedError<UpdateData, string>(__TAURI_INVOKE("check_for_updates", { includePrerelease })),
// 	installUpdate: (includePrerelease: boolean) => typedError<null, string>(__TAURI_INVOKE("install_update", { includePrerelease })),
// 	downloadUpdate: (includePrerelease: boolean) => typedError<string, string>(__TAURI_INVOKE("download_update", { includePrerelease })),
// 	applyDownloadedUpdate: () => typedError<null, string>(__TAURI_INVOKE("apply_downloaded_update")),
// 	getCurrentVersion: () => typedError<string, string>(__TAURI_INVOKE("get_current_version")),
// };
// ``` 
//

// Import the types (with type prefix) and commands from the generated api.ts file
import {
  commands,
  type CategorizedLauncherSettings,
  type CreateSymlinkRequest,
  type CustomIconTemplate,
  type KableAccount,
  type KableProfile,
  type KableProject_Deserialize, type Project_Deserialize,
  type ProjectSearch, type ProjectType
} from "./api";
// Import the unwrapAsync function from the utils/result.ts file
import { unwrapAsync } from "./utils/result";

// Export the api constant with all commands wrapped in unwrapAsync

export const api = {
  startAuthentication: () => unwrapAsync(commands.startAuthentication()),
  pollAuthentication: (deviceCode: string) => unwrapAsync(commands.pollAuthentication(deviceCode)),
  listAccounts: () => unwrapAsync(commands.listAccounts()),
  addAccount: (account: KableAccount) => unwrapAsync(commands.addAccount(account)),
  removeAccount: (account: KableAccount) => unwrapAsync(commands.removeAccount(account)),
  setActiveAccount: (account: KableAccount) => unwrapAsync(commands.setActiveAccount(account)),
  getActiveAccount: () => unwrapAsync(commands.getActiveAccount()),
  initializeDiscordRpc: () => unwrapAsync(commands.initializeDiscordRpc()),
  setDiscordEnabled: (enabled: boolean) => unwrapAsync(commands.setDiscordEnabled(enabled)),
  setDiscordPlaying: (profile: KableProfile) => unwrapAsync(commands.setDiscordPlaying(profile)),
  setDiscordBrowsing: (section: string) => unwrapAsync(commands.setDiscordBrowsing(section)),
  clearDiscordPlaying: () => unwrapAsync(commands.clearDiscordPlaying()),
  clearDiscordPresence: () => unwrapAsync(commands.clearDiscordPresence()),
  disconnectDiscord: () => unwrapAsync(commands.disconnectDiscord()),
  getCustomIconTemplates: () => unwrapAsync(commands.getCustomIconTemplates()),
  saveCustomIconTemplate: (template: CustomIconTemplate) => unwrapAsync(commands.saveCustomIconTemplate(template)),
  deleteCustomIconTemplate: (templateName: string) => unwrapAsync(commands.deleteCustomIconTemplate(templateName)),
  openIconsDirectory: () => unwrapAsync(commands.openIconsDirectory()),
  resolveImagePath: (key: string) => unwrapAsync(commands.resolveImagePath(key)),
  launchGame: (profile: KableProfile) => unwrapAsync(commands.launchGame(profile)),
  autoDetectJava: () => unwrapAsync(commands.autoDetectJava()),
  getJavaPath: (javaPath: string | null) => unwrapAsync(commands.getJavaPath(javaPath)),
  browse: (profile: KableProfile, search: ProjectSearch, smartFilter: boolean, projectType: ProjectType) => unwrapAsync(commands.browse(profile, search, smartFilter, projectType)),
  listProjects: (profile: KableProfile, projectType: ProjectType) => unwrapAsync(commands.listProjects(profile, projectType)),
  removeProject: (profile: KableProfile, kableProject: KableProject_Deserialize) => unwrapAsync(commands.removeProject(profile, kableProject)),
  downloadProject: (profile: KableProfile, project: Project_Deserialize, versionId: string | null) => unwrapAsync(commands.downloadProject(profile, project, versionId)),
  enableProject: (profile: KableProfile, kableProject: KableProject_Deserialize) => unwrapAsync(commands.enableProject(profile, kableProject)),
  disableProject: (profile: KableProfile, kableProject: KableProject_Deserialize) => unwrapAsync(commands.disableProject(profile, kableProject)),
  toggleProject: (profile: KableProfile, kableProject: KableProject_Deserialize) => unwrapAsync(commands.toggleProject(profile, kableProject)),
  checkForProjectUpdate: (kableProfile: KableProfile, kableProject: KableProject_Deserialize) => unwrapAsync(commands.checkForProjectUpdate(kableProfile, kableProject)),
  checkForProjectUpdates: (profile: KableProfile, projectType: ProjectType) => unwrapAsync(commands.checkForProjectUpdates(profile, projectType)),
  updateProject: (profile: KableProfile, kableProject: KableProject_Deserialize) => unwrapAsync(commands.updateProject(profile, kableProject)),
  updateAllProjects: (profile: KableProfile, projectType: ProjectType) => unwrapAsync(commands.updateAllProjects(profile, projectType)),
  getProfiles: () => unwrapAsync(commands.getProfiles()),
  getProfile: (id: string) => unwrapAsync(commands.getProfile(id)),
  modifyProfile: (oldProfile: KableProfile, newProfile: KableProfile) => unwrapAsync(commands.modifyProfile(oldProfile, newProfile)),
  deleteProfile: (id: string) => unwrapAsync(commands.deleteProfile(id)),
  getVersions: () => unwrapAsync(commands.getVersions()),
  getSettings: () => unwrapAsync(commands.getSettings()),
  setSettings: (settings: CategorizedLauncherSettings) => unwrapAsync(commands.setSettings(settings)),
  listSoundpacks: () => unwrapAsync(commands.listSoundpacks()),
  getSoundpackMetadata: (pack: string) => unwrapAsync(commands.getSoundpackMetadata(pack)),
  loadSoundpackFile: (pack: string, file: string) => unwrapAsync(commands.loadSoundpackFile(pack, file)),
  importSoundpackZip: (path: string) => unwrapAsync(commands.importSoundpackZip(path)),
  getSoundsDirectoryPath: () => unwrapAsync(commands.getSoundsDirectoryPath()),
  openSoundsDirectory: () => unwrapAsync(commands.openSoundsDirectory()),
  getSymlinks: () => unwrapAsync(commands.getSymlinks()),
  createSymlink: (request: CreateSymlinkRequest) => unwrapAsync(commands.createSymlink(request)),
  deleteSymlink: (id: string) => unwrapAsync(commands.deleteSymlink(id)),
  enableSymlink: (id: string) => unwrapAsync(commands.enableSymlink(id)),
  disableSymlink: (id: string) => unwrapAsync(commands.disableSymlink(id)),
  checkForUpdates: (includePrerelease: boolean) => unwrapAsync(commands.checkForUpdates(includePrerelease)),
  installUpdate: (includePrerelease: boolean) => unwrapAsync(commands.installUpdate(includePrerelease)),
  downloadUpdate: (includePrerelease: boolean) => unwrapAsync(commands.downloadUpdate(includePrerelease)),
  applyDownloadedUpdate: () => unwrapAsync(commands.applyDownloadedUpdate()),
  getCurrentVersion: () => unwrapAsync(commands.getCurrentVersion()),
};
