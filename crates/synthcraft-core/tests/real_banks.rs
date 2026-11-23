//! Loads a real downloaded bank when `SYNTHCRAFT_TEST_BANK` points at an `.sfz` (skipped otherwise).
//! Produce one with `synthcraft-mcp fetch-bank <id>`.

use synthcraft_core::sampler::load_bank;

#[test]
fn downloaded_sfz_bank_loads_with_audible_zones() {
    let Ok(spec) = std::env::var("SYNTHCRAFT_TEST_BANK") else {
        return;
    };
    let bank = match load_bank(&spec) {
        Ok(b) => b,
        Err(e) => {
            assert!(e.to_string().is_empty(), "bank {spec} failed to load: {e}");
            return;
        }
    };
    assert!(bank.zones.len() >= 8, "only {} zones", bank.zones.len());
    for z in &bank.zones {
        let peak = z.data.iter().fold(0.0_f32, |m, v| m.max(v.abs()));
        assert!(peak > 0.01, "silent zone at root {}", z.root_key);
        assert!(z.data.len() > 4410, "zone at root {} too short", z.root_key);
    }
    let z = bank.find(60, 0.7);
    assert!(z.is_some(), "no zone for middle C");
}
