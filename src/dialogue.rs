//! `InteractionAgent`'s two requests for one exchange with a person: the
//! character pass (`Character#interaction_instructions` and the per-turn
//! prompt, with the engine actions on offer) and the narrator pass that turns
//! the reaction into prose. They build the requests and send nothing.

use crate::arrival::prompt_details;
use crate::moment::Moment;
use crate::playthrough::Game;
use crate::records::{flag, id, int, string, text, Records, Row};
use crate::schemas;
use crate::text::{is_blank, presence};
use serde_json::{json, Map, Value};

/// How many recent exchanges the durable chat replays verbatim
/// (`Chat::HISTORY_EXCHANGES`).
pub const HISTORY_EXCHANGES: i64 = 2;

pub const NONE: &str = "none";

/// One pronoun set, form by form (`Character::PRONOUN_FORMS`).
pub struct Pronouns {
    pub rule: &'static str,
    pub subject: &'static str,
    pub object: &'static str,
    pub determiner: &'static str,
    pub plural: bool,
}

/// `Character#pronouns` and `#pronoun_forms`, from the stored sex.
pub fn pronouns(character: &Row) -> Pronouns {
    let he = Pronouns {
        rule: "he/him/his",
        subject: "he",
        object: "him",
        determiner: "his",
        plural: false,
    };
    let she = Pronouns {
        rule: "she/her/hers",
        subject: "she",
        object: "her",
        determiner: "her",
        plural: false,
    };
    let they = Pronouns {
        rule: "they/them/theirs",
        subject: "they",
        object: "them",
        determiner: "their",
        plural: true,
    };
    match string(character, "sex") {
        "male" | "trans man" => he,
        "female" | "trans woman" => she,
        "non-binary" => they,
        other => panic!("no pronouns for {other:?}"),
    }
}

/// `SanitizesGeneratedText#sanitize_string`: emoji dropped, any JSON
/// envelope debris at the end cut off, and the ends stripped.
pub fn sanitize(text: &str) -> String {
    let mut kept = String::new();
    let mut after_emoji = false;
    for c in text.chars() {
        let code = c as u32;
        let pictographic = matches!(code, 0x1F000..=0x1FAFF | 0x2600..=0x27BF);
        if pictographic {
            after_emoji = true;
            continue;
        }
        if after_emoji && (c == '\u{FE0F}' || c == '\u{200D}') {
            continue;
        }
        after_emoji = false;
        kept.push(c);
    }
    let debris = |c: char| c.is_ascii_whitespace() || ",\"'“”‘’}]".contains(c);
    let tail_start = kept
        .char_indices()
        .rev()
        .take_while(|(_, c)| debris(*c))
        .last()
        .map(|(at, _)| at);
    if let Some(at) = tail_start {
        if kept[at..].contains(['}', ']']) {
            kept.truncate(at);
        }
    }
    crate::text::ruby_strip(&kept).to_string()
}

fn protagonist<'a>(records: &'a Records, character: &Row) -> Option<&'a Row> {
    let story = int(character, "story_id");
    records.first("characters", |row| {
        int(row, "story_id") == story && flag(row, "is_protagonist")
    })
}

fn race<'a>(records: &'a Records, character: &Row) -> Option<&'a Row> {
    int(character, "race_id").and_then(|race| records.find("races", race))
}

