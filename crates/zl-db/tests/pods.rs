use zl_db::{Db, InstalledPod, PodState};

#[test]
fn pod_crud_and_trust_lifecycle() -> Result<(), Box<dyn std::error::Error>> {
    let db = Db::open_in_memory()?;

    let pod = InstalledPod {
        id: "sample.hello".to_string(),
        name: "Hello Pod".to_string(),
        version: "1.2.0".to_string(),
        api: 1,
        author_name: "Example Author".to_string(),
        signer: "56475aa75463474c0285df5dbf2bcab73da651358839e9b77481b2eab107708c".to_string(),
        entry: "dev.example.hello.HelloProvider".to_string(),
        description: Some("A test pod description.".to_string()),
        slots: vec!["start_menu".to_string(), "platform_grid_lead".to_string()],
        permissions: vec!["library.read".to_string()],
        abis: Some(vec!["arm64-v8a".to_string()]),
        homepage: Some("https://example.com/hello".to_string()),
        state: PodState::Active,
        installed_at: 1700000000,
        updated_at: 1700000000,
    };

    db.save_pod(&pod)?;
    db.save_pod_trust(&pod.id, &pod.signer, 1700000000)?;

    let read_back = db.get_pod("sample.hello")?.ok_or("pod must exist")?;
    assert_eq!(read_back, pod);

    let trusted_signer = db
        .get_pod_trust("sample.hello")?
        .ok_or("trust must exist")?;
    assert_eq!(trusted_signer, pod.signer);

    // Update state to DisabledByUser
    assert!(db.set_pod_state("sample.hello", PodState::DisabledByUser)?);
    let updated = db.get_pod("sample.hello")?.ok_or("pod must exist")?;
    assert_eq!(updated.state, PodState::DisabledByUser);

    // Record crashes
    db.record_pod_crash("sample.hello", 1700001000, Some("Crash 1"))?;
    db.record_pod_crash("sample.hello", 1700002000, Some("Crash 2"))?;
    db.record_pod_crash("sample.hello", 1700003000, Some("Crash 3"))?;

    let crashes = db.count_recent_pod_crashes("sample.hello", 1700001500)?;
    assert_eq!(crashes, 2);

    let all_crashes = db.count_recent_pod_crashes("sample.hello", 1700000000)?;
    assert_eq!(all_crashes, 3);

    db.clear_pod_crashes("sample.hello")?;
    assert_eq!(db.count_recent_pod_crashes("sample.hello", 0)?, 0);

    // List pods
    let list = db.list_pods()?;
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, "sample.hello");

    // Delete pod
    assert!(db.delete_pod("sample.hello")?);
    assert!(db.get_pod("sample.hello")?.is_none());
    assert!(db.get_pod_trust("sample.hello")?.is_none());

    Ok(())
}
