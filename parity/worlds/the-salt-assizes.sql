PRAGMA foreign_keys=OFF;
BEGIN TRANSACTION;
CREATE TABLE IF NOT EXISTS "characters_scenes" ("character_id" integer NOT NULL, "scene_id" integer NOT NULL);
INSERT INTO characters_scenes VALUES(1000000001,1000000001);
INSERT INTO characters_scenes VALUES(1000000002,1000000001);
CREATE TABLE IF NOT EXISTS "lab_exits_vantages" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "world" varchar NOT NULL, "name" varchar NOT NULL, "teaser" text NOT NULL, "reached_from" varchar, "danger" varchar, "absent" text, "expects_inside_quantifier" varchar, "expects_population" text, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL);
CREATE TABLE IF NOT EXISTS "lab_realization_kinds" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "world" varchar NOT NULL, "name" varchar NOT NULL, "teaser" text NOT NULL, "reached_from" varchar, "inside" varchar, "danger" varchar, "population" varchar, "expects_inside" text, "expects_population" text, "expects_storeys_above" text, "expects_storeys_below" text, "expects_danger" text, "expects_gradient" text, "expects_hazard" text, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL);
CREATE TABLE IF NOT EXISTS "players" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "name" varchar NOT NULL, "token_digest" varchar NOT NULL, "monthly_limit_usd" decimal(10,4) DEFAULT 1.0 NOT NULL, "revoked_at" datetime(6), "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL);
CREATE TABLE IF NOT EXISTS "playthrough_visits" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "playthrough_id" integer NOT NULL, "location_id" integer NOT NULL, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL);
CREATE TABLE IF NOT EXISTS "ruby_llm_batches" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "provider_batch_id" varchar NOT NULL, "provider" varchar NOT NULL, "status" varchar NOT NULL, "raw_status" varchar, "completed" boolean DEFAULT FALSE NOT NULL, "chat_type" varchar, "batch_protocol" varchar, "chat_ids" json DEFAULT '[]', "request_counts" json, "reported_cost" json, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL);
CREATE TABLE IF NOT EXISTS "ruby_llm_models" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "model_id" varchar NOT NULL, "name" varchar NOT NULL, "provider" varchar NOT NULL, "family" varchar, "model_created_at" datetime(6), "context_window" integer, "max_output_tokens" integer, "knowledge_cutoff" date, "modalities" json DEFAULT '{}', "capabilities" json DEFAULT '[]', "pricing" json DEFAULT '{}', "metadata" json DEFAULT '{}', "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, "unlisted_at" datetime(6));
CREATE TABLE IF NOT EXISTS "ruby_llm_tool_calls" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "message_id" integer NOT NULL, "tool_call_id" varchar NOT NULL, "name" varchar NOT NULL, "arguments" json DEFAULT '{}', "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, "message_type" varchar NOT NULL, "result_type" varchar, "result_id" integer, "thought_signature" text, "approval" varchar, "remote" boolean DEFAULT FALSE NOT NULL);
CREATE TABLE IF NOT EXISTS "ruby_llm_usages" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "chat_type" varchar NOT NULL, "chat_id" integer NOT NULL, "message_type" varchar, "message_id" integer, "operation" varchar NOT NULL, "provider" varchar NOT NULL, "model" varchar NOT NULL, "status" varchar NOT NULL, "input_tokens" integer, "output_tokens" integer, "cache_read_tokens" integer, "cache_write_tokens" integer, "thinking_tokens" integer, "input_cost" decimal(16,10), "output_cost" decimal(16,10), "cache_read_cost" decimal(16,10), "cache_write_cost" decimal(16,10), "thinking_cost" decimal(16,10), "total_cost" decimal(16,10), "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, "latency_ms" integer, CONSTRAINT chk_rails_71abd85d6e CHECK (operation IN ('chat', 'embedding', 'moderation', 'image', 'speech', 'transcription', 'ocr', 'rerank')), CONSTRAINT chk_rails_f50895962a CHECK (status IN ('pending', 'succeeded', 'failed', 'cancelled')));
CREATE TABLE IF NOT EXISTS "ruby_llm_v2_backfills" ("task" varchar NOT NULL, "last_id" integer, "completed" boolean DEFAULT FALSE NOT NULL);
CREATE TABLE IF NOT EXISTS "universes" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "physics" text, "technology" text, "weapons" text, "civilizations" text, "geographies" text, "history" text, "economics" text, "politics" text, "religion" text, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, "gravity" varchar, "carrying" varchar);
INSERT INTO universes VALUES(1000000001,'Standard physical laws, with one local exception the courts are built around: on this coast an oath spoken over standing salt water binds, and an oath spoken over dry ground does not. Nobody has explained it and everybody has arranged their life around it. A sworn statement made on the causeway at low water holds a man in place -- he cannot walk away from it, cannot contradict it, and will find his own mouth refusing the lie. The same words spoken inland are only words. The binding lasts exactly as long as the water that heard it stays where it is, which on this coast is about six hours.','Sail, tar, oil lamp and iron. Roughly eighteenth century and skewed entirely toward the sea: block and tackle, chain, dredging gear, tide tables computed by hand and reprinted every spring. There is no telegraph and no engine on the coast. The most advanced instrument in the county is the Assize bell, cast to a pitch that carries over surf, and the second is the tide-slate, a ruled board of black slate on which the day''s high water is chalked and which no one is permitted to erase but the Bell-Keeper.','Boat hooks, gutting knives, and the chain. Violence on this coast is mostly the sea''s, and what people do to each other they do with the law and the tide -- a man left chained at the post through one turn of the water is a man the county has killed without touching him. Firearms exist and are ruined by salt within a season, so nobody bothers. The Assizes carry no guard beyond the Bell-Keeper, because the causeway itself is the guard: at high water there is nowhere to run to.','One county, one court, and a hard division between the people of the shore and the people of the marsh. The shore lives by salvage, oysters and the wrecking trade and considers the law a weather event. The marsh lives by grazing and considers itself the county''s conscience. Above both sit the Justicars, appointed for life and travelling in circuit, and beside them the Bell-Keepers -- a guild of one family per coast, who read the water and are the only people whose word can rise a court. Everyone here can tell you how long they have until the water turns.','A drowned coast of mud flats, salt marsh and islands that are only islands twice a day. The Assize causeway runs a mile out from the shore to Gallows Rock and is walkable for about five hours either side of low water. On the rock stand the court, the tide post and the beached hulk of the *Vestry*, a revenue cutter that came ashore ninety years ago and was never floated off. Inland the marsh runs to grazing, and beyond that the county town, which the Assizes visit once and never twice.','The Assizes were founded four hundred years ago after a wrecking case collapsed twice on perjury, when somebody noticed that the men who had sworn on the flats had not been able to lie. The court moved onto the causeway the following spring and has sat there since. Twice the Crown has tried to move it inland and twice the verdicts stopped holding. Ninety years ago the revenue cutter *Vestry* came ashore across the court''s own steps during a hearing; the court adjourned, the hearing resumed inside the wreck, and the wreck has been the vestry ever since.','Oysters, salt, salvage and fees. Everything on this coast is paid at low water and settled before the turn, which makes credit almost unknown and grudges almost permanent. The Assizes are funded by the fees of the cases they hear, which means a court with nothing to try does not eat -- a fact everybody notices and nobody says. The wrecking trade is illegal, universal, and the single largest source of coin between the marsh and the sea.','The Justicars answer to the Crown and are ignored by it. Real power on the coast is the ability to say what time the water turns, which is the Bell-Keeper''s, and the ability to say what was sworn, which is the court''s. The two have been quietly at odds for a century: a Justicar who wants a verdict wants more water than the Keeper will give, and a Keeper who rings the court up early has ended a hearing that a life depended on. Neither can overrule the other and both know it.','The coast keeps the Turning: that the world is water and the only sin is being where you said you would not be when it comes in. Its clergy are tide-readers, its sacrament is the count of the last hour before high water, and its heresy -- held quietly by most of the marsh -- is that the sea does not care and never did. Oaths are religious business here as much as legal, which is why a Bell-Keeper is sworn and a Justicar is merely appointed.','2026-10-29 02:09:14.997573','2026-10-29 02:09:14.997573',NULL,NULL);
CREATE TABLE IF NOT EXISTS "chats" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "model_id_string" varchar, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, "ruby_llm_model_id" integer, "purpose" varchar, "playthrough_id" integer, "character_id" integer, "cancelled" boolean DEFAULT FALSE NOT NULL, "player_id" integer, CONSTRAINT "fk_rails_415520c982"
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
CREATE TABLE IF NOT EXISTS "interactions" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "character_id" integer NOT NULL, "scene_id" integer, "location_id" integer, "pre_thought" text, "pre_feeling" text, "action" text, "post_feeling" text, "post_thought" text, "inner_resolution" text, "summary" text, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, "user_input" text, "engine_action" varchar, "action_status" varchar, "action_fact" text, CONSTRAINT "fk_rails_294d4ae9f8"
FOREIGN KEY ("location_id")
  REFERENCES "locations" ("id")
