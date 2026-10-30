//! How the simulated player plays: from what they can see (their hit
//! points, the foes still fighting, what they carry) to one typed line.
//! A policy is a player's habit, never a rule: whatever it types, the turn
//! loop decides what happens.

use super::arena::Arena;
use crate::outcome::Named;
use yaml_rust2::Yaml;

/// A hit-point mark: a number, or a share of the player's most.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mark {
    Hp(i64),
    Share(f64),
}

impl Mark {
    fn reached(self, hp: i64, max_hp: i64) -> bool {
        match self {
            Mark::Hp(mark) => hp <= mark,
            Mark::Share(share) => (hp as f64) <= share * max_hp as f64,
        }
    }
}

/// A way of playing. `attack` is the default: swing at the first foe
/// standing, every line, to the end.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Policy {
    pub kind: String,
    /// Leave once hit points are at or below this.
    pub flee_below: Option<i64>,
    /// Drink a healing thing, while one is carried, once hit points are at
    /// or below this.
    pub drink_below: Option<Mark>,
    /// Try once to throw each throwable thing carried before swinging.
    pub throw_first: bool,
    /// Leave once one more round could kill: hit points at or below the sum
    /// of the hit dice of the foes still fighting.
    pub careful: bool,
}

impl Policy {
    /// A policy by name, before its options.
    pub fn named(kind: &str) -> Result<Policy, String> {
        let base = Policy {
            kind: kind.to_string(),
            ..Policy::default()
        };
        Ok(match kind {
            "attack" | "flee_below" | "drink_below" => base,
            "throw_first" => Policy {
                throw_first: true,
                ..base
            },
            // The dungeon studies' naive player, at the scale of one fight.
            "naive" => Policy {
                drink_below: Some(Mark::Share(0.3)),
                ..base
            },
            // The dungeon studies' careful player, at the scale of one fight.
            "careful" => Policy {
                careful: true,
                drink_below: Some(Mark::Share(0.5)),
                ..base
            },
            _ => {
                return Err(format!(
                "policy {kind:?} is attack, flee_below, drink_below, throw_first, naive or careful"
            ))
            }
        })
    }

    /// `{kind: flee_below, hp: 6, drink_below: 9}`; `hp` is the mark of the
    /// policy its kind names.
    pub fn read(yaml: &Yaml) -> Result<Policy, String> {
        if let Some(kind) = yaml.as_str() {
            return Policy::named(kind);
        }
        let kind = yaml["kind"].as_str().unwrap_or("attack");
        let mut policy = Policy::named(kind)?;
        let hp = yaml["hp"].as_i64();
        let mark = |key: &str| -> Result<Option<Mark>, String> {
            match &yaml[key] {
                Yaml::Integer(hp) => Ok(Some(Mark::Hp(*hp))),
                Yaml::Real(share) => share
                    .parse()
                    .map(|share| Some(Mark::Share(share)))
                    .map_err(|_| format!("policy.{key} is not a number")),
                Yaml::BadValue => Ok(None),
                _ => Err(format!("policy.{key} is not a number")),
            }
        };
        match kind {
            "flee_below" => {
                policy.flee_below = Some(hp.ok_or("policy flee_below needs hp")?);
            }
            "drink_below" => {
                policy.drink_below = Some(Mark::Hp(hp.ok_or("policy drink_below needs hp")?));
            }
            _ => {}
        }
        if let Some(Mark::Hp(hp)) = mark("flee_below")? {
            policy.flee_below = Some(hp);
        }
        if let Some(drink) = mark("drink_below")? {
            policy.drink_below = Some(drink);
        }
        if let Some(throw) = yaml["throw_first"].as_bool() {
            policy.throw_first = throw;
        }
        Ok(policy)
    }

    /// The next line.
    pub fn choose(&self, arena: &Arena, view: &View) -> Act {
        let Some(foe) = view.foes.first() else {
            return Act::Wait;
        };
        if let Some(drink) = self.drink_below {
            if drink.reached(view.hp, view.max_hp) {
                if let Some(thing) = view.carrying.iter().find(|thing| arena.heals(&thing.name)) {
                    return Act::Drink(thing.name.clone());
                }
            }
        }
        let flee = self.flee_below.is_some_and(|mark| view.hp <= mark);
        let risky = self.careful && {
            let threat: i64 = view
                .foes
                .iter()
                .map(|foe| arena.hit_die_of(&foe.name, view.seed))
                .sum();
            view.hp <= threat
        };
        if flee || risky {
            return Act::Leave(arena.exit_name.clone());
        }
        if self.throw_first {
            if let Some(thing) = view
                .carrying
                .iter()
                .find(|thing| arena.throws(&thing.name) && !view.thrown.contains(&thing.name))
            {
                return Act::Throw(thing.name.clone(), foe.name.clone());
            }
        }
        Act::Attack(foe.name.clone())
    }
}

/// What the player sees before a line.
#[derive(Clone, Copy, Debug)]
pub struct View<'a> {
    pub seed: i64,
    pub hp: i64,
    pub max_hp: i64,
    /// The foes still fighting here, sorted by name.
    pub foes: &'a [Named],
    pub carrying: &'a [Named],
    /// What the player has already tried to throw: a failed lift leaves the
    /// thing in hand, and it is not tried again.
    pub thrown: &'a [String],
}

/// One line, before it is typed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Act {
    Attack(String),
    Leave(String),
    Drink(String),
    Throw(String, String),
    Wait,
}

impl Act {
    /// The line as the grammar reads it.
    pub fn line(&self) -> String {
        match self {
            Act::Attack(foe) => format!("attack {foe}"),
            Act::Leave(room) => format!("go to the {room}"),
            Act::Drink(thing) => format!("drink {thing}"),
            Act::Throw(thing, foe) => format!("throw {thing} at {foe}"),
            Act::Wait => "wait".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yaml_rust2::YamlLoader;

    fn read(source: &str) -> Policy {
        Policy::read(&YamlLoader::load_from_str(source).unwrap()[0]).unwrap()
    }

    #[test]
    fn reads_each_kind() {
        assert_eq!(read("attack"), Policy::named("attack").unwrap());
        assert_eq!(read("{kind: flee_below, hp: 6}").flee_below, Some(6));
        assert_eq!(
            read("{kind: flee_below, hp: 6, drink_below: 9}").drink_below,
            Some(Mark::Hp(9))
        );
        assert_eq!(
            read("{kind: drink_below, hp: 4}").drink_below,
            Some(Mark::Hp(4))
        );
        assert!(read("throw_first").throw_first);
        assert_eq!(read("naive").drink_below, Some(Mark::Share(0.3)));
        let careful = read("careful");
        assert!(careful.careful);
        assert_eq!(careful.drink_below, Some(Mark::Share(0.5)));
        assert!(Policy::read(&YamlLoader::load_from_str("bold").unwrap()[0]).is_err());
    }

    #[test]
    fn marks() {
        assert!(Mark::Hp(6).reached(6, 18));
        assert!(!Mark::Hp(6).reached(7, 18));
        assert!(Mark::Share(0.5).reached(9, 18));
        assert!(!Mark::Share(0.5).reached(10, 18));
    }
}
