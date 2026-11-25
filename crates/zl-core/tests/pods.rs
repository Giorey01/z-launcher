//! Integration tests for Pods operations through the Service.

use std::error::Error;
use std::fs;
use std::path::PathBuf;

use zl_core::Service;

type Res = Result<(), Box<dyn Error>>;

fn fixtures_dir() -> Result<PathBuf, Box<dyn Error>> {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("no parent 1")?
        .parent()
        .ok_or("no parent 2")?
        .join("fixtures")
        .join("pods");
    Ok(p)
}

#[test]
fn test_service_pod_lifecycle() -> Res {
    let dir = tempfile::tempdir()?;
    let svc = Service::open(dir.path())?;
    let fix_dir = fixtures_dir()?;

    // 0. External pods setting defaults to false (§6)
    assert!(!svc.external_pods_enabled()?);

    // 1. Parse remote index
    let index_bytes = fs::read(fix_dir.join("indexes/valid.json"))?;
    let index_view = svc.parse_pod_index(&index_bytes)?;
    assert_eq!(index_view.schema, 1);
    assert!(!index_view.entries.is_empty());

    let entry = &index_view.entries[0];
    let package_path = fix_dir.join("packages/valid.zlpod");

    // 2. Inspect package
    let preview = svc.inspect_pod(&package_path)?;
    assert_eq!(preview.id, entry.id);
    assert_eq!(preview.signer, entry.signer);

    // 3. Install package
    let installed = svc.install_pod(&package_path, Some(&entry.sha256), Some(&entry.signer))?;
    assert_eq!(installed.id, entry.id);
    assert_eq!(installed.version, "1.2.0");
    assert_eq!(installed.state, "active");

    // 4. List active pods is empty when external pods disabled
    assert_eq!(svc.list_active_pods()?.len(), 0);
    assert_eq!(svc.list_pods()?.len(), 1);

    // Enable external pods
    svc.set_external_pods_enabled(true)?;
    assert!(svc.external_pods_enabled()?);
    let active = svc.list_active_pods()?;
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].id, entry.id);

    // 5. Update with higher version by same signer succeeds (§6)
    let update_path = fix_dir.join("packages/valid-v1.3.0.zlpod");
    let updated = svc.install_pod(&update_path, None, None)?;
    assert_eq!(updated.id, entry.id);
    assert_eq!(updated.version, "1.3.0");

    // 6. Update with different signer is rejected with SIGNER_CHANGED (§4, §6)
    let hostile_update_path = fix_dir.join("packages/valid-signed-by-b.zlpod");
    let res = svc.install_pod(&hostile_update_path, None, None);
    let err_str = match res {
        Err(e) => format!("{e:?}"),
        Ok(v) => return Err(format!("expected SIGNER_CHANGED, got {v:?}").into()),
    };
    assert!(err_str.contains("SIGNER_CHANGED"));

    // 7. Disable and Enable
    assert!(svc.set_pod_enabled(&entry.id, false)?);
    assert_eq!(svc.list_active_pods()?.len(), 0);
    assert_eq!(svc.list_pods()?.len(), 1);

    assert!(svc.set_pod_enabled(&entry.id, true)?);
    assert_eq!(svc.list_active_pods()?.len(), 1);

    // 8. Crash threshold
    assert!(!svc.record_pod_crash(&entry.id, Some("err 1"))?);
    assert!(!svc.record_pod_crash(&entry.id, Some("err 2"))?);
    assert!(svc.record_pod_crash(&entry.id, Some("err 3"))?);

    let pods = svc.list_pods()?;
    assert_eq!(pods[0].state, "disabled_by_crashes");

    // 9. Reset health
    svc.reset_pod_health(&entry.id)?;
    let pods = svc.list_pods()?;
    assert_eq!(pods[0].state, "active");

    // 10. Uninstall
    assert!(svc.uninstall_pod(&entry.id)?);
    assert!(svc.list_pods()?.is_empty());

    Ok(())
}

#[test]
fn test_index_hash_mismatch_rejected() -> Res {
    let dir = tempfile::tempdir()?;
    let svc = Service::open(dir.path())?;
    let fix_dir = fixtures_dir()?;

    let index_bytes = fs::read(fix_dir.join("indexes/install-hash-mismatch.json"))?;
    let index_view = svc.parse_pod_index(&index_bytes)?;
    let entry = &index_view.entries[0];
    let package_path = fix_dir.join("packages/valid.zlpod");

    let res = svc.install_pod(&package_path, Some(&entry.sha256), Some(&entry.signer));
    let err_str = match res {
        Err(e) => format!("{e:?}"),
        Ok(v) => return Err(format!("expected HASH_MISMATCH, got {v:?}").into()),
    };
    assert!(err_str.contains("HASH_MISMATCH"));

    Ok(())
}

#[test]
fn test_index_signer_mismatch_rejected() -> Res {
    let dir = tempfile::tempdir()?;
    let svc = Service::open(dir.path())?;
    let fix_dir = fixtures_dir()?;

    let index_bytes = fs::read(fix_dir.join("indexes/install-signer-mismatch.json"))?;
    let index_view = svc.parse_pod_index(&index_bytes)?;
    let entry = &index_view.entries[0];
    let package_path = fix_dir.join("packages/valid.zlpod");

    let res = svc.install_pod(&package_path, Some(&entry.sha256), Some(&entry.signer));
    let err_str = match res {
        Err(e) => format!("{e:?}"),
        Ok(v) => return Err(format!("expected SIGNER_MISMATCH, got {v:?}").into()),
    };
    assert!(err_str.contains("SIGNER_MISMATCH"));

    Ok(())
}