, CONSTRAINT "fk_rails_6b73a2cd17"
FOREIGN KEY ("character_id")
  REFERENCES "characters" ("id")
, CONSTRAINT "fk_rails_b73583a008"
FOREIGN KEY ("scene_id")
  REFERENCES "scenes" ("id")
);
CREATE TABLE IF NOT EXISTS "items" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "character_id" integer, "name" varchar, "description" text, "properties" text, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, "location_id" integer, "playthrough_id" integer, "readable" boolean DEFAULT FALSE NOT NULL, "inscription" text, "template_id" integer, "bulk" varchar DEFAULT 'handy' NOT NULL, "x" integer, "y" integer, "use_kind" varchar DEFAULT 'ordinary' NOT NULL, "combustible" boolean DEFAULT FALSE NOT NULL, "disposition" varchar DEFAULT 'intact' NOT NULL, "fragility" varchar DEFAULT 'sturdy' NOT NULL, "tier" varchar DEFAULT 'portable' NOT NULL, "holds" varchar, "within_id" integer, "how" varchar, "kit_key" varchar, "noticed_at" datetime(6), "searched_at" datetime(6), "hold" varchar, "material" varchar, "wear" varchar, "worn" boolean DEFAULT FALSE NOT NULL, "supply" varchar, CONSTRAINT "fk_rails_e8ed83a2e6"
FOREIGN KEY ("location_id")
  REFERENCES "locations" ("id")
, CONSTRAINT "fk_rails_35423c7ef8"
FOREIGN KEY ("character_id")
  REFERENCES "characters" ("id")
, CONSTRAINT "fk_rails_5248e92099"
FOREIGN KEY ("playthrough_id")
  REFERENCES "playthroughs" ("id")
);
INSERT INTO items VALUES(1000000001,NULL,'Assize hand-bell','The court''s bell: bronze, the size of two cupped hands, on a yew handle worn pale where nineteen years of your grip have been. It hangs on its hook at your shoulder, and the court rises and sits on it and on nothing else. Nobody but the Bell-Keeper takes it down.','{"rung": false}','2026-10-29 02:09:15.106685','2026-10-29 02:09:15.321819',1000000001,NULL,0,NULL,NULL,'light',2,4,'ordinary',0,'intact','sturdy','portable',NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,0,NULL);
INSERT INTO items VALUES(1000000002,1000000001,'Assize tide-slate','A ruled board of black slate, three hands wide, on which the day''s high water is chalked. Nobody but the Bell-Keeper may erase it. Today''s figure is in a hand that has her slant and none of her pressure.','{"ruled": true, "erasable_by": "bell-keeper", "chalk": "altered"}','2026-10-29 02:09:15.242381','2026-10-29 02:09:15.242381',NULL,NULL,1,'HIGH WATER, THIS DAY — three hours and forty minutes after noon. Court rises on the bell. Sworn: the Bell-Keeper of the Causeway.',NULL,'heavy',NULL,NULL,'ordinary',0,'intact','sturdy','portable',NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,0,NULL);
INSERT INTO items VALUES(1000000003,1000000001,'court black','A clerk''s suit of court black, plain and much brushed.',NULL,'2026-10-29 02:09:15.252680','2026-10-29 02:09:15.252680',NULL,NULL,0,NULL,NULL,'light',NULL,NULL,'ordinary',0,'intact','sturdy','portable',NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,'body',1,NULL);
INSERT INTO items VALUES(1000000004,1000000001,'oilskin','An oilskin coat stiff with salt, cut long for standing on the flats in a wind.',NULL,'2026-10-29 02:09:15.256673','2026-10-29 02:09:15.256673',NULL,NULL,0,NULL,NULL,'light',NULL,NULL,'ordinary',0,'intact','sturdy','portable',NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,'outer',1,NULL);
INSERT INTO items VALUES(1000000005,1000000002,'bands','Two white court bands at the throat, starched flat.',NULL,'2026-10-29 02:09:15.280114','2026-10-29 02:09:15.280114',NULL,NULL,0,NULL,NULL,'light',NULL,NULL,'ordinary',0,'intact','sturdy','portable',NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,'neck',1,NULL);
INSERT INTO items VALUES(1000000006,1000000002,'justicar''s gown','A black court gown, damp at the hem from the causeway.',NULL,'2026-10-29 02:09:15.286330','2026-10-29 02:09:15.286330',NULL,NULL,0,NULL,NULL,'light',NULL,NULL,'ordinary',0,'intact','sturdy','portable',NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,'outer',1,NULL);
INSERT INTO items VALUES(1000000007,1000000002,'town shoes','Town shoes of good leather, ruined by the causeway.',NULL,'2026-10-29 02:09:15.290224','2026-10-29 02:09:15.290224',NULL,NULL,0,NULL,NULL,'light',NULL,NULL,'ordinary',0,'intact','sturdy','portable',NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,'feet',1,NULL);
INSERT INTO items VALUES(1000000008,1000000003,'Neb''s knotted cord','A length of tarred line knotted along its whole length, wound round Neb''s free fist: one knot for every wave that has reached the stain band since the ebb, the way the Shorefolk have counted water for two hundred years without writing any of it down. Read against the band, it is the tide''s own figure for this morning, and it is not the one on the slate.','{"knots": 41, "written": false}','2026-10-29 02:09:15.298449','2026-10-29 02:09:15.298449',NULL,NULL,0,NULL,NULL,'light',NULL,NULL,'ordinary',0,'intact','sturdy','portable',NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,0,NULL);
INSERT INTO items VALUES(1000000009,1000000003,'shirt','A wrecker''s shirt, salt-stiff, with nothing over it.',NULL,'2026-10-29 02:09:15.300135','2026-10-29 02:09:15.300135',NULL,NULL,0,NULL,NULL,'light',NULL,NULL,'ordinary',0,'intact','sturdy','portable',NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,'body',1,NULL);
CREATE TABLE IF NOT EXISTS "lab_exits_judgements" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "vantage_id" integer NOT NULL, "name" varchar NOT NULL, "name_key" varchar NOT NULL, "expects_inside" text, "expects_population" text, "verdict" varchar, "aspects" text, "note" text, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_cbf2b734ae"
FOREIGN KEY ("vantage_id")
  REFERENCES "lab_exits_vantages" ("id")
);
CREATE TABLE IF NOT EXISTS "lab_exits_samples" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "vantage_id" integer NOT NULL, "row" json DEFAULT '{}' NOT NULL, "verdict" varchar, "aspects" text, "note" text, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_3cc36a20a9"
FOREIGN KEY ("vantage_id")
  REFERENCES "lab_exits_vantages" ("id")
);
CREATE TABLE IF NOT EXISTS "lab_realization_samples" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "kind_id" integer NOT NULL, "row" json DEFAULT '{}' NOT NULL, "verdict" varchar, "aspects" text, "note" text, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_7bb6f06ff4"
FOREIGN KEY ("kind_id")
  REFERENCES "lab_realization_kinds" ("id")
);
CREATE TABLE IF NOT EXISTS "location_connections" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "location_id" integer NOT NULL, "connected_location_id" integer NOT NULL, "distance" text, "time_to_travel" text, "travel_method" text, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, "hazard" varchar, "hazard_die" integer, "barrier" varchar DEFAULT 'open' NOT NULL, "key_template_id" integer, CONSTRAINT "fk_rails_99792c7943"
FOREIGN KEY ("location_id")
  REFERENCES "locations" ("id")
