PRAGMA foreign_keys=OFF;
BEGIN TRANSACTION;
CREATE TABLE IF NOT EXISTS "characters_scenes" ("character_id" integer NOT NULL, "scene_id" integer NOT NULL);
CREATE TABLE IF NOT EXISTS "lab_exits_vantages" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "absent" text, "created_at" datetime(6) NOT NULL, "danger" varchar, "expects_inside_quantifier" varchar, "expects_population" text, "name" varchar NOT NULL, "reached_from" varchar, "teaser" text NOT NULL, "updated_at" datetime(6) NOT NULL, "world" varchar NOT NULL);
CREATE TABLE IF NOT EXISTS "lab_realization_kinds" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "created_at" datetime(6) NOT NULL, "danger" varchar, "expects_danger" text, "expects_gradient" text, "expects_hazard" text, "expects_inside" text, "expects_population" text, "expects_storeys_above" text, "expects_storeys_below" text, "inside" varchar, "name" varchar NOT NULL, "population" varchar, "reached_from" varchar, "teaser" text NOT NULL, "updated_at" datetime(6) NOT NULL, "world" varchar NOT NULL);
CREATE TABLE IF NOT EXISTS "players" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "created_at" datetime(6) NOT NULL, "monthly_limit_usd" decimal(10,4) DEFAULT 1.0 NOT NULL, "name" varchar NOT NULL, "revoked_at" datetime(6), "token_digest" varchar NOT NULL, "updated_at" datetime(6) NOT NULL);
CREATE TABLE IF NOT EXISTS "playthrough_visits" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "created_at" datetime(6) NOT NULL, "location_id" integer NOT NULL, "playthrough_id" integer NOT NULL, "updated_at" datetime(6) NOT NULL);
CREATE TABLE IF NOT EXISTS "ruby_llm_batches" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "batch_protocol" varchar, "chat_ids" json DEFAULT '[]', "chat_type" varchar, "completed" boolean DEFAULT FALSE NOT NULL, "created_at" datetime(6) NOT NULL, "provider" varchar NOT NULL, "provider_batch_id" varchar NOT NULL, "raw_status" varchar, "reported_cost" json, "request_counts" json, "status" varchar NOT NULL, "updated_at" datetime(6) NOT NULL);
CREATE TABLE IF NOT EXISTS "ruby_llm_models" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "capabilities" json DEFAULT '[]', "context_window" integer, "created_at" datetime(6) NOT NULL, "family" varchar, "knowledge_cutoff" date, "max_output_tokens" integer, "metadata" json DEFAULT '{}', "modalities" json DEFAULT '{}', "model_created_at" datetime(6), "model_id" varchar NOT NULL, "name" varchar NOT NULL, "pricing" json DEFAULT '{}', "provider" varchar NOT NULL, "unlisted_at" datetime(6), "updated_at" datetime(6) NOT NULL);
CREATE TABLE IF NOT EXISTS "ruby_llm_tool_calls" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "approval" varchar, "arguments" json DEFAULT '{}', "created_at" datetime(6) NOT NULL, "message_id" integer NOT NULL, "message_type" varchar NOT NULL, "name" varchar NOT NULL, "remote" boolean DEFAULT FALSE NOT NULL, "result_id" integer, "result_type" varchar, "thought_signature" text, "tool_call_id" varchar NOT NULL, "updated_at" datetime(6) NOT NULL);
CREATE TABLE IF NOT EXISTS "ruby_llm_usages" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "cache_read_cost" decimal(16,10), "cache_read_tokens" integer, "cache_write_cost" decimal(16,10), "cache_write_tokens" integer, "chat_id" integer NOT NULL, "chat_type" varchar NOT NULL, "created_at" datetime(6) NOT NULL, "input_cost" decimal(16,10), "input_tokens" integer, "message_id" integer, "message_type" varchar, "model" varchar NOT NULL, "operation" varchar NOT NULL, "output_cost" decimal(16,10), "output_tokens" integer, "provider" varchar NOT NULL, "status" varchar NOT NULL, "thinking_cost" decimal(16,10), "thinking_tokens" integer, "total_cost" decimal(16,10), "updated_at" datetime(6) NOT NULL, CONSTRAINT chk_rails_71abd85d6e CHECK (operation IN ('chat', 'embedding', 'moderation', 'image', 'speech', 'transcription', 'ocr', 'rerank')), CONSTRAINT chk_rails_f50895962a CHECK (status IN ('pending', 'succeeded', 'failed', 'cancelled')));
CREATE TABLE IF NOT EXISTS "ruby_llm_v2_backfills" ("completed" boolean DEFAULT FALSE NOT NULL, "last_id" integer, "task" varchar NOT NULL);
CREATE TABLE IF NOT EXISTS "universes" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "civilizations" text, "created_at" datetime(6) NOT NULL, "economics" text, "geographies" text, "history" text, "physics" text, "politics" text, "religion" text, "technology" text, "updated_at" datetime(6) NOT NULL, "weapons" text, "gravity" varchar);
INSERT INTO universes VALUES(1000000001,'The Kingdom of Durnhold is ruled by the Iron Council, a cabal of knights and priests who enforce the Church’s decrees from the Obsidian Throne in the capital, Spire’s Rest. The goblins are divided into warring clans, the largest being the Blackfang, who control the dungeon where the prince is held. The Trollkin have no settlements, only nomadic bands, while the Veythi operate from floating barges on the river, their only permanent structure being the Rust Market, a black-market hub hidden in the river’s mist. The uneasy peace is maintained by the Iron Council’s threat of annihilation and the goblins’ fear of open war, but raids and executions are common.','2026-09-28 08:04:52.358932','Durnhold’s wealth comes from iron and coal, mined by convicts in the Spire’s shadow. The Church taxes all trade, taking a third of all goods in the name of the Eternal Pyre. Goblins trade in stolen Durnish steel and rare fungi, which they sell to the Veythi for clockwork parts. The Veythi deal in information and forbidden tech, but their currency is blood—each transaction requires a drop from the buyer, stored in vials as proof of the exchange. Trollkin have no economy, surviving by hunting and scavenging.','The kingdom of Durnhold is a land of jagged iron cliffs and black pine forests, its heart dominated by the Ashen Spire, a dead volcano where the Church burns heretics. The goblin caves lie beneath the Spire’s roots, a labyrinth of obsidian tunnels lit by bioluminescent fungi that dim every 12 hours. The nearest town, Hollow’s End, is a day’s ride north, its walls built from the bones of ancient trolls. The river Veyth, wide and slow, marks the border between Durnhold and the goblin territories, its waters undrinkable due to high iron content.','A century ago, the Durnish waged the War of the Spire, driving the goblins underground and sealing the caves with iron gates. The Church then burned the last great library of the Veythi, accusing them of heresy, which forced the river-dwellers into secrecy. Twenty years past, the Iron Council executed the last Trollkin king, scattering his people. Now, the prince’s capture by the Blackfang clan has shattered the fragile truce, as the Council debates whether to negotiate or burn the caves to the ground.','Gravity is consistent at 9.8 m/s², but the world is veined with ley lines—thin, invisible rivers of latent magic that pulse at a frequency of 0.3 Hz, detectable only by iron-forged compasses. These lines can be tapped by blood rituals, but each use drains the caster’s lifespan by exactly one day per minute of magic. The ley lines are finite; overuse in a region causes them to fade for a century. No magic can resurrect the dead or heal mortal wounds, only delay them.','The Iron Council holds power through fear, their knights enforcing the Church’s will with fire and steel. The Blackfang goblins are the only clan bold enough to defy them, using the prince as leverage to demand the return of their sacred obsidian, stolen during the War of the Spire. The Veythi play both sides, selling secrets to the Council while smuggling weapons to the goblins. The Trollkin are caught in the middle, hunted by the Church but too few in number to resist. The prince’s rescue could tip the balance—if he lives, the Council may strike; if he dies, the goblins will have nothing to lose.','The Church of the Eternal Pyre preaches that the world is a forge, and all life must be tempered by suffering to be made pure. They believe the ley lines are the veins of a sleeping god, and that magic is theft from the divine. The goblins worship the Silent Maw, a god who demands blood and silence, believing that all noise is a lie. The Veythi have no gods, only the River, which they see as a force of change and decay. The Trollkin revere the First Maw, a primordial troll said to have birthed their race, but their rituals are secret, performed only in the dark.','The world is locked in a late medieval stasis, with iron and steel as the primary metals. Clockwork mechanisms exist but are rare, limited to devices no larger than a loaf of bread due to the scarcity of precision gears. Gunpowder is known but unstable, often exploding prematurely in 1 out of 5 attempts. Most advancements are suppressed by the Church of the Eternal Pyre, which burns any invention deemed heretical.','2026-09-28 08:04:52.358932','Swords, axes, and maces dominate, with the knightly class wielding blades of cold-forged Veldarian steel, which never rusts but shatters if struck by goblin-wrought obsidian. Goblins favor curved daggers coated in paralyzing venom, effective only if the wound draws blood. Crossbows are used by mercenaries but require 30 seconds to reload, making them impractical in close quarters. No ranged weapon can pierce the hide of a cave troll, which guards the dungeon’s depths.',NULL);
CREATE TABLE IF NOT EXISTS "characters" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "age" integer, "appearance" text, "backstory" text, "conscious_desire" text, "created_at" datetime(6) NOT NULL, "deliberately_absent" boolean DEFAULT FALSE NOT NULL, "desire_pursuit" varchar, "dexterity" integer, "dislikes" text, "fears" text, "fullname" varchar, "hit_die" integer, "hostile" boolean DEFAULT FALSE NOT NULL, "is_companion" boolean, "is_protagonist" boolean DEFAULT FALSE NOT NULL, "level" integer, "likes" text, "location_id" integer, "need_pursuit" varchar, "nickname" varchar, "personality" text, "race_id" integer NOT NULL, "recognized_need" text, "sex" varchar, "story_id" integer NOT NULL, "strength" integer, "unconscious_desire" text, "unrecognized_need" text, "updated_at" datetime(6) NOT NULL, "will" integer, "x" integer, "y" integer, CONSTRAINT "fk_rails_56a213cbe2"
FOREIGN KEY ("race_id")
  REFERENCES "races" ("id")
, CONSTRAINT "fk_rails_0b4b445641"
FOREIGN KEY ("location_id")
  REFERENCES "locations" ("id")
