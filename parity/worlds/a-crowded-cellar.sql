PRAGMA foreign_keys=OFF;
BEGIN TRANSACTION;
CREATE TABLE IF NOT EXISTS "characters_scenes" ("character_id" integer NOT NULL, "scene_id" integer NOT NULL);
INSERT INTO characters_scenes VALUES(1000000001,1000000001);
INSERT INTO characters_scenes VALUES(1000000002,1000000001);
INSERT INTO characters_scenes VALUES(1000000003,1000000001);
CREATE TABLE IF NOT EXISTS "lab_exits_vantages" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "world" varchar NOT NULL, "name" varchar NOT NULL, "teaser" text NOT NULL, "reached_from" varchar, "danger" varchar, "absent" text, "expects_inside_quantifier" varchar, "expects_population" text, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL);
CREATE TABLE IF NOT EXISTS "lab_realization_kinds" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "world" varchar NOT NULL, "name" varchar NOT NULL, "teaser" text NOT NULL, "reached_from" varchar, "inside" varchar, "danger" varchar, "population" varchar, "expects_inside" text, "expects_population" text, "expects_storeys_above" text, "expects_storeys_below" text, "expects_danger" text, "expects_gradient" text, "expects_hazard" text, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL);
CREATE TABLE IF NOT EXISTS "players" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "name" varchar NOT NULL, "token_digest" varchar NOT NULL, "monthly_limit_usd" decimal(10,4) DEFAULT 1.0 NOT NULL, "revoked_at" datetime(6), "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL);
CREATE TABLE IF NOT EXISTS "playthrough_visits" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "playthrough_id" integer NOT NULL, "location_id" integer NOT NULL, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL);
CREATE TABLE IF NOT EXISTS "refused_lines" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "typed" text NOT NULL, "state" json NOT NULL, "answers" json, "reading" json NOT NULL, "act_confidence" float, "thing_confidence" float, "refusal" json NOT NULL, "place" varchar, "weights" varchar, "playthrough_id" integer, "location_id" integer, "status" varchar DEFAULT 'new' NOT NULL, "label" json, "labelled_at" datetime(6), "note" text, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL);
CREATE TABLE IF NOT EXISTS "ruby_llm_batches" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "provider_batch_id" varchar NOT NULL, "provider" varchar NOT NULL, "status" varchar NOT NULL, "raw_status" varchar, "completed" boolean DEFAULT FALSE NOT NULL, "chat_type" varchar, "batch_protocol" varchar, "chat_ids" json DEFAULT '[]', "request_counts" json, "reported_cost" json, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL);
CREATE TABLE IF NOT EXISTS "ruby_llm_models" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "model_id" varchar NOT NULL, "name" varchar NOT NULL, "provider" varchar NOT NULL, "family" varchar, "model_created_at" datetime(6), "context_window" integer, "max_output_tokens" integer, "knowledge_cutoff" date, "modalities" json DEFAULT '{}', "capabilities" json DEFAULT '[]', "pricing" json DEFAULT '{}', "metadata" json DEFAULT '{}', "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, "unlisted_at" datetime(6));
CREATE TABLE IF NOT EXISTS "ruby_llm_tool_calls" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "message_id" integer NOT NULL, "tool_call_id" varchar NOT NULL, "name" varchar NOT NULL, "arguments" json DEFAULT '{}', "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, "message_type" varchar NOT NULL, "result_type" varchar, "result_id" integer, "thought_signature" text, "approval" varchar, "remote" boolean DEFAULT FALSE NOT NULL);
CREATE TABLE IF NOT EXISTS "ruby_llm_usages" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "chat_type" varchar NOT NULL, "chat_id" integer NOT NULL, "message_type" varchar, "message_id" integer, "operation" varchar NOT NULL, "provider" varchar NOT NULL, "model" varchar NOT NULL, "status" varchar NOT NULL, "input_tokens" integer, "output_tokens" integer, "cache_read_tokens" integer, "cache_write_tokens" integer, "thinking_tokens" integer, "input_cost" decimal(16,10), "output_cost" decimal(16,10), "cache_read_cost" decimal(16,10), "cache_write_cost" decimal(16,10), "thinking_cost" decimal(16,10), "total_cost" decimal(16,10), "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, "latency_ms" integer, CONSTRAINT chk_rails_71abd85d6e CHECK (operation IN ('chat', 'embedding', 'moderation', 'image', 'speech', 'transcription', 'ocr', 'rerank')), CONSTRAINT chk_rails_f50895962a CHECK (status IN ('pending', 'succeeded', 'failed', 'cancelled')));
CREATE TABLE IF NOT EXISTS "ruby_llm_v2_backfills" ("task" varchar NOT NULL, "last_id" integer, "completed" boolean DEFAULT FALSE NOT NULL);
CREATE TABLE IF NOT EXISTS "universes" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "physics" text, "technology" text, "weapons" text, "civilizations" text, "geographies" text, "history" text, "economics" text, "politics" text, "religion" text, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, "gravity" varchar, "carrying" varchar);
INSERT INTO universes VALUES(1000000001,'Ordinary physics.','Lanterns, barrels and cart wheels.','Cudgels.','A market county.','A coaching inn on a wet road.','The inn has stood at the crossroads for a century.','Coin for beds, ale and stabling.','The county reeve keeps the road.','Harvest festivals.','2026-10-30 10:43:43.628832','2026-10-30 10:43:43.628832',NULL,NULL);
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
INSERT INTO location_connections VALUES(1000000001,1000000001,1000000002,'adjacent','about a minute','walking','2026-10-30 10:43:43.955954','2026-10-30 10:43:43.955954',NULL,NULL,'open',NULL);
INSERT INTO location_connections VALUES(1000000002,1000000002,1000000001,'adjacent','about a minute','walking','2026-10-30 10:43:43.959238','2026-10-30 10:43:43.959238',NULL,NULL,'open',NULL);
INSERT INTO location_connections VALUES(1000000003,1000000001,1000000003,'adjacent','about a minute','walking','2026-10-30 10:43:43.961243','2026-10-30 10:43:43.961243',NULL,NULL,'open',NULL);
INSERT INTO location_connections VALUES(1000000004,1000000003,1000000001,'adjacent','about a minute','walking','2026-10-30 10:43:43.965996','2026-10-30 10:43:43.965996',NULL,NULL,'open',NULL);
INSERT INTO location_connections VALUES(1000000005,1000000001,1000000004,'adjacent','about a minute','walking','2026-10-30 10:43:43.967724','2026-10-30 10:43:43.967724',NULL,NULL,'open',NULL);
INSERT INTO location_connections VALUES(1000000006,1000000004,1000000001,'adjacent','about a minute','walking','2026-10-30 10:43:43.969040','2026-10-30 10:43:43.969040',NULL,NULL,'open',NULL);
INSERT INTO location_connections VALUES(1000000007,1000000001,1000000005,'adjacent','about a minute','walking','2026-10-30 10:43:43.970085','2026-10-30 10:43:43.970085',NULL,NULL,'open',NULL);
INSERT INTO location_connections VALUES(1000000008,1000000005,1000000001,'adjacent','about a minute','walking','2026-10-30 10:43:43.979407','2026-10-30 10:43:43.979407',NULL,NULL,'open',NULL);
CREATE TABLE IF NOT EXISTS "locations" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "story_id" integer NOT NULL, "name" varchar, "description" text, "lore" text, "last_protagonist_visit" datetime(6), "parent_location_id" integer, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, "detail_level" varchar DEFAULT 'stub' NOT NULL, "teaser" text, "mobile" boolean DEFAULT FALSE NOT NULL, "danger" varchar DEFAULT 'safe' NOT NULL, "hazard" varchar, "hazard_die" integer, "x" integer, "y" integer, "z" integer, "width" integer, "depth" integer, "population" varchar, "generation_checkpoint" json, "surface" varchar, "kind" varchar, "density" varchar, "sanctuary" boolean DEFAULT FALSE NOT NULL, CONSTRAINT "fk_rails_5bc98acf09"
FOREIGN KEY ("parent_location_id")
  REFERENCES "locations" ("id")
