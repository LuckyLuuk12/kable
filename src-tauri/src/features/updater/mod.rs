use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::time::Duration;

use api_types::updater::{GitHubRelease, UpdateData};
use tauri_plugin_updater::UpdaterExt;
use url::Url;

use crate::constants::KABLE_GITHUB_RELEASES;
use crate::Logger;

const GITHUB_RELEASES_API: &str = "https://api.github.com/repos/LuckyLuuk12/kable/releases";
const GITHUB_RELEASE_DOWNLOADS: &str = "https://github.com/LuckyLuuk12/kable/releases/download";

const GITHUB_API_PAGE_SIZE: usize = 100;
const GITHUB_API_USER_AGENT: &str = "kable-updater";
const HTTP_TIMEOUT: Duration = Duration::from_secs(30);

fn log_debug(message: &str) {
    Logger::debug(message, None);
}

fn log_info(message: &str) {
    Logger::info(message, None);
}

fn log_warn(message: &str) {
    Logger::warn(message, None);
}

fn log_error(message: &str) {
    Logger::error(message, None);
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReleaseVersion {
    base: semver::Version,
    nightly_build: Option<u64>,
}

impl ReleaseVersion {
    fn as_string(&self) -> String {
        match self.nightly_build {
            Some(build) => format!("{}-{build}", self.base),
            None => self.base.to_string(),
        }
    }
}

/// Extracts the version from a release name.
///
/// Examples:
/// - "Kable v0.1.9" -> "0.1.9"
/// - "Nightly 0.1.9-33" -> "0.1.9-33"
fn extract_version(name: &str) -> &str {
    let start = name.find(|character: char| character.is_ascii_digit()).unwrap_or(name.len());

    &name[start..]
}

/// Parses Kable's version format.
///
/// Stable versions have no nightly suffix. Nightly versions use a numeric
/// suffix, such as 0.1.9-33.
fn parse_release_version(version: &str) -> Option<ReleaseVersion> {
    let version = extract_version(version).trim();

    if version.is_empty() {
        return None;
    }

    let (base, nightly_build) = match version.split_once('-') {
        Some((base, suffix)) => {
            if suffix.is_empty() || suffix.contains('-') {
                return None;
            }

            let build = suffix.parse::<u64>().ok()?;

            (base, Some(build))
        }
        None => (version, None),
    };

    let base = semver::Version::parse(base).ok()?;

    Some(ReleaseVersion { base, nightly_build })
}

/// Compares versions according to Kable's release policy.
///
/// For the same base version:
/// - 1.0.2-1 is newer than 1.0.2.
/// - 1.0.2-33 is newer than 1.0.2-32.
/// - 1.0.2 is older than 1.0.2-33.
///
/// Different base versions use semantic version ordering.
fn compare_versions(current: &str, update: &str) -> Option<Ordering> {
    let current = parse_release_version(current)?;
    let update = parse_release_version(update)?;

    match update.base.cmp(&current.base) {
        Ordering::Equal => {}
        ordering => return Some(ordering),
    }

    match (current.nightly_build, update.nightly_build) {
        (None, None) => Some(Ordering::Equal),
        (None, Some(_)) => Some(Ordering::Greater),
        (Some(_), None) => Some(Ordering::Less),
        (Some(current_build), Some(update_build)) => Some(update_build.cmp(&current_build)),
    }
}

fn is_update(current: &str, update: &str) -> bool {
    matches!(compare_versions(current, update), Some(Ordering::Greater))
}

fn release_ordering(a: &ReleaseVersion, b: &ReleaseVersion) -> Ordering {
    a.base.cmp(&b.base).then_with(|| match (a.nightly_build, b.nightly_build) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Less,
        (Some(_), None) => Ordering::Greater,
        (Some(a), Some(b)) => a.cmp(&b),
    })
}

fn latest_version_string(version: &ReleaseVersion) -> String {
    version.as_string()
}

/// Builds the updater manifest URL for a GitHub release.
fn release_endpoint(tag: &str) -> Result<Url, String> {
    let mut endpoint = Url::parse(&format!("{GITHUB_RELEASE_DOWNLOADS}/"))
        .map_err(|error| format!("Invalid GitHub release download base URL: {error}"))?;

    endpoint
        .path_segments_mut()
        .map_err(|_| "GitHub release download URL cannot accept path segments".to_string())?
        .push(tag)
        .push("latest.json");

    Ok(endpoint)
}

/// Creates a configured GitHub API client.
fn github_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(HTTP_TIMEOUT)
        .user_agent(GITHUB_API_USER_AGENT)
        .build()
        .map_err(|error| format!("Failed to initialize GitHub API client: {error}"))
}