/// `Character#interaction_instructions`: the character pass's system prompt.
pub fn interaction_instructions(records: &Records, character: &Row) -> String {
    let story = records
        .find("stories", int(character, "story_id").unwrap())
        .expect("the character's story");
    let universe = records
        .find("universes", int(story, "universe_id").unwrap())
        .expect("the story's universe");
    let race = race(records, character);
    let desires = [
        "conscious_desire",
        "unconscious_desire",
        "recognized_need",
        "unrecognized_need",
    ];
    let desire_section = if desires
        .iter()
        .all(|field| !is_blank(string(character, field)))
    {
        format!(
            "what you want, and would say out loud: {}\n\
             what you are really after, and would deny: {}\n\
             what you know you must do, whether you want to or not: {}\n\
             what you need and cannot see: {}\n",
            string(character, "conscious_desire"),
            string(character, "unconscious_desire"),
            string(character, "recognized_need"),
            string(character, "unrecognized_need"),
        )
    } else {
        String::new()
    };
    let addressee_section = match protagonist(records, character) {
        Some(them) if id(them) != id(character) => {
            let name = string(them, "fullname");
            let nickname = presence(text(them, "nickname"))
                .map(|nick| format!(" ({nick})"))
                .unwrap_or_default();
            format!(
                "\n## Who you are talking to\n\
                 The person speaking to you is {name}{nickname}.\n\
                 Refer to {name} as {}. Use those pronouns and no others.\n\
                 apparent age: about {}\n\
                 race: {}\n\
                 what you can see of them: {}\n\n\
                 That is what meeting {name} tells you, and it is the whole of\n\
                 what you know by sight. You do not know what {name} wants, is\n\
                 afraid of, or has done, and you do not invent any of it. Anything more\n\
                 you learn from what {name} says and does, here, now.\n",
                pronouns(them).rule,
                int(them, "age")
                    .map(|age| age.to_string())
                    .unwrap_or_default(),
                race_name(records, them),
                string(them, "appearance"),
            )
        }
        _ => String::new(),
    };
    let fullname = string(character, "fullname");
    let called = presence(text(character, "nickname")).unwrap_or(fullname);
    let forms = pronouns(character);
    format!(
        "\nThis is the universe in which you live\n## Universe Details\n{}\n\n\
         You are playing a character in a story. This is your character sheet.\n\
         Pretend you are this character in all of your responses.\n\n\
         ## Character Sheet\n\
         full name: {fullname}\n\
         nickname: {}\n\
         age: {}\n\
         sex: {}\n\
         race: {} -- {}\n\
         backstory: {}\n\
         personality: {}\n\
         appearance: {}\n\
         likes: {}\n\
         dislikes: {}\n\
         fears: {}\n\
         {desire_section}{addressee_section}\n\
         If someone asks you a question, you should respond as if you are the character. NEVER BREAK CHARACTER.\n\n\
         ## Voice\n\
         You answer in six fields, and each one has its own register.\n\n\
         pre_thought and post_thought are your private thoughts. Think them in the first person, as \"I\". \
         Nobody hears them, so nothing in them is a line of speech.\n\
         pre_feeling and post_feeling are two or three words each. No sentences.\n\
         inner_resolution is what you have decided to do, in the first person.\n\
         action is the one field anybody can see: what you visibly do and what you say out loud. \
         Speech goes inside quotes, and only speech does.\n\
         Inside the quotes you are talking out loud: say \"I\", never your own name. \
         Nobody says \"{fullname} does not know\" out loud.\n\
         Outside the quotes you are described from outside: {called}, {}, {}. \
         A bare \"I\" out there reads as the person you are talking to rather than as you.\n\n\
         Answer AS {fullname}, not about {}. Do not plan the answer and do not weigh what {called} \
         would probably think -- think the thought and say the line.\n\n",
        prompt_details(records, universe, &["physics", "race_names", "civilizations", "politics", "religion"]),
        string(character, "nickname"),
        int(character, "age").map(|age| age.to_string()).unwrap_or_default(),
        string(character, "sex"),
        race.map(|race| string(race, "name")).unwrap_or(""),
        race.map(|race| string(race, "description")).unwrap_or(""),
        string(character, "backstory"),
        string(character, "personality"),
        string(character, "appearance"),
        string(character, "likes"),
        string(character, "dislikes"),
        string(character, "fears"),
        forms.subject,
        forms.determiner,
        forms.object,
    )
}

fn race_name<'a>(records: &'a Records, character: &Row) -> &'a str {
    race(records, character)
        .map(|race| string(race, "name"))
        .unwrap_or("")
}

