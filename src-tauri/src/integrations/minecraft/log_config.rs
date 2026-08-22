use crate::integrations::minecraft::versions::types::McVersionManifest;
use crate::Logger;
use quick_xml::{
    events::{BytesStart, Event},
    Reader, Writer,
};
use sha1::{Digest, Sha1};
use std::path::{Path, PathBuf};

pub struct LogConfigResolver {
    root: PathBuf,
    kable_config_path: PathBuf,
}

impl LogConfigResolver {
    pub fn new() -> Result<Self, String> {
        let root = crate::system::fs::mc_dir()?.join(crate::constants::ASSETS_DIR).join("log_configs");

        let kable_config_path = crate::system::fs::launcher_dir()?.join("kable_log_config.xml");

        Ok(Self { root, kable_config_path })
    }

    pub async fn ensure_config(&self, manifest: &McVersionManifest) -> Result<Option<PathBuf>, String> {
        let Some(logging) = &manifest.logging else {
            return Ok(None);
        };

        let Some(client) = &logging.client else {
            return Ok(None);
        };

        let Some(file) = &client.file else {
            return Ok(None);
        };

        let Some(id) = &file.id else {
            return Err(format!("Minecraft manifest {} has a logging configuration without a file id", manifest.id));
        };

        let Some(url) = &file.url else {
            return Err(format!("Minecraft manifest {} has a logging configuration without a file URL", manifest.id));
        };

        let source_path = self.root.join(id);

        if source_path.is_file() {
            if !self.verify_file(&source_path, file.sha1.as_deref()).await? {
                Logger::debug_global(
                    format!("Logging configuration {} failed verification, downloading again", source_path.display()).as_str(),
                    None,
                );

                crate::system::fs::remove_file(&source_path).await?;
            }
        }

        if !source_path.is_file() {
            crate::system::fs::create_dir(&self.root).await?;

            Logger::debug_global(format!("Downloading Minecraft logging configuration {} from {}", id, url).as_str(), None);

            crate::system::net::download_to_file(url, &source_path).await?;

            if !source_path.is_file() {
                return Err(format!("Logging configuration download completed but {} does not exist", source_path.display()));
            }

            if !self.verify_file(&source_path, file.sha1.as_deref()).await? {
                crate::system::fs::remove_file(&source_path).await?;

                return Err(format!("Downloaded logging configuration {} failed SHA-1 verification", source_path.display()));
            }
        }

        self.create_kable_config(&source_path).await?;

        Ok(Some(self.kable_config_path.clone()))
    }

    async fn create_kable_config(&self, source_path: &Path) -> Result<(), String> {
        let source = crate::system::fs::read_str(source_path).await?;

        let config = self.modify_config(&source)?;

        let launcher_dir = crate::system::fs::launcher_dir()?;

        crate::system::fs::create_dir(&launcher_dir).await?;

        crate::system::fs::write_str(&self.kable_config_path, &config, false).await?;

        Logger::debug_global(format!("Generated Kable logging configuration at {}", self.kable_config_path.display()).as_str(), None);

        Ok(())
    }