, CONSTRAINT "fk_rails_6608c248e8"
FOREIGN KEY ("story_id")
  REFERENCES "stories" ("id")
);
INSERT INTO characters VALUES(1000000001,107,'Towering over most, Veythra’s gray fur is streaked with silver, a sign of her age, and her broad frame is draped in a tattered cloak woven from river reeds. Her tusks are filed to points, and her amber eyes glow faintly in the dark, a trait of her lineage. Scars crisscross her arms, each a story she refuses to tell.','Veythra was born on a Veythi barge, the only Trollkin child among river traders who saw her as an omen. Raised by an exiled Durnish knight turned mentor, she learned the ways of the wild and the weight of secrets. The Church’s hunts forced her into hiding, but she never forgot the debt she owed to the river for sparing her life. Now, she wanders the banks of the Veyth, tracking the movements of the Blackfang goblins, driven by a quiet vow to protect the last of her kind. The prince’s capture has drawn her attention, as the goblins’ boldness threatens the fragile balance she has spent a lifetime maintaining. She seeks not glory, but survival—for herself and the scattered remnants of the Trollkin.',NULL,'2026-09-28 08:04:53.082792',0,NULL,7,'fire, the Church’s hymns, the taste of cooked meat, being watched, iron chains','being trapped underground, the Church’s pyres, losing her sense of smell, the Silent Maw’s hunger','Veythra the Silent',8,0,0,1,1,'raw fish, the sound of flowing water, the scent of rain on stone, the weight of a well-balanced axe, solitude',NULL,NULL,'The River’s Shadow','Veythra is reserved and deliberate, speaking only when necessary and often in riddles. She carries a quiet intensity, observing the world with a hunter’s patience, though she treats others with a cautious respect born of survival. Her loyalty is rare but absolute once given.',1000000003,NULL,'female',1000000001,13,NULL,NULL,'2026-09-28 08:04:53.082792',10,NULL,NULL);
INSERT INTO characters VALUES(1000000002,54,'A tall Trollkin woman with weathered, grayish skin and deep-set, amber eyes. She wears a patched cloak of stitched hides and carries a notched axe at her belt. Her hair, streaked with white, is tied back in a loose braid.','Gorva was once part of a nomadic band, but age and injury left her behind. She now drifts between towns like Hollow’s End, trading odd jobs for coin or shelter. She knows the paths through the black pines better than most, and she has a grudging respect for the Iron Council’s reach.',NULL,'2026-09-28 08:04:53.122359',0,NULL,14,'fools, the Church, wasted time','fire, deep water','Gorva the Wanderer',6,0,0,0,1,'quiet, strong drink, sharp blades',1000000007,NULL,'Gorva','Gorva speaks little but watches everything, her voice a low rumble when she does. She treats strangers with cautious curiosity, though her patience wears thin quickly.',1000000003,NULL,'female',1000000001,10,NULL,NULL,'2026-09-28 08:04:53.122359',6,NULL,NULL);
INSERT INTO characters VALUES(1000000003,41,'A Veythi woman with dark, braided hair streaked with silver, her sharp features framed by a high-collared coat of river-worn leather. Her hands are calloused from years of hauling nets and ropes, and her eyes flicker with the restlessness of someone used to the water.','Lyssa was born on the barges of the Veythi but left the river life after a falling-out with her kin. Now she trades in Hollow’s End, smuggling goods between the town and the Rust Market hidden in the river’s mist. She knows the Iron Council’s patrols and the goblin raids better than anyone.',NULL,'2026-09-28 08:04:53.131360',0,NULL,7,'the Church, betrayal, cold nights','being trapped, the Spire’s shadow','Lyssa of the Mist',10,0,0,0,1,'bargaining, river stories, fine cloth',1000000007,NULL,'Lyssa','Lyssa is quick with a joke and quicker with a deal, her voice smooth as the river’s current. She sizes up strangers with a practiced eye, always weighing what they might be worth.',1000000004,NULL,'trans woman',1000000001,10,NULL,NULL,'2026-09-28 08:04:53.131360',9,NULL,NULL);
INSERT INTO characters VALUES(1000000004,19,'A tall Durnish youth gone gaunt, his ashen skin greyer still for a season without sun, and his dark hair cropped to the scalp where the goblins took it for their own uses. What is left of a doublet of Council blue hangs off him, rotted at the seams and stiff with old blood at the collar. His wrists carry two bright rings of scar from the chain, and the third finger of his left hand is scarred around the base where the signet was pulled over the knuckle. His eyes have adjusted to the fungi’s glow and water in any brighter light.','Aurel Durn is the last living son of the Iron Council’s ruling line, and until the spring he had never been further from the Obsidian Throne than the smelters at the Spire’s foot. He was taken on the north road to Hollow’s End with eleven men-at-arms around him, none of whom outlived the ambush; the Blackfang came out of the black pines without a sound, which is how the clan hunts, and cut the horses first. Somewhere outside the iron gate they wrenched the signet from his left hand, and he understood, watching it go into the mud, that they meant to be believed rather than paid. He has been in the dark since. The clan keeps him whole and keeps him fed on fungus-mash because a corpse buys nothing, and they have told him plainly what he is worth: the sacred obsidian the Durnish took in the War of the Spire, weighed out and returned. He counts the dimming of the fungi to keep the days, and he has counted enough of them to know the arithmetic the Church will do. Rescue is not what he expects. Being burned with the caves is.',NULL,'2026-09-28 08:04:53.141565',0,NULL,10,'fungus-mash, the Church’s arithmetic, being spoken to as a price, pity, torchlight held close to his face','the Council burning the caves with him inside them, the Silent Maw’s hunger, losing his count of the hours, hearing his own name used as a sum','Prince Aurel Durn',6,0,0,0,1,'the twelve-hour dimming of the fungi, which is his only clock, clean water, the sound of iron being worked, his mother’s salt-and-iron psalm, being told the truth about numbers',1000000014,NULL,'Aurel','Aurel is courteous in a way that has stopped costing him anything, and he has learned the goblin habit of silence well enough to make Durnish visitors uneasy. He listens first, at length, before he says a word, and what he says is usually a price or a warning. He does not beg and he does not thank; he bargains, even from the floor of a cell, because bargaining is the only office left to him and he was raised to hold one.',1000000001,NULL,'male',1000000001,8,NULL,NULL,'2026-09-28 08:04:53.141565',16,10,7);
CREATE TABLE IF NOT EXISTS "chats" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "cancelled" boolean DEFAULT FALSE NOT NULL, "character_id" integer, "created_at" datetime(6) NOT NULL, "model_id_string" varchar, "player_id" integer, "playthrough_id" integer, "purpose" varchar, "ruby_llm_model_id" integer, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_415520c982"
FOREIGN KEY ("playthrough_id")
  REFERENCES "playthroughs" ("id")
, CONSTRAINT "fk_rails_5c87bd0780"
FOREIGN KEY ("character_id")
  REFERENCES "characters" ("id")
, CONSTRAINT "fk_rails_ea73c47d5b"
FOREIGN KEY ("player_id")
  REFERENCES "players" ("id")
, CONSTRAINT "fk_rails_d8a12df187"
FOREIGN KEY ("ruby_llm_model_id")
  REFERENCES "ruby_llm_models" ("id")
);
CREATE TABLE IF NOT EXISTS "interactions" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "action" text, "action_fact" text, "action_status" varchar, "character_id" integer NOT NULL, "created_at" datetime(6) NOT NULL, "engine_action" varchar, "inner_resolution" text, "location_id" integer, "post_feeling" text, "post_thought" text, "pre_feeling" text, "pre_thought" text, "scene_id" integer, "summary" text, "updated_at" datetime(6) NOT NULL, "user_input" text, CONSTRAINT "fk_rails_294d4ae9f8"
FOREIGN KEY ("location_id")
  REFERENCES "locations" ("id")
, CONSTRAINT "fk_rails_6b73a2cd17"
FOREIGN KEY ("character_id")
  REFERENCES "characters" ("id")
, CONSTRAINT "fk_rails_b73583a008"
FOREIGN KEY ("scene_id")
  REFERENCES "scenes" ("id")
);
CREATE TABLE IF NOT EXISTS "items" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "bulk" varchar DEFAULT 'handy' NOT NULL, "character_id" integer, "combustible" boolean DEFAULT FALSE NOT NULL, "created_at" datetime(6) NOT NULL, "description" text, "disposition" varchar DEFAULT 'intact' NOT NULL, "inscription" text, "location_id" integer, "name" varchar, "playthrough_id" integer, "properties" text, "readable" boolean DEFAULT FALSE NOT NULL, "template_id" integer, "updated_at" datetime(6) NOT NULL, "use_kind" varchar DEFAULT 'ordinary' NOT NULL, "x" integer, "y" integer, "fragility" varchar DEFAULT 'sturdy' NOT NULL, CONSTRAINT "fk_rails_e8ed83a2e6"
FOREIGN KEY ("location_id")
  REFERENCES "locations" ("id")
, CONSTRAINT "fk_rails_35423c7ef8"
FOREIGN KEY ("character_id")
  REFERENCES "characters" ("id")
, CONSTRAINT "fk_rails_5248e92099"
FOREIGN KEY ("playthrough_id")
  REFERENCES "playthroughs" ("id")
);
INSERT INTO items VALUES(1000000001,'handy',NULL,0,'2026-09-28 08:04:52.545111','A small, rusted key lying near the gate’s hinge, its bow shaped like a serpent’s head. It looks too delicate to have opened this gate.','intact',NULL,1000000001,'iron key',NULL,NULL,0,NULL,'2026-09-28 08:04:52.545111','ordinary',NULL,NULL,'sturdy');
INSERT INTO items VALUES(1000000002,'handy',NULL,0,'2026-09-28 08:04:52.570080','A heavy gold ring, its surface etched with the royal crest of Durnhold, still caked with mud. The metal is cold and slightly bent, as if wrenched from a finger.','intact',NULL,1000000001,'prince''s signet ring',NULL,NULL,0,NULL,'2026-09-28 08:04:52.570080','ordinary',NULL,NULL,'sturdy');
INSERT INTO items VALUES(1000000003,'handy',NULL,0,'2026-09-28 08:04:52.626339','A rolled parchment, its edges frayed and its surface stained with moisture. The ink is smudged but still legible in places.','intact','Beware the teeth of the dark. They hunger.',1000000003,'damp scroll',NULL,NULL,1,NULL,'2026-09-28 08:04:52.626339','ordinary',NULL,NULL,'sturdy');
INSERT INTO items VALUES(1000000004,'handy',NULL,0,'2026-09-28 08:04:52.663889','A jagged fragment of blackened metal, its edges sharp and its surface pitted with age. It looks like it was broken from something larger, perhaps a blade or a tool.','intact',NULL,1000000003,'iron shard',NULL,NULL,0,NULL,'2026-09-28 08:04:52.663889','ordinary',NULL,NULL,'sturdy');
INSERT INTO items VALUES(1000000005,'handy',NULL,0,'2026-09-28 08:04:52.695964','A small, pitted blade of cheap iron, its edge dulled by time and neglect. The hilt is wrapped in frayed leather, still damp to the touch.','intact',NULL,1000000004,'rusted dagger',NULL,NULL,0,NULL,'2026-09-28 08:04:52.695964','ordinary',NULL,NULL,'sturdy');
INSERT INTO items VALUES(1000000006,'handy',NULL,0,'2026-09-28 08:04:52.705565','A strip of coarse, once-red fabric, now faded and stained with something dark. It smells faintly of sweat and iron.','intact',NULL,1000000004,'tattered cloth',NULL,NULL,0,NULL,'2026-09-28 08:04:52.705565','ordinary',NULL,NULL,'sturdy');
INSERT INTO items VALUES(1000000007,'handy',NULL,0,'2026-09-28 08:04:52.723908','A blackened fragment of bone, likely from an animal, its surface cracked and brittle from the heat of the pyres. It’s light enough to carry but offers no immediate use.','intact',NULL,1000000005,'charred bone',NULL,NULL,0,NULL,'2026-09-28 08:04:52.723908','ordinary',NULL,NULL,'sturdy');
INSERT INTO items VALUES(1000000008,'handy',NULL,0,'2026-09-28 08:04:52.733267','A thin, rusted shard of iron, no larger than a palm, broken off from the Spire’s cliffs. It’s sharp on one edge but too small to be a weapon.','intact',NULL,1000000005,'iron flake',NULL,NULL,0,NULL,'2026-09-28 08:04:52.733267','ordinary',NULL,NULL,'sturdy');
INSERT INTO items VALUES(1000000009,'handy',NULL,0,'2026-09-28 08:04:52.819938','A single, tarnished coin bearing the faded crest of the Iron Council, its edges worn smooth by time. It lies half-buried in the dirt near the gate.','intact',NULL,1000000007,'iron coin',NULL,NULL,0,NULL,'2026-09-28 08:04:52.819938','ordinary',NULL,NULL,'sturdy');
INSERT INTO items VALUES(1000000010,'handy',NULL,0,'2026-09-28 08:04:52.842890','A small, splintered fragment of bone, likely broken from the wall during construction or repair. It is light but sharp at the edges.','intact',NULL,1000000007,'troll-bone shard',NULL,NULL,0,NULL,'2026-09-28 08:04:52.842890','ordinary',NULL,NULL,'sturdy');
INSERT INTO items VALUES(1000000011,'handy',NULL,0,'2026-09-28 08:04:52.865958','A small, blackened pendant of twisted iron, its surface pitted and warped by fire. The chain is broken, leaving only a jagged stub.','intact',NULL,1000000008,'charred amulet',NULL,NULL,0,NULL,'2026-09-28 08:04:52.865958','ordinary',NULL,NULL,'sturdy');
INSERT INTO items VALUES(1000000012,'handy',NULL,0,'2026-09-28 08:04:52.874436','A single, rusted nail, its head bent and its tip blunt from years of exposure to the elements. It’s cold to the touch, as if it has absorbed the chill of the Spire’s shadow.','intact',NULL,1000000008,'iron nail',NULL,NULL,0,NULL,'2026-09-28 08:04:52.874436','ordinary',NULL,NULL,'sturdy');
CREATE TABLE IF NOT EXISTS "lab_exits_judgements" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "aspects" text, "created_at" datetime(6) NOT NULL, "expects_inside" text, "expects_population" text, "name" varchar NOT NULL, "name_key" varchar NOT NULL, "note" text, "updated_at" datetime(6) NOT NULL, "vantage_id" integer NOT NULL, "verdict" varchar, CONSTRAINT "fk_rails_cbf2b734ae"
FOREIGN KEY ("vantage_id")
  REFERENCES "lab_exits_vantages" ("id")
);
CREATE TABLE IF NOT EXISTS "lab_exits_samples" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "aspects" text, "created_at" datetime(6) NOT NULL, "note" text, "row" json DEFAULT '{}' NOT NULL, "updated_at" datetime(6) NOT NULL, "vantage_id" integer NOT NULL, "verdict" varchar, CONSTRAINT "fk_rails_3cc36a20a9"
FOREIGN KEY ("vantage_id")
  REFERENCES "lab_exits_vantages" ("id")
);
CREATE TABLE IF NOT EXISTS "lab_realization_samples" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "aspects" text, "created_at" datetime(6) NOT NULL, "kind_id" integer NOT NULL, "note" text, "row" json DEFAULT '{}' NOT NULL, "updated_at" datetime(6) NOT NULL, "verdict" varchar, CONSTRAINT "fk_rails_7bb6f06ff4"
FOREIGN KEY ("kind_id")
  REFERENCES "lab_realization_kinds" ("id")
);
CREATE TABLE IF NOT EXISTS "location_connections" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "barrier" varchar DEFAULT 'open' NOT NULL, "connected_location_id" integer NOT NULL, "created_at" datetime(6) NOT NULL, "distance" text, "hazard" varchar, "hazard_die" integer, "key_template_id" integer, "location_id" integer NOT NULL, "time_to_travel" text, "travel_method" text, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_99792c7943"
FOREIGN KEY ("location_id")
  REFERENCES "locations" ("id")