/// Fetches all published GitHub releases.
///
/// Results are explicitly paginated. Drafts are always excluded, and
/// prereleases are excluded when `include_prerelease` is false.
async fn fetch_releases(include_prerelease: bool) -> Result<Vec<GitHubRelease>, String> {
    let client = github_client()?;
    let mut releases = Vec::new();
    let mut page = 1usize;

    loop {
        let response = client
            .get(GITHUB_RELEASES_API)
            .header("Accept", "application/vnd.github+json")
            .query(&[("per_page", GITHUB_API_PAGE_SIZE), ("page", page)])
            .send()
            .await
            .map_err(|error| {
                let message = format!("Failed to request GitHub releases (page {page}): {error}");
                log_error(&message);
                message
            })?;

        let status = response.status();

        if !status.is_success() {
            let message = format!("GitHub releases API returned HTTP {status} while fetching page {page}");
            log_error(&message);
            return Err(message);
        }

        let page_releases: Vec<GitHubRelease> = response.json().await.map_err(|error| {
            let message = format!("Failed to parse GitHub releases response (page {page}): {error}");
            log_error(&message);
            message
        })?;

        let page_len = page_releases.len();
        releases.extend(page_releases);

        if page_len < GITHUB_API_PAGE_SIZE {
            break;
        }

        page = page.checked_add(1).ok_or_else(|| "GitHub release pagination overflowed".to_string())?;
    }

    let total = releases.len();

    releases.retain(|release| !release.draft && (include_prerelease || !release.prerelease));

    log_debug(&format!("Fetched {total} GitHub releases; {} remain after filtering drafts and prereleases", releases.len()));

    Ok(releases)
}

/// Selects the highest published release newer than the supplied version.
///
/// Selection is based on parsed versions, never GitHub API ordering.
async fn select_latest_release(current: &str, include_prerelease: bool) -> Result<GitHubRelease, String> {
    if parse_release_version(current).is_none() {
        let message = format!("Cannot check for updates: current version '{current}' is invalid");
        log_error(&message);
        return Err(message);
    }

    let releases = fetch_releases(include_prerelease).await?;
    let mut candidates: Vec<(GitHubRelease, ReleaseVersion)> = Vec::new();

    for release in releases {
        let Some(version) = parse_release_version(&release.name) else {
            log_warn(&format!("Ignoring release '{}' (tag '{}'): unsupported version format", release.name, release.tag_name));
            continue;
        };

        if is_update(current, &release.name) {
            candidates.push((release, version));
        }
    }

    let (latest, latest_version) = candidates.into_iter().max_by(|(_, a), (_, b)| release_ordering(a, b)).ok_or_else(|| {
        let message =
            format!("No update available for version {current}{}", if include_prerelease { " (including prereleases)" } else { "" });
        log_info(&message);
        message
    })?;

    log_info(&format!(
        "Selected update {} (tag '{}') for installed version {}",
        latest_version_string(&latest_version),
        latest.tag_name,
        current
    ));

    Ok(latest)
}

/// Selects the latest published nightly for the current Cargo base version,
/// along with a simulated older version for development update-check testing.
///
/// If multiple nightlies exist, the simulated installed version is the
/// second-highest nightly. If only one exists, the stable base version is
/// used as the simulated installed version.
async fn select_development_update() -> Result<(GitHubRelease, String), String> {
    let base_version = semver::Version::parse(env!("CARGO_PKG_VERSION"))
        .map_err(|error| format!("Invalid Cargo package version '{}': {error}", env!("CARGO_PKG_VERSION")))?;

    let releases = fetch_releases(true).await?;
    let mut nightlies: Vec<(GitHubRelease, ReleaseVersion)> = Vec::new();

    for release in releases {
        let Some(version) = parse_release_version(&release.name) else {
            continue;
        };

        if version.base != base_version || version.nightly_build.is_none() {
            continue;
        }

        nightlies.push((release, version));
    }

    nightlies.sort_by(|(_, a), (_, b)| release_ordering(a, b));

    let Some((latest_release, latest_version)) = nightlies.pop() else {
        let message = format!("No published nightly releases found for development base version {base_version}");
        log_warn(&message);
        return Err(message);
    };

    let simulated_current = nightlies.last().map(|(_, version)| version.as_string()).unwrap_or_else(|| base_version.to_string());

    log_info(&format!(
        "Development update-check simulation: installed version {simulated_current}, \
         available release {} (tag '{}')",
        latest_version.as_string(),
        latest_release.tag_name
    ));

    Ok((latest_release, simulated_current))
}