, CONSTRAINT "fk_rails_34dc49debe"
FOREIGN KEY ("key_template_id")
  REFERENCES "items" ("id")
 ON DELETE SET NULL, CONSTRAINT "fk_rails_37a61eef17"
FOREIGN KEY ("connected_location_id")
  REFERENCES "locations" ("id")
);
INSERT INTO location_connections VALUES(1000000001,1000000001,1000000002,'adjacent','about a minute','walking','2026-10-29 02:09:15.398462','2026-10-29 02:09:15.398462',NULL,NULL,'open',NULL);
INSERT INTO location_connections VALUES(1000000002,1000000002,1000000001,'adjacent','about a minute','walking','2026-10-29 02:09:15.407368','2026-10-29 02:09:15.407368',NULL,NULL,'open',NULL);
INSERT INTO location_connections VALUES(1000000003,1000000001,1000000003,'adjacent','about a minute','walking','2026-10-29 02:09:15.409612','2026-10-29 02:09:15.409612','drop',4,'open',NULL);
INSERT INTO location_connections VALUES(1000000004,1000000003,1000000001,'adjacent','about a minute','walking','2026-10-29 02:09:15.438196','2026-10-29 02:09:15.438196',NULL,NULL,'open',NULL);
CREATE TABLE IF NOT EXISTS "locations" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "story_id" integer NOT NULL, "name" varchar, "description" text, "lore" text, "last_protagonist_visit" datetime(6), "parent_location_id" integer, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, "detail_level" varchar DEFAULT 'stub' NOT NULL, "teaser" text, "mobile" boolean DEFAULT FALSE NOT NULL, "danger" varchar DEFAULT 'safe' NOT NULL, "hazard" varchar, "hazard_die" integer, "x" integer, "y" integer, "z" integer, "width" integer, "depth" integer, "population" varchar, "generation_checkpoint" json, "surface" varchar, "kind" varchar, "density" varchar, "sanctuary" boolean DEFAULT FALSE NOT NULL, CONSTRAINT "fk_rails_5bc98acf09"
FOREIGN KEY ("parent_location_id")
  REFERENCES "locations" ("id")
, CONSTRAINT "fk_rails_fedd9b21a0"
FOREIGN KEY ("story_id")
  REFERENCES "stories" ("id")
);
INSERT INTO locations VALUES(1000000001,1000000001,'The Causeway Court',unistr('You are standing in the open court on Gallows Rock, which is four walls of dressed stone about waist high, a flagged floor, a bench, and no roof at all -- roofs, on this coast, are what the sea takes first. The Justicar''s bench faces seaward so that whoever is being tried has the water behind them. The day''s cases are chalked on the tide-slate propped against the wall at your left hand, with the high-water figure above them in somebody else''s writing.\u000a\u000aBehind you the causeway runs a mile back to the shore and its edges have already stopped being edges. Seaward, the rock narrows to the tide post, and you can see the chain from here. The beached hulk of the *Vestry* lies canted across the northern side of the court with its hatch propped open on a boat hook, close enough that the court''s own papers are kept in it. The bell hangs on its hook at your shoulder. Nobody in this court but you can tell how long any of it has.'),unistr('The court has sat on this rock for four hundred years because an oath sworn over standing salt water binds and an oath sworn on dry ground does not. Everything about the building follows from that: no roof, a low wall so the water can be seen, a floor that drains, and a bench positioned so the Justicar is the only person in the court who is not watching the tide.\u000a\u000aThe tide-slate is the one object here with a rule about it. Only the Bell-Keeper may erase the day''s figure, which in four hundred years has been broken twice, both times in the same case, and the case is not in the records because the record was the thing that was altered.'),NULL,NULL,'2026-10-29 02:09:15.058497','2026-10-29 02:09:15.311191','realized','A bench, a slate, and a mile of causeway going under.',0,'safe',NULL,NULL,0,0,0,11,9,NULL,NULL,NULL,NULL,NULL,0);
INSERT INTO locations VALUES(1000000002,1000000001,'The Tide Post',unistr('The seaward end of Gallows Rock is a shelf of granite about twelve feet across, sloping, worn smooth, with the post set into it: an iron column the thickness of a mast, sunk four hundred years and stained to the colour of the rock in a band that tells you exactly where the water gets to. Neb Halloran is chained to it by the left wrist with about six feet of play, standing where the shelf is highest, and he has arranged himself so as not to be sitting when you arrive.\u000a\u000aThere is nothing else out here. No shelter, no rail, no second way off -- the shelf runs back to the court and that is all. The surf is working at the low side of the rock in a way it was not doing an hour ago, and the stain band on the post is a hand and a half above the wet.'),unistr('The post predates the court. It was a mooring for the boats that worked the rock before anybody thought of trying anyone here, and the practice of chaining the accused to it during a hearing began as a way of stopping men from walking off down the causeway mid-testimony. That it also puts them where the water will reach was noticed early and never corrected, and the county''s phrase for a hung jury is still *the post decided*.\u000a\u000aThree men have died at it in the court''s records, all three during adjournments, all three with the Bell-Keeper of the day recorded as having given the correct figure.'),NULL,NULL,'2026-10-29 02:09:15.120925','2026-10-29 02:09:15.333151','realized','A chain, a man, and about five feet of dry rock.',0,'safe','flooded',4,0,0,0,9,7,NULL,NULL,NULL,NULL,NULL,0);
INSERT INTO locations VALUES(1000000003,1000000001,'The Vestry Hulk',NULL,NULL,NULL,NULL,'2026-10-29 02:09:15.133718','2026-10-29 02:09:15.353125','stub','A revenue cutter that came ashore in 1836 and is now where the papers live.',0,'safe',NULL,NULL,0,0,0,11,9,NULL,NULL,NULL,NULL,NULL,0);
CREATE TABLE IF NOT EXISTS "locations_world_events" ("location_id" integer NOT NULL, "world_event_id" integer NOT NULL, CONSTRAINT "fk_rails_861146f35c"
FOREIGN KEY ("location_id")
  REFERENCES "locations" ("id")
, CONSTRAINT "fk_rails_16317bfdf9"
FOREIGN KEY ("world_event_id")
  REFERENCES "world_events" ("id")
);
CREATE TABLE IF NOT EXISTS "messages" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "chat_id" integer NOT NULL, "role" varchar, "content" text, "model_id_string" varchar, "input_tokens" integer, "output_tokens" integer, "tool_call_id" integer, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, "model_id" integer, "scene_id" integer, "content_raw" json, "cache_until_here" boolean DEFAULT FALSE NOT NULL, "finish_reason" varchar, "citations" json, "server_tool_calls" json, "raw_content" json, "raw_reasoning" json, CONSTRAINT "fk_rails_c02b47ad97"
FOREIGN KEY ("model_id")
  REFERENCES "ruby_llm_models" ("id")