, CONSTRAINT "fk_rails_34dc49debe"
FOREIGN KEY ("key_template_id")
  REFERENCES "items" ("id")
 ON DELETE SET NULL, CONSTRAINT "fk_rails_37a61eef17"
FOREIGN KEY ("connected_location_id")
  REFERENCES "locations" ("id")
);
INSERT INTO location_connections VALUES(1000000001,'open',1000000002,'2026-09-28 08:04:53.268256','a short walk',NULL,NULL,NULL,1000000001,'about 5 minutes','walking','2026-09-28 08:04:53.268256');
INSERT INTO location_connections VALUES(1000000002,'open',1000000001,'2026-09-28 08:04:53.275773','a short walk',NULL,NULL,NULL,1000000002,'about 5 minutes','walking','2026-09-28 08:04:53.275773');
INSERT INTO location_connections VALUES(1000000003,'open',1000000003,'2026-09-28 08:04:53.281288','adjacent',NULL,NULL,NULL,1000000001,'about a minute','walking','2026-09-28 08:04:53.281288');
INSERT INTO location_connections VALUES(1000000004,'open',1000000001,'2026-09-28 08:04:53.286293','adjacent',NULL,NULL,NULL,1000000003,'about a minute','walking','2026-09-28 08:04:53.286293');
INSERT INTO location_connections VALUES(1000000005,'open',1000000004,'2026-09-28 08:04:53.289665','a short walk',NULL,NULL,NULL,1000000002,'about 5 minutes','walking','2026-09-28 08:04:53.289665');
INSERT INTO location_connections VALUES(1000000006,'open',1000000002,'2026-09-28 08:04:53.298873','a short walk',NULL,NULL,NULL,1000000004,'about 5 minutes','walking','2026-09-28 08:04:53.298873');
INSERT INTO location_connections VALUES(1000000007,'open',1000000005,'2026-09-28 08:04:53.302511','a short walk',NULL,NULL,NULL,1000000002,'about 5 minutes','walking','2026-09-28 08:04:53.302511');
INSERT INTO location_connections VALUES(1000000008,'open',1000000002,'2026-09-28 08:04:53.305176','a short walk',NULL,NULL,NULL,1000000005,'about 5 minutes','walking','2026-09-28 08:04:53.305176');
INSERT INTO location_connections VALUES(1000000009,'open',1000000004,'2026-09-28 08:04:53.308078','a short walk',NULL,NULL,NULL,1000000003,'about 5 minutes','walking','2026-09-28 08:04:53.308078');
INSERT INTO location_connections VALUES(1000000010,'open',1000000003,'2026-09-28 08:04:53.311375','a short walk',NULL,NULL,NULL,1000000004,'about 5 minutes','walking','2026-09-28 08:04:53.311375');
INSERT INTO location_connections VALUES(1000000011,'open',1000000012,'2026-09-28 08:04:53.313567','adjacent',NULL,NULL,NULL,1000000004,'about a minute','walking','2026-09-28 08:04:53.313567');
INSERT INTO location_connections VALUES(1000000012,'open',1000000004,'2026-09-28 08:04:53.315747','adjacent',NULL,NULL,NULL,1000000012,'about a minute','walking','2026-09-28 08:04:53.315747');
INSERT INTO location_connections VALUES(1000000013,'open',1000000006,'2026-09-28 08:04:53.318777','a long journey',NULL,NULL,NULL,1000000005,'about 48 minutes','riding','2026-09-28 08:04:53.318777');
INSERT INTO location_connections VALUES(1000000014,'open',1000000005,'2026-09-28 08:04:53.324552','a long journey',NULL,NULL,NULL,1000000006,'about 48 minutes','riding','2026-09-28 08:04:53.324552');
INSERT INTO location_connections VALUES(1000000015,'open',1000000007,'2026-09-28 08:04:53.327352','a long journey',NULL,NULL,NULL,1000000005,'about 48 minutes','riding','2026-09-28 08:04:53.327352');
INSERT INTO location_connections VALUES(1000000016,'open',1000000005,'2026-09-28 08:04:53.333419','a long journey',NULL,NULL,NULL,1000000007,'about 48 minutes','riding','2026-09-28 08:04:53.333419');
INSERT INTO location_connections VALUES(1000000017,'open',1000000007,'2026-09-28 08:04:53.338611','a long journey',NULL,NULL,NULL,1000000006,'about 48 minutes','riding','2026-09-28 08:04:53.338611');
INSERT INTO location_connections VALUES(1000000018,'open',1000000006,'2026-09-28 08:04:53.341784','a long journey',NULL,NULL,NULL,1000000007,'about 48 minutes','riding','2026-09-28 08:04:53.341784');
INSERT INTO location_connections VALUES(1000000019,'open',1000000008,'2026-09-28 08:04:53.345171','a short walk',NULL,NULL,NULL,1000000006,'about 5 minutes','walking','2026-09-28 08:04:53.345171');
INSERT INTO location_connections VALUES(1000000020,'open',1000000006,'2026-09-28 08:04:53.350271','a short walk',NULL,NULL,NULL,1000000008,'about 5 minutes','walking','2026-09-28 08:04:53.350271');
INSERT INTO location_connections VALUES(1000000021,'open',1000000009,'2026-09-28 08:04:53.355702','across the district',NULL,NULL,NULL,1000000006,'about 20 minutes','walking','2026-09-28 08:04:53.355702');
INSERT INTO location_connections VALUES(1000000022,'open',1000000006,'2026-09-28 08:04:53.370119','across the district',NULL,NULL,NULL,1000000009,'about 20 minutes','walking','2026-09-28 08:04:53.370119');
INSERT INTO location_connections VALUES(1000000023,'open',1000000009,'2026-09-28 08:04:53.372911','a short walk',NULL,NULL,NULL,1000000007,'about 5 minutes','walking','2026-09-28 08:04:53.372911');
INSERT INTO location_connections VALUES(1000000024,'open',1000000007,'2026-09-28 08:04:53.377105','a short walk',NULL,NULL,NULL,1000000009,'about 5 minutes','walking','2026-09-28 08:04:53.377105');
INSERT INTO location_connections VALUES(1000000025,'open',1000000010,'2026-09-28 08:04:53.380799','a long journey',NULL,NULL,NULL,1000000007,'about 48 minutes','riding','2026-09-28 08:04:53.380799');
INSERT INTO location_connections VALUES(1000000026,'open',1000000007,'2026-09-28 08:04:53.383624','a long journey',NULL,NULL,NULL,1000000010,'about 48 minutes','riding','2026-09-28 08:04:53.383624');
INSERT INTO location_connections VALUES(1000000027,'open',1000000009,'2026-09-28 08:04:53.387766','a long journey',NULL,NULL,NULL,1000000008,'about 48 minutes','riding','2026-09-28 08:04:53.387766');
INSERT INTO location_connections VALUES(1000000028,'open',1000000008,'2026-09-28 08:04:53.390587','a long journey',NULL,NULL,NULL,1000000009,'about 48 minutes','riding','2026-09-28 08:04:53.390587');
INSERT INTO location_connections VALUES(1000000029,'open',1000000013,'2026-09-28 08:04:53.394305','adjacent',NULL,NULL,NULL,1000000012,'about a minute','walking','2026-09-28 08:04:53.394305');
INSERT INTO location_connections VALUES(1000000030,'open',1000000012,'2026-09-28 08:04:53.397121','adjacent',NULL,NULL,NULL,1000000013,'about a minute','walking','2026-09-28 08:04:53.397121');
INSERT INTO location_connections VALUES(1000000031,'open',1000000015,'2026-09-28 08:04:53.400653','adjacent',NULL,NULL,NULL,1000000012,'about a minute','walking','2026-09-28 08:04:53.400653');
INSERT INTO location_connections VALUES(1000000032,'open',1000000012,'2026-09-28 08:04:53.403499','adjacent',NULL,NULL,NULL,1000000015,'about a minute','walking','2026-09-28 08:04:53.403499');
INSERT INTO location_connections VALUES(1000000033,'open',1000000014,'2026-09-28 08:04:53.406731','adjacent',NULL,NULL,NULL,1000000013,'about a minute','walking','2026-09-28 08:04:53.406731');
INSERT INTO location_connections VALUES(1000000034,'open',1000000013,'2026-09-28 08:04:53.411910','adjacent',NULL,NULL,NULL,1000000014,'about a minute','walking','2026-09-28 08:04:53.411910');
INSERT INTO location_connections VALUES(1000000035,'open',1000000015,'2026-09-28 08:04:53.416790','adjacent',NULL,NULL,NULL,1000000014,'about a minute','walking','2026-09-28 08:04:53.416790');
INSERT INTO location_connections VALUES(1000000036,'open',1000000014,'2026-09-28 08:04:53.418945','adjacent',NULL,NULL,NULL,1000000015,'about a minute','walking','2026-09-28 08:04:53.418945');
CREATE TABLE IF NOT EXISTS "locations" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "created_at" datetime(6) NOT NULL, "danger" varchar DEFAULT 'safe' NOT NULL, "depth" integer, "description" text, "detail_level" varchar DEFAULT 'stub' NOT NULL, "generation_checkpoint" json, "hazard" varchar, "hazard_die" integer, "last_protagonist_visit" datetime(6), "lore" text, "mobile" boolean DEFAULT FALSE NOT NULL, "name" varchar, "parent_location_id" integer, "population" varchar, "story_id" integer NOT NULL, "teaser" text, "updated_at" datetime(6) NOT NULL, "width" integer, "x" integer, "y" integer, "z" integer, "surface" varchar, CONSTRAINT "fk_rails_5bc98acf09"
FOREIGN KEY ("parent_location_id")
  REFERENCES "locations" ("id")