/// Builds an updater configured for the selected release.
///
/// `comparison_version` overrides the binary's actual version for update
/// comparison. Development checks use this to simulate an older installation.
fn build_updater(
    app: &tauri::AppHandle,
    release: &GitHubRelease,
    comparison_version: &str,
) -> Result<tauri_plugin_updater::Updater, String> {
    let endpoint = release_endpoint(&release.tag_name)?;

    log_debug(&format!("Using updater manifest endpoint: {endpoint}; comparison version: {comparison_version}"));

    let mut builder = app
        .updater_builder()
        .endpoints(vec![endpoint])
        .map_err(|error| format!("Failed to configure updater endpoint for release '{}': {error}", release.tag_name))?;

    let comparison_version = comparison_version.to_string();

    builder = builder.version_comparator(move |_current, update| is_update(&comparison_version, &update.version.to_string()));

    builder.build().map_err(|error| format!("Failed to initialize updater: {error}"))
}

/// Checks whether an update is available.
///
/// In development builds, this intentionally simulates an older installed
/// nightly so the frontend can exercise its update notification flow.
/// This does not permit downloading or installing the update.
pub async fn check_for_updates(include_prerelease: bool) -> Result<UpdateData, String> {
    let (release, comparison_version) = if cfg!(debug_assertions) {
        log_info(&format!("Running development update check; include_prerelease={include_prerelease}"));

        select_development_update().await?
    } else {
        let current = env!("CARGO_PKG_VERSION").to_string();
        let release = select_latest_release(&current, include_prerelease).await?;

        (release, current)
    };

    let app = crate::app_handle()?;
    let updater = build_updater(&app, &release, &comparison_version)?;

    match updater.check().await {
        Ok(Some(update)) => {
            let version = update.version.to_string();

            log_info(&format!(
                "Update available: {comparison_version} -> {version}{}",
                if cfg!(debug_assertions) { " (development simulation; installation is disabled)" } else { "" }
            ));

            Ok(UpdateData {
                version: update.version,
                date: update.date.map(|date| date.to_string()),
                body: update.body.unwrap_or_default(),
                current_version: comparison_version,
            })
        }
        Ok(None) => {
            let message = format!(
                "The selected release '{}' did not provide an applicable update. \
                 Check that its latest.json version matches the release name and \
                 is newer than comparison version {}.",
                release.name, comparison_version
            );

            log_warn(&message);
            Err(message)
        }
        Err(error) => {
            let message = format!("Failed to check update manifest for release '{}' (tag '{}'): {error}", release.name, release.tag_name);

            log_error(&message);
            Err(message)
        }
    }
}

/// Downloads and installs the selected update immediately.
///
/// This operation is deliberately unavailable in development builds.
pub async fn install_update(include_prerelease: bool) -> Result<(), String> {
    if cfg!(debug_assertions) {
        let message = "Update installation is disabled in development builds. Use a packaged build to test installation.";
        log_warn(message);
        return Err(message.to_string());
    }

    let current = env!("CARGO_PKG_VERSION").to_string();
    let release = select_latest_release(&current, include_prerelease).await?;

    let app = crate::app_handle()?;
    let updater = build_updater(&app, &release, &current)?;

    match updater.check().await {
        Ok(Some(update)) => {
            let version = update.version.to_string();

            log_info(&format!("Installing update {current} -> {version}"));

            update.download_and_install(|_, _| {}, || {}).await.map_err(|error| {
                let message = format!("Failed to download or install update {version}: {error}");
                log_error(&message);
                message
            })?;

            log_info(&format!("Update {version} installed successfully"));
            Ok(())
        }
        Ok(None) => {
            let message = format!("No applicable update found in release '{}'", release.name);
            log_warn(&message);
            Err(message)
        }
        Err(error) => {
            let message = format!("Failed to check update before installation for release '{}': {error}", release.name);
            log_error(&message);
            Err(message)
        }
    }
}