    fn modify_config(&self, source: &str) -> Result<String, String> {
        let mut reader = Reader::from_str(source);

        reader.config_mut().trim_text(false);

        let mut writer = Writer::new(Vec::new());
        let mut buffer = Vec::new();

        let mut inside_console = false;

        loop {
            match reader.read_event_into(&mut buffer) {
                Ok(Event::Start(event)) => {
                    let name = event.name();

                    if name.as_ref() == b"Root" {
                        let mut root = event.into_owned();

                        Self::set_attribute(&mut root, b"level", b"debug");

                        writer
                            .write_event(Event::Start(root))
                            .map_err(|error| format!("Failed to write modified Root element: {}", error))?;
                    } else if name.as_ref() == b"Console" {
                        inside_console = true;

                        writer
                            .write_event(Event::Start(event.into_owned()))
                            .map_err(|error| format!("Failed to write Console element: {}", error))?;
                    } else {
                        writer
                            .write_event(Event::Start(event.into_owned()))
                            .map_err(|error| format!("Failed to write XML element: {}", error))?;
                    }
                }

                Ok(Event::Empty(event)) => {
                    let name = event.name();

                    if name.as_ref() == b"LegacyXMLLayout" && inside_console {
                        let mut pattern = BytesStart::new("PatternLayout");

                        pattern.push_attribute(("pattern", "[%d{HH:mm:ss}] [%t/%level]: %msg{nolookups}%n"));

                        writer
                            .write_event(Event::Empty(pattern))
                            .map_err(|error| format!("Failed to write replacement PatternLayout: {}", error))?;
                    } else {
                        writer
                            .write_event(Event::Empty(event.into_owned()))
                            .map_err(|error| format!("Failed to write XML empty element: {}", error))?;
                    }
                }

                Ok(Event::End(event)) => {
                    let name = event.name();

                    if name.as_ref() == b"Console" {
                        inside_console = false;
                    }

                    writer
                        .write_event(Event::End(event.into_owned()))
                        .map_err(|error| format!("Failed to write XML closing element: {}", error))?;
                }

                Ok(Event::Text(event)) => {
                    writer.write_event(Event::Text(event.into_owned())).map_err(|error| format!("Failed to write XML text: {}", error))?;
                }

                Ok(Event::CData(event)) => {
                    writer
                        .write_event(Event::CData(event.into_owned()))
                        .map_err(|error| format!("Failed to write XML CDATA: {}", error))?;
                }

                Ok(Event::Comment(event)) => {
                    writer
                        .write_event(Event::Comment(event.into_owned()))
                        .map_err(|error| format!("Failed to write XML comment: {}", error))?;
                }

                Ok(Event::Decl(event)) => {
                    writer
                        .write_event(Event::Decl(event.into_owned()))
                        .map_err(|error| format!("Failed to write XML declaration: {}", error))?;
                }

                Ok(Event::PI(event)) => {
                    writer
                        .write_event(Event::PI(event.into_owned()))
                        .map_err(|error| format!("Failed to write XML processing instruction: {}", error))?;
                }

                Ok(Event::DocType(event)) => {
                    writer
                        .write_event(Event::DocType(event.into_owned()))
                        .map_err(|error| format!("Failed to write XML doctype: {}", error))?;
                }

                Ok(Event::GeneralRef(event)) => {
                    writer
                        .write_event(Event::GeneralRef(event.into_owned()))
                        .map_err(|error| format!("Failed to write XML general reference: {}", error))?;
                }

                Ok(Event::Eof) => break,

                Err(error) => {
                    return Err(format!("Failed to parse Minecraft logging configuration: {}", error));
                }
            }

            buffer.clear();
        }

        String::from_utf8(writer.into_inner())
            .map_err(|error| format!("Generated Kable logging configuration was not valid UTF-8: {}", error))
    }

    fn set_attribute(element: &mut BytesStart<'static>, key: &[u8], value: &[u8]) {
        let attributes: Vec<(Vec<u8>, Vec<u8>)> = element
            .attributes()
            .flatten()
            .filter(|attribute| attribute.key.as_ref() != key)
            .map(|attribute| (attribute.key.as_ref().to_vec(), attribute.value.as_ref().to_vec()))
            .collect();

        element.clear_attributes();

        for (attribute_key, attribute_value) in attributes {
            element.push_attribute((attribute_key.as_slice(), attribute_value.as_slice()));
        }

        element.push_attribute((key, value));
    }

    async fn verify_file(&self, path: &Path, expected_sha1: Option<&str>) -> Result<bool, String> {
        let Some(expected_sha1) = expected_sha1 else {
            return Ok(true);
        };

        let bytes = crate::system::fs::read(path).await?;

        let mut hasher = Sha1::new();
        hasher.update(&bytes);

        let actual_sha1 = format!("{:x}", hasher.finalize());

        Ok(actual_sha1.eq_ignore_ascii_case(expected_sha1))
    }
}
