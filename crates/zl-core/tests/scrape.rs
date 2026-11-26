//! Downloading art from SteamGridDB over a mock transport: only the missing kinds are asked, the
//! files land in the data folder and become the game's art, and a refused or missing key stops.

use std::error::Error;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::Duration;

use zl_core::{ImageScrapeReport, KeyCheck, ScrapeProgress, ScrapeStop, ScrapeTarget, Service};
use zl_library::NoProgress;
use zl_net::MockHttpClient;

/// Result type of the tests.
type Res = Result<(), Box<dyn Error>>;

/// A key of a plausible shape.
const KEY: &str = "0123456789abcdef0123456789abcdef";

/// Counts the progress calls and cancels when asked to.
#[derive(Default)]
struct Progress {
    /// Games reported finished.
    done: AtomicU32,
    /// Answer of `is_cancelled`.
    cancel: AtomicBool,
}

impl ScrapeProgress for Progress {
    fn on_game(&self, done: u32, _total: u32) {
        self.done.store(done, Ordering::SeqCst);
    }
    fn is_cancelled(&self) -> bool {
        self.cancel.load(Ordering::SeqCst)
    }
}

/// A library with two GBA games, scanned, in a temporary folder.
fn library(dir: &Path) -> Result<Service, Box<dyn Error>> {
    let gba = dir.join("roms/gba");
    std::fs::create_dir_all(&gba)?;
    std::fs::write(gba.join("Advance Wars (USA).gba"), b"rom")?;
    std::fs::write(gba.join("Nothing Like This.gba"), b"rom")?;
    let svc = Service::open(&dir.join("data"))?;
    svc.add_root(&dir.join("roms").to_string_lossy(), None)?;
    svc.scan(&NoProgress)?;
    Ok(svc)
}