/// Downloads an update and saves it for installation on the next restart.
///
/// This operation is deliberately unavailable in development builds.
pub async fn download_update(include_prerelease: bool) -> Result<String, String> {
    if cfg!(debug_assertions) {
        let message = "Update downloads are disabled in development builds. Use a packaged build to test downloading.";
        log_warn(message);
        return Err(message.to_string());
    }

    let current = env!("CARGO_PKG_VERSION").to_string();
    let release = select_latest_release(&current, include_prerelease).await?;

    let app = crate::app_handle()?;
    let updater = build_updater(&app, &release, &current)?;

    match updater.check().await {
        Ok(Some(update)) => {
            let version = update.version.to_string();

            log_info(&format!("Downloading update {current} -> {version}"));

            let downloaded_bytes = update.download(|_, _| {}, || {}).await.map_err(|error| {
                let message = format!("Failed to download update {version} from release '{}': {error}", release.name);
                log_error(&message);
                message
            })?;

            if downloaded_bytes.is_empty() {
                let message = format!("Downloaded update {version} contains no data");
                log_error(&message);
                return Err(message);
            }

            let launcher_dir = crate::system::fs::launcher_dir()?;
            let filename = format!("kable-installer-{version}.bin");
            let download_path = launcher_dir.join(&filename);

            crate::system::fs::write(&download_path, &downloaded_bytes, false).await.map_err(|error| {
                let message = format!("Failed to save downloaded update to '{}': {error}", download_path.display());
                log_error(&message);
                message
            })?;

            let pending_path = launcher_dir.join(crate::constants::PENDING_UPDATE_FILE);

            let payload = serde_json::json!({
                "installer_path": download_path.display().to_string(),
                "version": version
            });

            let serialized_payload =
                serde_json::to_vec_pretty(&payload).map_err(|error| format!("Failed to serialize pending update metadata: {error}"))?;

            crate::system::fs::write(&pending_path, &serialized_payload, false).await.map_err(|error| {
                let message = format!(
                    "Downloaded update was saved to '{}', but pending update metadata \
                         could not be written to '{}': {error}",
                    download_path.display(),
                    pending_path.display()
                );
                log_error(&message);
                message
            })?;

            log_info(&format!("Update {version} downloaded to '{}' and scheduled for installation", download_path.display()));

            Ok(download_path.display().to_string())
        }
        Ok(None) => {
            let message = format!("No applicable update found in release '{}'", release.name);
            log_warn(&message);
            Err(message)
        }
        Err(error) => {
            let message = format!("Failed to check update before downloading release '{}': {error}", release.name);
            log_error(&message);
            Err(message)
        }
    }
}

/// Validates the saved installer path before executing it.
fn validate_installer_path(installer_path: &Path, launcher_dir: &Path) -> Result<(), String> {
    if !installer_path.is_absolute() {
        return Err(format!("Pending installer path must be absolute: '{}'", installer_path.display()));
    }

    let canonical_launcher_dir = std::fs::canonicalize(launcher_dir)
        .map_err(|error| format!("Failed to resolve launcher directory '{}': {error}", launcher_dir.display()))?;

    let canonical_installer_path = std::fs::canonicalize(installer_path)
        .map_err(|error| format!("Failed to resolve installer path '{}': {error}", installer_path.display()))?;

    if !canonical_installer_path.starts_with(&canonical_launcher_dir) {
        return Err(format!(
            "Pending installer path '{}' is outside the launcher directory '{}'",
            canonical_installer_path.display(),
            canonical_launcher_dir.display()
        ));
    }

    if !canonical_installer_path.is_file() {
        return Err(format!("Downloaded installer does not exist or is not a file: '{}'", canonical_installer_path.display()));
    }

    Ok(())
}

/// Starts the installer for a previously downloaded update and exits the app.
///
/// This operation is deliberately unavailable in development builds.
pub async fn apply_downloaded_update() -> Result<(), String> {
    if cfg!(debug_assertions) {
        let message = "Applying a downloaded update is disabled in development builds.";
        log_warn(message);
        return Err(message.to_string());
    }

    let launcher_dir = crate::system::fs::launcher_dir()?;
    let pending_path = launcher_dir.join(crate::constants::PENDING_UPDATE_FILE);

    if !pending_path.exists() {
        return Err("No downloaded update is pending installation".into());
    }

    let contents = crate::system::fs::read_str(&pending_path)
        .await
        .map_err(|error| format!("Failed to read pending update metadata '{}': {error}", pending_path.display()))?;

    let payload: serde_json::Value = serde_json::from_str(&contents)
        .map_err(|error| format!("Pending update metadata '{}' contains invalid JSON: {error}", pending_path.display()))?;

    let installer = payload
        .get("installer_path")
        .and_then(|value| value.as_str())
        .ok_or_else(|| format!("Pending update metadata '{}' is missing a valid installer_path", pending_path.display()))?;

    let installer_path = PathBuf::from(installer);
    validate_installer_path(&installer_path, &launcher_dir)?;

    let version = payload.get("version").and_then(|value| value.as_str()).unwrap_or("unknown");

    log_info(&format!("Starting downloaded installer for version {version}: '{}'", installer_path.display()));

    match std::process::Command::new(&installer_path).spawn() {
        Ok(_) => {
            if let Err(error) = crate::system::fs::remove_file(&pending_path).await {
                log_warn(&format!(
                    "Installer was started, but pending metadata '{}' could not be removed: {error}",
                    pending_path.display()
                ));
            }

            std::process::exit(0);
        }
        Err(error) => {
            let message = format!("Failed to start downloaded installer '{}': {error}", installer_path.display());
            log_error(&message);
            Err(message)
        }
    }
}

