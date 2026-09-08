//! Zed extension for CoHDL — the RFC-019 discipline (DR-025) applied to a
//! second editor: ZERO compiler changes, a grammar plus `cohdl lsp` wiring.
//!
//! Server resolution, in order:
//! 1. the user's `lsp.cohdl.binary` setting (path + optional arguments);
//! 2. `cohdl` on the worktree's PATH (the install.sh / self-update binary);
//! 3. the newest GitHub compiler release, downloaded once per version into
//!    the extension's work directory. The asset names follow the
//!    three-place artifact contract (install.sh / `cohdl self-update` /
//!    release-cohdl.yml): `cohdl-vX.Y.Z-<target>.tar.gz`, one binary at
//!    the archive root. VS Code / explorer releases pass `--latest=false`,
//!    so "latest" is always a compiler release.

use std::fs;
use zed_extension_api::settings::LspSettings;
use zed_extension_api::{self as zed, LanguageServerId, Result};

struct CohdlExtension {
    cached_binary_path: Option<String>,
}

impl CohdlExtension {
    fn language_server_binary_path(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<String> {
        if let Some(path) = worktree.which("cohdl") {
            return Ok(path);
        }

        if let Some(path) = &self.cached_binary_path {
            if fs::metadata(path).is_ok_and(|stat| stat.is_file()) {
                return Ok(path.clone());
            }
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );
        let release = zed::latest_github_release(
            "conol-ai/cohdl",
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let (platform, arch) = zed::current_platform();
        let target = match (platform, arch) {
            (zed::Os::Mac, zed::Architecture::Aarch64) => "aarch64-apple-darwin",
            (zed::Os::Mac, zed::Architecture::X8664) => "x86_64-apple-darwin",
            (zed::Os::Linux, zed::Architecture::Aarch64) => "aarch64-unknown-linux-musl",
            (zed::Os::Linux, zed::Architecture::X8664) => "x86_64-unknown-linux-musl",
            (zed::Os::Windows, zed::Architecture::X8664) => "x86_64-pc-windows-msvc",
            (platform, arch) => {
                return Err(format!(
                    "cohdl publishes no release binary for {platform:?}/{arch:?} — \
                     install it another way and set the `lsp.cohdl.binary` setting"
                ));
            }
        };
        // The release tag is `vX.Y.Z`; guard against a bare version in case
        // the API ever normalizes it.
        let tag = if release.version.starts_with('v') {
            release.version.clone()
        } else {
            format!("v{}", release.version)
        };
        let asset_name = format!("cohdl-{tag}-{target}.tar.gz");
        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| format!("release {tag} has no asset named {asset_name}"))?;

        let version_dir = format!("cohdl-{tag}");
        let binary_name = match platform {
            zed::Os::Windows => "cohdl.exe",
            _ => "cohdl",
        };
        let binary_path = format!("{version_dir}/{binary_name}");

        if !fs::metadata(&binary_path).is_ok_and(|stat| stat.is_file()) {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            zed::download_file(
                &asset.download_url,
                &version_dir,
                zed::DownloadedFileType::GzipTar,
            )
            .map_err(|e| format!("failed to download {asset_name}: {e}"))?;

            if !matches!(platform, zed::Os::Windows) {
                zed::make_file_executable(&binary_path)?;
            }

            // One version on disk at a time — prune the rest of the work dir.
            let entries =
                fs::read_dir(".").map_err(|e| format!("failed to list working directory: {e}"))?;
            for entry in entries {
                let entry = entry.map_err(|e| format!("failed to load directory entry: {e}"))?;
                if entry.file_name().to_str() != Some(&version_dir) {
                    fs::remove_dir_all(entry.path()).ok();
                }
            }
        }

        self.cached_binary_path = Some(binary_path.clone());
        Ok(binary_path)
    }
}

impl zed::Extension for CohdlExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let binary_settings = LspSettings::for_worktree("cohdl", worktree)
            .ok()
            .and_then(|settings| settings.binary);
        if let Some(path) = binary_settings
            .as_ref()
            .and_then(|binary| binary.path.clone())
        {
            return Ok(zed::Command {
                command: path,
                args: binary_settings
                    .and_then(|binary| binary.arguments)
                    .unwrap_or_else(|| vec!["lsp".to_string()]),
                env: Default::default(),
            });
        }

        Ok(zed::Command {
            command: self.language_server_binary_path(language_server_id, worktree)?,
            args: vec!["lsp".to_string()],
            env: Default::default(),
        })
    }
}

zed::register_extension!(CohdlExtension);
