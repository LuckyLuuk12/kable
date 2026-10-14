use crate::{
    integrations::minecraft::{natives::NativeResolver, versions::Library},
    tests::{runtime_generator::TestContext, server::TestServer},
};

#[tokio::test]
async fn test_asset_resolution_from_index() {
    let ctx = TestContext::new().await.unwrap();
    let server = TestServer::start(&ctx).await.unwrap();

    let contents = b"asset-data-123";

    let hash = ctx.create_asset("test/model.json", contents, "test-index").await.unwrap();

    let url = server.asset_index_url("test-index");

    let manifest = reqwest::get(&url).await.unwrap().json::<serde_json::Value>().await.unwrap();

    let entry = &manifest["objects"]["test/model.json"];

    assert_eq!(entry["hash"], hash);
    assert_eq!(entry["size"], contents.len());

    let object_url = server.assets_url(format!("objects/{}/{}", &hash[..2], hash));

    let downloaded = reqwest::get(&object_url).await.unwrap().bytes().await.unwrap();

    assert_eq!(&downloaded[..], contents);
}

#[tokio::test]
async fn test_library_maven_resolution() {
    let ctx = TestContext::new().await.unwrap();
    let server = TestServer::start(&ctx).await.unwrap();

    let jar_path = ctx.create_library_jar("net/fabricmc/test/1.0/test-1.0.jar").await.unwrap();

    let url = server.library_url("net/fabricmc/test/1.0/test-1.0.jar");

    let resp = reqwest::get(&url).await.unwrap().bytes().await.unwrap();

    let expected = tokio::fs::read(jar_path).await.unwrap();

    assert_eq!(&resp[..], &expected[..]);

    // sanity check: jar is not empty and contains manifest
    let contains_manifest = zip::ZipArchive::new(std::io::Cursor::new(resp)).unwrap().file_names().any(|n| n == "META-INF/MANIFEST.MF");

    assert!(contains_manifest);
}

#[tokio::test]
async fn test_native_extraction_and_loader_isolation() {
    let ctx = TestContext::new().await.unwrap();
    let _server = TestServer::start(&ctx).await.unwrap();

    ctx.create_native_jar("net/fabricmc/natives/1.0/natives.jar").await.unwrap();

    let mut resolver =
        NativeResolver { mc_root: ctx.mc_dir.clone(), libraries_dir: ctx.libraries_dir(), natives_dir: ctx.mc_dir.join("natives") };

    let libs = vec![Library {
        name: Some("net.fabricmc:natives:1.0".to_string()),
        url: None,
        downloads: None,
        rules: None,
        sha1: None,
        sha256: None,
        sha512: None,
        md5: None,
        size: None,
    }];

    resolver.ensure_natives(&libs, "test-loader").await.expect("native extraction failed");

    let out_dir = ctx.mc_dir.join("natives").join("test-loader");

    assert!(out_dir.exists());

    let files: Vec<_> = std::fs::read_dir(&out_dir).unwrap().map(|e| e.unwrap().file_name()).collect();

    // must extract at least the native file, not just metadata
    let has_native = files.iter().any(|f| {
        let s = f.to_string_lossy();
        s == "test.dll" || s == "test.so" || s == "test.dylib"
    });

    assert!(has_native);
}