/// Checks for a pending update during application startup.
///
/// Debug builds never launch pending installers.
pub fn launch_pending_update() -> Result<(), String> {
    if cfg!(debug_assertions) {
        log_debug("Skipping pending-update launch in development mode");
        return Ok(());
    }

    let launcher_dir = crate::system::fs::launcher_dir()?;
    let pending_path = launcher_dir.join(crate::constants::PENDING_UPDATE_FILE);

    if !pending_path.exists() {
        return Ok(());
    }

    let contents = std::fs::read_to_string(&pending_path)
        .map_err(|error| format!("Failed to read pending update metadata '{}': {error}", pending_path.display()))?;

    let payload: serde_json::Value = serde_json::from_str(&contents)
        .map_err(|error| format!("Pending update metadata '{}' contains invalid JSON: {error}", pending_path.display()))?;

    let installer = payload
        .get("installer_path")
        .and_then(|value| value.as_str())
        .ok_or_else(|| format!("Pending update metadata '{}' is missing a valid installer_path", pending_path.display()))?;

    let installer_path = PathBuf::from(installer);
    validate_installer_path(&installer_path, &launcher_dir)?;

    let version = payload.get("version").and_then(|value| value.as_str()).unwrap_or("unknown");

    log_info(&format!("Launching pending installer for version {version}: '{}'", installer_path.display()));

    std::process::Command::new(&installer_path).spawn().map_err(|error| {
        let message = format!("Failed to launch pending installer '{}': {error}", installer_path.display());
        log_error(&message);
        message
    })?;

    if let Err(error) = std::fs::remove_file(&pending_path) {
        log_warn(&format!("Installer was launched, but pending metadata '{}' could not be removed: {error}", pending_path.display()));
    }

    std::process::exit(0);
}

/// Returns the current version.
///
/// In release builds, this returns the version embedded in Cargo metadata.
///
/// In development builds, it returns the next predicted nightly number based
/// on the highest published nightly for this exact base version.
pub async fn get_current_version() -> Result<String, String> {
    let base_version = env!("CARGO_PKG_VERSION");

    if !cfg!(debug_assertions) {
        log_debug(&format!("Current Kable version: {base_version}"));
        return Ok(base_version.to_string());
    }

    let base = semver::Version::parse(base_version).map_err(|error| {
        let message = format!("Invalid Cargo package version '{base_version}': {error}");
        log_error(&message);
        message
    })?;

    let client = github_client()?;
    let mut page = 1usize;
    let mut highest_build = 0u64;

    loop {
        let response = client
            .get(KABLE_GITHUB_RELEASES)
            .header("Accept", "application/vnd.github+json")
            .query(&[("per_page", GITHUB_API_PAGE_SIZE), ("page", page)])
            .send()
            .await
            .map_err(|error| {
                let message = format!("Failed to fetch GitHub releases while computing the development version (page {page}): {error}");
                log_error(&message);
                message
            })?;

        let status = response.status();

        if !status.is_success() {
            let message = format!("GitHub releases API returned HTTP {status} while computing the development version (page {page})");
            log_error(&message);
            return Err(message);
        }

        let releases: Vec<serde_json::Value> = response.json().await.map_err(|error| {
            let message = format!("Failed to parse GitHub releases while computing the development version: {error}");
            log_error(&message);
            message
        })?;

        if releases.is_empty() {
            break;
        }

        for release in &releases {
            if release["draft"].as_bool().unwrap_or(true) {
                continue;
            }

            let Some(name) = release["name"].as_str() else {
                continue;
            };

            let Some(version) = parse_release_version(name) else {
                continue;
            };

            if version.base != base {
                continue;
            }

            if let Some(build) = version.nightly_build {
                highest_build = highest_build.max(build);
            }
        }

        if releases.len() < GITHUB_API_PAGE_SIZE {
            break;
        }

        page = page.checked_add(1).ok_or_else(|| "GitHub release pagination overflowed".to_string())?;
    }

    let next_build = highest_build.checked_add(1).ok_or_else(|| format!("Nightly build number overflow for version {base_version}"))?;

    let version = format!("{base_version}-{next_build}");

    log_info(&format!("Computed Kable development version: {version} (highest published nightly: {highest_build})"));

    Ok(version)
}