, CONSTRAINT "fk_rails_0f670de7ba"
FOREIGN KEY ("chat_id")
  REFERENCES "chats" ("id")
, CONSTRAINT "fk_rails_5c4d747b35"
FOREIGN KEY ("scene_id")
  REFERENCES "scenes" ("id")
);
CREATE TABLE IF NOT EXISTS "playthrough_beats" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "playthrough_id" integer NOT NULL, "quest_step_id" integer NOT NULL, "reached_at" datetime(6) NOT NULL, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_d479d5ffac"
FOREIGN KEY ("playthrough_id")
  REFERENCES "playthroughs" ("id")
, CONSTRAINT "fk_rails_c536cc5a9f"
FOREIGN KEY ("quest_step_id")
  REFERENCES "quest_steps" ("id")
);
CREATE TABLE IF NOT EXISTS "playthrough_blows" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "playthrough_id" integer NOT NULL, "attacker_id" integer NOT NULL, "target_id" integer NOT NULL, "location_id" integer NOT NULL, "damage" integer NOT NULL, "hp_after" integer NOT NULL, "round" integer NOT NULL, "sequence" integer NOT NULL, "story_timestamp" datetime(6) NOT NULL, "scene_id" integer, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_f0e8a189c5"
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
CREATE TABLE IF NOT EXISTS "playthrough_commands" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "playthrough_id" integer NOT NULL, "request_token" varchar NOT NULL, "command" text NOT NULL, "status" varchar DEFAULT 'pending' NOT NULL, "result_scene_id" integer, "refusal" json DEFAULT '{}' NOT NULL, "error_kind" varchar, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, "journal" json DEFAULT '{}' NOT NULL, CONSTRAINT "fk_rails_53f23eb49f"
FOREIGN KEY ("playthrough_id")
  REFERENCES "playthroughs" ("id")
, CONSTRAINT "fk_rails_69c7d16a1a"
FOREIGN KEY ("result_scene_id")
  REFERENCES "scenes" ("id")
);
CREATE TABLE IF NOT EXISTS "playthrough_drifts" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "playthrough_id" integer NOT NULL, "scene_id" integer, "location_id" integer, "action" varchar NOT NULL, "command" text NOT NULL, "offered" text, "story_timestamp" datetime(6), "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_018f29e0ac"
FOREIGN KEY ("playthrough_id")
  REFERENCES "playthroughs" ("id")
, CONSTRAINT "fk_rails_094a605342"
FOREIGN KEY ("location_id")
  REFERENCES "locations" ("id")
, CONSTRAINT "fk_rails_45044856a9"
FOREIGN KEY ("scene_id")
  REFERENCES "scenes" ("id")
);
CREATE TABLE IF NOT EXISTS "playthrough_due_endings" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "playthrough_id" integer NOT NULL, "quest_outcome_id" integer NOT NULL, "due_at" datetime(6) NOT NULL, "declined_at" datetime(6), "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_c471335e2f"
FOREIGN KEY ("playthrough_id")
  REFERENCES "playthroughs" ("id")
, CONSTRAINT "fk_rails_b8e95530cf"
FOREIGN KEY ("quest_outcome_id")
  REFERENCES "quest_outcomes" ("id")
);
CREATE TABLE IF NOT EXISTS "playthrough_endings" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "playthrough_id" integer NOT NULL, "quest_outcome_id" integer NOT NULL, "reached_at" datetime(6) NOT NULL, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_762b78023d"
FOREIGN KEY ("playthrough_id")
  REFERENCES "playthroughs" ("id")
, CONSTRAINT "fk_rails_2220d324b0"
FOREIGN KEY ("quest_outcome_id")
  REFERENCES "quest_outcomes" ("id")
);
CREATE TABLE IF NOT EXISTS "playthrough_feedbacks" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "playthrough_id" integer NOT NULL, "scene_id" integer NOT NULL, "verdict" varchar NOT NULL, "note" text, "prose_model" varchar, "prose_purpose" varchar, "prose_models" text, "answering_models" text, "input_tokens" integer, "output_tokens" integer, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, "prose_prompt_digest" varchar, CONSTRAINT "fk_rails_99ee3b49ea"
FOREIGN KEY ("playthrough_id")
  REFERENCES "playthroughs" ("id")
, CONSTRAINT "fk_rails_247709935a"
FOREIGN KEY ("scene_id")
  REFERENCES "scenes" ("id")
);
CREATE TABLE IF NOT EXISTS "playthrough_npc_states" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "playthrough_id" integer NOT NULL, "character_id" integer NOT NULL, "location_id" integer, "following" boolean DEFAULT FALSE NOT NULL, "ceasefire" boolean DEFAULT FALSE NOT NULL, "peace_after_blow_id" integer DEFAULT 0 NOT NULL, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, "noticed_at" datetime(6), CONSTRAINT "fk_rails_ef100d7841"
FOREIGN KEY ("location_id")
  REFERENCES "locations" ("id")
, CONSTRAINT "fk_rails_71a135780d"
FOREIGN KEY ("character_id")
  REFERENCES "characters" ("id")
, CONSTRAINT "fk_rails_984efad0e9"
FOREIGN KEY ("playthrough_id")
  REFERENCES "playthroughs" ("id")
);
CREATE TABLE IF NOT EXISTS "playthrough_overreaches" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "playthrough_id" integer NOT NULL, "scene_id" integer, "location_id" integer, "action" varchar NOT NULL, "command" text NOT NULL, "acted" text NOT NULL, "unacted" text NOT NULL, "story_timestamp" datetime(6), "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_b903d1bc16"
FOREIGN KEY ("playthrough_id")
  REFERENCES "playthroughs" ("id")
, CONSTRAINT "fk_rails_9aac75595a"
FOREIGN KEY ("location_id")
  REFERENCES "locations" ("id")
, CONSTRAINT "fk_rails_879cc0f78c"
FOREIGN KEY ("scene_id")
  REFERENCES "scenes" ("id")
);
CREATE TABLE IF NOT EXISTS "playthrough_paragraphs" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "playthrough_id" integer NOT NULL, "scene_id" integer NOT NULL, "author" varchar DEFAULT 'player' NOT NULL, "text" text NOT NULL, "audit" json, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, "consent" varchar DEFAULT 'none' NOT NULL, "requests" json, "prompt_digest" varchar, CONSTRAINT "fk_rails_db3983d32c"
FOREIGN KEY ("playthrough_id")
  REFERENCES "playthroughs" ("id")
