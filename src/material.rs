//! What a thing is made of (`items.material`): one of a closed list of ten
//! (`data/item/kits.yml`'s `materials`), or nothing, which is unknown and
//! plays exactly as a thing always has.
//!
//! NO ACT READS IT YET. A kit writes it from the table's `names`, a seed file
//! sets it by hand, and a game's copy of a thing brings it along; nothing
//! offered, rolled or told turns on it, and no request a model is handed
//! carries it. What it will decide -- what may be torn, smashed or cut, and
//! what that leaves -- comes with the acts that read it.
//!
//! What it does decide now is what a thing's other columns may say. A
//! material may settle whether a thing of it burns (paper always does, glass
//! never) and which fragilities it may have (glass is fragile or brittle,
//! never sturdy). The table is checked against itself as it is read, so a
//! kit never writes a contradiction; [`findings`] is what the game's doctor
//! reports of a story whose rows do.

use crate::data::{self, Material};
use crate::noticed::Finding;
use crate::records::{id, int, string, text, Records, Row};

/// The doctor's code for a thing whose material its other columns
/// contradict, or whose material is none of the list.
pub const FINDING: &str = "item_material_contradicts";

/// The materials, in the table's order.
pub fn materials() -> Vec<&'static str> {
    data::kits()
        .materials
        .iter()
        .map(|(material, _)| material.as_str())
        .collect()
}

/// What a thing of `material` always is, or none for a word that is no
/// material.
pub fn rule(material: &str) -> Option<&'static Material> {
    data::kits()
        .materials
        .iter()
        .find(|(name, _)| name == material)
        .map(|(_, rule)| rule)
}

/// The material a kit writes a thing of this name with, or none where the
/// table names it under no material.
pub fn of(name: &str) -> Option<&'static str> {
    data::kits()
        .materials
        .iter()
        .find(|(_, rule)| rule.names.iter().any(|listed| listed == name))
        .map(|(material, _)| material.as_str())
}

/// What a thing of `material`, combustible or not and of `fragility`, says
/// that the material's rule denies: one clause for each, empty when they
/// agree.
pub fn contradictions(
    material: &str,
    rule: &Material,
    combustible: bool,
    fragility: &str,
) -> Vec<String> {
    let mut wrong = Vec::new();
    match rule.combustible {
        Some(true) if !combustible => {
            wrong.push(format!("it is not combustible, and {material} always is"));
        }
        Some(false) if combustible => {
            wrong.push(format!("it is combustible, and {material} never is"));
        }
        _ => {}
    }
    if let Some(allowed) = &rule.fragility {
        if !allowed.iter().any(|word| word == fragility) {
            wrong.push(format!(
                "it is {fragility}, and {material} is only ever {}",
                allowed.join(" or ")
            ));
        }
    }
    wrong
}

/// What a row's material says against the row: nothing for a row with no
/// material, one clause for a material that is none of the list, and
/// otherwise [`contradictions`].
pub fn against(row: &Row) -> Vec<String> {
    let Some(material) = text(row, "material") else {
        return Vec::new();
    };
    let Some(rule) = rule(material) else {
        return vec![format!(
            "{}, which is none of the materials ({})",
            crate::text::inspect(material),
            materials().join(", ")
        )];
    };
    contradictions(
        material,
        rule,
        crate::records::flag(row, "combustible"),
        text(row, "fragility").unwrap_or_default(),
    )
}

/// Whether an item row is in the story, on whichever leg it sits: a room, a
/// person, or a game of the story (`Item.in_story`).
fn in_story(records: &Records, item: &Row, story: i64) -> bool {
    [
        ("locations", "location_id"),
        ("characters", "character_id"),
        ("playthroughs", "playthrough_id"),
    ]
    .iter()
    .any(|(table, column)| {
        int(item, column)
            .and_then(|other| records.find(table, other))
            .is_some_and(|other| int(other, "story_id") == Some(story))
    })
}

/// Every thing in a story, both layers, whose material its other columns
/// contradict, or whose material is none of the list. A game's copy that
/// says what its template says is the template's finding, reported once.
pub fn findings(records: &Records, story: i64) -> Vec<Finding> {
    let same = |row: &Row, other: &Row| {
        ["material", "combustible", "fragility"]
            .iter()
            .all(|column| row.get(*column) == other.get(*column))
    };
    records
        .select("items", |item| in_story(records, item, story))
        .into_iter()
        .filter(|item| {
            int(item, "template_id")
                .and_then(|template| records.find("items", template))
                .is_none_or(|template| !same(item, template))
        })
        .filter_map(|item| {
            let wrong = against(item);
            let material = string(item, "material");
            let message = if rule(material).is_none() {
                format!("{:?} is made of {}", string(item, "name"), wrong.join("; "))
            } else {
                format!(
                    "{:?} is made of {material}, but {}",
                    string(item, "name"),
                    wrong.join("; ")
                )
            };
            (!wrong.is_empty()).then(|| Finding {
                code: FINDING,
                severity: "warning",
                message,
                remedy: "manual",
                subject: ("items", id(item)),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    fn against(row: &Value) -> Vec<String> {
        super::against(row.as_object().expect("a row"))
    }

    #[test]
    fn the_list_is_ten_materials() {
        assert_eq!(
            materials(),
            [
                "paper", "cloth", "rope", "wood", "leather", "glass", "pottery", "metal", "stone",
                "food"
            ]
        );
    }

    #[test]
    fn a_kit_name_has_the_material_the_table_gives_it_and_others_none() {
        assert_eq!(of("letter"), Some("paper"));
        assert_eq!(of("branch"), Some("wood"));
        assert_eq!(of("jug"), None);
        assert_eq!(of("a thing no kit makes"), None);
    }

    #[test]
    fn what_each_material_denies() {
        let row = |material: &str, combustible: bool, fragility: &str| {
            json!({
                "material": material,
                "combustible": combustible,
                "fragility": fragility,
            })
        };
        assert!(against(&json!({ "material": null, "fragility": "sturdy" })).is_empty());
        assert!(against(&row("paper", true, "sturdy")).is_empty());
        assert!(against(&row("glass", false, "brittle")).is_empty());
        assert!(against(&row("wood", false, "fragile")).is_empty());
        assert_eq!(
            against(&row("paper", false, "sturdy")),
            ["it is not combustible, and paper always is"]
        );
        assert_eq!(
            against(&row("glass", true, "sturdy")),
            [
                "it is combustible, and glass never is",
                "it is sturdy, and glass is only ever fragile or brittle"
            ]
        );
        assert_eq!(
            against(&row("stoneware", false, "sturdy")),
            [
                "\"stoneware\", which is none of the materials (paper, cloth, rope, wood, leather, \
                 glass, pottery, metal, stone, food)"
            ]
        );
    }
}
