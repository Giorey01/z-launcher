//! Tests for Pod lifecycle, registry, health monitor, and trust store.

use std::error::Error;
use std::fs;
use tempfile::tempdir;
use zl_db::{Db, InstalledPod, PodState};
use zl_pods::{HealthMonitor, PodRegistry, TrustStore};

type Res = Result<(), Box<dyn Error>>;

#[test]
fn test_health_monitor_threshold() -> Res {
    let db = Db::open_in_memory()?;
    let health = HealthMonitor::new(&db);

    let pod = InstalledPod {
        id: "crash.test".to_string(),
        name: "Crash Test".to_string(),
        version: "1.0.0".to_string(),
        api: 1,
        author_name: "Tester".to_string(),
        signer: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        entry: "dev.test.CrashProvider".to_string(),
        description: None,
        slots: vec!["start_menu".to_string()],
        permissions: vec![],
        abis: None,
        homepage: None,
        state: PodState::Active,
        installed_at: 1000,
        updated_at: 1000,
    };
    db.save_pod(&pod)?;

    // 1st crash: below threshold
    assert!(!health.record_crash("crash.test", Some("error 1"))?);
    assert_eq!(
        db.get_pod("crash.test")?.ok_or("pod missing")?.state,
        PodState::Active
    );

    // 2nd crash: below threshold
    assert!(!health.record_crash("crash.test", Some("error 2"))?);
    assert_eq!(
        db.get_pod("crash.test")?.ok_or("pod missing")?.state,
        PodState::Active
    );

    // 3rd crash: reaches MAX_CRASHES (3) within 24h window
    assert!(health.record_crash("crash.test", Some("error 3"))?);
    assert_eq!(
        db.get_pod("crash.test")?.ok_or("pod missing")?.state,
        PodState::DisabledByCrashes
    );

    // Reset health reactivates Pod
    health.reset_health("crash.test")?;
    assert_eq!(
        db.get_pod("crash.test")?.ok_or("pod missing")?.state,
        PodState::Active
    );
    assert_eq!(db.count_recent_pod_crashes("crash.test", 0)?, 0);
    Ok(())
}

#[test]
fn test_registry_lifecycle_and_safe_mode() -> Res {
    let db = Db::open_in_memory()?;
    let temp_pods = tempdir()?;
    let registry = PodRegistry::new(&db, temp_pods.path());

    let pod = InstalledPod {
        id: "sample.reg".to_string(),
        name: "Registry Test".to_string(),
        version: "1.0.0".to_string(),
        api: 1,
        author_name: "Tester".to_string(),
        signer: "1111111111111111111111111111111111111111111111111111111111111111".to_string(),
        entry: "dev.test.RegProvider".to_string(),
        description: None,
        slots: vec!["start_menu".to_string()],
        permissions: vec![],
        abis: None,
        homepage: None,
        state: PodState::Active,
        installed_at: 1000,
        updated_at: 1000,
    };
    db.save_pod(&pod)?;

    // Create deployed folder on filesystem
    let pod_folder = temp_pods.path().join("sample.reg");
    fs::create_dir_all(&pod_folder)?;
    fs::write(pod_folder.join("test.txt"), "hello")?;

    // Verify active pods
    let active = registry.list_active()?;
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].id, "sample.reg");

    // Disable Pod
    assert!(registry.set_enabled("sample.reg", false)?);
    assert_eq!(registry.list_active()?.len(), 0);
    assert_eq!(registry.list_installed()?.len(), 1);

    // Enable Pod
    assert!(registry.set_enabled("sample.reg", true)?);
    assert_eq!(registry.list_active()?.len(), 1);

    // Safe mode crash sentinel test
    let marker_file = temp_pods.path().join(".loading_pod");
    registry.set_loading_marker(&marker_file, "sample.reg")?;
    assert!(marker_file.exists());

    let crashed_pod = registry.check_and_clear_loading_marker(&marker_file)?;
    assert_eq!(crashed_pod.as_deref(), Some("sample.reg"));
    assert!(!marker_file.exists());
    assert_eq!(
        db.get_pod("sample.reg")?.ok_or("pod missing")?.state,
        PodState::DisabledByCrashes
    );

    // Uninstall
    assert!(registry.uninstall("sample.reg")?);
    assert!(!pod_folder.exists());
    assert_eq!(registry.list_installed()?.len(), 0);
    Ok(())
}

#[test]
fn test_trust_store() -> Res {
    let db = Db::open_in_memory()?;
    let trust = TrustStore::new(&db);

    assert_eq!(trust.get_trusted_signer("sample.trust")?, None);

    let key_fp = "56475aa75463474c0285df5dbf2bcab73da651358839e9b77481b2eab107708c";
    trust.pin_trust("sample.trust", key_fp)?;

    assert_eq!(
        trust.get_trusted_signer("sample.trust")?.as_deref(),
        Some(key_fp)
    );

    assert!(trust.clear_trust("sample.trust")?);
    assert_eq!(trust.get_trusted_signer("sample.trust")?, None);
    Ok(())
}