, CONSTRAINT "fk_rails_52ef8100a0"
FOREIGN KEY ("scene_id")
  REFERENCES "scenes" ("id")
);
CREATE TABLE IF NOT EXISTS "playthrough_passages" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "playthrough_id" integer NOT NULL, "location_connection_id" integer NOT NULL, "opened_by_item_id" integer, "means" varchar NOT NULL, "opened_at" datetime(6) NOT NULL, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_7bdf6a743a"
FOREIGN KEY ("location_connection_id")
  REFERENCES "location_connections" ("id")
 ON DELETE CASCADE, CONSTRAINT "fk_rails_75c2355411"
FOREIGN KEY ("opened_by_item_id")
  REFERENCES "items" ("id")
 ON DELETE SET NULL, CONSTRAINT "fk_rails_995c25e2fd"
FOREIGN KEY ("playthrough_id")
  REFERENCES "playthroughs" ("id")
 ON DELETE CASCADE);
CREATE TABLE IF NOT EXISTS "playthrough_tolls" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "playthrough_id" integer NOT NULL, "character_id" integer NOT NULL, "location_id" integer NOT NULL, "location_connection_id" integer, "hazard" varchar NOT NULL, "saved" boolean DEFAULT FALSE NOT NULL, "damage" integer NOT NULL, "hp_after" integer NOT NULL, "sequence" integer NOT NULL, "story_timestamp" datetime(6) NOT NULL, "scene_id" integer, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_83dfd307ab"
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
CREATE TABLE IF NOT EXISTS "playthrough_turn_events" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "playthrough_command_id" integer NOT NULL, "sequence" integer NOT NULL, "kind" varchar NOT NULL, "data" json DEFAULT '{}' NOT NULL, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_c7a6770f24"
FOREIGN KEY ("playthrough_command_id")
  REFERENCES "playthrough_commands" ("id")
);
CREATE TABLE IF NOT EXISTS "playthrough_vitals" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "playthrough_id" integer NOT NULL, "character_id" integer NOT NULL, "hp_current" integer NOT NULL, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, "provoked_at" datetime(6), CONSTRAINT "fk_rails_03cf4b1b8e"
FOREIGN KEY ("character_id")
  REFERENCES "characters" ("id")
, CONSTRAINT "fk_rails_9956f79aa5"
FOREIGN KEY ("playthrough_id")
  REFERENCES "playthroughs" ("id")
);
CREATE TABLE IF NOT EXISTS "playthrough_volitions" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "playthrough_id" integer NOT NULL, "character_id" integer NOT NULL, "location_id" integer NOT NULL, "scene_id" integer, "chosen" varchar NOT NULL, "status" varchar NOT NULL, "fact" text NOT NULL, "serves" varchar NOT NULL, "round" integer NOT NULL, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, "decided_by" varchar, "system_one_error" varchar, CONSTRAINT "fk_rails_55e6f9e865"
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
CREATE TABLE IF NOT EXISTS "playthroughs" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "story_id" integer NOT NULL, "character_id" integer, "current_location_id" integer, "current_scene_id" integer, "token" varchar NOT NULL, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, "ended_at" datetime(6), "player_id" integer, "mode" varchar DEFAULT 'narrated' NOT NULL, CONSTRAINT "fk_rails_9b48509224"
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
CREATE TABLE IF NOT EXISTS "quest_outcomes" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "quest_id" integer NOT NULL, "name" varchar NOT NULL, "summary" text NOT NULL, "is_default" boolean DEFAULT FALSE NOT NULL, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, "condition" varchar, "minutes" integer, "ramification_summary" text, "ramification_minutes" integer, "step_position" integer, "character_id" integer, CONSTRAINT "fk_rails_acf8ece7c5"
FOREIGN KEY ("quest_id")
  REFERENCES "quests" ("id")
);
INSERT INTO quest_outcomes VALUES(1000000001,1000000001,'the-court-rises-true','You ring the court up on the water''s own figure, with Neb''s cord for the count and nobody''s chalk for the hour. Justicar Brace hears him with the stain band still dry above his head, and the seventy minutes go into the record as what they were -- somebody''s hand, and not the sea''s.',1,'2026-10-29 02:09:15.542185','2026-10-29 02:09:15.542185',NULL,NULL,NULL,NULL,NULL,NULL);
INSERT INTO quest_outcomes VALUES(1000000002,1000000001,'rung-on-the-chalk','You rang the court up before you had been out to the water, so it sat on the chalk. Neb Halloran is heard at last from the causeway''s edge, soaked to the chest; the hearing is lawful in every particular, and the figure it sat on is the forger''s.',0,'2026-10-29 02:09:15.544188','2026-10-29 02:09:15.544188','out_of_order',NULL,NULL,NULL,NULL,NULL);
CREATE TABLE IF NOT EXISTS "quest_steps" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "quest_id" integer NOT NULL, "position" integer NOT NULL, "summary" text NOT NULL, "trigger_kind" varchar NOT NULL, "target_name" varchar, "teaser" text, "target_type" varchar, "target_id" integer, "minutes" integer, "bound_at" datetime(6), "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, "companion_id" integer, CONSTRAINT "fk_rails_ba1603d17b"
FOREIGN KEY ("quest_id")
  REFERENCES "quests" ("id")
);
INSERT INTO quest_steps VALUES(1000000001,1000000001,1,'Go out to the tide post and read the stain band against the water yourself.','reach_location','The Tide Post','The water at the post will say whether the slate is lying.','Location',1000000002,NULL,'2026-09-01 05:20:00','2026-10-29 02:09:15.488636','2026-10-29 02:09:15.506587',NULL);
INSERT INTO quest_steps VALUES(1000000002,1000000001,2,'Get the knotted cord Neb has been keeping since the ebb, the water''s own count.','hold_item','Neb''s knotted cord',NULL,'Item',1000000008,NULL,'2026-09-01 05:20:00','2026-10-29 02:09:15.508994','2026-10-29 02:09:15.516959',NULL);
INSERT INTO quest_steps VALUES(1000000003,1000000001,3,'Take the hand-bell off its hook and ring the court up on the true figure.','hold_item','Assize hand-bell',NULL,'Item',1000000001,NULL,'2026-09-01 05:20:00','2026-10-29 02:09:15.518856','2026-10-29 02:09:15.521887',NULL);
CREATE TABLE IF NOT EXISTS "quests" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "story_id" integer NOT NULL, "parent_quest_id" integer, "title" varchar NOT NULL, "premise" text NOT NULL, "status" varchar DEFAULT 'open' NOT NULL, "contributes" boolean DEFAULT TRUE NOT NULL, "origin" varchar DEFAULT 'seeded' NOT NULL, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_a64954ae79"
FOREIGN KEY ("parent_quest_id")
  REFERENCES "quests" ("id")
