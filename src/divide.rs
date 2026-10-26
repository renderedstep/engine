//! Breaking a thing apart: tearing paper or cloth, smashing glass or pottery.
//! The thing ends (`items.disposition` `divided`) and one row of what it
//! leaves begins where it was: "scraps of the letter", "shards of the jar".
//!
//! WHAT A THING IS MADE OF DECIDES IT, AND NOTHING ELSE DOES. The material's
//! row in `data/item/kits.yml` says whether it tears or smashes, what that
//! leaves and the check it takes, if any (cloth tears against strength less
//! two); a material that names neither act, and a thing whose material is
//! unknown, is never offered one. A model never sets any of it: a reader only
//! points at an attempt the room offers.
//!
//! WHAT IS OFFERED IS WHAT IS POSSIBLE FROM WHERE THE PLAYER STANDS
//! ([`offered`]): a portable thing in the player's hands or lying here, of a
//! material that allows the act, by a player whose ability reaches the
//! check at all -- a check whose target is zero or less is not offered,
//! rather than offered and refused. What it leaves is not broken apart
//! again: scraps of the letter are no letter to tear.
//!
//! A THING THE STORY STILL NEEDS IS NOT BROKEN APART, OR BURNED. While a
//! step of an open arc this game has not reached names a thing, neither act
//! is offered on it, and a line that asks for one is refused in the engine's
//! words ([`crate::refusal::still_needed`]), until arcs can branch on a
//! thing being gone (the owner's call of 2026-10-25).
//!
//! WHAT IT LEAVES IS ONE ROW, NOT ONE A SCRAP ([`pieces`]): a plural thing a
//! player can pick up, drop or burn, of the source's material and fragility,
//! one bulk lighter, with no words on it, burning where its material does.
//! It is this game's own and copies no row of the world's, so it has no
//! template; what it was broken from is kept on it instead
//! (`items.properties`, [`PIECES_OF`]).

use crate::data::Divide;
use crate::records::{flag, string, text, Row};
use crate::room::Thing;
use serde_json::Value;

/// Tearing apart: paper, cloth.
pub const TEAR: &str = "tear";
/// Smashing apart: glass, pottery.
pub const SMASH: &str = "smash";
/// The ways a thing breaks apart, in the order a room offers them.
pub const KINDS: [&str; 2] = [TEAR, SMASH];

/// The disposition of a thing broken apart, beside `consumed`, `burned` and
/// `broken`: it is in no place, and its pieces are a row of their own.
pub const DIVIDED: &str = "divided";

/// How a thing of `material` breaks apart by `kind`, or none where it does
/// not.
pub fn act(material: &str, kind: &str) -> Option<&'static Divide> {
    let rule = crate::material::rule(material)?;
    match kind {
        TEAR => rule.tear.as_ref(),
        SMASH => rule.smash.as_ref(),
        _ => None,
    }
}

/// Whether a check is in reach: none to take, or `score` less what it takes
/// off leaves something to roll under. A score of none is a body with no
/// abilities, which reaches no check.
pub fn reachable(act: &Divide, score: Option<i64>) -> bool {
    match &act.check {
        None => true,
        Some((_, penalty)) => score.is_some_and(|score| score - penalty > 0),
    }
}

/// The name of what `kind` leaves of a thing named `name`: "scraps of the
/// letter".
pub fn leaves_name(act: &Divide, name: &str) -> String {
    format!("{} of {}", act.leaves, crate::moment::definite_name(name))
}

/// Whether a thing is already what a break of its material leaves, which is
/// not broken apart again.
fn already_pieces(name: &str, material: &str) -> bool {
    KINDS
        .iter()
        .filter_map(|kind| act(material, kind))
        .any(|act| name.starts_with(&format!("{} of ", act.leaves)))
}

/// Whether `kind` is offered on a thing, by a player whose abilities are
/// `scores` (ability and score; empty for a body with none): a portable
/// thing of a material that allows it, not worn (it comes off first), not
/// already in pieces, that no open
/// beat of the story is bound to, by a player whose ability reaches the
/// check.
pub fn offered(kind: &str, thing: &Thing, scores: &[(String, i64)]) -> bool {
    possible(kind, thing, scores) && !thing.bound
}

/// [`offered`], but for the beat guard: what the room would offer if the
/// story did not still need the thing.
pub fn possible(kind: &str, thing: &Thing, scores: &[(String, i64)]) -> bool {
    let Some(material) = thing.material.as_deref() else {
        return false;
    };
    let Some(act) = act(material, kind) else {
        return false;
    };
    if thing.fixture || thing.worn || already_pieces(&thing.name, material) {
        return false;
    }
    let score = act.check.as_ref().and_then(|(ability, _)| {
        scores
            .iter()
            .find(|(name, _)| name == ability)
            .map(|(_, score)| *score)
    });
    reachable(act, score)
}

/// One bulk lighter, as far as `light`: what a thing's pieces weigh.
pub fn lighter(bulk: &str) -> &str {
    match bulk {
        "heavy" => "handy",
        "handy" | "light" => "light",
        other => other,
    }
}

/// The key of `items.properties` that names the row a game's own pieces were
/// broken from: the pieces record. A row this game began has no template,
/// and this is what it is instead.
pub const PIECES_OF: &str = "pieces_of";

