use std::{
    fs, io,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

use anyhow::{Context as _, Result};
use gpui::{AppContext, Context, Task};
use semver::Version;
use serde::Deserialize;

const REPOSITORY: &str = "tuist/gesttalt";
const POLL_INTERVAL: Duration = Duration::from_secs(60 * 60);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AutoUpdateStatus {
    Idle,
    Checking,
    UpToDate,
    Available {
        version: Version,
        release_url: String,
        asset_name: Option<String>,
        asset_url: Option<String>,
    },
    Downloading {
        version: Version,
    },
    Installing {
        version: Version,
    },
    Updated {
        version: Version,
    },
    Errored(String),
}

pub struct AutoUpdater {
    status: AutoUpdateStatus,
    current_version: Version,
    pending_check: Option<Task<()>>,
    polling: Option<Task<()>>,
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    html_url: String,
    draft: bool,
    prerelease: bool,
    assets: Vec<GitHubReleaseAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubReleaseAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Debug)]
struct ReleaseUpdate {
    version: Version,
    release_url: String,
    asset_name: Option<String>,
    asset_url: Option<String>,
}

impl AutoUpdater {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let current_version = Version::parse(env!("CARGO_PKG_VERSION"))
            .expect("CARGO_PKG_VERSION must be a semantic version");

        let mut updater = Self {
            status: AutoUpdateStatus::Idle,
            current_version,
            pending_check: None,
            polling: None,
        };
        updater.start_polling(cx);
        updater
    }

    pub fn status(&self) -> &AutoUpdateStatus {
        &self.status
    }

    pub fn current_version(&self) -> &Version {
        &self.current_version
    }

    pub fn check_now(&mut self, cx: &mut Context<Self>) {
        self.check(false, cx);
    }

    pub fn install_update(&mut self, cx: &mut Context<Self>) {
        let (version, asset_url, asset_name) = match &self.status {
            AutoUpdateStatus::Available {
                version,
                asset_url: Some(asset_url),
                asset_name,
                ..
            } => (version.clone(), asset_url.clone(), asset_name.clone()),
            AutoUpdateStatus::Available { release_url, .. } => {
                self.status = AutoUpdateStatus::Errored(format!(
                    "No installable Gesttalt asset found. View the release at {release_url}"
                ));
                cx.notify();
                return;
            }
            _ => return,
        };

        self.status = AutoUpdateStatus::Downloading {
            version: version.clone(),
        };
        cx.notify();

        self.pending_check = Some(cx.spawn(async move |this, cx| {
            let result: Result<Option<PathBuf>> = async {
                let running_app_path = cx.update(|cx| cx.app_path())??;
                let downloaded_archive = cx
                    .background_spawn({
                        let asset_url = asset_url.clone();
                        let asset_name = asset_name.clone();
                        async move { download_release_asset(&asset_url, asset_name.as_deref()) }
                    })
                    .await?;

                this.update(cx, |this, cx| {
                    this.status = AutoUpdateStatus::Installing {
                        version: version.clone(),
                    };
                    cx.notify();
                })?;

                cx.background_spawn(async move {
                    install_release_archive(&downloaded_archive, &running_app_path)
                })
                .await
            }
            .await;

            this.update(cx, |this, cx| {
                this.pending_check = None;
                match result {
                    Ok(restart_path) => {
                        if let Some(restart_path) = restart_path {
                            cx.set_restart_path(restart_path);
                        }
                        this.status = AutoUpdateStatus::Updated { version };
                    }
                    Err(error) => {
                        this.status = AutoUpdateStatus::Errored(error.to_string());
                    }
                }
                cx.notify();
            })
            .ok();
        }));
    }

    pub fn restart(&self, cx: &mut gpui::App) {
        cx.restart();
    }

    fn start_polling(&mut self, cx: &mut Context<Self>) {
        if self.polling.is_some() {
            return;
        }

        self.polling = Some(cx.spawn(async move |this, cx| {
            loop {
                this.update(cx, |this, cx| this.check(true, cx)).ok();
                cx.background_executor().timer(POLL_INTERVAL).await;
            }
        }));
    }

    fn check(&mut self, quiet: bool, cx: &mut Context<Self>) {
        if self.pending_check.is_some() {
            return;
        }

        self.status = AutoUpdateStatus::Checking;
        cx.notify();

        let current_version = self.current_version.clone();

        self.pending_check = Some(cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { fetch_update(REPOSITORY, &current_version) })
                .await;

            this.update(cx, |this, cx| {
                this.pending_check = None;
                this.status = match result {
                    Ok(Some(update)) => AutoUpdateStatus::Available {
                        version: update.version,
                        release_url: update.release_url,
                        asset_name: update.asset_name,
                        asset_url: update.asset_url,
                    },
                    Ok(None) => {
                        if quiet {
                            AutoUpdateStatus::Idle
                        } else {
                            AutoUpdateStatus::UpToDate
                        }
                    }
                    Err(error) => {
                        if quiet {
                            log::info!("auto-update check failed: {error:#}");
                            AutoUpdateStatus::Idle
                        } else {
                            AutoUpdateStatus::Errored(error.to_string())
                        }
                    }
                };
                cx.notify();
            })
            .ok();
        }));
    }
}