, CONSTRAINT "fk_rails_fedd9b21a0"
FOREIGN KEY ("story_id")
  REFERENCES "stories" ("id")
);
INSERT INTO locations VALUES(1000000001,'2026-09-28 08:04:52.461303','safe',NULL,'The iron gate lies half-buried in the stone floor, its jagged teeth glinting dully in the pale glow of bioluminescent fungi clinging to the cavern walls. The air is thick with the scent of damp earth and rust, the taste of iron sharp on your tongue. The tunnel ahead yawns black and slick with moisture, the faint sound of skittering claws echoing from its depths. Your boot sinks slightly into the muddy threshold, where a single, deep groove cuts across the stone—fresh, as if something heavy was dragged through. The dim light flickers, casting long, wavering shadows that seem to crawl along the obsidian walls.','realized',NULL,NULL,NULL,NULL,'This chamber marks the final Durnish outpost before the goblin tunnels, a last line of defense built into the roots of the Ashen Spire. The gate was forged by the Iron Council to seal the caves after the Blackfang raids grew bolder, its mechanism designed to lock from the outside. The groove in the stone is new, likely made by the prince’s captors as they hauled him deeper into the labyrinth. The fungi here pulse faintly in time with the ley lines, though their glow is too weak to reveal what lurks beyond. The Church’s decree to retrieve the prince has driven many knights this far, but few return.',0,'Iron Gate Chamber',NULL,NULL,1000000001,'A cavernous entryway where the last iron gate of Durnhold’s defense has just been forced open, its teeth embedded in the stone floor.','2026-09-28 08:04:52.461303',NULL,NULL,NULL,NULL,NULL);
INSERT INTO locations VALUES(1000000002,'2026-09-28 08:04:52.590330','safe',NULL,'The tunnel slopes upward, its rough-hewn walls slick with moisture and streaked with veins of iron ore. You feel the faintest draft of cooler air from above, carrying the scent of torch smoke and damp stone. The bioluminescent fungi here are sparse, their dim glow barely illuminating the uneven footing under your boots. The ceiling presses low in places, forcing you to duck as you ascend. Somewhere ahead, a distant flicker of torchlight hints at the world beyond the caves.','realized',NULL,NULL,NULL,NULL,'This passage was carved by goblin labor under the lash of Durnish overseers, a forced route to the surface for the Blackfang clan’s tribute and prisoners. The iron veins in the stone were once mined, but the seams played out decades ago, leaving only the damp, echoing tunnel. The Church occasionally uses it to drag heretics down to the Spire’s roots, though most never return. The torchlight above belongs to the watch posted at the cave’s mouth, a reminder of the world you left behind.',0,'tunnel upward',NULL,NULL,1000000001,'The sloping passage back toward the surface, where the faintest glimmer of torchlight hints at the world above.','2026-09-28 08:04:52.590330',NULL,NULL,NULL,NULL,NULL);
INSERT INTO locations VALUES(1000000003,'2026-09-28 08:04:52.602325','safe',NULL,'You stand at the mouth of a tunnel carved from black obsidian, its walls glistening with moisture that drips in slow, uneven rhythms. The air is thick with the scent of damp stone and something metallic, like old blood. Faint blue-green fungi cling to the ceiling, casting a sickly glow that barely reaches the slick, uneven floor. From the depths, the sound of skittering claws echoes, sharp and deliberate, as if something is testing the edges of the dark. The cold seeps through your armor, and the silence between the sounds feels heavier than the stone itself.','realized',NULL,NULL,NULL,NULL,'This tunnel is part of the Blackfang goblin clan’s labyrinth beneath the Ashen Spire, a network of passages hewn from the volcano’s ancient roots. The obsidian here was formed by a long-dead eruption, its surface polished smooth by centuries of water and the passage of countless goblin feet. The fungi are a symbiotic growth, fed by the iron-rich waters seeping from the stone, and their dim light is all that keeps the tunnels from absolute darkness. The skittering is likely a goblin scout or a cave-dwelling predator, both drawn to the scent of intruders. The Church’s inquisitors have long suspected these tunnels lead to the prince’s prison, but none have returned to confirm it.',0,'obsidian maw',NULL,NULL,1000000001,'The yawning black tunnel ahead, its walls slick with moisture and the distant sound of skittering claws echoing from the dark.','2026-09-28 08:04:52.602325',NULL,NULL,NULL,NULL,NULL);
INSERT INTO locations VALUES(1000000004,'2026-09-28 08:04:52.677993','safe',NULL,'The obsidian walls of the tunnel glisten with moisture, their jagged edges catching the dim glow of bioluminescent fungi clinging to the ceiling. The air is thick with the scent of damp earth and something metallic, like old blood. The passage slopes downward, the floor uneven and slick underfoot, and the skittering sounds echo from deeper within the dark, growing louder as you listen. The faint pulse of a ley line hums beneath your boots, detectable only by the faintest vibration in the iron of your armor. A cold draft curls around your ankles, carrying the musk of wet stone and something faintly rotten.','realized',NULL,NULL,NULL,NULL,'This tunnel is part of the Blackfang clan’s network beneath the Ashen Spire, carved by goblin hands over centuries into the obsidian veins of the dead volcano. The ley line here is weak but active, often used by goblin shamans for minor rituals, though its proximity to the surface has left it nearly spent. The skittering sounds are likely cave rats or worse—goblin sentries, or something that has made its home in the dark. The dampness never fully dries, and the fungi thrive in the absence of sunlight.',0,'Blackfang Tunnel',NULL,NULL,1000000001,'The obsidian passage slopes downward, the skittering sounds growing louder as the light fades.','2026-09-28 08:04:52.677993',NULL,NULL,NULL,NULL,NULL);
INSERT INTO locations VALUES(1000000005,'2026-09-28 08:04:52.712332','dangerous',NULL,'You stand on the ashen slope of the Spire, the wind biting at your exposed skin as it carries the acrid tang of burning pitch from the distant pyres. The cave mouth yawns behind you, its darkness a stark contrast to the pale, powdery ground underfoot. The air is dry here, the heat of the sun baking the iron-rich earth into a brittle crust that cracks beneath your boots. To the north, the black pines of Durnhold stretch like skeletal fingers, while the river Veyth glints far below, its surface a dull, rusted mirror. The only sound is the whisper of the wind and the occasional groan of the Spire’s iron cliffs shifting in the heat.','realized',NULL,NULL,NULL,NULL,'This is the surface entrance to the goblin caves, a jagged scar in the Ashen Spire’s flank where the earth has been worn away by centuries of wind and water. The Church’s heretic burnings have left the air thick with the scent of charred flesh and pitch, and the iron-rich soil here is barren, unable to sustain life beyond the hardy black pines. The cave itself was once a natural fissure, but goblin hands have widened it over generations, carving a path into the depths below. The Spire’s slopes are treacherous, littered with loose rock and the occasional bone of some unfortunate who misjudged their footing.',0,'surface',NULL,NULL,1000000001,'The mouth of the cave opens to the ashen slopes of the Spire, where the wind carries the scent of burning pitch.','2026-09-28 08:04:52.712332',NULL,NULL,NULL,NULL,NULL);
INSERT INTO locations VALUES(1000000006,'2026-09-28 08:04:52.739087','safe',NULL,'You stand in the shadow of Spire’s Rest, its iron walls looming high and unbroken, their surfaces pitted with age and the scars of old sieges. The air carries the sharp tang of smelted iron and the faint, acrid bite of burning pitch from the Church’s ever-lit braziers atop the towers. Black banners bearing the sigil of the Eternal Pyre snap in the cold wind, their edges frayed but defiant. The cobbled streets beneath your boots are slick with recent rain, reflecting the dim glow of lanterns that flicker in the gathering dusk. Somewhere beyond the walls, the distant clang of a smith’s hammer rings out, steady and unyielding.','realized',NULL,NULL,NULL,NULL,'Spire’s Rest is the capital of Durnhold, built at the foot of the Ashen Spire by the Iron Council to serve as the seat of their power and the heart of the Church of the Eternal Pyre. Its walls were forged from the iron of the Spire itself, said to be blessed by the first High Inquisitor to repel both blade and spell. The city thrives on the labor of its forges and the fear of its people, with the Church’s influence woven into every street and tower. Here, heresy is met with fire, and progress is measured in the weight of chains and the sharpness of swords.',0,'Spire’s Rest',NULL,NULL,1000000001,'The capital’s iron walls rise in the distance, its towers black against the ashen sky, where the Church’s banners snap in the wind.','2026-09-28 08:04:52.739087',NULL,NULL,NULL,NULL,NULL);
INSERT INTO locations VALUES(1000000007,'2026-09-28 08:04:52.744240','dangerous',NULL,'You stand before the troll-bone walls of Hollow’s End, their jagged edges bleached white by time and weather. The air smells of damp pine and the faint, metallic tang of the river Veyth to the south. The town’s gates, a day’s ride north through the black pines, loom ahead, their iron reinforcements rusted but still imposing. A few torches flicker in the gathering dusk, casting long shadows across the packed earth. The wind carries the distant clatter of a blacksmith’s hammer and the murmur of hushed voices from within.','realized',NULL,NULL,NULL,NULL,'Hollow’s End was built from the bones of ancient trolls, a grim testament to the Iron Council’s victory over the nomadic Trollkin centuries ago. The town serves as the last outpost of Durnhold before the goblin caves and the river Veyth, a place of uneasy trade and whispered deals. The Church’s influence is thin here, its decrees enforced only when convenient, and the townsfolk have learned to look the other way. The walls, though imposing, are more for show than defense, as the true threats lie beyond the pines.',0,'Hollow’s End',NULL,NULL,1000000001,'The troll-bone walls of the nearest town loom to the north, its gates a day’s hard ride away through the black pines.','2026-09-28 08:04:52.744240',NULL,NULL,NULL,NULL,NULL);
INSERT INTO locations VALUES(1000000008,'2026-09-28 08:04:52.852425','uneasy',NULL,'You stand on the ashen slope of the dead volcano, the air thick with the acrid tang of old smoke and the metallic bite of iron. The ground beneath your boots is a mosaic of blackened stone and rusted pyre remnants, the charred bones of heretics still half-buried in the soot. Above, the Spire’s jagged rim cuts the sky like a broken crown, its edges lined with the iron teeth of the capital’s defenses. The wind carries the distant crackle of flames from the Church’s eternal pyres, and the faint, rhythmic pulse of ley lines hums beneath your feet. The only movement is the slow drift of ash, settling like a shroud over everything.','realized',NULL,NULL,NULL,NULL,'The Ashen Spire is the hollowed heart of a long-dead volcano, repurposed by the Church of the Eternal Pyre as a site for executions and heretical burnings. Its slopes are scarred by centuries of iron forging and ritual fires, the stone itself darkened by the blood and ash of countless victims. The Church’s defenses—iron spikes and barbed wire—bristle along its rim, a warning to any who might defy their rule. Beneath its surface, the ley lines run thick, though tapping them here is forbidden, punishable by the pyre. The Spire overlooks Spire’s Rest, the capital of Durnhold, where the Iron Council rules from the Obsidian Throne.',0,'Ashen Spire',NULL,NULL,1000000001,'The dead volcano looms over the city, its slopes scarred by the Church’s pyres and the iron teeth of the capital’s defenses.','2026-09-28 08:04:52.852425',NULL,NULL,NULL,NULL,NULL);
INSERT INTO locations VALUES(1000000009,'2026-09-28 08:04:52.881631','safe',NULL,NULL,'stub',NULL,NULL,NULL,NULL,NULL,0,'Rust Market',NULL,'a crowd',1000000001,'A thin path descends toward the river, where the mist clings to the water and the scent of iron and trade lingers.','2026-09-28 08:04:52.881631',NULL,NULL,NULL,NULL,NULL);
INSERT INTO locations VALUES(1000000010,'2026-09-28 08:04:52.893150','safe',NULL,NULL,'stub',NULL,NULL,NULL,NULL,NULL,0,'black pines',NULL,'nobody',1000000001,'The dense forest stretches north, its gnarled branches blotting out the sky and the scent of iron-rich earth thick in the air.','2026-09-28 08:04:52.893150',NULL,NULL,NULL,NULL,NULL);
INSERT INTO locations VALUES(1000000011,'2026-09-28 08:04:52.915458','safe',9,NULL,'stub',NULL,NULL,NULL,NULL,NULL,0,'Blackfang Warren',NULL,NULL,1000000001,'A low door of iron-strapped obsidian at the tunnel''s lowest turn, and behind it the clan''s own warren cut back into the rock.','2026-09-28 08:04:52.915458',12,NULL,NULL,NULL,NULL);
INSERT INTO locations VALUES(1000000012,'2026-09-28 08:04:52.938020','safe',5,NULL,'stub',NULL,NULL,NULL,NULL,NULL,0,'Blackfang Warren room 1',1000000011,NULL,1000000001,'A room inside Blackfang Warren, 6 by 5 paces on storey 0.','2026-09-28 08:04:52.976797',6,0,0,0,NULL);
INSERT INTO locations VALUES(1000000013,'2026-09-28 08:04:52.947439','uneasy',4,NULL,'stub',NULL,NULL,NULL,NULL,NULL,0,'Blackfang Warren room 2',1000000011,NULL,1000000001,'A room inside Blackfang Warren, 6 by 4 paces on storey 0.','2026-09-28 08:04:52.998020',6,0,5,0,NULL);
INSERT INTO locations VALUES(1000000014,'2026-09-28 08:04:52.955610','uneasy',4,'The warren''s far corner, 6 by 4 paces of dry obsidian on storey 0, and the only room down here whose walls have been scraped clear of the fungi — grey patches of bare stone, and just enough of a glow left to see by. A door in the north wall goes back through to the room the guards keep, and the one in the west wall is the way you came; nothing else opens anywhere. The stone is scored with rows of short marks cut by something blunter than a blade, and a chain runs from a ring hammered into the east wall to a shackle lying open on the floor. The floor is dry, which nothing else beneath the Spire is. Against the far wall, where the light gives out, a young man in the rags of Council blue sits with his back to the stone and his hands loose in his lap, watching the doorway as though he has watched it every hour he has been awake.','realized',NULL,NULL,NULL,NULL,'This is where the Blackfang keep what they intend to trade. The clan learned in the War of the Spire that a hostage held in the wet tunnels dies of the damp before any Durnish answer arrives, so this corner of the warren was cut above the seep and floored dry, and the ring in its wall is Durnish iron taken off the gate above. The marks scratched in the stone are days. The shackle is left open because the Silent Maw''s priests hold that a thing which has been given a choice and stays has been offered properly, and because nobody has walked this far in uninvited in a hundred years. The Iron Council has been told the price twice — the sacred obsidian, weighed out and returned — and has answered neither time.',0,'the dry cell',1000000011,NULL,1000000001,'A room inside Blackfang Warren, 6 by 4 paces on storey 0.','2026-09-28 08:04:53.000901',6,6,5,0,NULL);
INSERT INTO locations VALUES(1000000015,'2026-09-28 08:04:52.965213','uneasy',5,NULL,'stub',NULL,NULL,NULL,NULL,NULL,0,'Blackfang Warren room 4',1000000011,NULL,1000000001,'A room inside Blackfang Warren, 6 by 5 paces on storey 0.','2026-09-28 08:04:53.003516',6,6,0,0,NULL);
CREATE TABLE IF NOT EXISTS "locations_world_events" ("location_id" integer NOT NULL, "world_event_id" integer NOT NULL, CONSTRAINT "fk_rails_861146f35c"
FOREIGN KEY ("location_id")
  REFERENCES "locations" ("id")
, CONSTRAINT "fk_rails_16317bfdf9"
FOREIGN KEY ("world_event_id")
  REFERENCES "world_events" ("id")
);
CREATE TABLE IF NOT EXISTS "messages" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "cache_until_here" boolean DEFAULT FALSE NOT NULL, "chat_id" integer NOT NULL, "citations" json, "content" text, "content_raw" json, "created_at" datetime(6) NOT NULL, "finish_reason" varchar, "input_tokens" integer, "model_id" integer, "model_id_string" varchar, "output_tokens" integer, "raw_content" json, "raw_reasoning" json, "role" varchar, "scene_id" integer, "server_tool_calls" json, "tool_call_id" integer, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_c02b47ad97"
FOREIGN KEY ("model_id")
  REFERENCES "ruby_llm_models" ("id")