/// The columns of the row a thing broken apart by `kind` leaves, before
/// where it lies: its name, a plain description, portable, the source's
/// material and fragility, one bulk lighter, no words on it, burning where
/// its material always does or the source did, and the source's id under
/// [`PIECES_OF`].
pub fn pieces(source: &Row, kind: &str) -> Option<Vec<(&'static str, Value)>> {
    let material = text(source, "material")?;
    let act = act(material, kind)?;
    let name = leaves_name(act, string(source, "name"));
    let combustible = crate::material::rule(material)
        .and_then(|rule| rule.combustible)
        .unwrap_or_else(|| flag(source, "combustible"));
    Some(vec![
        ("name", Value::from(name.as_str())),
        (
            "description",
            Value::from(format!(
                "What is left of {} after it was {}.",
                crate::moment::definite_name(string(source, "name")),
                done(kind)
            )),
        ),
        ("tier", Value::from(crate::kit::PORTABLE)),
        (
            "bulk",
            Value::from(lighter(text(source, "bulk").unwrap_or("handy"))),
        ),
        ("material", Value::from(material)),
        (
            "fragility",
            Value::from(text(source, "fragility").unwrap_or("sturdy")),
        ),
        ("combustible", Value::Bool(combustible)),
        ("readable", Value::Bool(false)),
        ("use_kind", Value::from("ordinary")),
        (
            "properties",
            Value::from(
                serde_json::json!({ PIECES_OF: source.get("id").cloned().unwrap_or(Value::Null) })
                    .to_string(),
            ),
        ),
    ])
}

/// The act done, as a sentence says it: "torn apart", "smashed".
pub fn done(kind: &str) -> &'static str {
    match kind {
        TEAR => "torn apart",
        SMASH => "smashed",
        _ => "burned",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn thing(name: &str, material: Option<&str>) -> Thing {
        Thing {
            material: material.map(str::to_string),
            ..Thing::new(1, name, true)
        }
    }

    fn strength(score: i64) -> Vec<(String, i64)> {
        vec![("strength".to_string(), score)]
    }

    #[test]
    fn paper_and_cloth_tear_glass_and_pottery_smash_and_nothing_else_does() {
        let tears: Vec<&str> = crate::material::materials()
            .into_iter()
            .filter(|material| act(material, TEAR).is_some())
            .collect();
        let smashes: Vec<&str> = crate::material::materials()
            .into_iter()
            .filter(|material| act(material, SMASH).is_some())
            .collect();
        assert_eq!(tears, ["paper", "cloth"]);
        assert_eq!(smashes, ["glass", "pottery"]);
        assert_eq!(
            act("cloth", TEAR),
            Some(&Divide {
                leaves: "strips".into(),
                check: Some(("strength".into(), 2)),
            })
        );
        assert_eq!(act("paper", TEAR).and_then(|a| a.check.as_ref()), None);
    }

    #[test]
    fn a_thing_of_no_material_or_the_wrong_one_is_offered_nothing() {
        assert!(!offered(TEAR, &thing("writ", None), &strength(12)));
        assert!(!offered(TEAR, &thing("key", Some("metal")), &strength(12)));
        assert!(!offered(
            SMASH,
            &thing("letter", Some("paper")),
            &strength(12)
        ));
        assert!(offered(
            TEAR,
            &thing("letter", Some("paper")),
            &strength(12)
        ));
        assert!(offered(SMASH, &thing("jar", Some("glass")), &[]));
    }

    #[test]
    fn a_check_out_of_reach_is_not_offered() {
        let apron = thing("apron", Some("cloth"));
        assert!(offered(TEAR, &apron, &strength(3)));
        assert!(!offered(TEAR, &apron, &strength(2)));
        assert!(!offered(TEAR, &apron, &[]));
        assert!(offered(TEAR, &thing("letter", Some("paper")), &[]));
    }

    #[test]
    fn pieces_fixed_things_and_what_the_story_needs_are_not_offered() {
        assert!(!offered(
            TEAR,
            &thing("scraps of the letter", Some("paper")),
            &strength(12)
        ));
        let banner = Thing {
            fixture: true,
            ..thing("banner", Some("cloth"))
        };
        assert!(!offered(TEAR, &banner, &strength(12)));
        let writ = Thing {
            bound: true,
            ..thing("writ", Some("paper"))
        };
        assert!(!offered(TEAR, &writ, &strength(12)));
        assert!(possible(TEAR, &writ, &strength(12)));
    }

    #[test]
    fn what_a_letter_torn_apart_leaves() {
        let letter = json!({
            "name": "a sealed letter",
            "material": "paper",
            "bulk": "light",
            "fragility": "sturdy",
            "combustible": true,
            "readable": true,
        });
        let row = letter.as_object().unwrap();
        let pieces: serde_json::Map<String, Value> = pieces(row, TEAR)
            .unwrap()
            .into_iter()
            .map(|(column, value)| (column.to_string(), value))
            .collect();
        assert_eq!(pieces["name"], "scraps of the sealed letter");
        assert_eq!(pieces["bulk"], "light");
        assert_eq!(pieces["material"], "paper");
        assert_eq!(pieces["combustible"], true);
        assert_eq!(pieces["readable"], false);
        assert_eq!(pieces["tier"], "portable");
        assert!(super::pieces(row, SMASH).is_none());
    }
}