fn fetch_update(repository: &str, current_version: &Version) -> Result<Option<ReleaseUpdate>> {
    let api_url = format!("https://api.github.com/repos/{repository}/releases/latest");
    let release: GitHubRelease = ureq::get(&api_url)
        .set("Accept", "application/vnd.github+json")
        .set("User-Agent", "gesttalt-auto-update")
        .call()
        .with_context(|| format!("failed to fetch latest release from {api_url}"))?
        .into_json()
        .context("failed to decode latest GitHub release")?;

    release_update_from_github_release(release, current_version)
}

fn release_update_from_github_release(
    release: GitHubRelease,
    current_version: &Version,
) -> Result<Option<ReleaseUpdate>> {
    if release.draft || release.prerelease {
        return Ok(None);
    }

    let latest_version = parse_release_version(&release.tag_name)?;
    if latest_version <= *current_version {
        return Ok(None);
    }

    let asset = preferred_asset(&release.assets);
    Ok(Some(ReleaseUpdate {
        version: latest_version,
        release_url: release.html_url,
        asset_name: asset.map(|asset| asset.name.clone()),
        asset_url: asset.map(|asset| asset.browser_download_url.clone()),
    }))
}

fn parse_release_version(tag: &str) -> Result<Version> {
    Version::parse(tag.trim_start_matches('v'))
        .with_context(|| format!("release tag {tag:?} is not a semantic version"))
}

fn preferred_asset(assets: &[GitHubReleaseAsset]) -> Option<&GitHubReleaseAsset> {
    let platform = format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH);
    assets.iter().find(|asset| {
        asset.name.starts_with("gesttalt-")
            && asset.name.contains(&platform)
            && !asset.browser_download_url.trim().is_empty()
    })
}

fn download_release_asset(asset_url: &str, asset_name: Option<&str>) -> Result<PathBuf> {
    let extension = archive_extension(asset_name.unwrap_or("gesttalt-update.tar.gz"))?;
    let response = ureq::get(asset_url)
        .set("Accept", "application/octet-stream")
        .set("User-Agent", "gesttalt-auto-update")
        .call()
        .with_context(|| format!("failed to download update from {asset_url}"))?;

    anyhow::ensure!(
        response.status() < 400,
        "failed to download update: HTTP {}",
        response.status()
    );

    let mut temp_file = tempfile::Builder::new()
        .prefix("gesttalt-auto-update-")
        .suffix(extension)
        .tempfile()?;
    let mut reader = response.into_reader();
    io::copy(&mut reader, &mut temp_file)?;
    let (_file, path) = temp_file.keep()?;
    Ok(path)
}

fn install_release_archive(
    archive_path: &Path,
    running_app_path: &Path,
) -> Result<Option<PathBuf>> {
    #[cfg(target_os = "windows")]
    {
        let _ = (archive_path, running_app_path);
        anyhow::bail!("Windows auto-update requires an update helper and is not implemented yet");
    }

    #[cfg(target_os = "macos")]
    {
        let extracted = tempfile::Builder::new()
            .prefix("gesttalt-auto-update-install-")
            .tempdir()?;
        extract_release_archive(archive_path, extracted.path())?;
        let app_bundle = find_release_app_bundle(extracted.path())?
            .context("downloaded update did not contain a Gesttalt.app bundle")?;
        replace_app_bundle(&app_bundle, running_app_path)?;
        Ok(None)
    }

    #[cfg(target_os = "linux")]
    {
        let extracted = tempfile::Builder::new()
            .prefix("gesttalt-auto-update-install-")
            .tempdir()?;
        extract_release_archive(archive_path, extracted.path())?;
        let new_binary = find_release_binary(extracted.path())?
            .context("downloaded update did not contain a gesttalt binary")?;
        replace_running_binary(&new_binary, running_app_path)?;
        Ok(None)
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        let _ = (archive_path, running_app_path);
        anyhow::bail!("auto-update is not implemented for this platform");
    }
}

