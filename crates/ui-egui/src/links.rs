//! Project links (Help menu, About dialog). This build is a fork of storytold/lightcraft; its links
//! point at the fork's repository.

/// The app's id (repository name).
pub const APP: &str = "lightcraft";
/// This app's source repository.
pub const GITHUB: &str = "https://github.com/685ks/lightcraft";
/// The project this build is a fork of (named in the About box).
pub const UPSTREAM: &str = "https://github.com/storytold/lightcraft";
/// The About box's provenance line. A legal notice, shown in English in every UI language.
pub const FORK_NOTE: &str = "A fork of storytold/lightcraft, not affiliated with or endorsed by ArtCraft.";
/// Shown in About when the Arthavan Projects launcher started the app (`ARTHAVAN_LAUNCHER=1`).
/// LightCraft has no updater of its own and makes no update requests; the launcher installs and
/// updates it from the 685ks/lightcraft releases.
pub const LAUNCHER_UPDATES_NOTE: &str = "Updates are managed by Arthavan Projects";

/// The user documentation (docs/ in the repository).
pub const HELP: &str = "https://github.com/685ks/lightcraft/tree/main/docs";

/// (UI command id, menu label, URL) for each link, in Help-menu order.
pub const LINKS: &[(&str, &str, &str)] =
    &[("app.help", "LightCraft Help", HELP), ("app.github", "LightCraft on GitHub", GITHUB), ("app.feedback", "Send Feedback…", FEEDBACK)];

/// Where feedback and bug reports go.
pub const FEEDBACK: &str = "https://github.com/685ks/lightcraft/issues/new";

/// The URL behind a link command id.
pub fn url_of(cmd: &str) -> Option<&'static str> {
    LINKS.iter().find(|(id, _, _)| *id == cmd).map(|(_, _, u)| *u)
}

/// Whether the Arthavan Projects launcher started this process.
pub fn launcher_managed() -> bool {
    launcher_managed_from(std::env::var("ARTHAVAN_LAUNCHER").ok().as_deref())
}

fn launcher_managed_from(value: Option<&str>) -> bool {
    value.is_some_and(|v| v.trim() == "1")
}

/// Open `url` in the user's browser (through the host's `open_url` service).
pub fn open(app: &mut crate::LightcraftApp, url: &str) -> Result<serde_json::Value, String> {
    let open = app.services.open_url.as_mut().ok_or("can't open links here")?;
    open(url)?;
    Ok(serde_json::json!({ "url": url }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_are_https_and_name_this_app() {
        for (id, label, url) in LINKS {
            assert!(url.starts_with("https://"), "{id}");
            assert!(!label.is_empty());
            assert_eq!(url_of(id), Some(*url));
        }
        assert!(GITHUB.ends_with(&format!("/685ks/{APP}")));
        assert!(UPSTREAM.ends_with(&format!("/storytold/{APP}")));
    }

    #[test]
    fn no_artcraft_links_remain() {
        for (id, label, url) in LINKS {
            for s in [id, label, url] {
                assert!(!s.to_ascii_lowercase().contains("artcraft"), "{s}");
                assert!(!s.to_ascii_lowercase().contains("discord"), "{s}");
            }
        }
        assert!(FORK_NOTE.contains("storytold/lightcraft") && FORK_NOTE.contains("not affiliated"));
    }

    #[test]
    fn launcher_flag_is_read_from_arthavan_launcher() {
        assert!(launcher_managed_from(Some("1")));
        assert!(!launcher_managed_from(None));
        assert!(!launcher_managed_from(Some("0")));
        assert!(!launcher_managed_from(Some("")));
    }
}