/// `Playthrough::NpcAction#choices`: token => what it does, in the order
/// offered.
pub fn choices(game: &Game, character: &Row) -> Vec<(String, String)> {
    let mut available = vec![(
        NONE.to_string(),
        "Speak without transferring anything or changing an agreement.".to_string(),
    )];
    let player = game.protagonist();
    let here = game.current_location();
    let present = int(game.row, "ended_at").is_none()
        && player.map(id) != Some(id(character))
        && int(character, "story_id") == Some(game.story_id())
        && game
            .cast_in(here)
            .iter()
            .any(|who| id(who) == id(character));
    if !present {
        return available;
    }
    let player_name = player
        .map(|who| string(who, "fullname"))
        .unwrap_or("the player");
    if let Some(player) = player {
        for item in game.items_held_by(character) {
            available.push((
                format!("give:{}", id(item)),
                format!(
                    "Give {} to {}.",
                    string(item, "name"),
                    string(player, "fullname")
                ),
            ));
        }
    }
    let state = game
        .own("playthrough_npc_states")
        .into_iter()
        .find(|row| int(row, "character_id") == Some(id(character)));
    let following = match state {
        Some(row) => flag(row, "following"),
        None => flag(character, "is_companion"),
    };
    if game
        .foes_in(here)
        .iter()
        .any(|who| id(who) == id(character))
    {
        available.push((
            "ceasefire".into(),
            format!("Stop fighting {player_name}; another attack can break the truce."),
        ));
    } else if following {
        available.push((
            "stop_following".into(),
            format!(
                "Stay in {} when the player leaves.",
                here.map(|room| string(room, "name")).unwrap_or("")
            ),
        ));
    } else {
        available.push((
            "follow".into(),
            format!("Accompany {player_name} when they leave this room."),
        ));
    }
    available
}

/// `Playthrough::NpcAction#apply!`'s receipt sentence for a choice, read off
/// the game as it stood before the choice was applied.
pub fn receipt(game: &Game, character: &Row, choice: &str) -> String {
    let name = string(character, "fullname");
    if choice == NONE {
        return format!("{name} changes no possessions, travel agreement or ceasefire.");
    }
    if !choices(game, character)
        .iter()
        .any(|(token, _)| token == choice)
    {
        return "The proposed action was rejected: it is unavailable. No possessions, travel agreement or ceasefire changed."
            .into();
    }
    let player = game.protagonist().map(|who| string(who, "fullname"));
    let item_name = |item: &str| {
        let item: i64 = item.parse().expect("an item id");
        string(game.records.find("items", item).expect("the item"), "name").to_string()
    };
    if let Some(item) = choice.strip_prefix("give:") {
        return format!(
            "{name} gave {} to {}; the player now carries it.",
            item_name(item),
            player.unwrap_or("")
        );
    }
    if let Some(item) = choice.strip_prefix("accept:") {
        return format!(
            "{name} accepted {}; the player no longer carries it and {name} now holds it.",
            item_name(item)
        );
    }
    match choice {
        "follow" => format!(
            "{name} is now accompanying {} and will travel with them.",
            player.unwrap_or("the player")
        ),
        "stop_following" => format!(
            "{name} stopped accompanying the player and remains in {}.",
            game.current_location().map(|room| string(room, "name")).unwrap_or("")
        ),
        "ceasefire" => format!("{name} stopped fighting the player. The ceasefire holds unless the player attacks again."),
        other => panic!("no receipt for {other}"),
    }
}

fn addressee_name(records: &Records, character: &Row) -> String {
    match protagonist(records, character) {
        Some(them) if id(them) != id(character) => string(them, "fullname").to_string(),
        _ => "the person in front of you".into(),
    }
}

/// The character's conversation with this game, oldest message first, as
/// `{role, content}` (`Message#text`).
fn history(game: &Game, character: &Row) -> Vec<Value> {
    let chat = game.records.first("chats", |chat| {
        int(chat, "character_id") == Some(id(character))
            && int(chat, "playthrough_id") == Some(game.id())
    });
    let Some(chat) = chat else {
        return Vec::new();
    };
    game.records
        .select("messages", |message| {
            int(message, "chat_id") == Some(id(chat))
        })
        .iter()
        .map(|message| {
            let content = match presence(text(message, "content")) {
                Some(content) => Value::String(content.to_string()),
                None => match message.get("content_raw") {
                    Some(Value::String(raw)) => Value::String(raw.clone()),
                    Some(raw) if !raw.is_null() => {
                        Value::String(serde_json::to_string_pretty(raw).expect("JSON"))
                    }
                    _ => Value::Null,
                },
            };
            json!({ "role": string(message, "role"), "content": content })
        })
        .collect()
}

