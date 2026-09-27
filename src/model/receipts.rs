//! What a model call leaves in the database: the conversation (`chats`), its
//! messages, and a usage row for every attempt, written the way RubyLLM's
//! ActiveRecord integration writes them, so the Ruby app's spend meter, its
//! debug view and its checks read a conversation this engine had exactly as
//! they read one of their own.
//!
//! A conversation is a row once something is asked in it, not before. A
//! failed attempt is taken back out (its messages deleted, its usage row
//! kept, as RubyLLM keeps a billed failure), so the next attempt asks the
//! same question in the same context.

use super::wire::Usage;
use super::Book;
use crate::engine::Error;
use crate::records::{int, text, Row};
use serde_json::Value;

/// `Chat::CHARACTER`: the one conversation that is picked up again.
pub const CHARACTER: &str = "character";

/// `Chat::HISTORY_EXCHANGES`: how many exchanges a picked-up conversation
/// replays.
pub const HISTORY_EXCHANGES: usize = 2;

/// Who a conversation is filed under (`BaseAgent.new`'s `purpose`,
/// `playthrough` and `character`, and the game's player).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Filed {
    pub purpose: String,
    pub playthrough: Option<i64>,
    pub character: Option<i64>,
    pub player: Option<i64>,
}

impl Filed {
    /// A conversation of this purpose in this game, filed under the game's
    /// player, as `Current.player` files it.
    pub fn in_game(book: &Book, purpose: &str, playthrough: i64) -> Filed {
        Filed {
            purpose: purpose.to_string(),
            playthrough: Some(playthrough),
            character: None,
            player: book
                .records
                .find("playthroughs", playthrough)
                .and_then(|row| int(row, "player_id")),
        }
    }
}

/// The durable conversation this game already has with this character
/// (`Chat.conversation_with`), if it has one.
pub fn conversation_with(book: &Book, character: i64, playthrough: i64) -> Option<i64> {
    book.records
        .first("chats", |chat| {
            text(chat, "purpose") == Some(CHARACTER)
                && int(chat, "character_id") == Some(character)
                && int(chat, "playthrough_id") == Some(playthrough)
        })
        .and_then(|chat| int(chat, "id"))
}

fn messages_of(book: &Book, chat: i64) -> Vec<Row> {
    book.records
        .select("messages", |message| int(message, "chat_id") == Some(chat))
        .into_iter()
        .cloned()
        .collect()
}

fn delete_messages(book: &mut Book, ids: &[i64]) -> Result<(), Error> {
    for id in ids {
        book.store.connection().execute(
            "UPDATE ruby_llm_usages SET message_id = NULL WHERE message_type = 'Message' AND message_id = ?1",
            [id],
        )?;
        book.store.delete("messages", *id)?;
        book.records.remove("messages", *id);
    }
    Ok(())
}

/// Picking a conversation up (`BaseAgent#build_chat` on a persisted chat):
/// a killed attempt's unanswered prompt comes out (`Chat#drop_unanswered!`),
/// and, except for a room being written, the replay is trimmed to
/// [`HISTORY_EXCHANGES`] (`Chat#prune_history!`). Call it before building a
/// request from the conversation, so the request replays what is left.
pub fn pick_up(book: &mut Book, chat: i64) -> Result<(), Error> {
    let messages = messages_of(book, chat);
    let answered = messages
        .iter()
        .filter(|m| {
            text(m, "role") == Some("assistant")
                && (text(m, "content").is_some_and(|c| !c.is_empty())
                    || m.get("content_raw").is_some_and(|raw| !raw.is_null()))
        })
        .filter_map(|m| int(m, "id"))
        .max()
        .unwrap_or(0);
    let unanswered: Vec<i64> = messages
        .iter()
        .filter(|m| text(m, "role") != Some("system"))
        .filter_map(|m| int(m, "id"))
        .filter(|id| *id > answered)
        .collect();
    delete_messages(book, &unanswered)?;

    let purpose = book
        .records
        .find("chats", chat)
        .and_then(|row| text(row, "purpose").map(str::to_string));
    if purpose.as_deref() == Some("location") {
        return Ok(());
    }
    let exchange: Vec<i64> = messages_of(book, chat)
        .iter()
        .filter(|m| text(m, "role") != Some("system"))
        .filter_map(|m| int(m, "id"))
        .collect();
    let keep = HISTORY_EXCHANGES * 2;
    let doomed = if keep == 0 {
        exchange
    } else {
        exchange[..exchange.len().saturating_sub(keep)].to_vec()
    };
    delete_messages(book, &doomed)
}

