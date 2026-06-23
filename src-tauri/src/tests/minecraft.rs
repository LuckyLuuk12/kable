use crate::integrations::minecraft::{
    assets::AssetResolver,
    libraries::LibraryResolver,
    natives::NativeResolver,
    versions::{Library, McVersionManifest},
};
use api_types::profiles::{KableProfile, LoaderKind, ProfileVersion, VersionType};
use axum::Router; // Outside sources: standard web framework for mocking
use std::{collections::HashMap, path::PathBuf};
use std::{fs, net::SocketAddr};
use tokio::net::TcpListener;
use tower_http::services::ServeDir; // Outside sources: for serving local files

// Helper to get the local test server path [1]
fn test_server_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/tests/server")
}

async fn start_mock_server() -> SocketAddr {
    let app = Router::new()
        .nest_service("/assets", ServeDir::new(test_server_root().join("assets")))
        .nest_service("/libraries", ServeDir::new(test_server_root().join("libraries")));

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    addr
}

// Here we test crate::integrations::minecraft functionality
const TEST_MC_DIR: &str = ".minecraft";

fn test_mc_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/tests").join(TEST_MC_DIR)
}

pub fn mock_profile() -> KableProfile {
    let mut parameters_map = HashMap::new();

    parameters_map.insert("demo".to_string(), "false".to_string());

    KableProfile {
        id: "test-profile".to_string(),
        name: "Test Profile".to_string(),
        icon: None,

        version: ProfileVersion {
            id: "test-loader".to_string(),
            version_type: Some(VersionType::Release),
            loader: LoaderKind::Vanilla,
            display_name: "Test Loader".to_string(),
            minecraft_version: Some("1.21.8".to_string()),
            loader_version: None,
            stable: Some(true),
            release_time: None,
            updated_time: None,
            url: None,
            sha1: None,
            compliance_level: None,
            recommended: None,
        },

        created: "2026-01-01T00:00:00Z".to_string(),
        last_used: "2026-01-01T00:00:00Z".to_string(),

        java_args: vec!["-Xms512M".to_string(), "-Xmx1G".to_string()],

        dedicated_mods_folder: Some("mods".to_string()),
        dedicated_resource_pack_folder: None,
        dedicated_shaders_folder: None,
        dedicated_config_folder: None,

        favorite: false,
        total_time_played_ms: 0,
        times_launched: 0,

        parameters_map,

        description: Some("Test profile for launcher pipeline validation".to_string()),

        enable_pack_merging: false,
        pack_order: Vec::new(),
        merged_packs: Vec::new(),
    }
}

#[tokio::test]
async fn test_manifest_chain_resolution() {
    let base_path = test_mc_root();

    let version_path = base_path.join("versions").join("test-loader").join("test-loader.json");

    let raw = crate::system::fs::read_str(version_path).await.expect("read manifest");

    let manifest: McVersionManifest = crate::integrations::minecraft::versions::load_version_manifest(raw).await.expect("parse manifest");

    let resolved = crate::integrations::minecraft::manifest::resolve_manifest_chain(manifest).await.expect("resolve chain");

    // structural guarantees
    assert!(resolved.libraries.is_some());
    assert!(resolved.arguments.is_some());

    let libs = resolved.libraries.unwrap();
    assert!(!libs.is_empty(), "no libraries after merge");

    // stronger: ensure inheritance happened
    let lib_names: Vec<_> = libs.iter().filter_map(|l| l.name.as_deref()).collect();
    assert!(
        lib_names.iter().any(|n| n.contains("asm") || n.contains("fabric") || n.contains("minecraft")),
        "expected inherited libraries missing"
    );
}

#[tokio::test]
async fn test_library_resolution_and_download() {
    let resolver = LibraryResolver::new().expect("resolver init");

    let libs = vec![Library {
        name: Some("org.ow2.asm:asm:9.8".to_string()),
        url: Some("https://maven.fabricmc.net/".to_string()),
        downloads: None,
        rules: None,
        sha1: None,
        sha256: None,
        sha512: None,
        md5: None,
        size: None,
    }];

    resolver.ensure_downloaded(&libs).await.expect("download libs");

    let cp = resolver.resolve_classpath(&libs).expect("classpath build");

    let sep = if cfg!(windows) { ";" } else { ":" };

    // stricter: must contain jar path structure, not just "asm"
    assert!(cp.split(sep).any(|p| p.contains("org/ow2/asm") && p.ends_with(".jar")), "classpath missing resolved jar path");

    // sanity: no duplicates
    let parts: Vec<_> = cp.split(sep).collect();
    let mut unique = std::collections::HashSet::new();
    assert!(parts.iter().all(|p| unique.insert(p.to_string())));
}

#[tokio::test]
async fn test_asset_resolution() {
    // Start a mock server to serve asset index and objects for testing
    let _server_addr = start_mock_server().await;

    let resolver = AssetResolver::new().expect("asset resolver");

    let manifest_path = test_mc_root().join("versions").join("test-loader").join("test-loader.json");

    let raw = crate::system::fs::read_str(manifest_path).await.expect("read manifest");

    let manifest: McVersionManifest = crate::integrations::minecraft::versions::load_version_manifest(raw).await.expect("parse manifest");

    let result = resolver.ensure_assets(&manifest).await.expect("assets");

    assert!(result.exists());

    // stricter: index must exist
    let index_id = manifest.asset_index.as_ref().unwrap().id.as_deref().unwrap_or("unknown");
    let index_path = result.join("indexes").join(format!("{index_id}.json"));

    assert!(index_path.exists(), "missing asset index file");
}

#[tokio::test]
async fn test_native_extraction() {
    // Start a mock server to serve native libraries for testing
    let _server_addr = start_mock_server().await;

    // Now test the native resolver:
    let resolver = NativeResolver::new().expect("native resolver");

    let libs = vec![Library {
        name: Some("net.fabricmc:something-natives:1.0".to_string()),
        url: None,
        downloads: None,
        rules: None,
        sha1: None,
        sha256: None,
        sha512: None,
        md5: None,
        size: None,
    }];

    resolver.ensure_natives(&libs, "test-loader").await.expect("native extraction");

    let native_dir = resolver.natives_dir.join("test-loader");

    assert!(native_dir.exists(), "native output folder missing");

    let has_files = std::fs::read_dir(native_dir).map(|mut r| r.next().is_some()).unwrap_or(false);

    assert!(has_files, "no extracted natives");
}

// #[tokio::test]
// async fn test_full_resolution_pipeline() {
//     let profile = crate::tests::minecraft::mock_profile();

//     let result = crate::features::launcher::resolver::resolve(profile).await;

//     assert!(result.is_ok(), "pipeline failed");

//     let cmd = result.unwrap();

//     let debug = format!("{:?}", cmd);

//     assert!(debug.contains("java"));

//     // stronger: must contain version jar or main class
//     assert!(
//         debug.contains("minecraft") || debug.contains("net.minecraft") || debug.contains("fabric") || debug.contains("forge"),
//         "missing expected launch target"
//     );
// }
