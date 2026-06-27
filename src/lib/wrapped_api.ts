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
// 	getActiveAccount: () => typedError<{
// 	access_token: string,
// 	access_token_expires_at: string,
// 	/**  AES-encrypted refresh token for "persistent" accounts. */
// 	encrypted_refresh_token: string | null,
// 	avatar: string,
// 	eligible_for_free_trials: boolean,
// 	eligible_for_migration: boolean,
// 	franchise_inventory_id: string,
// 	has_multiple_profiles: boolean,
// 	in_forced_migration: boolean,
// 	legacy: boolean,
// 	license_product_ids: string[],
// 	/**  This is usually the same as the user's UUID but for correctness use the Minecraft Profile's ID! */
// 	local_id: string,
// 	minecraft_profile: KableMinecraftProfile,
// 	persistent: boolean,
// 	remote_id: string,
// 	type: string,
// 	user_properties: null[],
// 	username: string,
// } | null, string>(__TAURI_INVOKE("get_active_account")),
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
// 	checkForProjectUpdate: (kableProfile: KableProfile, kableProject: KableProject_Deserialize) => typedError<{
// 	/**  The slug of a project, used for vanity URLs. Regex: ```^[\\w!@$()`.+,\"\\-']{3,64}$``` */
// 	slug: string,
// 	/**  The title or name of the project */
// 	title: string,
// 	/**  A short description of the project */
// 	description: string,
// 	/**  A list of the categories that the project has */
// 	categories?: string[] | null,
// 	/**  The client side support of the project */
// 	client_side: ClientSide,
// 	/**  The server side support of the project */
// 	server_side: ServerSide,
// 	/**  The project type of the project */
// 	project_type: ProjectType,
// 	/**  The total number of downloads of the project */
// 	downloads: number,
// 	/**  The URL of the project's icon */
// 	icon_url?: string | null,
// 	/**  The RGB color of the project, automatically generated from the project icon */
// 	color?: number | null,
// 	/**  The ID of the moderation thread associated with this project */
// 	thread_id?: string | null,
// 	monetization_status?: MonetizationStatus | null,
// 	/**  The ID of the project */
// 	project_id: string,
// 	/**  The username of the project's author */
// 	author: string,
// 	/**  A list of the categories that the project has which are not secondary */
// 	display_categories?: string[] | null,
// 	/**  A list of the minecraft versions supported by the project */
// 	versions: ProjectVersion_Serialize[],
// 	/**  The total number of users following the project */
// 	follows: number,
// 	/**  The date the project was added to search */
// 	date_created: string,
// 	/**  The date the project was last modified */
// 	date_modified: string,
// 	/**  The latest version of minecraft that this project supports */
// 	latest_version?: string | null,
// 	/**  The SPDX license ID of a project */
// 	license: string,
// 	/**  All gallery images attached to the project */
// 	gallery?: string[] | null,
// 	/**  The featured gallery image of the project */
// 	featured_gallery?: string | null,
// } | null, string>(__TAURI_INVOKE("check_for_project_update", { kableProfile, kableProject })),
// 	checkForProjectUpdates: (profile: KableProfile, projectType: ProjectType) => typedError<([KableProject_Serialize, Project_Serialize])[], string>(__TAURI_INVOKE("check_for_project_updates", { profile, projectType })),
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
// 	checkForUpdates: (includePrerelease: boolean) => typedError<{
// 	version: string,
// 	date: string | null,
// 	body: string,
// 	current_version: string,
// } | null, string>(__TAURI_INVOKE("check_for_updates", { includePrerelease })),
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