/// SteamGridDB knows Advance Wars (cover and logo, no hero) and nothing else.
fn mock() -> MockHttpClient {
    let http = MockHttpClient::new();
    http.register(
        "/search/autocomplete/Advance%20Wars",
        r#"{"success":true,"data":[{"id":11,"name":"Advance Wars"}]}"#,
    );
    http.register(
        "/search/autocomplete/Nothing%20Like%20This",
        r#"{"success":true,"data":[]}"#,
    );
    http.register(
        "/grids/game/11?dimensions=1024x1024",
        r#"{"success":true,"data":[{"url":"https://cdn2.steamgriddb.com/grid/aw.webp"}]}"#,
    );
    http.register("/heroes/game/11", r#"{"success":true,"data":[]}"#);
    http.register(
        "/logos/game/11",
        r#"{"success":true,"data":[{"url":"https://cdn2.steamgriddb.com/logo/aw.png"}]}"#,
    );
    http.register("https://cdn2.steamgriddb.com/grid/aw.webp", "cover-bytes");
    http.register("https://cdn2.steamgriddb.com/logo/aw.png", "logo-bytes");
    http
}

/// One run downloads the cover and the logo of the known game and reports the other as not
/// found; a second run asks nothing new for the art already there.
#[test]
fn downloads_missing_art() -> Res {
    let dir = tempfile::tempdir()?;
    let svc = library(dir.path())?;
    svc.set_steamgriddb_key(Some(KEY))?;
    let http = mock();
    let progress = Progress::default();

    let report = svc.scrape_images(&ScrapeTarget::All, &http, &progress, Duration::ZERO)?;
    let want = ImageScrapeReport {
        games: 2,
        updated: 1,
        images: 2,
        not_found: 1,
        ..ImageScrapeReport::default()
    };
    assert_eq!(report, want);
    assert_eq!(progress.done.load(Ordering::SeqCst), 2);

    let games = svc.games(Some("gba"))?;
    let aw = games
        .iter()
        .find(|g| g.display_title == "Advance Wars")
        .ok_or("game")?;
    let cover = aw.icon.as_deref().ok_or("no cover")?;
    assert!(cover.ends_with(&format!("media/sgdb/gba/icon/{}.webp", aw.id)));
    assert_eq!(std::fs::read(cover)?, b"cover-bytes");
    assert!(aw.logo.is_some());
    assert!(aw.hero.is_none());

    let before = http.requested_urls().len();
    let again = svc.scrape_images(&ScrapeTarget::Game(aw.id), &http, &progress, Duration::ZERO)?;
    assert_eq!((again.games, again.images), (1, 0));
    let asked: Vec<String> = http.requested_urls()[before..].to_vec();
    assert!(asked.iter().any(|u| u.contains("/heroes/")), "{asked:?}");
    assert!(!asked.iter().any(|u| u.contains("/grids/")), "{asked:?}");
    Ok(())
}

/// Without a key nothing is asked; a refused key stops at the first game.
#[test]
fn missing_or_refused_key_stops() -> Res {
    let dir = tempfile::tempdir()?;
    let svc = library(dir.path())?;
    let http = MockHttpClient::new();
    let progress = Progress::default();
    let report = svc.scrape_images(&ScrapeTarget::All, &http, &progress, Duration::ZERO)?;
    assert_eq!(report.stopped, Some(ScrapeStop::MissingKey));
    assert!(http.requested_urls().is_empty());

    svc.set_steamgriddb_key(Some(KEY))?;
    http.register_status("/search/", 401, "");
    let report = svc.scrape_images(&ScrapeTarget::All, &http, &progress, Duration::ZERO)?;
    assert_eq!(report.stopped, Some(ScrapeStop::KeyRefused));
    assert_eq!(http.requested_urls().len(), 1);
    assert_eq!(svc.check_steamgriddb_key(KEY, &http), KeyCheck::Refused);
    Ok(())
}

/// A cancelled run stops before the first game; a malformed key is refused before storing.
#[test]
fn cancel_and_key_shape() -> Res {
    let dir = tempfile::tempdir()?;
    let svc = library(dir.path())?;
    assert!(svc.set_steamgriddb_key(Some("short")).is_err());
    assert!(svc
        .set_steamgriddb_key(Some("has spaces in it 0123456789"))
        .is_err());
    svc.set_steamgriddb_key(Some(KEY))?;
    let progress = Progress::default();
    progress.cancel.store(true, Ordering::SeqCst);
    let http = mock();
    let report = svc.scrape_images(&ScrapeTarget::All, &http, &progress, Duration::ZERO)?;
    assert_eq!(report.stopped, Some(ScrapeStop::Cancelled));
    assert!(http.requested_urls().is_empty());
    svc.set_steamgriddb_key(None)?;
    assert_eq!(svc.scraper_config()?.steamgriddb_key, None);
    Ok(())
}

/// After a scan only the games it added are looked up: the older game SteamGridDB does not know
/// is not asked again.
#[test]
fn only_the_games_a_scan_added() -> Res {
    let dir = tempfile::tempdir()?;
    let svc = library(dir.path())?;
    svc.set_steamgriddb_key(Some(KEY))?;
    let since = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as i64
        + 1;
    std::thread::sleep(Duration::from_millis(1100));
    std::fs::write(
        dir.path().join("roms/gba/Advance Wars 2 (USA).gba"),
        b"rom2",
    )?;
    svc.scan(&NoProgress)?;
    let http = mock();
    http.register(
        "/search/autocomplete/Advance%20Wars%202",
        r#"{"success":true,"data":[]}"#,
    );
    let report = svc.scrape_images(
        &ScrapeTarget::AddedSince(since),
        &http,
        &Progress::default(),
        Duration::ZERO,
    )?;
    assert_eq!(report.games, 1);
    let asked = http.requested_urls();
    assert!(
        !asked.iter().any(|u| u.contains("Nothing%20Like%20This")),
        "{asked:?}"
    );
    Ok(())
}
