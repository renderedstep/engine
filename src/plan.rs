//! `Location::Plan`: what the engine knows about the shape of one room, said
//! in sentences. Every one of them is a record read out: the room's own box,
//! the wall each doorway is in, the storey, and the place it stands in.

use crate::boxes::{Box, WALLS};
use crate::records::{id, int, string, Records, Row};
use serde_json::{json, Value};

/// How a way out is described when it is neither a door nor a stair.
pub const WAY_OUT: &str = "a way out";

/// `Location::Box.of`: the box a row places, or none unless all five columns
/// are there.
pub fn box_of(location: &Row) -> Option<Box> {
    let column = |name: &str| int(location, name);
    Some(Box {
        x: column("x")?,
        y: column("y")?,
        z: column("z")?,
        width: column("width")?,
        depth: column("depth")?,
    })
}

/// `Location#interior?`: whether a place has an extent.
pub fn interior(location: &Row) -> bool {
    int(location, "width").is_some() && int(location, "depth").is_some()
}

/// One way out of the room.
struct Way<'a> {
    to: &'a Row,
    wall: Option<&'static str>,
    storey: Option<i64>,
    bearing: Option<String>,
    from_storey: i64,
}

impl Way<'_> {
    fn door(&self) -> bool {
        self.wall.is_some()
    }

    fn stair(&self) -> bool {
        self.storey.is_some()
    }

    fn up(&self) -> bool {
        self.storey.is_some_and(|storey| storey > self.from_storey)
    }
}

pub struct Plan<'a> {
    room: &'a Row,
    place: &'a Row,
    r#box: Box,
    ways_out: Vec<Way<'a>>,
}

impl<'a> Plan<'a> {
    /// `Location::Plan.for`: a placed room with a parent, or none.
    pub fn of(records: &'a Records, room: &'a Row) -> Option<Plan<'a>> {
        let r#box = box_of(room)?;
        let place = records.find("locations", int(room, "parent_location_id")?)?;
        let here = id(room);
        let mut ways_out: Vec<Way<'a>> = records
            .select("location_connections", |edge| {
                int(edge, "location_id") == Some(here)
            })
            .iter()
            .map(|edge| {
                let other = records
                    .find(
                        "locations",
                        int(edge, "connected_location_id").expect("a far end"),
                    )
                    .expect("the far end's row");
                way_for(room, &r#box, other)
            })
            .collect();
        ways_out.sort_by(|a, b| {
            let key = |way: &Way| {
                (
                    way.wall
                        .and_then(|wall| WALLS.iter().position(|w| *w == wall))
                        .unwrap_or(WALLS.len()),
                    if way.stair() { 0 } else { 1 },
                    string(way.to, "name").to_string(),
                )
            };
            key(a).cmp(&key(b))
        });
        Some(Plan {
            room,
            place,
            r#box,
            ways_out,
        })
    }

    pub fn sentences(&self) -> Vec<String> {
        vec![
            self.size_sentence(),
            self.storey_sentence(),
            self.ways_out_sentence(),
        ]
    }

    pub fn to_prompt(&self) -> String {
        self.sentences().join(" ")
    }

    pub fn to_json(&self) -> Value {
        let doors: Vec<Value> = self
            .ways_out
            .iter()
            .filter(|way| way.door())
            .map(|way| json!({ "wall": way.wall, "to": string(way.to, "name") }))
            .collect();
        let stairs: Vec<Value> = self
            .ways_out
            .iter()
            .filter(|way| way.stair())
            .map(|way| {
                json!({
                    "to": string(way.to, "name"),
                    "up": way.up(),
                    "storey": way.storey,
                    "bearing": way.bearing,
                })
            })
            .collect();
        let others: Vec<&str> = self.others().map(|way| string(way.to, "name")).collect();
        json!({
            "room": string(self.room, "name"),
            "place": string(self.place, "name"),
            "storey": self.r#box.z,
            "place_width": int(self.place, "width"),
            "place_depth": int(self.place, "depth"),
            "width": self.r#box.width,
            "depth": self.r#box.depth,
            "doors": doors,
            "stairs": stairs,
            "other_ways_out": others,
        })
    }

    fn others(&self) -> impl Iterator<Item = &Way<'a>> {
        self.ways_out
            .iter()
            .filter(|way| !way.door() && !way.stair())
    }

    fn size_sentence(&self) -> String {
        let [wide, deep] = self.r#box.metres();
        format!(
            "This room is {} by {} paces -- about {wide} by {deep} metres.",
            self.r#box.width, self.r#box.depth
        )
    }

    fn storey_sentence(&self) -> String {
        let footprint = if interior(self.place) {
            format!(
                ", which is {} by {} paces across",
                int(self.place, "width").unwrap(),
                int(self.place, "depth").unwrap()
            )
        } else {
            String::new()
        };
        format!(
            "It is on storey {} of {}{footprint}; storey 0 is the ground floor.",
            self.r#box.z,
            string(self.place, "name")
        )
    }

    fn ways_out_sentence(&self) -> String {
        if self.ways_out.is_empty() {
            return "Nothing leads out of this room yet.".into();
        }
        let clauses: Vec<String> = self
            .ways_out
            .iter()
            .map(|way| self.clause_for(way))
            .collect();
        let closed = if self.others().next().is_some() {
            ""
        } else {
            ", and no other wall of it holds a door"
        };
        format!(
            "Its ways out: {}. Those are every way out of this room{closed}.",
            clauses.join("; ")
        )
    }

    fn clause_for(&self, way: &Way) -> String {
        let to = string(way.to, "name");
        if let Some(wall) = way.wall {
            return format!("a door in the {wall} wall, to {to}");
        }
        if let Some(storey) = way.storey {
            let bearing = way
                .bearing
                .as_ref()
                .map(|bearing| format!(", in the {bearing} of this room"))
                .unwrap_or_default();
            let direction = if way.up() { "up" } else { "down" };
            return format!("a stair {direction} to {to}, on storey {storey}{bearing}");
        }
        let outside = if int(way.to, "parent_location_id") == Some(id(self.place)) {
            String::new()
        } else {
            format!(", which is outside {}", string(self.place, "name"))
        };
        format!("{WAY_OUT} to {to}{outside}")
    }
}

fn way_for<'a>(room: &Row, r#box: &Box, other: &'a Row) -> Way<'a> {
    let plain = |wall, storey, bearing| Way {
        to: other,
        wall,
        storey,
        bearing,
        from_storey: r#box.z,
    };
    let far = match box_of(other) {
        Some(far) if int(other, "parent_location_id") == int(room, "parent_location_id") => far,
        _ => return plain(None, None, None),
    };
    if far.z == r#box.z {
        return plain(r#box.wall_towards(&far), None, None);
    }
    let bearing = r#box
        .shared_ground(&far)
        .and_then(|ground| r#box.bearing_of(&ground));
    plain(None, Some(far.z), bearing)
}