/// The `ruby_llm_models` row for an OpenRouter model, if the database has
/// the registry loaded.
fn model_row(book: &Book, model: &str) -> Option<(i64, Value)> {
    book.store
        .connection()
        .query_row(
            "SELECT id, pricing FROM ruby_llm_models WHERE provider = 'openrouter' AND model_id = ?1 ORDER BY id LIMIT 1",
            [model],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, Option<String>>(1)?)),
        )
        .ok()
        .map(|(id, pricing)| {
            let pricing = pricing
                .and_then(|text| serde_json::from_str(&text).ok())
                .unwrap_or(Value::Null);
            (id, pricing)
        })
}

fn model_columns(book: &Book, model: &str) -> [(&'static str, Value); 2] {
    match model_row(book, model) {
        Some((id, _)) => [
            ("ruby_llm_model_id", Value::from(id)),
            ("model_id_string", Value::Null),
        ],
        None => [
            ("ruby_llm_model_id", Value::Null),
            ("model_id_string", Value::from(model)),
        ],
    }
}

/// The conversation an attempt is asked in: created on the first ask, filed
/// under `filed`, with the instructions as its system message; or the one
/// being continued. Either way it is pointed at `model`.
pub fn conversation(
    book: &mut Book,
    chat: Option<i64>,
    filed: &Filed,
    model: &str,
    instructions: Option<&str>,
) -> Result<i64, Error> {
    let columns = model_columns(book, model);
    let chat = match chat {
        Some(chat) => {
            let written = book.store.update("chats", chat, &columns)?;
            for (column, value) in written {
                book.records.set("chats", chat, &column, value);
            }
            chat
        }
        None => {
            let mut values = vec![
                ("purpose", Value::from(filed.purpose.as_str())),
                (
                    "playthrough_id",
                    filed.playthrough.map_or(Value::Null, Value::from),
                ),
                (
                    "character_id",
                    filed.character.map_or(Value::Null, Value::from),
                ),
                ("player_id", filed.player.map_or(Value::Null, Value::from)),
            ];
            values.extend(columns);
            let row = book.store.insert("chats", &values)?;
            let chat = int(&row, "id").expect("a chat has an id");
            book.records.push("chats", row);
            chat
        }
    };
    if let Some(instructions) = instructions {
        let systems: Vec<i64> = messages_of(book, chat)
            .iter()
            .filter(|m| text(m, "role") == Some("system"))
            .filter_map(|m| int(m, "id"))
            .collect();
        let same = systems.len() == 1
            && book
                .records
                .find("messages", systems[0])
                .and_then(|m| text(m, "content"))
                == Some(instructions);
        if !same {
            delete_messages(book, &systems)?;
            message(book, chat, "system", instructions, None)?;
        }
    }
    Ok(chat)
}

/// Where the conversation stands before an attempt.
pub fn mark(book: &Book, chat: i64) -> i64 {
    messages_of(book, chat)
        .iter()
        .filter_map(|m| int(m, "id"))
        .max()
        .unwrap_or(0)
}

/// Every message written since `mark`.
pub fn since(book: &Book, chat: i64, mark: i64) -> Vec<i64> {
    messages_of(book, chat)
        .iter()
        .filter_map(|m| int(m, "id"))
        .filter(|id| *id > mark)
        .collect()
}

/// Takes an attempt back out: every message after `mark`.
pub fn rewind(book: &mut Book, chat: i64, mark: i64) -> Result<(), Error> {
    let ids = since(book, chat, mark);
    delete_messages(book, &ids)
}

/// Writes one message and returns its id.
pub fn message(
    book: &mut Book,
    chat: i64,
    role: &str,
    content: &str,
    finish_reason: Option<&str>,
) -> Result<i64, Error> {
    let mut values = vec![
        ("chat_id", Value::from(chat)),
        ("role", Value::from(role)),
        ("content", Value::from(content)),
    ];
    if let Some(reason) = finish_reason {
        values.push(("finish_reason", Value::from(reason)));
    }
    let row = book.store.insert("messages", &values)?;
    let id = int(&row, "id").expect("a message has an id");
    book.records.push("messages", row);
    Ok(id)
}

fn price(pricing: &Value, key: &str) -> Option<f64> {
    pricing["text_tokens"]["standard"][key].as_f64()
}

/// One attempt's usage row: what it cost when it was answered (`message`
/// the answer), or a failed attempt with nothing counted.
pub fn usage(
    book: &mut Book,
    chat: i64,
    message: Option<i64>,
    model: &str,
    used: Option<&Usage>,
) -> Result<(), Error> {
    let mut values = vec![
        ("chat_id", Value::from(chat)),
        ("chat_type", Value::from("Chat")),
        ("operation", Value::from("chat")),
        ("provider", Value::from("openrouter")),
        ("model", Value::from(model)),
    ];
    if let Some(message) = message {
        values.push(("message_id", Value::from(message)));
        values.push(("message_type", Value::from("Message")));
    }
    match used {
        None => values.push(("status", Value::from("failed"))),
        Some(used) => {
            let pricing = model_row(book, model)
                .map(|(_, pricing)| pricing)
                .unwrap_or(Value::Null);
            let cost = |tokens: Option<i64>, key: &str| {
                Some(tokens? as f64 * price(&pricing, key)? / 1_000_000.0)
            };
            let input = cost(used.input_tokens, "input_per_million");
            let output = cost(used.output_tokens, "output_per_million");
            let cache_read = cost(used.cache_read_tokens, "cache_read_input_per_million");
            let total = used.cost.or_else(|| match (input, output) {
                (None, None) => None,
                _ => Some(input.unwrap_or(0.0) + output.unwrap_or(0.0) + cache_read.unwrap_or(0.0)),
            });
            let number = |value: Option<f64>| value.map_or(Value::Null, Value::from);
            let tokens = |value: Option<i64>| value.map_or(Value::Null, Value::from);
            values.extend([
                ("status", Value::from("succeeded")),
                ("input_tokens", tokens(used.input_tokens)),
                ("output_tokens", tokens(used.output_tokens)),
                ("cache_read_tokens", tokens(used.cache_read_tokens)),
                ("cache_write_tokens", tokens(used.cache_write_tokens)),
                ("thinking_tokens", tokens(used.thinking_tokens)),
                ("input_cost", number(input)),
                ("output_cost", number(output)),
                ("cache_read_cost", number(cache_read)),
                ("total_cost", number(total)),
            ]);
        }
    }
    book.store.insert("ruby_llm_usages", &values)?;
    Ok(())
}

/// Stamps these messages with the turn they were exchanged on
/// (`BaseAgent#attribute_to!`), where no turn has claimed them yet.
pub fn attribute(book: &mut Book, messages: &[i64], scene: i64) -> Result<(), Error> {
    for id in messages {
        let unclaimed = book
            .records
            .find("messages", *id)
            .is_some_and(|m| m.get("scene_id").is_none_or(Value::is_null));
        if unclaimed {
            let written =
                book.store
                    .update("messages", *id, &[("scene_id", Value::from(scene))])?;
            for (column, value) in written {
                book.records.set("messages", *id, &column, value);
            }
        }
    }
    Ok(())
}

/// `SystemOneReceipt::COST_PER_REQUEST_USD`: what a System One request is
/// charged at before its answer says otherwise.
pub const SYSTEM_ONE_COST: f64 = 0.002;

/// The System One receipt, written before the request goes, because a
/// request that fails after it was sent may still have been billed.
pub fn system_one(book: &mut Book, filed: &Filed, transport: &str) -> Result<i64, Error> {
    let row = book.store.insert(
        "system_one_receipts",
        &[
            ("player_id", filed.player.map_or(Value::Null, Value::from)),
            (
                "playthrough_id",
                filed.playthrough.map_or(Value::Null, Value::from),
            ),
            ("purpose", Value::from(filed.purpose.as_str())),
            ("transport", Value::from(transport)),
            ("cost_usd", Value::from(SYSTEM_ONE_COST)),
        ],
    )?;
    Ok(int(&row, "id").expect("a receipt has an id"))
}

/// The cost the answer reported, where it is more than the receipt holds
/// (`SystemOneReceipt#reported!`).
pub fn system_one_reported(book: &mut Book, receipt: i64, usage: &Value) -> Result<(), Error> {
    if let Some(cost) = usage.get("cost").and_then(Value::as_f64) {
        if cost > SYSTEM_ONE_COST {
            book.store.update(
                "system_one_receipts",
                receipt,
                &[("cost_usd", Value::from(cost))],
            )?;
        }
    }
    Ok(())
}
