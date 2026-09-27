//! The text-adventure engine's pure rules: the dice, the geometry, reading a
//! typed line against the room the player stands in, and building the
//! requests the engine hands a model.
//!
//! Every function here takes values and returns values. Nothing reads a
//! database, the network or the clock, so each rule can be checked against
//! the golden vectors the Ruby engine writes (see `vectors/README.md`).
//!
//! Reading a line (`grammar`, `intent`, `refusal`, `slash_menu`, `cascade`)
//! works over a [`room::Room`] of plain records. The System One cascade
//! builds its request and composes answers it is handed; it sends nothing.
//!
//! The request builders (`moment`, `ledger`, `memory`, `plan`, `volition`,
//! `arrival`, `realization`, `dialogue`) read a [`records::Records`]: the
//! rows a database would hold, handed in as values. They return each request
//! as JSON, keys in the order it is sent, and send nothing.
//!
//! Draws come from [`random::Random`], a reproduction of Ruby's `Random`, and
//! never from any other generator: the same seed has to roll the same dice in
//! both engines.

pub mod arrival;
pub mod boxes;
pub mod cascade;
pub mod cast;
pub mod danger;
mod data;
pub mod deadline;
pub mod dialogue;
pub mod grammar;
pub mod identity;
pub mod intent;
pub mod interior;
pub mod ledger;
pub mod memory;
pub mod moment;
pub mod parameters;
pub mod plan;
pub mod playthrough;
pub mod population;
pub mod random;
pub mod realization;
pub mod records;
pub mod refusal;
pub mod roll;
pub mod room;
pub mod schemas;
pub mod shuffle_connections;
pub mod slash_menu;
pub mod spot;
pub mod stat_block;
pub mod text;
pub mod volition;
pub mod world_mechanic;