, CONSTRAINT "fk_rails_fedd9b21a0"
FOREIGN KEY ("story_id")
  REFERENCES "stories" ("id")
);
INSERT INTO locations VALUES(1000000001,1000000001,'Taproom','Benches packed shoulder to shoulder under smoky beams.','The Lantern''s taproom has never been this full.',NULL,NULL,'2026-10-30 10:43:43.674420','2026-10-30 10:43:43.674420','realized','A packed taproom.',0,'safe',NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,0);
INSERT INTO locations VALUES(1000000002,1000000001,'Kitchen','Steam, copper pans and a roaring range.','The range was built with the inn.',NULL,NULL,'2026-10-30 10:43:43.686293','2026-10-30 10:43:43.686293','realized','A hot kitchen.',0,'safe',NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,0);
INSERT INTO locations VALUES(1000000003,1000000001,'Stable','Stalls of steaming horses and a hayloft ladder.','Every stall is let tonight.',NULL,NULL,'2026-10-30 10:43:43.694934','2026-10-30 10:43:43.694934','realized','A busy stable.',0,'safe',NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,0);
INSERT INTO locations VALUES(1000000004,1000000001,'Loft','Pallets laid end to end under the eaves.','The loft is let by the pallet.',NULL,NULL,'2026-10-30 10:43:43.707747','2026-10-30 10:43:43.707747','realized','A crowded loft.',0,'safe',NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,0);
INSERT INTO locations VALUES(1000000005,1000000001,'Cellar',NULL,NULL,NULL,NULL,'2026-10-30 10:43:43.722943','2026-10-30 10:43:43.722943','stub','Steps down to the barrels.',0,'safe',NULL,NULL,NULL,NULL,NULL,NULL,NULL,'a crowd',NULL,NULL,NULL,NULL,0);
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
CREATE TABLE IF NOT EXISTS "quest_steps" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "quest_id" integer NOT NULL, "position" integer NOT NULL, "summary" text NOT NULL, "trigger_kind" varchar NOT NULL, "target_name" varchar, "teaser" text, "target_type" varchar, "target_id" integer, "minutes" integer, "bound_at" datetime(6), "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, "companion_id" integer, CONSTRAINT "fk_rails_ba1603d17b"
FOREIGN KEY ("quest_id")
  REFERENCES "quests" ("id")
);
CREATE TABLE IF NOT EXISTS "quests" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "story_id" integer NOT NULL, "parent_quest_id" integer, "title" varchar NOT NULL, "premise" text NOT NULL, "status" varchar DEFAULT 'open' NOT NULL, "contributes" boolean DEFAULT TRUE NOT NULL, "origin" varchar DEFAULT 'seeded' NOT NULL, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_a64954ae79"
FOREIGN KEY ("parent_quest_id")
  REFERENCES "quests" ("id")
