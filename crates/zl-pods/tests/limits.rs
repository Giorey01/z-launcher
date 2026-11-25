//! Package limits integration tests.

use std::error::Error;
use std::io::Cursor;
use zl_pods::{inspect_package, PodIssue};

type Res = Result<(), Box<dyn Error>>;

#[test]
fn test_package_too_large() -> Res {
    let dummy_data = b"PK\x05\x06\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0"; // empty zip
    let limit_exceeded_size = zl_config::pods::MAX_PACKAGE_SIZE_BYTES + 1;

    let res = inspect_package(Cursor::new(dummy_data), limit_exceeded_size);
    assert_eq!(res, Err(vec![PodIssue::PackageTooLarge]));
    Ok(())
}
