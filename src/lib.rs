//! The text-adventure engine: the dice, the geometry, reading a typed line
//! against the room the player stands in, building the requests the engine
//! hands a model, and a turn loop that plays a line with no model over the
//! game's own database.
//!
//! The rules take values and return values. Nothing but the turn loop
//! ([`store`], [`turn`], [`outcome`], [`engine`]) reads a database, and
//! nothing but the model client ([`model`]) reads the network, so each rule
//! can be checked against the golden vectors the Ruby engine writes (see
//! `vectors/README.md`) and the loop against the engine sweep (see
//! `parity/README.md`).
//!
//! Reading a line (`grammar`, `intent`, `refusal`, `slash_menu`, `cascade`)
//! works over a [`room::Room`] of plain records. The System One cascade
//! builds its request and composes answers it is handed; it sends nothing.
//!
//! The request builders (`moment`, `ledger`, `memory`, `plan`, `volition`,
//! `arrival`, `realization`, `dialogue`) read a [`records::Records`]: the
//! rows a database would hold, handed in as values. They return each request
//! as JSON, keys in the order it is sent, and send nothing: [`model`] sends
//! them, over the player's own key or the owner's relay, or answers them from
//! fixed replies.
//!
//! Draws come from [`random::Random`], a reproduction of Ruby's `Random`, and
//! never from any other generator: the same seed has to roll the same dice in
//! both engines.

pub mod arrival;
pub mod boxes;
pub mod cascade;
pub mod cast;
pub mod classifier;
pub mod clock;
pub mod command;
pub mod danger;
pub mod data;
pub mod deadline;
pub mod dialogue;
pub mod engine;
pub mod facts;
pub mod glance;
pub mod grammar;
pub mod identity;
pub mod intent;
pub mod interior;
pub mod kind;
pub mod kit;
pub mod ledger;
pub mod memory;
pub mod model;
pub mod moment;
pub mod narration;
pub mod outcome;
pub mod parameters;
pub mod parity;
pub mod physics;
pub mod plan;
pub mod playthrough;
pub mod population;
pub mod prompt_version;
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
pub mod store;
pub mod text;
pub mod turn;
pub mod volition;
pub mod world_mechanic;