fn archive_extension(asset_name: &str) -> Result<&'static str> {
    if asset_name.ends_with(".tar.gz") {
        Ok(".tar.gz")
    } else if asset_name.ends_with(".zip") {
        Ok(".zip")
    } else {
        anyhow::bail!("unsupported update archive format: {asset_name}")
    }
}

fn extract_release_archive(archive_path: &Path, destination: &Path) -> Result<()> {
    let archive_name = archive_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();

    let output = if archive_name.ends_with(".tar.gz") {
        Command::new("tar")
            .arg("-xzf")
            .arg(archive_path)
            .arg("-C")
            .arg(destination)
            .output()
            .context("failed to launch tar")?
    } else if archive_name.ends_with(".zip") {
        Command::new("unzip")
            .arg("-q")
            .arg(archive_path)
            .arg("-d")
            .arg(destination)
            .output()
            .context("failed to launch unzip")?
    } else {
        anyhow::bail!("unsupported update archive format: {archive_name}");
    };

    anyhow::ensure!(
        output.status.success(),
        "failed to extract update archive: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

#[cfg(target_os = "linux")]
fn find_release_binary(root: &Path) -> Result<Option<PathBuf>> {
    let mut entries = fs::read_dir(root)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            if let Some(binary) = find_release_binary(&path)? {
                return Ok(Some(binary));
            }
        } else if is_gesttalt_binary(&path) {
            return Ok(Some(path));
        }
    }

    Ok(None)
}

#[cfg(any(test, target_os = "macos"))]
fn find_release_app_bundle(root: &Path) -> Result<Option<PathBuf>> {
    let mut entries = fs::read_dir(root)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        if is_gesttalt_app_bundle(&path) {
            return Ok(Some(path));
        }

        if path.is_dir() {
            if let Some(app_bundle) = find_release_app_bundle(&path)? {
                return Ok(Some(app_bundle));
            }
        }
    }

    Ok(None)
}

#[cfg(any(test, target_os = "linux"))]
fn is_gesttalt_binary(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name == "gesttalt" || name == "gesttalt.exe")
}

#[cfg(any(test, target_os = "macos"))]
fn is_gesttalt_app_bundle(path: &Path) -> bool {
    path.is_dir()
        && path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.eq_ignore_ascii_case("Gesttalt.app"))
}

