//! Project links (Help menu, About dialog, header GitHub button, Home screen).
//!
//! Reelwright is a fork of FilmCraft by the ArtCraft Team and contributors; its own pages come first
//! and the upstream project is credited last (see `NOTICE`).

/// The app's short name, used in the website and repository URLs.
pub const APP: &str = "reelwright";

/// The website of the fork's maintainer.
pub const WEBSITE: &str = "https://www.jonmcgee.online";
/// Reelwright's page on that website (with the live browser build).
pub const APP_PAGE: &str = "https://www.jonmcgee.online/apps/reelwright/";
/// Reelwright's source repository.
pub const GITHUB: &str = "https://github.com/game7ballJT/reelwright";
/// New issue on Reelwright's repository.
pub const ISSUES: &str = "https://github.com/game7ballJT/reelwright/issues";
/// The project this fork is based on.
pub const UPSTREAM: &str = "https://github.com/storytold/filmcraft";

/// (command id, label, url) for every link, primary first.
pub const ALL: [(&str, &str, &str); 4] = [
    ("help.github", "Reelwright on GitHub", GITHUB),
    ("help.website", "Reelwright Website", APP_PAGE),
    ("help.upstream", "Based on FilmCraft…", UPSTREAM),
    ("help.reportIssue", "Report an Issue…", ISSUES),
];

/// The URL a `help.*` link command opens.
pub fn url_for(command: &str) -> Option<&'static str> {
    ALL.iter().find(|(id, _, _)| *id == command).map(|(_, _, u)| *u)
}

/// Open `url` in the system browser (a new tab on the web).
pub fn open(ctx: &egui::Context, url: &str) {
    ctx.open_url(egui::OpenUrl::new_tab(url));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urls_follow_the_fork_scheme() {
        assert_eq!(APP_PAGE, format!("{WEBSITE}/apps/{APP}/"));
        assert_eq!(GITHUB, format!("https://github.com/game7ballJT/{APP}"));
        assert_eq!(ISSUES, format!("{GITHUB}/issues"));
        assert!(ALL.iter().all(|(id, _, u)| id.starts_with("help.") && u.starts_with("https://")));
        assert_eq!(url_for("help.github"), Some(GITHUB));
        assert_eq!(url_for("help.upstream"), Some(UPSTREAM));
        assert_eq!(url_for("help.nope"), None);
    }
}
