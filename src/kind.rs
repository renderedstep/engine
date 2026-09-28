//! What sort of place a room is, how much small stuff lies about in it, and
//! which sort each room of a building is (`Location::Kind`): two closed lists
//! a model picks from and a table a building's rooms are dealt their words
//! out of, all read from `data/location/kind.yml`, which the game reads too.
//!
//! Nothing plays on these words yet: a stub keeps the ones its exits answer
//! picked, and a room of a building keeps the one it was dealt.

use crate::data;

/// `Location::Kind::KINDS`: what sort of place a room may be.
pub fn kinds() -> &'static [String] {
    &data::location_kind().kinds
}

/// `Location::Kind::DENSITIES`: how cluttered, quietest first.
pub fn densities() -> &'static [String] {
    &data::location_kind().densities
}

/// `Location::Kind::BUILDINGS`: what sort of building a place may be.
pub fn buildings() -> Vec<&'static str> {
    data::location_kind()
        .buildings
        .iter()
        .map(|(name, _)| name.as_str())
        .collect()
}

/// `Location::Kind.deal`: the word each room of a building is born with,
/// given each room's storey in the order the rooms were written, entry room
/// first. The entry room is the building's `entry`; every other room takes
/// the next word of its storey's band in turn, starting again at the top when
/// the band runs out. No die is thrown. `None` for every room when the
/// building is not one the table has.
pub fn deal(building: Option<&str>, storeys: &[i64]) -> Vec<Option<&'static str>> {
    let Some(plan) = building.and_then(|name| {
        data::location_kind()
            .buildings
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, plan)| plan)
    }) else {
        return vec![None; storeys.len()];
    };
    let mut dealt = [0usize; 3];
    storeys
        .iter()
        .enumerate()
        .map(|(index, &storey)| {
            if index == 0 {
                return Some(plan.entry.as_str());
            }
            let band = match storey.signum() {
                1 => 1,
                -1 => 2,
                _ => 0,
            };
            let words = &plan.bands[band];
            let word = &words[dealt[band] % words.len()];
            dealt[band] += 1;
            Some(word.as_str())
        })
        .collect()
}

/// One of `words` if `picked` is one of them (`Location::Generator#word`).
pub fn word<'w, S: AsRef<str>>(picked: &str, words: &'w [S]) -> Option<&'w str> {
    words.iter().map(AsRef::as_ref).find(|word| *word == picked)
}