, CONSTRAINT "fk_rails_fa77fc496b"
FOREIGN KEY ("story_id")
  REFERENCES "stories" ("id")
);
CREATE TABLE IF NOT EXISTS "races" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "universe_id" integer NOT NULL, "name" varchar NOT NULL, "description" text NOT NULL, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, "monstrous" boolean DEFAULT FALSE NOT NULL, CONSTRAINT "fk_rails_c25ac61605"
FOREIGN KEY ("universe_id")
  REFERENCES "universes" ("id")
);
INSERT INTO races VALUES(1000000001,1000000001,'Countryfolk','Ordinary people of the county.','2026-10-30 10:43:43.632356','2026-10-30 10:43:43.632356',0);
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
INSERT INTO scenes VALUES(1000000001,1000000001,1000000001,NULL,'You stand dripping by the taproom door.','2026-09-21 19:00:00','Nell comes in out of the rain.','2026-10-30 10:43:44.133098','2026-10-30 10:43:44.133098',1,NULL,NULL,NULL,NULL,NULL,0,NULL);
CREATE TABLE IF NOT EXISTS "stories" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "title" varchar, "genre" varchar, "preface" text, "summary" text, "start_time" datetime(6), "universe_id" integer NOT NULL, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, "generation_snapshot" text, "people_limit" integer, "things_limit" integer, CONSTRAINT "fk_rails_2a912ea846"
FOREIGN KEY ("universe_id")
  REFERENCES "universes" ("id")
);
INSERT INTO stories VALUES(1000000001,'A Crowded Cellar (engine sweep)','country adventure','The Lantern Inn is full to the rafters on a wet night.','Nell looks for room in a full inn.','2026-09-21 19:00:00',1000000001,'2026-10-30 10:43:43.651026','2026-10-30 10:43:43.651026',NULL,20,NULL);
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
INSERT INTO schema_migrations VALUES('20261030120000');
INSERT INTO schema_migrations VALUES('20261029154235');
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
INSERT INTO schema_migrations VALUES('20261030200000');
INSERT INTO schema_migrations VALUES('20261030230000');
INSERT INTO schema_migrations VALUES('20261031090000');
INSERT INTO schema_migrations VALUES('20261031120000');
INSERT INTO schema_migrations VALUES('20261031150000');
CREATE TABLE IF NOT EXISTS "ar_internal_metadata" ("key" varchar NOT NULL PRIMARY KEY, "value" varchar, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL);
INSERT INTO ar_internal_metadata VALUES('environment','test','2026-10-30 10:43:42.317995','2026-10-30 10:43:42.317999');
INSERT INTO ar_internal_metadata VALUES('schema_sha1','ebdda9bf12dc2d5493a426a2e64de0ed551652b9','2026-10-30 10:43:42.324377','2026-10-30 10:43:42.324380');
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
INSERT INTO characters VALUES(1000000001,1000000001,'Nell','Nell',33,'female','Plain-spoken.','A soaked cloak.','A dry corner.','Crowds.','Sleeping in the road.','A tinker caught out by the rain.',0,'2026-10-30 10:43:43.797067','2026-10-30 10:43:43.797067',1000000001,1,NULL,0,10,8,12,12,12,0,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL);
INSERT INTO characters VALUES(1000000002,1000000001,'Hob Allard','Hob',44,'male','Gruff and fair.','A stained apron.','A full house.','Unpaid tabs.','A cold hearth.','The innkeeper, who has run the Lantern for twenty years.',0,'2026-10-30 10:43:43.829152','2026-10-30 10:43:43.829152',1000000001,0,1000000001,0,3,8,12,12,12,0,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL);
INSERT INTO characters VALUES(1000000003,1000000001,'Ida Fenn','Ida',27,'female','Quick and wry.','Rolled sleeves.','Tips.','Spilled ale.','Deep water.','A barmaid saving for a boat of her own.',0,'2026-10-30 10:43:43.834649','2026-10-30 10:43:43.834649',1000000001,0,1000000001,0,3,8,12,12,12,0,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL);
INSERT INTO characters VALUES(1000000004,1000000001,'Moss Tarrow','Moss',51,'male','Slow and certain.','A floury shirt.','Fresh bread.','Haste.','Fire in the flue.','The cook, who came with the inn.',0,'2026-10-30 10:43:43.839521','2026-10-30 10:43:43.839521',1000000001,0,1000000002,0,3,8,12,12,12,0,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL);
INSERT INTO characters VALUES(1000000005,1000000001,'Pell Quill','Pell',19,'non-binary','Eager and clumsy.','Wet cuffs.','Praise.','Scouring pots.','The cook''s temper.','A scullion learning the trade.',0,'2026-10-30 10:43:43.851223','2026-10-30 10:43:43.851223',1000000001,0,1000000002,0,3,8,12,12,12,0,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL);
INSERT INTO characters VALUES(1000000006,1000000001,'Rue Ambry','Rue',36,'female','Tired and patient.','A dusty coat.','A hot meal.','Delays.','A lame horse.','A carter waiting for her supper.',0,'2026-10-30 10:43:43.857509','2026-10-30 10:43:43.857509',1000000001,0,1000000002,0,3,8,12,12,12,0,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL);
INSERT INTO characters VALUES(1000000007,1000000001,'Tam Brisk','Tam',23,'male','Cheerful and loud.','Straw in his hair.','Horses.','Tight fists.','Being kicked.','The ostler who minds the guests'' horses.',0,'2026-10-30 10:43:43.861686','2026-10-30 10:43:43.861686',1000000001,0,1000000003,0,3,8,12,12,12,0,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL);
INSERT INTO characters VALUES(1000000008,1000000001,'Vey Corran','Vey',40,'female','Watchful.','A long oilskin.','Open roads.','Fences.','Wolves.','A drover passing through with her herd.',0,'2026-10-30 10:43:43.865582','2026-10-30 10:43:43.865582',1000000001,0,1000000003,0,3,8,12,12,12,0,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL);
INSERT INTO characters VALUES(1000000009,1000000001,'Wick Dunmore','Wick',62,'male','Talkative.','A patched pack.','A good story.','Paying for a bed.','Dying alone.','An old pedlar sleeping in the hay to save a coin.',0,'2026-10-30 10:43:43.869814','2026-10-30 10:43:43.869814',1000000001,0,1000000003,0,3,8,12,12,12,0,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL);
INSERT INTO characters VALUES(1000000010,1000000001,'Yara Holt','Yara',31,'female','Restless.','Riding boots.','Speed.','Waiting.','A lost letter.','A courier resting between runs.',0,'2026-10-30 10:43:43.874924','2026-10-30 10:43:43.874924',1000000001,0,1000000004,0,3,8,12,12,12,0,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL);
INSERT INTO characters VALUES(1000000011,1000000001,'Zeb Marrow','Zeb',29,'male','Easygoing.','A green waistcoat.','Applause.','Silence.','A broken string.','A fiddler hired for the evening.',0,'2026-10-30 10:43:43.878653','2026-10-30 10:43:43.878653',1000000001,0,1000000004,0,3,8,12,12,12,0,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL);
INSERT INTO characters VALUES(1000000012,1000000001,'Ash Penrose','Ash',34,'trans man','Precise.','Ink on his fingers.','Straight lines.','Muddle.','Getting lost.','A surveyor laid up by the rain.',0,'2026-10-30 10:43:43.882211','2026-10-30 10:43:43.882211',1000000001,0,1000000004,0,3,8,12,12,12,0,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL);
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
PRAGMA writable_schema=ON;
CREATE TABLE IF NOT EXISTS sqlite_sequence(name,seq);
DELETE FROM sqlite_sequence;
INSERT INTO sqlite_sequence VALUES('lab_exits_vantages',1000000000);
INSERT INTO sqlite_sequence VALUES('lab_realization_kinds',1000000000);
INSERT INTO sqlite_sequence VALUES('players',1000000000);
INSERT INTO sqlite_sequence VALUES('playthrough_visits',1000000000);
INSERT INTO sqlite_sequence VALUES('refused_lines',1000000000);
INSERT INTO sqlite_sequence VALUES('ruby_llm_batches',1000000000);
INSERT INTO sqlite_sequence VALUES('ruby_llm_models',1000000000);
INSERT INTO sqlite_sequence VALUES('ruby_llm_tool_calls',1000000000);
INSERT INTO sqlite_sequence VALUES('ruby_llm_usages',1000000000);
INSERT INTO sqlite_sequence VALUES('universes',1000000001);
INSERT INTO sqlite_sequence VALUES('chats',1000000000);
INSERT INTO sqlite_sequence VALUES('interactions',1000000000);
INSERT INTO sqlite_sequence VALUES('items',1000000000);
INSERT INTO sqlite_sequence VALUES('lab_exits_judgements',1000000000);
INSERT INTO sqlite_sequence VALUES('lab_exits_samples',1000000000);
INSERT INTO sqlite_sequence VALUES('lab_realization_samples',1000000000);
INSERT INTO sqlite_sequence VALUES('location_connections',1000000008);
INSERT INTO sqlite_sequence VALUES('locations',1000000005);
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
INSERT INTO sqlite_sequence VALUES('quest_outcomes',1000000000);
INSERT INTO sqlite_sequence VALUES('quest_steps',1000000000);
INSERT INTO sqlite_sequence VALUES('quests',1000000000);
INSERT INTO sqlite_sequence VALUES('races',1000000001);
INSERT INTO sqlite_sequence VALUES('relay_receipts',1000000000);
INSERT INTO sqlite_sequence VALUES('scenes',1000000001);
INSERT INTO sqlite_sequence VALUES('stories',1000000001);
INSERT INTO sqlite_sequence VALUES('system_one_receipts',1000000000);
INSERT INTO sqlite_sequence VALUES('world_events',1000000000);
INSERT INTO sqlite_sequence VALUES('world_mechanics',1000000000);
INSERT INTO sqlite_sequence VALUES('characters',1000000012);
CREATE INDEX "index_characters_scenes_on_character_id_and_scene_id" ON "characters_scenes" ("character_id", "scene_id");
CREATE INDEX "index_characters_scenes_on_scene_id_and_character_id" ON "characters_scenes" ("scene_id", "character_id");
CREATE UNIQUE INDEX "index_players_on_name" ON "players" ("name");
CREATE UNIQUE INDEX "index_players_on_token_digest" ON "players" ("token_digest");
CREATE INDEX "index_playthrough_visits_on_location_id" ON "playthrough_visits" ("location_id");
CREATE UNIQUE INDEX "index_playthrough_visits_on_playthrough_and_location" ON "playthrough_visits" ("playthrough_id", "location_id");
CREATE INDEX "index_playthrough_visits_on_playthrough_id" ON "playthrough_visits" ("playthrough_id");
CREATE INDEX "index_refused_lines_on_status" ON "refused_lines" ("status");
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
PRAGMA writable_schema=OFF;
COMMIT;