, CONSTRAINT "fk_rails_0f670de7ba"
FOREIGN KEY ("chat_id")
  REFERENCES "chats" ("id")
, CONSTRAINT "fk_rails_5c4d747b35"
FOREIGN KEY ("scene_id")
  REFERENCES "scenes" ("id")
);
CREATE TABLE IF NOT EXISTS "playthrough_beats" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "created_at" datetime(6) NOT NULL, "playthrough_id" integer NOT NULL, "quest_step_id" integer NOT NULL, "reached_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_d479d5ffac"
FOREIGN KEY ("playthrough_id")
  REFERENCES "playthroughs" ("id")
, CONSTRAINT "fk_rails_c536cc5a9f"
FOREIGN KEY ("quest_step_id")
  REFERENCES "quest_steps" ("id")
);
CREATE TABLE IF NOT EXISTS "playthrough_blows" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "attacker_id" integer NOT NULL, "created_at" datetime(6) NOT NULL, "damage" integer NOT NULL, "hp_after" integer NOT NULL, "location_id" integer NOT NULL, "playthrough_id" integer NOT NULL, "round" integer NOT NULL, "scene_id" integer, "sequence" integer NOT NULL, "story_timestamp" datetime(6) NOT NULL, "target_id" integer NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_f0e8a189c5"
FOREIGN KEY ("playthrough_id")
  REFERENCES "playthroughs" ("id")
, CONSTRAINT "fk_rails_9a2b0679db"
FOREIGN KEY ("target_id")
  REFERENCES "characters" ("id")
, CONSTRAINT "fk_rails_d02f38aa6d"
FOREIGN KEY ("attacker_id")
  REFERENCES "characters" ("id")
, CONSTRAINT "fk_rails_104e085e3d"
FOREIGN KEY ("location_id")
  REFERENCES "locations" ("id")
, CONSTRAINT "fk_rails_83e55c02a1"
FOREIGN KEY ("scene_id")
  REFERENCES "scenes" ("id")
);
CREATE TABLE IF NOT EXISTS "playthrough_commands" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "command" text NOT NULL, "created_at" datetime(6) NOT NULL, "error_kind" varchar, "journal" json DEFAULT '{}' NOT NULL, "playthrough_id" integer NOT NULL, "refusal" json DEFAULT '{}' NOT NULL, "request_token" varchar NOT NULL, "result_scene_id" integer, "status" varchar DEFAULT 'pending' NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_53f23eb49f"
FOREIGN KEY ("playthrough_id")
  REFERENCES "playthroughs" ("id")
, CONSTRAINT "fk_rails_69c7d16a1a"
FOREIGN KEY ("result_scene_id")
  REFERENCES "scenes" ("id")
);
CREATE TABLE IF NOT EXISTS "playthrough_drifts" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "action" varchar NOT NULL, "command" text NOT NULL, "created_at" datetime(6) NOT NULL, "location_id" integer, "offered" text, "playthrough_id" integer NOT NULL, "scene_id" integer, "story_timestamp" datetime(6), "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_018f29e0ac"
FOREIGN KEY ("playthrough_id")
  REFERENCES "playthroughs" ("id")
, CONSTRAINT "fk_rails_094a605342"
FOREIGN KEY ("location_id")
  REFERENCES "locations" ("id")
, CONSTRAINT "fk_rails_45044856a9"
FOREIGN KEY ("scene_id")
  REFERENCES "scenes" ("id")
);
CREATE TABLE IF NOT EXISTS "playthrough_endings" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "created_at" datetime(6) NOT NULL, "playthrough_id" integer NOT NULL, "quest_outcome_id" integer NOT NULL, "reached_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_762b78023d"
FOREIGN KEY ("playthrough_id")
  REFERENCES "playthroughs" ("id")
, CONSTRAINT "fk_rails_2220d324b0"
FOREIGN KEY ("quest_outcome_id")
  REFERENCES "quest_outcomes" ("id")
);
CREATE TABLE IF NOT EXISTS "playthrough_feedbacks" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "answering_models" text, "created_at" datetime(6) NOT NULL, "input_tokens" integer, "note" text, "output_tokens" integer, "playthrough_id" integer NOT NULL, "prose_model" varchar, "prose_models" text, "prose_prompt_digest" varchar, "prose_purpose" varchar, "scene_id" integer NOT NULL, "updated_at" datetime(6) NOT NULL, "verdict" varchar NOT NULL, CONSTRAINT "fk_rails_99ee3b49ea"
FOREIGN KEY ("playthrough_id")
  REFERENCES "playthroughs" ("id")
, CONSTRAINT "fk_rails_247709935a"
FOREIGN KEY ("scene_id")
  REFERENCES "scenes" ("id")
);
CREATE TABLE IF NOT EXISTS "playthrough_npc_states" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "ceasefire" boolean DEFAULT FALSE NOT NULL, "character_id" integer NOT NULL, "created_at" datetime(6) NOT NULL, "following" boolean DEFAULT FALSE NOT NULL, "location_id" integer, "peace_after_blow_id" integer DEFAULT 0 NOT NULL, "playthrough_id" integer NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_ef100d7841"
FOREIGN KEY ("location_id")
  REFERENCES "locations" ("id")
, CONSTRAINT "fk_rails_71a135780d"
FOREIGN KEY ("character_id")
  REFERENCES "characters" ("id")
, CONSTRAINT "fk_rails_984efad0e9"
FOREIGN KEY ("playthrough_id")
  REFERENCES "playthroughs" ("id")
);
CREATE TABLE IF NOT EXISTS "playthrough_overreaches" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "acted" text NOT NULL, "action" varchar NOT NULL, "command" text NOT NULL, "created_at" datetime(6) NOT NULL, "location_id" integer, "playthrough_id" integer NOT NULL, "scene_id" integer, "story_timestamp" datetime(6), "unacted" text NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_b903d1bc16"
FOREIGN KEY ("playthrough_id")
  REFERENCES "playthroughs" ("id")
, CONSTRAINT "fk_rails_9aac75595a"
FOREIGN KEY ("location_id")
  REFERENCES "locations" ("id")
, CONSTRAINT "fk_rails_879cc0f78c"
FOREIGN KEY ("scene_id")
  REFERENCES "scenes" ("id")
);
CREATE TABLE IF NOT EXISTS "playthrough_passages" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "created_at" datetime(6) NOT NULL, "location_connection_id" integer NOT NULL, "means" varchar NOT NULL, "opened_at" datetime(6) NOT NULL, "opened_by_item_id" integer, "playthrough_id" integer NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_7bdf6a743a"
FOREIGN KEY ("location_connection_id")
  REFERENCES "location_connections" ("id")
 ON DELETE CASCADE, CONSTRAINT "fk_rails_75c2355411"
FOREIGN KEY ("opened_by_item_id")
  REFERENCES "items" ("id")
 ON DELETE SET NULL, CONSTRAINT "fk_rails_995c25e2fd"
FOREIGN KEY ("playthrough_id")
  REFERENCES "playthroughs" ("id")
 ON DELETE CASCADE);
CREATE TABLE IF NOT EXISTS "playthrough_tolls" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "character_id" integer NOT NULL, "created_at" datetime(6) NOT NULL, "damage" integer NOT NULL, "hazard" varchar NOT NULL, "hp_after" integer NOT NULL, "location_connection_id" integer, "location_id" integer NOT NULL, "playthrough_id" integer NOT NULL, "saved" boolean DEFAULT FALSE NOT NULL, "scene_id" integer, "sequence" integer NOT NULL, "story_timestamp" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_83dfd307ab"
FOREIGN KEY ("playthrough_id")
  REFERENCES "playthroughs" ("id")
, CONSTRAINT "fk_rails_443984de3a"
FOREIGN KEY ("location_connection_id")
  REFERENCES "location_connections" ("id")
, CONSTRAINT "fk_rails_8a2212da5c"
FOREIGN KEY ("character_id")
  REFERENCES "characters" ("id")
, CONSTRAINT "fk_rails_6ec4904a86"
FOREIGN KEY ("location_id")
  REFERENCES "locations" ("id")
, CONSTRAINT "fk_rails_af4dd03665"
FOREIGN KEY ("scene_id")
  REFERENCES "scenes" ("id")
);
CREATE TABLE IF NOT EXISTS "playthrough_turn_events" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "created_at" datetime(6) NOT NULL, "data" json DEFAULT '{}' NOT NULL, "kind" varchar NOT NULL, "playthrough_command_id" integer NOT NULL, "sequence" integer NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_c7a6770f24"
FOREIGN KEY ("playthrough_command_id")
  REFERENCES "playthrough_commands" ("id")
);
CREATE TABLE IF NOT EXISTS "playthrough_vitals" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "character_id" integer NOT NULL, "created_at" datetime(6) NOT NULL, "hp_current" integer NOT NULL, "playthrough_id" integer NOT NULL, "provoked_at" datetime(6), "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_03cf4b1b8e"
FOREIGN KEY ("character_id")
  REFERENCES "characters" ("id")
, CONSTRAINT "fk_rails_9956f79aa5"
FOREIGN KEY ("playthrough_id")
  REFERENCES "playthroughs" ("id")
);
CREATE TABLE IF NOT EXISTS "playthrough_volitions" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "character_id" integer NOT NULL, "chosen" varchar NOT NULL, "created_at" datetime(6) NOT NULL, "decided_by" varchar, "fact" text NOT NULL, "location_id" integer NOT NULL, "playthrough_id" integer NOT NULL, "round" integer NOT NULL, "scene_id" integer, "serves" varchar NOT NULL, "status" varchar NOT NULL, "system_one_error" varchar, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_55e6f9e865"
FOREIGN KEY ("playthrough_id")
  REFERENCES "playthroughs" ("id")
, CONSTRAINT "fk_rails_0c408b6253"
FOREIGN KEY ("character_id")
  REFERENCES "characters" ("id")
, CONSTRAINT "fk_rails_1824dea9f6"
FOREIGN KEY ("location_id")
  REFERENCES "locations" ("id")
, CONSTRAINT "fk_rails_a054701564"
FOREIGN KEY ("scene_id")
  REFERENCES "scenes" ("id")
);
CREATE TABLE IF NOT EXISTS "playthroughs" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "character_id" integer, "created_at" datetime(6) NOT NULL, "current_location_id" integer, "current_scene_id" integer, "ended_at" datetime(6), "player_id" integer, "story_id" integer NOT NULL, "token" varchar NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_9b48509224"
FOREIGN KEY ("current_scene_id")
  REFERENCES "scenes" ("id")
, CONSTRAINT "fk_rails_3a7e48fa36"
FOREIGN KEY ("current_location_id")
  REFERENCES "locations" ("id")
, CONSTRAINT "fk_rails_5b54078c4b"
FOREIGN KEY ("character_id")
  REFERENCES "characters" ("id")
, CONSTRAINT "fk_rails_ab75586f4b"
FOREIGN KEY ("player_id")
  REFERENCES "players" ("id")