, CONSTRAINT "fk_rails_fa77fc496b"
FOREIGN KEY ("story_id")
  REFERENCES "stories" ("id")
);
INSERT INTO quests VALUES(1000000001,1000000001,NULL,'The Altered Figure','Prove the high water on the tide-slate is not yours before the court sits on it and the post decides.','open',1,'seeded','2026-10-29 02:09:15.457594','2026-10-29 02:09:15.457594');
CREATE TABLE IF NOT EXISTS "races" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "universe_id" integer NOT NULL, "name" varchar NOT NULL, "description" text NOT NULL, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, "monstrous" boolean DEFAULT FALSE NOT NULL, CONSTRAINT "fk_rails_c25ac61605"
FOREIGN KEY ("universe_id")
  REFERENCES "universes" ("id")
);
INSERT INTO races VALUES(1000000001,1000000001,'Bell-Keepers','One family to a coast, sworn at seven and never released. They are bred and trained to read water, and they do it the way other people hear music: a Keeper standing on the causeway can tell you the minute of the turn without a table, and cannot stop doing it. They are tall, weathered early, deaf in the ear that faces the bell, and constitutionally unable to give a number they are not sure of, which makes them useless liars and excellent witnesses.','2026-10-29 02:09:15.003245','2026-10-29 02:09:15.003245',0);
INSERT INTO races VALUES(1000000002,1000000001,'Circuit Justicars','Not a bloodline but effectively a caste: appointed for life, moved every season, and forbidden to hear the same county twice. Decades of never staying leave them precise, courteous and profoundly disconnected -- a Justicar knows the law of forty parishes and the name of nobody in any of them. They are the only people on the coast who cannot tell you when the water turns.','2026-10-29 02:09:15.008603','2026-10-29 02:09:15.008603',0);
INSERT INTO races VALUES(1000000003,1000000001,'Marshborn','Graziers and dyke-menders from the inland levels, drier and stiffer than the shore in every sense. They keep records, keep grudges, and supply most of the county''s witnesses and all of its informers. A Marshborn on the causeway is somebody who came a long way to see a thing done properly.','2026-10-29 02:09:15.016906','2026-10-29 02:09:15.016906',0);
INSERT INTO races VALUES(1000000004,1000000001,'Shorefolk','The people of the flats: oystermen, salvagers, wreckers. Broad, salt-cured and cheerfully fatalistic, with an oral memory of every hull lost on this coast for two hundred years and no written record of anything at all. They regard the Assizes as a kind of weather that occasionally takes somebody, and they turn out for every hearing.','2026-10-29 02:09:15.018293','2026-10-29 02:09:15.018293',0);
CREATE TABLE IF NOT EXISTS "relay_receipts" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "player_id" integer NOT NULL, "route" varchar NOT NULL, "model" varchar NOT NULL, "stream" boolean DEFAULT FALSE NOT NULL, "status" varchar DEFAULT 'open' NOT NULL, "reserved_usd" decimal(12,6) NOT NULL, "cost_usd" decimal(12,6), "cost_source" varchar, "input_tokens" integer, "output_tokens" integer, "upstream_status" integer, "finished_at" datetime(6), "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_deae27bd71"
FOREIGN KEY ("player_id")
  REFERENCES "players" ("id")
);
CREATE TABLE IF NOT EXISTS "scenes" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "story_id" integer NOT NULL, "location_id" integer NOT NULL, "previous_scene_id" integer, "description" text, "story_timestamp" datetime(6), "summary" text, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, "is_opening" boolean DEFAULT FALSE NOT NULL, "typed" text, "resolved_action" varchar, "acted_on_type" varchar, "acted_on_id" integer, "resolved_by" varchar, "engine_fallback" boolean DEFAULT FALSE NOT NULL, "engine_fact" text, CONSTRAINT "fk_rails_5cc24f985a"
FOREIGN KEY ("previous_scene_id")
  REFERENCES "scenes" ("id")
, CONSTRAINT "fk_rails_abfe1c0369"
FOREIGN KEY ("location_id")
  REFERENCES "locations" ("id")
, CONSTRAINT "fk_rails_38ea512481"
FOREIGN KEY ("story_id")
  REFERENCES "stories" ("id")
);
INSERT INTO scenes VALUES(1000000001,1000000001,1000000001,NULL,'The chalk is still under your thumb and it is still not your handwriting. You are standing at the slate in the open court on Gallows Rock with the mile of causeway behind you already darkening at its edges, and Justicar Brace is on the bench in his gown with his hands folded, waiting to be told how long he has. Out past the low wall the flats are silvering over the way they do an hour before they stop being flats. The bell hangs on its hook at your shoulder, cold, and the figure chalked above the day''s cases says ten minutes past eight in a hand that has your slant and none of your pressure. From the post at the seaward end of the rock, faint over the surf, somebody is calling a name, and it is not yours.','2026-09-01 05:20:00','The story opens in the Causeway Court with the day''s high water altered on the tide-slate by seventy minutes, the Justicar waiting on the bench for a number, and the chained man audible from the tide post.','2026-10-29 02:09:15.636465','2026-10-29 02:09:15.636465',1,NULL,NULL,NULL,NULL,NULL,0,NULL);
CREATE TABLE IF NOT EXISTS "stories" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "title" varchar, "genre" varchar, "preface" text, "summary" text, "start_time" datetime(6), "universe_id" integer NOT NULL, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, "generation_snapshot" text, "people_limit" integer, "things_limit" integer, CONSTRAINT "fk_rails_2a912ea846"
FOREIGN KEY ("universe_id")
  REFERENCES "universes" ("id")
);
INSERT INTO stories VALUES(1000000001,'The Salt Assizes (engine sweep)','tidal legal mystery','You have rung this court up and down for nineteen years and you have never once been wrong about the water, which is why the chalk on the tide-slate is wrong in somebody else''s hand. High water is written as ten minutes past eight. High water is at seven. You wrote seven yourself, last night, on a dry board in a still room, and the figure on the slate now is not yours and is close enough to yours to pass. Out at the post, Neb Halloran has been chained since the ebb for a wrecking nobody on this coast believes he did alone, and his hearing is set for the last hour before the turn -- an hour that, on that chalk, does not exist. The Justicar is already in the court with his gown on. The bell is on its hook. Whoever moved seventy minutes has given the water time to do what the court would have had to sign for.','The player is Coraith Vell, Bell-Keeper of the Assize causeway, who finds the day''s high water altered by seventy minutes in a hand that is almost her own. The chained man at the tide post has a hearing in an hour the false chalk has erased, so working out who changed it means reading the court''s own papers against the water: the Justicar who needs a verdict, the wreck that serves as the vestry, and the question of who benefits from a drowning the record will call a delay. The tone is cold, procedural and out of doors, and the clock is the sea.','2026-09-01 05:20:00',1000000001,'2026-10-29 02:09:15.036495','2026-10-29 02:09:15.036495',NULL,NULL,NULL);
CREATE TABLE IF NOT EXISTS "system_one_receipts" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "player_id" integer, "playthrough_id" integer, "purpose" varchar, "transport" varchar, "cost_usd" decimal(12,6) NOT NULL, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, "cost_source" varchar DEFAULT 'estimated' NOT NULL, "latency_ms" integer, CONSTRAINT "fk_rails_4ae941c854"
FOREIGN KEY ("player_id")
  REFERENCES "players" ("id")
, CONSTRAINT "fk_rails_75847e1556"
FOREIGN KEY ("playthrough_id")
  REFERENCES "playthroughs" ("id")
);
CREATE TABLE IF NOT EXISTS "world_events" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "world_mechanic_id" integer, "story_id" integer NOT NULL, "occurred_at" datetime(6) NOT NULL, "summary" text NOT NULL, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, "source" varchar NOT NULL, "playthrough_id" integer, "scheduled_for" datetime(6), "fired_at" datetime(6), CONSTRAINT "fk_rails_cafc0959e8"
FOREIGN KEY ("story_id")
  REFERENCES "stories" ("id")