/// The character pass's request: `{system, user, schema, history}`.
pub fn character_request(game: &Game, character: &Row, line: &str) -> Value {
    let records = game.records;
    let context = Moment::new(*game).character_context(character, HISTORY_EXCHANGES, Some(line));
    let moment = if is_blank(&context) {
        String::new()
    } else {
        format!("## The moment\n{context}\n")
    };
    let offered = choices(game, character);
    let tokens: Vec<&str> = offered.iter().map(|(token, _)| token.as_str()).collect();
    let listed: Vec<String> = offered
        .iter()
        .map(|(token, meaning)| format!("{token}: {meaning}"))
        .collect();
    let user = format!(
        "{moment}\n## What {} says or does\n{line}\n\nReact as {}: what you think, feel, do and decide.\n\
         \n## Immediate actions available to you\n\
         Choose one engine_action according to your own motivations and the exchange.\n\
         You may refuse a request by choosing none. Copy a token exactly:\n{}\n\
         Only the selected action changes possessions or agreements now. The other fields describe speech,\n\
         expression and private thought; do not describe an unselected transfer, journey or truce as completed.\n",
        addressee_name(records, character),
        string(character, "fullname"),
        listed.join("\n"),
    );
    json!({
        "system": interaction_instructions(records, character),
        "user": user,
        "schema": schemas::interaction_with_actions(&tokens),
        "history": history(game, character),
    })
}

/// The narrator pass's request, for the reaction the character answered
/// with and the engine's receipt for what it applied.
pub fn narrator_request(
    game: &Game,
    character: &Row,
    line: &str,
    reaction: &Map<String, Value>,
    fact: &str,
) -> Value {
    let records = game.records;
    let field = |name: &str| sanitize(reaction.get(name).and_then(Value::as_str).unwrap_or(""));
    let fullname = string(character, "fullname");
    let name = presence(text(character, "nickname")).unwrap_or(fullname);
    let forms = pronouns(character);
    let context = Moment::new(*game).narration_context(false, false);
    let moment = if is_blank(&context) {
        String::new()
    } else {
        format!("## Where this happens\n{context}\n")
    };
    let addressee = addressee_name(records, character);
    let (subject, determiner) = (forms.subject, forms.determiner);
    let says = if forms.plural { "say" } else { "says" };
    let user = format!(
        "{moment}\n## The exchange\n\
         {addressee} says or does: {line}\n\
         {fullname}'s reaction, in {determiner} own words:\n\
         pre_thought: {}\npre_feeling: {}\naction: {}\npost_thought: {}\npost_feeling: {}\n\n\
         ## Instructions\n\
         Write what happens in the second person, present tense, from the player's side. The player is \"you\".\n\
         Refer to the character as {name}. Refer to {fullname} as {}. Use those pronouns and no others.\n\
         The exchange is the subject and the place above is where it happens: use it for what the character\n\
         does with {determiner} hands and eyes, not for a tour. Add nobody who is not listed above.\n\n\
         Everything you know about {name} is the reaction above and the exchange itself. Render the thoughts\n\
         and feelings as what they look like from outside -- a pause, a glance, a change of tone -- and put\n\
         the speech in {name}'s mouth as written. Do not add facts about {name}, do not narrate what\n\
         {subject} will do next, and do not answer for the player.\n\n\
         One or two short paragraphs. Do not offer the player choices and do not end on a question to them.\n\n\
         ## Example input\n\
         {addressee} says or does: \"What are you doing??\"\n\
         {fullname}'s reaction, in {determiner} own words:\n\
         pre_thought: Is that a question for me? I think so. I should probably answer.\n\
         pre_feeling: surprised, nervous\n\
         action: {name} says, \"Huh?\"\n\
         post_thought: Why did I just make that noise?\n\
         post_feeling: anxious, embarrassed\n\n\
         ## Example output\n\
         {name} turns to you, {determiner} eyes wide. It seems you startled {}.\n\
         \"Huh?\" {subject} {says}.\n\
         Immediately {name} shuts {determiner} eyes, apparently embarrassed by the noise {subject} just made.\n\n\
         \n## What the engine actually applied\n{fact}\n\
         This receipt is authoritative about possessions, accompanying the player and fighting.\n\
         If the reaction claims another such change, render that part as a proposal or intention,\n\
         never as a completed act. Do not add a transfer, a journey or a ceasefire to this receipt.\n",
        field("pre_thought"),
        field("pre_feeling"),
        field("action"),
        field("post_thought"),
        field("post_feeling"),
        forms.rule,
        forms.object,
    );
    json!({ "system": null, "user": user, "schema": null, "history": [] })
}
