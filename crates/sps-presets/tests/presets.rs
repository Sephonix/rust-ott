use sps_dsp::params::Spec;
use sps_presets::*;
const SPECS: [Spec; 1] = [Spec {
    id: "mix",
    name: "Mix",
    min: 0.0,
    max: 100.0,
    default: 35.0,
    unit: "%",
    logarithmic: false,
    discrete: false,
}];
#[test]
fn round_trip_no_clobber_schema_isolation_and_corrupt_input() {
    let dir = std::env::temp_dir().join(format!("sps-presets-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    save(&dir, "Warm room", "room", &SPECS, &[42.0]).unwrap();
    assert_eq!(list(&dir).unwrap(), ["Warm room"]);
    assert_eq!(load(&dir, "Warm room", "room", &SPECS).unwrap(), [42.0]);
    assert!(save(&dir, "Warm room", "room", &SPECS, &[50.0]).is_err());
    assert_eq!(load(&dir, "Warm room", "room", &SPECS).unwrap(), [42.0]);
    assert!(load(&dir, "Warm room", "echo", &SPECS).is_err());
    assert!(save(&dir, "../escape", "room", &SPECS, &[50.0]).is_err());
    for text in [
        r#"{"version":2,"plugin":"room","parameters":{"mix":42}}"#,
        r#"{"version":1,"plugin":"room","parameters":{"mix":101}}"#,
        r#"{"version":1,"plugin":"room","parameters":{"mix":null}}"#,
        r#"{"version":1,"plugin":"room","parameters":{"mix":42,"unknown":1}}"#,
        "{",
        &"x".repeat(65537),
    ] {
        std::fs::write(dir.join("Broken.json"), text).unwrap();
        assert!(load(&dir, "Broken", "room", &SPECS).is_err());
    }
    assert!(!std::fs::read_dir(&dir).unwrap().any(|e| e
        .unwrap()
        .file_name()
        .to_string_lossy()
        .ends_with(".tmp")));
    std::fs::remove_dir_all(dir).unwrap();
}