, CONSTRAINT "fk_rails_efc6cdec27"
FOREIGN KEY ("world_mechanic_id")
  REFERENCES "world_mechanics" ("id")
);
CREATE TABLE IF NOT EXISTS "world_mechanics" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "story_id" integer NOT NULL, "name" varchar NOT NULL, "kind" varchar NOT NULL, "cadence" varchar NOT NULL, "description" text, "last_run_at" datetime(6), "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_d7328bbbe5"
FOREIGN KEY ("story_id")
  REFERENCES "stories" ("id")
);
CREATE TABLE IF NOT EXISTS "schema_migrations" ("version" varchar NOT NULL PRIMARY KEY);
INSERT INTO schema_migrations VALUES('20261027120117');
INSERT INTO schema_migrations VALUES('20261025230000');
INSERT INTO schema_migrations VALUES('20261025200000');
INSERT INTO schema_migrations VALUES('20261025120000');
INSERT INTO schema_migrations VALUES('20261024120000');
INSERT INTO schema_migrations VALUES('20261015213000');
INSERT INTO schema_migrations VALUES('20261015200000');
INSERT INTO schema_migrations VALUES('20261015180000');
INSERT INTO schema_migrations VALUES('20261015120000');
INSERT INTO schema_migrations VALUES('20261010220000');
INSERT INTO schema_migrations VALUES('20261010180000');
INSERT INTO schema_migrations VALUES('20261010120000');
INSERT INTO schema_migrations VALUES('20260929023507');
INSERT INTO schema_migrations VALUES('20260928124453');
INSERT INTO schema_migrations VALUES('20260928102823');
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
INSERT INTO schema_migrations VALUES('20261030120000');
INSERT INTO schema_migrations VALUES('20261030200000');
INSERT INTO schema_migrations VALUES('20261030230000');
INSERT INTO schema_migrations VALUES('20261031090000');
INSERT INTO schema_migrations VALUES('20261031120000');
INSERT INTO schema_migrations VALUES('20261031150000');
INSERT INTO schema_migrations VALUES('20261031180000');
CREATE TABLE IF NOT EXISTS "ar_internal_metadata" ("key" varchar NOT NULL PRIMARY KEY, "value" varchar, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL);
INSERT INTO ar_internal_metadata VALUES('environment','test','2026-10-29 02:06:59.131845','2026-10-29 02:06:59.131848');
INSERT INTO ar_internal_metadata VALUES('schema_sha1','9ea7d9a7496841168065535777683ed7c64300e0','2026-10-29 02:06:59.136646','2026-10-29 02:06:59.136649');
CREATE TABLE IF NOT EXISTS "floors" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "location_id" integer NOT NULL, "rooms" integer NOT NULL, "danger" varchar, "hazard" varchar, "down_location_id" integer, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, "monster_level" integer, "levels_up" boolean DEFAULT FALSE NOT NULL, "minutes" integer, "safe_rooms" integer DEFAULT 0 NOT NULL, "healing_count" integer DEFAULT 0 NOT NULL, "healing_name" varchar, "healing_description" text, CONSTRAINT "fk_rails_ce8b005cb2"
FOREIGN KEY ("location_id")
  REFERENCES "locations" ("id")
 ON DELETE CASCADE, CONSTRAINT "fk_rails_8a0268155a"
FOREIGN KEY ("down_location_id")
  REFERENCES "locations" ("id")
 ON DELETE SET NULL);
CREATE TABLE IF NOT EXISTS "characters" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "story_id" integer NOT NULL, "fullname" varchar, "nickname" varchar, "age" integer, "sex" varchar, "personality" text, "appearance" text, "likes" text, "dislikes" text, "fears" text, "backstory" text, "is_companion" boolean, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, "race_id" integer NOT NULL, "is_protagonist" boolean DEFAULT FALSE NOT NULL, "location_id" integer, "deliberately_absent" boolean DEFAULT FALSE NOT NULL, "level" integer, "hit_die" integer, "strength" integer, "dexterity" integer, "will" integer, "hostile" boolean DEFAULT FALSE NOT NULL, "x" integer, "y" integer, "conscious_desire" text, "unconscious_desire" text, "recognized_need" text, "unrecognized_need" text, "desire_pursuit" varchar, "need_pursuit" varchar, "posted_to_id" integer, CONSTRAINT "fk_rails_6608c248e8"
FOREIGN KEY ("story_id")
  REFERENCES "stories" ("id")
, CONSTRAINT "fk_rails_0b4b445641"
FOREIGN KEY ("location_id")
  REFERENCES "locations" ("id")
, CONSTRAINT "fk_rails_56a213cbe2"
FOREIGN KEY ("race_id")
  REFERENCES "races" ("id")
, CONSTRAINT "fk_rails_4c54fc506e"
FOREIGN KEY ("posted_to_id")
  REFERENCES "locations" ("id")
 ON DELETE SET NULL);
INSERT INTO characters VALUES(1000000001,1000000001,'Coraith Vell','Vell',44,'female','Exact, literal and slow to speak, with the Keeper''s inability to give a figure she is not sure of. She does not argue; she states the water and lets people arrange themselves around it. Under pressure she gets quieter and more precise, which people mistake for calm and is closer to fury.','Tall, salt-weathered a decade early, with cropped grey hair and a permanent squint to seaward. Deaf in the left ear, which faces the bell. Chalk is worn into the creases of her right hand so deeply that it does not wash out.','A slate that agrees with the water, the ten minutes before a turn, the sound of a court rising on time','A figure given without a source, being asked to round, anyone standing between her and the flats','Ringing a court up and finding afterwards that the water had not turned','Sworn Bell-Keeper at seven, as her mother and her mother''s mother were, and keeper of this causeway alone since she was twenty-five. She has rung nineteen years of Assizes up and down and has ended two hearings early, one of which cost a man his acquittal and is the only thing she has ever apologised for. She writes the slate herself every night, in a still room, on a dry board, and has never had cause to check it in the morning.',0,'2026-10-29 02:09:15.196954','2026-10-29 02:09:15.196954',1000000001,1,NULL,0,3,8,13,10,14,0,NULL,NULL,'Coraith Vell wants to keep the tide-slate true through the day''s high water.','Coraith Vell wants to reach the source of the hand that altered her figure.','Coraith Vell needs to attend the bell and ring the court on time.','Coraith Vell needs to reach the flats and measure the water herself.','keep','reach',NULL);
INSERT INTO characters VALUES(1000000002,1000000001,'Ammon Brace','Justicar Brace',58,'male','Courteous, unhurried, genuinely fair-minded within the law and entirely unable to see outside it. He will hear anybody out. He wants the hearing to happen because a hearing not held is a man held without one, and he will not notice that the fastest way to have it is to trust a number nobody sourced.','Neat, dry and out of place: gown, bands, town shoes ruined by the causeway, and a face that has been polite in bad weather for thirty years. He keeps his hands folded because they are cold.','A court that sits when it says it will, a witness who answers the question, being given a figure','Adjournments, the smell of the flats, being asked to wait without a reason','Signing off on a delay that turns out to have been a sentence','Appointed at thirty-one and moved every season since, through forty parishes he could recite and not one he could find a friend in. He has heard eleven wrecking cases on this coast and returned nine convictions, and he is fairly sure that is the correct proportion. He has never in his life known when the water would turn, and he has spent thirty years being told.',0,'2026-10-29 02:09:15.273680','2026-10-29 02:09:15.330754',1000000002,0,1000000001,0,1,6,8,9,16,0,0,3,'Ammon Brace wants to reach the hearing before the tide closes the causeway.','Ammon Brace wants to obtain certainty that the law has not failed the accused.','Ammon Brace needs to attend every witness until the hearing is complete.','Ammon Brace needs to reach a judgment that accounts for the water beyond the law.','reach','attend',NULL);
INSERT INTO characters VALUES(1000000003,1000000001,'Neb Halloran','Neb',33,'male','Loud, funny and shrewd, and getting quieter the higher the water gets. He bargains out of habit even when he has nothing, and he tells the truth in fragments, testing what each one buys before he parts with the next.','Broad, sunburnt over salt-burn, with a wrecker''s hands and a rope-scar around the left wrist that predates the chain by ten years. Shirt, no coat; they took the coat.','A tide that comes in slow, being called by his name, an argument he can win','Being counted with the marsh, questions about his cousins, the sound of the bell','Drowning at the post while the court is still deciding whether to hear him','Third-generation salvager out of the shore villages, chained at the tide post since the ebb for the wrecking of the *Marianne Dowe* -- a job everybody on this coast agrees took six men and which only he has been charged with. He knows all six names. He has said none of them, because five of them are his cousins and the sixth pays for his mother''s roof.',0,'2026-10-29 02:09:15.295749','2026-10-29 02:09:15.351873',1000000004,0,1000000002,0,1,10,15,13,9,0,3,3,'Neb Halloran wants to obtain a hearing before the water reaches the tide post.','Neb Halloran wants to avoid naming the five cousins who paid for his mother''s roof.','Neb Halloran needs to keep his mother''s roof secure through the trial.','Neb Halloran needs to reach the truth without leaving his cousins to drown in his place.','obtain','avoid',NULL);
CREATE TABLE IF NOT EXISTS "playthrough_floors" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "playthrough_id" integer NOT NULL, "location_id" integer NOT NULL, "arrived_at" datetime(6) NOT NULL, "level" integer, "scene_id" integer, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, "fell_at" datetime(6), CONSTRAINT "fk_rails_340e8cdc38"
FOREIGN KEY ("playthrough_id")
  REFERENCES "playthroughs" ("id")