#[cfg(target_os = "macos")]
fn replace_app_bundle(new_app_bundle: &Path, running_app_path: &Path) -> Result<()> {
    anyhow::ensure!(
        running_app_path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("app")),
        "macOS auto-update requires Gesttalt to be running from an app bundle"
    );

    let source = path_with_trailing_slash(new_app_bundle);
    let destination = path_with_trailing_slash(running_app_path);
    let output = Command::new("rsync")
        .arg("-a")
        .arg("--delete")
        .arg(source)
        .arg(destination)
        .output()
        .context("failed to launch rsync")?;

    anyhow::ensure!(
        output.status.success(),
        "failed to install app bundle update: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

#[cfg(target_os = "macos")]
fn path_with_trailing_slash(path: &Path) -> std::ffi::OsString {
    let mut path = path.as_os_str().to_os_string();
    path.push("/");
    path
}

#[cfg(target_os = "linux")]
fn replace_running_binary(new_binary: &Path, running_app_path: &Path) -> Result<()> {
    let replacement_path = running_app_path.with_extension("new");
    fs::copy(new_binary, &replacement_path).with_context(|| {
        format!(
            "failed to copy update from {} to {}",
            new_binary.display(),
            replacement_path.display()
        )
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;

        let mode = fs::metadata(new_binary)?.permissions().mode();
        fs::set_permissions(&replacement_path, fs::Permissions::from_mode(mode | 0o755))?;
    }

    fs::rename(&replacement_path, running_app_path).with_context(|| {
        format!(
            "failed to replace running binary at {}",
            running_app_path.display()
        )
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release(tag_name: &str, draft: bool, prerelease: bool) -> GitHubRelease {
        GitHubRelease {
            tag_name: tag_name.to_string(),
            html_url: format!("https://github.com/tuist/gesttalt/releases/tag/{tag_name}"),
            draft,
            prerelease,
            assets: Vec::new(),
        }
    }

    fn asset(name: &str, url: &str) -> GitHubReleaseAsset {
        GitHubReleaseAsset {
            name: name.to_string(),
            browser_download_url: url.to_string(),
        }
    }

    #[test]
    fn parses_release_tags_with_or_without_v_prefix() {
        assert_eq!(
            parse_release_version("v1.2.3").unwrap(),
            Version::new(1, 2, 3)
        );
        assert_eq!(
            parse_release_version("1.2.3").unwrap(),
            Version::new(1, 2, 3)
        );
    }

    #[test]
    fn rejects_non_semver_release_tags() {
        let error = parse_release_version("latest").unwrap_err();

        assert!(error.to_string().contains("release tag \"latest\""));
    }

    #[test]
    fn ignores_draft_and_prerelease_releases() {
        let current = Version::new(1, 0, 0);

        assert!(
            release_update_from_github_release(release("v1.1.0", true, false), &current)
                .unwrap()
                .is_none()
        );
        assert!(
            release_update_from_github_release(release("v1.1.0", false, true), &current)
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn ignores_releases_that_are_not_newer_than_current_version() {
        let current = Version::new(1, 2, 3);

        assert!(
            release_update_from_github_release(release("v1.2.3", false, false), &current)
                .unwrap()
                .is_none()
        );
        assert!(
            release_update_from_github_release(release("v1.2.2", false, false), &current)
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn returns_release_update_for_newer_release() {
        let current = Version::new(1, 2, 3);
        let update = release_update_from_github_release(release("v1.3.0", false, false), &current)
            .unwrap()
            .unwrap();

        assert_eq!(update.version, Version::new(1, 3, 0));
        assert_eq!(
            update.release_url,
            "https://github.com/tuist/gesttalt/releases/tag/v1.3.0"
        );
        assert_eq!(update.asset_name, None);
        assert_eq!(update.asset_url, None);
    }

    #[test]
    fn picks_first_matching_platform_asset_with_download_url() {
        let platform = format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH);
        let mut github_release = release("v1.3.0", false, false);
        github_release.assets = vec![
            asset(
                "gesttalt-other-platform.tar.gz",
                "https://example.com/other",
            ),
            asset(&format!("gesttalt-v1.3.0-{platform}.tar.gz"), ""),
            asset(
                &format!("gesttalt-v1.3.0-{platform}.tar.gz"),
                "https://example.com/current",
            ),
        ];

        let update = release_update_from_github_release(github_release, &Version::new(1, 2, 3))
            .unwrap()
            .unwrap();

        assert_eq!(
            update.asset_name,
            Some(format!("gesttalt-v1.3.0-{platform}.tar.gz"))
        );
        assert_eq!(
            update.asset_url,
            Some("https://example.com/current".to_string())
        );
    }

    #[test]
    fn only_picks_gesttalt_release_assets() {
        let platform = format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH);
        let assets = vec![
            asset(
                &format!("zed-remote-server-{platform}.gz"),
                "https://example.com/zed",
            ),
            asset(
                &format!("gesttalt-v1.3.0-{platform}.tar.gz"),
                "https://example.com/gesttalt",
            ),
        ];

        let selected = preferred_asset(&assets).unwrap();

        assert_eq!(
            selected.browser_download_url,
            "https://example.com/gesttalt"
        );
    }

    #[test]
    fn recognizes_supported_archive_extensions() {
        assert_eq!(
            archive_extension("gesttalt-linux-x86_64.tar.gz").unwrap(),
            ".tar.gz"
        );
        assert_eq!(
            archive_extension("gesttalt-windows-x86_64.zip").unwrap(),
            ".zip"
        );
        assert!(archive_extension("gesttalt-linux-x86_64.gz").is_err());
    }

    #[test]
    fn recognizes_gesttalt_binaries() {
        assert!(is_gesttalt_binary(Path::new("/tmp/gesttalt")));
        assert!(is_gesttalt_binary(Path::new("/tmp/gesttalt.exe")));
        assert!(!is_gesttalt_binary(Path::new("/tmp/zed")));
    }

    #[test]
    fn recognizes_gesttalt_app_bundles() {
        let temp_dir = tempfile::tempdir().unwrap();
        let app_bundle = temp_dir.path().join("Gesttalt.app");
        let other_bundle = temp_dir.path().join("Zed.app");
        let app_bundle_file = temp_dir.path().join("NotADirectory.app");
        fs::create_dir(&app_bundle).unwrap();
        fs::create_dir(&other_bundle).unwrap();
        fs::write(&app_bundle_file, "").unwrap();

        assert!(is_gesttalt_app_bundle(&app_bundle));
        assert!(!is_gesttalt_app_bundle(&other_bundle));
        assert!(!is_gesttalt_app_bundle(&app_bundle_file));
    }
}