, CONSTRAINT "fk_rails_3d6d3a9c5a"
FOREIGN KEY ("story_id")
  REFERENCES "stories" ("id")
);
CREATE TABLE IF NOT EXISTS "quest_outcomes" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "condition" varchar, "created_at" datetime(6) NOT NULL, "is_default" boolean DEFAULT FALSE NOT NULL, "minutes" integer, "name" varchar NOT NULL, "quest_id" integer NOT NULL, "ramification_minutes" integer, "ramification_summary" text, "summary" text NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_acf8ece7c5"
FOREIGN KEY ("quest_id")
  REFERENCES "quests" ("id")
);
INSERT INTO quest_outcomes VALUES(1000000001,NULL,'2026-09-28 08:04:53.748823',1,NULL,'rescued',1000000001,NULL,NULL,'The prince is found alive in the dry cell, and the iron gate opens outward at last.','2026-09-28 08:04:53.748823');
INSERT INTO quest_outcomes VALUES(1000000002,'out_of_order','2026-09-28 08:04:53.754493',0,NULL,'too-late',1000000001,180,'The Blackfang answer for the cell being opened, and the low door at the tunnel''s end is barred from the inside.','The cell is opened and the prince is already cold; the decree is answered, and not the way anybody wanted.','2026-09-28 08:04:53.754493');
CREATE TABLE IF NOT EXISTS "quest_steps" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "bound_at" datetime(6), "created_at" datetime(6) NOT NULL, "minutes" integer, "position" integer NOT NULL, "quest_id" integer NOT NULL, "summary" text NOT NULL, "target_id" integer, "target_name" varchar, "target_type" varchar, "teaser" text, "trigger_kind" varchar NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_ba1603d17b"
FOREIGN KEY ("quest_id")
  REFERENCES "quests" ("id")
);
INSERT INTO quest_steps VALUES(1000000001,'2026-09-05 20:17:00','2026-09-28 08:04:53.552247',NULL,1,1000000001,'Take the signet ring that proves he came this way.',1000000002,'prince''s signet ring','Item',NULL,'hold_item','2026-09-28 08:04:53.620567');
INSERT INTO quest_steps VALUES(1000000002,'2026-09-05 20:17:00','2026-09-28 08:04:53.628674',NULL,2,1000000001,'Get inside the warren where the Blackfang keep what they take.',1000000011,'Blackfang Warren','Location','The Blackfang hold him somewhere below the old workings.','reach_location','2026-09-28 08:04:53.692480');
INSERT INTO quest_steps VALUES(1000000003,'2026-09-05 20:17:00','2026-09-28 08:04:53.696088',NULL,3,1000000001,'Find the cell they are keeping him in.',1000000014,'the dry cell','Location',NULL,'reach_location','2026-09-28 08:04:53.705400');
CREATE TABLE IF NOT EXISTS "quests" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "contributes" boolean DEFAULT TRUE NOT NULL, "created_at" datetime(6) NOT NULL, "origin" varchar DEFAULT 'seeded' NOT NULL, "parent_quest_id" integer, "premise" text NOT NULL, "status" varchar DEFAULT 'open' NOT NULL, "story_id" integer NOT NULL, "title" varchar NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_a64954ae79"
FOREIGN KEY ("parent_quest_id")
  REFERENCES "quests" ("id")
, CONSTRAINT "fk_rails_fa77fc496b"
FOREIGN KEY ("story_id")
  REFERENCES "stories" ("id")
);
INSERT INTO quests VALUES(1000000001,1,'2026-09-28 08:04:53.475736','seeded',NULL,'Retrieve the prince from the goblin caves beneath the Ashen Spire, or die trying.','open',1000000001,'The Church''s Decree','2026-09-28 08:04:53.475736');
CREATE TABLE IF NOT EXISTS "races" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "created_at" datetime(6) NOT NULL, "description" text NOT NULL, "monstrous" boolean DEFAULT FALSE NOT NULL, "name" varchar NOT NULL, "universe_id" integer NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_c25ac61605"
FOREIGN KEY ("universe_id")
  REFERENCES "universes" ("id")
);
INSERT INTO races VALUES(1000000001,'2026-09-28 08:04:52.368447','Humans of the kingdom of Durnhold, tall and broad-shouldered with ashen skin from generations of iron-smoke. They revere the Church of the Eternal Pyre, whose priests brand heretics with white-hot irons. Their knights wear blackened plate armor, forged from Veldarian steel, but it is heavy—no Durnish warrior can run for more than a minute in full gear.',0,'Durnish',1000000001,'2026-09-28 08:04:52.368447');
INSERT INTO races VALUES(1000000002,'2026-09-28 08:04:52.374645','Squat, green-skinned humanoids with elongated fingers and needle teeth, dwelling in the obsidian caves beneath the Ashen Spire. They worship the Silent Maw, a god of hunger, and believe all surface-dwellers are already dead. Their venomous daggers are their only advantage; without them, they are weak in direct combat.',0,'Goblins',1000000001,'2026-09-28 08:04:52.374645');
INSERT INTO races VALUES(1000000003,'2026-09-28 08:04:52.391747','Hulking, gray-furred descendants of cave trolls, standing twice the height of a Durnish knight. They are immune to poison and can regenerate wounds, but only if they consume raw meat within an hour of injury. The Church hunts them as abominations, though they keep to the riverbanks and rarely attack unless provoked.',0,'Trollkin',1000000001,'2026-09-28 08:04:52.391747');
INSERT INTO races VALUES(1000000004,'2026-09-28 08:04:52.394752','Pale, gaunt humans who live along the iron-tainted River Veyth, their skin stained rust-red from the water. They trade in forbidden knowledge, smuggling clockwork devices and alchemical texts to the highest bidder. The Church executes them on sight, but the goblins tolerate them as suppliers of surface-world goods.',0,'Veythi',1000000001,'2026-09-28 08:04:52.394752');
CREATE TABLE IF NOT EXISTS "relay_receipts" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "cost_source" varchar, "cost_usd" decimal(12,6), "created_at" datetime(6) NOT NULL, "finished_at" datetime(6), "input_tokens" integer, "model" varchar NOT NULL, "output_tokens" integer, "player_id" integer NOT NULL, "reserved_usd" decimal(12,6) NOT NULL, "route" varchar NOT NULL, "status" varchar DEFAULT 'open' NOT NULL, "stream" boolean DEFAULT FALSE NOT NULL, "updated_at" datetime(6) NOT NULL, "upstream_status" integer, CONSTRAINT "fk_rails_deae27bd71"
FOREIGN KEY ("player_id")
  REFERENCES "players" ("id")
);
CREATE TABLE IF NOT EXISTS "scenes" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "acted_on_id" integer, "acted_on_type" varchar, "created_at" datetime(6) NOT NULL, "description" text, "engine_fact" text, "engine_fallback" boolean DEFAULT FALSE NOT NULL, "is_opening" boolean DEFAULT FALSE NOT NULL, "location_id" integer NOT NULL, "previous_scene_id" integer, "resolved_action" varchar, "resolved_by" varchar, "story_id" integer NOT NULL, "story_timestamp" datetime(6), "summary" text, "typed" text, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_5cc24f985a"
FOREIGN KEY ("previous_scene_id")
  REFERENCES "scenes" ("id")
, CONSTRAINT "fk_rails_abfe1c0369"
FOREIGN KEY ("location_id")
  REFERENCES "locations" ("id")
, CONSTRAINT "fk_rails_38ea512481"
FOREIGN KEY ("story_id")
  REFERENCES "stories" ("id")
);
INSERT INTO scenes VALUES(1000000001,NULL,NULL,'2026-09-28 08:04:53.882318','The first thing that hits you is the stench of wet iron and old blood, thick enough to coat your throat. Then comes the cold—damp, seeping through the seams of your armor as your boot crunches on something brittle in the mud. The air hums faintly, a vibration in your teeth, and the dim glow of the fungi pulses like a slow, dying heartbeat against the black stone. The gate’s jagged teeth yawn behind you, but the tunnel ahead swallows the light whole, leaving only the sound of something shifting in the dark.',NULL,0,1,1000000001,NULL,NULL,NULL,1000000001,'2026-09-05 20:17:00','The player steps into the Iron Gate Chamber, encountering the sensory overload of the cavern’s atmosphere and the immediate presence of the tunnel ahead.',NULL,'2026-09-28 08:04:53.882318');
CREATE TABLE IF NOT EXISTS "stories" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "created_at" datetime(6) NOT NULL, "generation_snapshot" text, "genre" varchar, "preface" text, "start_time" datetime(6), "summary" text, "title" varchar, "universe_id" integer NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_2a912ea846"
FOREIGN KEY ("universe_id")
  REFERENCES "universes" ("id")
);
INSERT INTO stories VALUES(1000000001,'2026-09-28 08:04:52.424261',NULL,'dark fantasy','The iron gate groans as it settles into the stone, its teeth biting into the threshold with a finality that echoes through the cavern. You stand in the dim glow of bioluminescent fungi, the air thick with the scent of damp earth and old blood. Your gauntleted hand rests on the hilt of your Veldarian sword, its edge still sharp despite the weight of the blackened plate armor pressing into your shoulders. The prince’s signet ring, cold in your palm, was found in the mud outside this gate—proof he passed this way. Behind you, the tunnel slopes upward, but the way forward is a yawning maw of obsidian, the walls slick with moisture. Somewhere in the dark, something skitters.','2026-09-05 20:17:00','A Durnish knight, clad in heavy blackened plate, stands at the threshold of the goblin caves beneath the Ashen Spire, having just forced open an iron gate. The prince’s signet ring, discovered in the mud, confirms his passage into the tunnels. The air is thick with the scent of damp earth and the faint glow of fungi, while the distant sound of movement hints at unseen dangers ahead. The Church’s decree to retrieve the prince or die trying weighs heavily, but the immediate threat is the darkness itself.','The Iron Gate Descends (engine sweep)',1000000001,'2026-09-28 08:04:52.424261');
CREATE TABLE IF NOT EXISTS "system_one_receipts" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "cost_usd" decimal(12,6) NOT NULL, "created_at" datetime(6) NOT NULL, "player_id" integer, "playthrough_id" integer, "purpose" varchar, "transport" varchar, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_4ae941c854"
FOREIGN KEY ("player_id")
  REFERENCES "players" ("id")
, CONSTRAINT "fk_rails_75847e1556"
FOREIGN KEY ("playthrough_id")
  REFERENCES "playthroughs" ("id")
);
CREATE TABLE IF NOT EXISTS "world_events" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "created_at" datetime(6) NOT NULL, "fired_at" datetime(6), "occurred_at" datetime(6) NOT NULL, "playthrough_id" integer, "scheduled_for" datetime(6), "source" varchar NOT NULL, "story_id" integer NOT NULL, "summary" text NOT NULL, "updated_at" datetime(6) NOT NULL, "world_mechanic_id" integer, CONSTRAINT "fk_rails_cafc0959e8"
FOREIGN KEY ("story_id")
  REFERENCES "stories" ("id")