, CONSTRAINT "fk_rails_f82b72b677"
FOREIGN KEY ("location_id")
  REFERENCES "locations" ("id")
, CONSTRAINT "fk_rails_b4a8914069"
FOREIGN KEY ("scene_id")
  REFERENCES "scenes" ("id")
);
CREATE TABLE IF NOT EXISTS "playthrough_rests" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "playthrough_id" integer NOT NULL, "location_id" integer NOT NULL, "recovered" integer NOT NULL, "hp_after" integer NOT NULL, "story_timestamp" datetime(6) NOT NULL, "scene_id" integer, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_d40226d075"
FOREIGN KEY ("playthrough_id")
  REFERENCES "playthroughs" ("id")
, CONSTRAINT "fk_rails_52d7e18410"
FOREIGN KEY ("location_id")
  REFERENCES "locations" ("id")
, CONSTRAINT "fk_rails_68b103131a"
FOREIGN KEY ("scene_id")
  REFERENCES "scenes" ("id")
);
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
INSERT INTO sqlite_sequence VALUES('chats',1000000000);
INSERT INTO sqlite_sequence VALUES('interactions',1000000000);
INSERT INTO sqlite_sequence VALUES('items',1000000009);
INSERT INTO sqlite_sequence VALUES('lab_exits_judgements',1000000000);
INSERT INTO sqlite_sequence VALUES('lab_exits_samples',1000000000);
INSERT INTO sqlite_sequence VALUES('lab_realization_samples',1000000000);
INSERT INTO sqlite_sequence VALUES('location_connections',1000000004);
INSERT INTO sqlite_sequence VALUES('locations',1000000003);
INSERT INTO sqlite_sequence VALUES('messages',1000000000);
INSERT INTO sqlite_sequence VALUES('playthrough_beats',1000000000);
INSERT INTO sqlite_sequence VALUES('playthrough_blows',1000000000);
INSERT INTO sqlite_sequence VALUES('playthrough_commands',1000000000);
INSERT INTO sqlite_sequence VALUES('playthrough_drifts',1000000000);
INSERT INTO sqlite_sequence VALUES('playthrough_due_endings',1000000000);
INSERT INTO sqlite_sequence VALUES('playthrough_endings',1000000000);
INSERT INTO sqlite_sequence VALUES('playthrough_feedbacks',1000000000);
INSERT INTO sqlite_sequence VALUES('playthrough_npc_states',1000000000);
INSERT INTO sqlite_sequence VALUES('playthrough_overreaches',1000000000);
INSERT INTO sqlite_sequence VALUES('playthrough_paragraphs',1000000000);
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
INSERT INTO sqlite_sequence VALUES('characters',1000000003);
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
CREATE INDEX "index_items_on_within_id" ON "items" ("within_id");
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
CREATE UNIQUE INDEX "index_playthrough_due_endings_on_playthrough_id" ON "playthrough_due_endings" ("playthrough_id");
CREATE INDEX "index_playthrough_due_endings_on_quest_outcome_id" ON "playthrough_due_endings" ("quest_outcome_id");
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
CREATE INDEX "index_playthrough_paragraphs_on_playthrough_id" ON "playthrough_paragraphs" ("playthrough_id");
CREATE UNIQUE INDEX "index_playthrough_paragraphs_on_scene_id_and_author" ON "playthrough_paragraphs" ("scene_id", "author");
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
CREATE INDEX "index_quest_outcomes_on_character_id" ON "quest_outcomes" ("character_id");
CREATE UNIQUE INDEX "index_quest_outcomes_on_quest_id_and_name" ON "quest_outcomes" ("quest_id", "name");
CREATE INDEX "index_quest_outcomes_on_quest_id" ON "quest_outcomes" ("quest_id");
CREATE INDEX "index_quest_steps_on_companion_id" ON "quest_steps" ("companion_id");
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
CREATE UNIQUE INDEX "index_floors_on_location_id" ON "floors" ("location_id");
CREATE INDEX "index_floors_on_down_location_id" ON "floors" ("down_location_id");
CREATE UNIQUE INDEX "index_characters_on_story_id_and_lower_fullname" ON "characters" (story_id, LOWER(fullname));
CREATE INDEX "index_characters_on_location_id_and_hostile" ON "characters" ("location_id", "hostile");
CREATE INDEX "index_characters_on_location_id_and_id" ON "characters" ("location_id", "id");
CREATE INDEX "index_characters_on_location_id" ON "characters" ("location_id");
CREATE INDEX "index_characters_on_race_id" ON "characters" ("race_id");
CREATE INDEX "index_characters_on_story_id_and_is_protagonist" ON "characters" ("story_id", "is_protagonist");
CREATE INDEX "index_characters_on_story_id" ON "characters" ("story_id");
CREATE INDEX "index_characters_on_posted_to_id" ON "characters" ("posted_to_id");
CREATE INDEX "index_playthrough_floors_on_playthrough_id" ON "playthrough_floors" ("playthrough_id");
CREATE INDEX "index_playthrough_floors_on_location_id" ON "playthrough_floors" ("location_id");
CREATE INDEX "index_playthrough_floors_on_scene_id" ON "playthrough_floors" ("scene_id");
CREATE UNIQUE INDEX "index_playthrough_floors_on_playthrough_and_location" ON "playthrough_floors" ("playthrough_id", "location_id");
CREATE INDEX "index_playthrough_rests_on_playthrough_id" ON "playthrough_rests" ("playthrough_id");
CREATE INDEX "index_playthrough_rests_on_location_id" ON "playthrough_rests" ("location_id");
CREATE INDEX "index_playthrough_rests_on_scene_id" ON "playthrough_rests" ("scene_id");
PRAGMA writable_schema=OFF;
COMMIT;