, CONSTRAINT "fk_rails_efc6cdec27"
FOREIGN KEY ("world_mechanic_id")
  REFERENCES "world_mechanics" ("id")
);
CREATE TABLE IF NOT EXISTS "world_mechanics" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "cadence" varchar NOT NULL, "created_at" datetime(6) NOT NULL, "description" text, "kind" varchar NOT NULL, "last_run_at" datetime(6), "name" varchar NOT NULL, "story_id" integer NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_d7328bbbe5"
FOREIGN KEY ("story_id")
  REFERENCES "stories" ("id")
);
CREATE TABLE IF NOT EXISTS "schema_migrations" ("version" varchar NOT NULL PRIMARY KEY);
INSERT INTO schema_migrations VALUES('20260928035434');
INSERT INTO schema_migrations VALUES('20260928035429');
INSERT INTO schema_migrations VALUES('20260928031026');
INSERT INTO schema_migrations VALUES('20260928025950');
INSERT INTO schema_migrations VALUES('20260928020741');
INSERT INTO schema_migrations VALUES('20260927152056');
INSERT INTO schema_migrations VALUES('20260927031841');
INSERT INTO schema_migrations VALUES('20260927031839');
INSERT INTO schema_migrations VALUES('20260927031837');
INSERT INTO schema_migrations VALUES('20260927031835');
INSERT INTO schema_migrations VALUES('20260927031834');
INSERT INTO schema_migrations VALUES('20260926195041');
INSERT INTO schema_migrations VALUES('20260922141619');
INSERT INTO schema_migrations VALUES('20260922141618');
INSERT INTO schema_migrations VALUES('20260922141617');
INSERT INTO schema_migrations VALUES('20260919202606');
INSERT INTO schema_migrations VALUES('20260919202605');
INSERT INTO schema_migrations VALUES('20260917133823');
INSERT INTO schema_migrations VALUES('20260910041334');
INSERT INTO schema_migrations VALUES('20260910025040');
INSERT INTO schema_migrations VALUES('20260909030346');
INSERT INTO schema_migrations VALUES('20260909023028');
INSERT INTO schema_migrations VALUES('20260909023014');
INSERT INTO schema_migrations VALUES('20260908203558');
INSERT INTO schema_migrations VALUES('20260908155546');
INSERT INTO schema_migrations VALUES('20260908120000');
INSERT INTO schema_migrations VALUES('20260907150000');
INSERT INTO schema_migrations VALUES('20260907140000');
INSERT INTO schema_migrations VALUES('20260907120000');
INSERT INTO schema_migrations VALUES('20260906130000');
INSERT INTO schema_migrations VALUES('20260906120000');
INSERT INTO schema_migrations VALUES('20260905150000');
INSERT INTO schema_migrations VALUES('20260905140000');
INSERT INTO schema_migrations VALUES('20260905130000');
INSERT INTO schema_migrations VALUES('20260905120000');
INSERT INTO schema_migrations VALUES('20260905110000');
INSERT INTO schema_migrations VALUES('20260905100001');
INSERT INTO schema_migrations VALUES('20260905100000');
INSERT INTO schema_migrations VALUES('20260905090100');
INSERT INTO schema_migrations VALUES('20260905090000');
INSERT INTO schema_migrations VALUES('20260904190000');
INSERT INTO schema_migrations VALUES('20260904180000');
INSERT INTO schema_migrations VALUES('20260904170000');
INSERT INTO schema_migrations VALUES('20260904160000');
INSERT INTO schema_migrations VALUES('20260904120000');
INSERT INTO schema_migrations VALUES('20260903170000');
INSERT INTO schema_migrations VALUES('20260903120000');
INSERT INTO schema_migrations VALUES('20260902120000');
INSERT INTO schema_migrations VALUES('20260901170000');
INSERT INTO schema_migrations VALUES('20260901160000');
INSERT INTO schema_migrations VALUES('20260901120000');
INSERT INTO schema_migrations VALUES('20260831140000');
INSERT INTO schema_migrations VALUES('20260831130000');
INSERT INTO schema_migrations VALUES('20260831120000');
INSERT INTO schema_migrations VALUES('20260831000000');
INSERT INTO schema_migrations VALUES('20260830220946');
INSERT INTO schema_migrations VALUES('20260830220945');
INSERT INTO schema_migrations VALUES('20260830210000');
INSERT INTO schema_migrations VALUES('20260830200001');
INSERT INTO schema_migrations VALUES('20260830200000');
INSERT INTO schema_migrations VALUES('20260830192215');
INSERT INTO schema_migrations VALUES('20260830192214');
INSERT INTO schema_migrations VALUES('20250824193519');
INSERT INTO schema_migrations VALUES('20250823024100');
INSERT INTO schema_migrations VALUES('20250823024050');
INSERT INTO schema_migrations VALUES('20250823024037');
INSERT INTO schema_migrations VALUES('20250823024029');
INSERT INTO schema_migrations VALUES('20250823024021');
INSERT INTO schema_migrations VALUES('20250823024013');
INSERT INTO schema_migrations VALUES('20250823023950');
INSERT INTO schema_migrations VALUES('20250823023942');
INSERT INTO schema_migrations VALUES('20250823023925');
INSERT INTO schema_migrations VALUES('20250822184651');
INSERT INTO schema_migrations VALUES('20250822184650');
INSERT INTO schema_migrations VALUES('20250822184649');
CREATE TABLE IF NOT EXISTS "ar_internal_metadata" ("key" varchar NOT NULL PRIMARY KEY, "value" varchar, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL);
INSERT INTO ar_internal_metadata VALUES('environment','test','2026-09-28 08:04:11.981278','2026-09-28 08:04:11.981283');
INSERT INTO ar_internal_metadata VALUES('schema_sha1','0ac19d803e60477f0a66d948e0255683b378ac78','2026-09-28 08:04:11.990182','2026-09-28 08:04:11.990187');
PRAGMA writable_schema=ON;
CREATE TABLE IF NOT EXISTS sqlite_sequence(name,seq);
DELETE FROM sqlite_sequence;
INSERT INTO sqlite_sequence VALUES('lab_exits_vantages',1000000000);
INSERT INTO sqlite_sequence VALUES('lab_realization_kinds',1000000000);
INSERT INTO sqlite_sequence VALUES('players',1000000000);
INSERT INTO sqlite_sequence VALUES('playthrough_visits',1000000000);
INSERT INTO sqlite_sequence VALUES('ruby_llm_batches',1000000000);
INSERT INTO sqlite_sequence VALUES('ruby_llm_models',1000000000);
INSERT INTO sqlite_sequence VALUES('ruby_llm_tool_calls',1000000000);
INSERT INTO sqlite_sequence VALUES('ruby_llm_usages',1000000000);
INSERT INTO sqlite_sequence VALUES('universes',1000000001);
INSERT INTO sqlite_sequence VALUES('characters',1000000004);
INSERT INTO sqlite_sequence VALUES('chats',1000000000);
INSERT INTO sqlite_sequence VALUES('interactions',1000000000);
INSERT INTO sqlite_sequence VALUES('items',1000000012);
INSERT INTO sqlite_sequence VALUES('lab_exits_judgements',1000000000);
INSERT INTO sqlite_sequence VALUES('lab_exits_samples',1000000000);
INSERT INTO sqlite_sequence VALUES('lab_realization_samples',1000000000);
INSERT INTO sqlite_sequence VALUES('location_connections',1000000036);
INSERT INTO sqlite_sequence VALUES('locations',1000000015);
INSERT INTO sqlite_sequence VALUES('messages',1000000000);
INSERT INTO sqlite_sequence VALUES('playthrough_beats',1000000000);
INSERT INTO sqlite_sequence VALUES('playthrough_blows',1000000000);
INSERT INTO sqlite_sequence VALUES('playthrough_commands',1000000000);
INSERT INTO sqlite_sequence VALUES('playthrough_drifts',1000000000);
INSERT INTO sqlite_sequence VALUES('playthrough_endings',1000000000);
INSERT INTO sqlite_sequence VALUES('playthrough_feedbacks',1000000000);
INSERT INTO sqlite_sequence VALUES('playthrough_npc_states',1000000000);
INSERT INTO sqlite_sequence VALUES('playthrough_overreaches',1000000000);
INSERT INTO sqlite_sequence VALUES('playthrough_passages',1000000000);
INSERT INTO sqlite_sequence VALUES('playthrough_tolls',1000000000);
INSERT INTO sqlite_sequence VALUES('playthrough_turn_events',1000000000);
INSERT INTO sqlite_sequence VALUES('playthrough_vitals',1000000000);
INSERT INTO sqlite_sequence VALUES('playthrough_volitions',1000000000);
INSERT INTO sqlite_sequence VALUES('playthroughs',1000000000);
INSERT INTO sqlite_sequence VALUES('quest_outcomes',1000000002);
INSERT INTO sqlite_sequence VALUES('quest_steps',1000000003);
INSERT INTO sqlite_sequence VALUES('quests',1000000001);
INSERT INTO sqlite_sequence VALUES('races',1000000004);
INSERT INTO sqlite_sequence VALUES('relay_receipts',1000000000);
INSERT INTO sqlite_sequence VALUES('scenes',1000000001);
INSERT INTO sqlite_sequence VALUES('stories',1000000001);
INSERT INTO sqlite_sequence VALUES('system_one_receipts',1000000000);
INSERT INTO sqlite_sequence VALUES('world_events',1000000000);
INSERT INTO sqlite_sequence VALUES('world_mechanics',1000000000);
CREATE INDEX "index_characters_scenes_on_character_id_and_scene_id" ON "characters_scenes" ("character_id", "scene_id");
CREATE INDEX "index_characters_scenes_on_scene_id_and_character_id" ON "characters_scenes" ("scene_id", "character_id");
CREATE UNIQUE INDEX "index_players_on_name" ON "players" ("name");
CREATE UNIQUE INDEX "index_players_on_token_digest" ON "players" ("token_digest");
CREATE INDEX "index_playthrough_visits_on_location_id" ON "playthrough_visits" ("location_id");
CREATE UNIQUE INDEX "index_playthrough_visits_on_playthrough_and_location" ON "playthrough_visits" ("playthrough_id", "location_id");
CREATE INDEX "index_playthrough_visits_on_playthrough_id" ON "playthrough_visits" ("playthrough_id");
CREATE UNIQUE INDEX "index_ruby_llm_batches_on_provider_and_provider_batch_id" ON "ruby_llm_batches" ("provider", "provider_batch_id");
CREATE INDEX "index_ruby_llm_batches_on_status" ON "ruby_llm_batches" ("status");
CREATE INDEX "index_ruby_llm_models_on_family" ON "ruby_llm_models" ("family");
CREATE UNIQUE INDEX "index_ruby_llm_models_on_provider_and_model_id" ON "ruby_llm_models" ("provider", "model_id");
CREATE INDEX "index_ruby_llm_models_on_provider" ON "ruby_llm_models" ("provider");
CREATE INDEX "index_ruby_llm_tool_calls_on_message_type_and_message_id" ON "ruby_llm_tool_calls" ("message_type", "message_id");
CREATE INDEX "index_ruby_llm_tool_calls_on_name" ON "ruby_llm_tool_calls" ("name");
CREATE INDEX "index_ruby_llm_tool_calls_on_result_type_and_result_id" ON "ruby_llm_tool_calls" ("result_type", "result_id");
CREATE UNIQUE INDEX "index_ruby_llm_tool_calls_on_tool_call_id" ON "ruby_llm_tool_calls" ("tool_call_id");
CREATE INDEX "index_ruby_llm_usages_on_chat_type_and_chat_id" ON "ruby_llm_usages" ("chat_type", "chat_id");
CREATE INDEX "index_ruby_llm_usages_on_message_type_and_message_id" ON "ruby_llm_usages" ("message_type", "message_id");
CREATE INDEX "index_ruby_llm_usages_on_status" ON "ruby_llm_usages" ("status");
CREATE UNIQUE INDEX "index_ruby_llm_v2_backfills_on_task" ON "ruby_llm_v2_backfills" ("task");
CREATE UNIQUE INDEX "index_characters_on_story_id_and_lower_fullname" ON "characters" (story_id, LOWER(fullname));
CREATE INDEX "index_characters_on_location_id_and_hostile" ON "characters" ("location_id", "hostile");
CREATE INDEX "index_characters_on_location_id_and_id" ON "characters" ("location_id", "id");
CREATE INDEX "index_characters_on_location_id" ON "characters" ("location_id");
CREATE INDEX "index_characters_on_race_id" ON "characters" ("race_id");
CREATE INDEX "index_characters_on_story_id_and_is_protagonist" ON "characters" ("story_id", "is_protagonist");
CREATE INDEX "index_characters_on_story_id" ON "characters" ("story_id");
CREATE INDEX "index_chats_on_character_id" ON "chats" ("character_id");
CREATE INDEX "index_chats_on_player_id" ON "chats" ("player_id");
CREATE INDEX "index_chats_on_conversation_key" ON "chats" ("playthrough_id", "character_id", "purpose");
CREATE INDEX "index_chats_on_playthrough_id" ON "chats" ("playthrough_id");
CREATE INDEX "index_chats_on_ruby_llm_model_id" ON "chats" ("ruby_llm_model_id");
CREATE INDEX "index_interactions_on_character_id" ON "interactions" ("character_id");
CREATE INDEX "index_interactions_on_location_id" ON "interactions" ("location_id");
CREATE INDEX "index_interactions_on_scene_id" ON "interactions" ("scene_id");
CREATE INDEX "index_items_on_character_id" ON "items" ("character_id");
CREATE INDEX "index_items_on_location_id_and_character_id" ON "items" ("location_id", "character_id");
CREATE INDEX "index_items_on_location_id" ON "items" ("location_id");
CREATE INDEX "index_items_on_playthrough_id_and_id" ON "items" ("playthrough_id", "id");
CREATE INDEX "index_items_on_playthrough_id_and_template_id" ON "items" ("playthrough_id", "template_id");
CREATE INDEX "index_items_on_playthrough_id" ON "items" ("playthrough_id");
CREATE INDEX "index_items_on_template_id" ON "items" ("template_id");
CREATE UNIQUE INDEX "index_lab_exits_judgements_on_place" ON "lab_exits_judgements" ("vantage_id", "name_key");
CREATE INDEX "index_lab_exits_judgements_on_vantage_id" ON "lab_exits_judgements" ("vantage_id");
CREATE INDEX "index_lab_exits_samples_on_vantage_id" ON "lab_exits_samples" ("vantage_id");
CREATE INDEX "index_lab_realization_samples_on_kind_id" ON "lab_realization_samples" ("kind_id");
CREATE INDEX "index_location_connections_on_connected_location_id" ON "location_connections" ("connected_location_id");
CREATE INDEX "index_location_connections_on_key_template_id" ON "location_connections" ("key_template_id");
CREATE UNIQUE INDEX "idx_on_location_id_connected_location_id_a0efda2bf6" ON "location_connections" ("location_id", "connected_location_id");
CREATE INDEX "index_location_connections_on_location_id" ON "location_connections" ("location_id");
CREATE UNIQUE INDEX "index_locations_on_story_id_and_lower_name" ON "locations" (story_id, lower(name));
CREATE INDEX "index_locations_on_parent_location_id" ON "locations" ("parent_location_id");
CREATE INDEX "index_locations_on_story_id_and_detail_level" ON "locations" ("story_id", "detail_level");
CREATE INDEX "index_locations_on_story_id" ON "locations" ("story_id");
CREATE INDEX "index_locations_world_events_on_location_id" ON "locations_world_events" ("location_id");
CREATE UNIQUE INDEX "index_locations_world_events_on_world_event_id_and_location_id" ON "locations_world_events" ("world_event_id", "location_id");
CREATE INDEX "index_locations_world_events_on_world_event_id" ON "locations_world_events" ("world_event_id");
CREATE INDEX "index_messages_on_chat_id" ON "messages" ("chat_id");
CREATE INDEX "index_messages_on_model_id" ON "messages" ("model_id");
CREATE INDEX "index_messages_on_scene_id" ON "messages" ("scene_id");
CREATE INDEX "index_messages_on_tool_call_id" ON "messages" ("tool_call_id");
CREATE UNIQUE INDEX "index_playthrough_beats_on_playthrough_id_and_quest_step_id" ON "playthrough_beats" ("playthrough_id", "quest_step_id");
CREATE INDEX "index_playthrough_beats_on_playthrough_id" ON "playthrough_beats" ("playthrough_id");
CREATE INDEX "index_playthrough_beats_on_quest_step_id" ON "playthrough_beats" ("quest_step_id");
CREATE INDEX "index_playthrough_blows_on_attacker_id" ON "playthrough_blows" ("attacker_id");
CREATE INDEX "index_playthrough_blows_on_location_id" ON "playthrough_blows" ("location_id");
CREATE INDEX "index_playthrough_blows_on_playthrough_and_scene" ON "playthrough_blows" ("playthrough_id", "scene_id", "id");
CREATE INDEX "index_playthrough_blows_on_playthrough_id" ON "playthrough_blows" ("playthrough_id");
CREATE INDEX "index_playthrough_blows_on_scene_id" ON "playthrough_blows" ("scene_id");
CREATE INDEX "index_playthrough_blows_on_target_id" ON "playthrough_blows" ("target_id");
CREATE UNIQUE INDEX "index_playthrough_commands_on_submission" ON "playthrough_commands" ("playthrough_id", "request_token", "command");
CREATE INDEX "index_playthrough_commands_on_playthrough_id" ON "playthrough_commands" ("playthrough_id");
CREATE INDEX "index_playthrough_commands_on_result_scene_id" ON "playthrough_commands" ("result_scene_id");
CREATE INDEX "index_playthrough_drifts_on_action" ON "playthrough_drifts" ("action");
CREATE INDEX "index_playthrough_drifts_on_location_id" ON "playthrough_drifts" ("location_id");
CREATE INDEX "index_playthrough_drifts_on_playthrough_id_and_story_timestamp" ON "playthrough_drifts" ("playthrough_id", "story_timestamp");
CREATE INDEX "index_playthrough_drifts_on_playthrough_id" ON "playthrough_drifts" ("playthrough_id");
CREATE INDEX "index_playthrough_drifts_on_scene_id" ON "playthrough_drifts" ("scene_id");
CREATE UNIQUE INDEX "idx_on_playthrough_id_quest_outcome_id_7ea31b4171" ON "playthrough_endings" ("playthrough_id", "quest_outcome_id");
CREATE INDEX "index_playthrough_endings_on_playthrough_id" ON "playthrough_endings" ("playthrough_id");
CREATE INDEX "index_playthrough_endings_on_quest_outcome_id" ON "playthrough_endings" ("quest_outcome_id");
CREATE UNIQUE INDEX "index_playthrough_feedbacks_on_playthrough_id_and_scene_id" ON "playthrough_feedbacks" ("playthrough_id", "scene_id");
CREATE INDEX "index_playthrough_feedbacks_on_playthrough_id" ON "playthrough_feedbacks" ("playthrough_id");
CREATE INDEX "index_playthrough_feedbacks_on_prose_model" ON "playthrough_feedbacks" ("prose_model");
CREATE INDEX "index_playthrough_feedbacks_on_prose_prompt_digest" ON "playthrough_feedbacks" ("prose_prompt_digest");
CREATE INDEX "index_playthrough_feedbacks_on_scene_id" ON "playthrough_feedbacks" ("scene_id");
CREATE INDEX "index_playthrough_feedbacks_on_verdict" ON "playthrough_feedbacks" ("verdict");
CREATE INDEX "index_playthrough_npc_states_on_character_id" ON "playthrough_npc_states" ("character_id");
CREATE INDEX "index_playthrough_npc_states_on_location_id" ON "playthrough_npc_states" ("location_id");
CREATE UNIQUE INDEX "idx_on_playthrough_id_character_id_008ae03862" ON "playthrough_npc_states" ("playthrough_id", "character_id");
CREATE INDEX "index_playthrough_npc_states_on_playthrough_id" ON "playthrough_npc_states" ("playthrough_id");
CREATE INDEX "index_playthrough_overreaches_on_action" ON "playthrough_overreaches" ("action");
CREATE INDEX "index_playthrough_overreaches_on_location_id" ON "playthrough_overreaches" ("location_id");
CREATE INDEX "idx_on_playthrough_id_story_timestamp_b54cd36315" ON "playthrough_overreaches" ("playthrough_id", "story_timestamp");
CREATE INDEX "index_playthrough_overreaches_on_playthrough_id" ON "playthrough_overreaches" ("playthrough_id");
CREATE INDEX "index_playthrough_overreaches_on_scene_id" ON "playthrough_overreaches" ("scene_id");
CREATE INDEX "index_playthrough_passages_on_location_connection_id" ON "playthrough_passages" ("location_connection_id");
CREATE INDEX "index_playthrough_passages_on_opened_by_item_id" ON "playthrough_passages" ("opened_by_item_id");
CREATE UNIQUE INDEX "idx_on_playthrough_id_location_connection_id_6d3a29d910" ON "playthrough_passages" ("playthrough_id", "location_connection_id");
CREATE INDEX "index_playthrough_passages_on_playthrough_id" ON "playthrough_passages" ("playthrough_id");
CREATE INDEX "index_playthrough_tolls_on_character_id" ON "playthrough_tolls" ("character_id");
CREATE INDEX "index_playthrough_tolls_on_location_connection_id" ON "playthrough_tolls" ("location_connection_id");
CREATE INDEX "index_playthrough_tolls_on_location_id" ON "playthrough_tolls" ("location_id");
CREATE INDEX "index_playthrough_tolls_on_playthrough_and_scene" ON "playthrough_tolls" ("playthrough_id", "scene_id", "id");
CREATE INDEX "index_playthrough_tolls_on_playthrough_id" ON "playthrough_tolls" ("playthrough_id");
CREATE INDEX "index_playthrough_tolls_on_scene_id" ON "playthrough_tolls" ("scene_id");
CREATE UNIQUE INDEX "index_playthrough_turn_events_on_command_and_sequence" ON "playthrough_turn_events" ("playthrough_command_id", "sequence");
CREATE INDEX "index_playthrough_turn_events_on_playthrough_command_id" ON "playthrough_turn_events" ("playthrough_command_id");
CREATE INDEX "index_playthrough_vitals_on_character_id" ON "playthrough_vitals" ("character_id");
CREATE UNIQUE INDEX "index_playthrough_vitals_on_playthrough_and_character" ON "playthrough_vitals" ("playthrough_id", "character_id");
CREATE INDEX "index_playthrough_vitals_on_playthrough_id" ON "playthrough_vitals" ("playthrough_id");
CREATE INDEX "index_playthrough_volitions_on_character_id" ON "playthrough_volitions" ("character_id");
CREATE INDEX "index_playthrough_volitions_on_location_id" ON "playthrough_volitions" ("location_id");
CREATE INDEX "index_playthrough_volitions_on_playthrough_and_scene" ON "playthrough_volitions" ("playthrough_id", "scene_id", "id");
CREATE INDEX "index_playthrough_volitions_on_playthrough_id" ON "playthrough_volitions" ("playthrough_id");
CREATE INDEX "index_playthrough_volitions_on_scene_id" ON "playthrough_volitions" ("scene_id");
CREATE INDEX "index_playthroughs_on_character_id" ON "playthroughs" ("character_id");
CREATE INDEX "index_playthroughs_on_current_location_id" ON "playthroughs" ("current_location_id");
CREATE INDEX "index_playthroughs_on_current_scene_id" ON "playthroughs" ("current_scene_id");
CREATE INDEX "index_playthroughs_on_player_id" ON "playthroughs" ("player_id");
CREATE INDEX "index_playthroughs_on_story_id" ON "playthroughs" ("story_id");
CREATE UNIQUE INDEX "index_playthroughs_on_token" ON "playthroughs" ("token");
CREATE UNIQUE INDEX "index_quest_outcomes_on_quest_id_and_name" ON "quest_outcomes" ("quest_id", "name");
CREATE INDEX "index_quest_outcomes_on_quest_id" ON "quest_outcomes" ("quest_id");
CREATE UNIQUE INDEX "index_quest_steps_on_quest_id_and_position" ON "quest_steps" ("quest_id", "position");
CREATE INDEX "index_quest_steps_on_quest_id" ON "quest_steps" ("quest_id");
CREATE INDEX "index_quest_steps_on_target" ON "quest_steps" ("target_type", "target_id");
CREATE INDEX "index_quests_on_parent_quest_id" ON "quests" ("parent_quest_id");
CREATE UNIQUE INDEX "index_quests_on_story_id_and_title" ON "quests" ("story_id", "title");
CREATE INDEX "index_quests_on_story_id" ON "quests" ("story_id");
CREATE UNIQUE INDEX "index_races_on_universe_id_and_name" ON "races" ("universe_id", "name");
CREATE INDEX "index_races_on_universe_id" ON "races" ("universe_id");
CREATE INDEX "index_relay_receipts_on_player_id_and_created_at" ON "relay_receipts" ("player_id", "created_at");
CREATE INDEX "index_relay_receipts_on_player_id_and_status" ON "relay_receipts" ("player_id", "status");
CREATE INDEX "index_relay_receipts_on_player_id" ON "relay_receipts" ("player_id");
CREATE INDEX "index_scenes_on_acted_on" ON "scenes" ("acted_on_type", "acted_on_id");
CREATE INDEX "index_scenes_on_location_id" ON "scenes" ("location_id");
CREATE INDEX "index_scenes_on_previous_scene_id" ON "scenes" ("previous_scene_id");
CREATE INDEX "index_scenes_on_story_id_and_is_opening" ON "scenes" ("story_id", "is_opening");
CREATE INDEX "index_scenes_on_story_id_and_resolved_action" ON "scenes" ("story_id", "resolved_action");
CREATE INDEX "index_scenes_on_story_id_and_story_timestamp" ON "scenes" ("story_id", "story_timestamp");
CREATE INDEX "index_scenes_on_story_id" ON "scenes" ("story_id");
CREATE INDEX "index_stories_on_universe_id" ON "stories" ("universe_id");
CREATE INDEX "index_system_one_receipts_on_player_id_and_created_at" ON "system_one_receipts" ("player_id", "created_at");
CREATE INDEX "index_system_one_receipts_on_player_id" ON "system_one_receipts" ("player_id");
CREATE INDEX "index_system_one_receipts_on_playthrough_id" ON "system_one_receipts" ("playthrough_id");
CREATE INDEX "index_world_events_on_playthrough_id" ON "world_events" ("playthrough_id");
CREATE INDEX "index_world_events_on_story_id_and_occurred_at" ON "world_events" ("story_id", "occurred_at");
CREATE INDEX "index_world_events_on_story_id_and_scheduled_for" ON "world_events" ("story_id", "scheduled_for");
CREATE INDEX "index_world_events_on_story_id" ON "world_events" ("story_id");
CREATE INDEX "index_world_events_on_world_mechanic_id" ON "world_events" ("world_mechanic_id");
CREATE UNIQUE INDEX "index_world_mechanics_on_story_id_and_name" ON "world_mechanics" ("story_id", "name");
CREATE INDEX "index_world_mechanics_on_story_id" ON "world_mechanics" ("story_id");
PRAGMA writable_schema=OFF;
COMMIT;
