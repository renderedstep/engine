PRAGMA foreign_keys=OFF;
BEGIN TRANSACTION;
CREATE TABLE IF NOT EXISTS "characters_scenes" ("character_id" integer NOT NULL, "scene_id" integer NOT NULL);
INSERT INTO characters_scenes VALUES(1000000001,1000000001);
INSERT INTO characters_scenes VALUES(1000000002,1000000001);
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
INSERT INTO universes VALUES(1000000001,'The city of Nocturnis is the center of civilization, ruled by the Lunar Sovereigns who control the supply of moonstones and advanced technology. The Sovereigns live in the opulent Sovereign''s Circle, while the rest of the population resides in the ever-changing districts. The Scorch Nomads roam the wasteland, trading scavenged pre-war technology and resources with the Verdant Folk and Crystal Dwarves. The Verdant Folk live in harmony with nature in the Verdant Expanse, while the Crystal Dwarves mine moonstones in the Crystal Peaks. Tensions exist between the Lunar Sovereigns and the other factions due to the Sovereigns'' control of resources and technology, as well as their secretive and hierarchical nature.','2026-09-28 13:13:47.764978','The economy of this world revolves around the trade of moonstones, technology, and resources. The Lunar Sovereigns control the supply of moonstones, which are essential for powering advanced technology and Lunar Compasses. The Crystal Dwarves mine these moonstones and trade them for resources with the other factions. The Scorch Nomads scavenge pre-war technology and resources from the Scorch, trading them with the Verdant Folk for food and other necessities. The Verdant Folk, in turn, trade with the other factions for technology and resources they cannot produce themselves.','The world is dominated by the ever-changing city of Nocturnis, surrounded by a vast, desolate landscape known as the Scorch. The Scorch is a reminder of the Great War, a wasteland where little life thrives. To the north lies the Crystal Peaks, a mountain range rich in moonstones. To the east, the Verdant Expanse is a vast forest that has begun to reclaim the land. Within Nocturnis, notable landmarks include the Ever-Shifting Bazaar, a marketplace that changes location and layout every night, and the Celestial Spire, a tower that remains static despite the city''s rearrangements. The city is divided into districts, each with its unique characteristics, such as the Industrial Quarter, the Residential Loop, and the Sovereign''s Circle.','The Great War, which ended 75 years ago, was a cataclysmic event that reshaped the world. It was fought between the ancient precursors to the Lunar Sovereigns and an alliance of other races over control of Nocturna and advanced technology. The war resulted in the devastation of the landscape, now known as the Scorch, and the creation of the ever-changing city of Nocturnis. In the aftermath, the Lunar Sovereigns seized control of the city and its resources, leading to the current societal structure and tensions.','This world adheres to standard physical laws, with one exception: the city of Nocturnis, built on the ruins of an ancient war, rearranges itself every night due to the residual magical energy from the Great War. This energy, called Nocturna, is harnessed from the moon and causes buildings, streets, and landmarks to shift and change. The effects of Nocturna are limited to the city boundaries, and its influence wanes during the day, allowing the city to remain static until nightfall. Nocturna does not affect living beings directly, but prolonged exposure can cause disorientation and memory loss. The city''s rearrangement follows certain patterns, often influenced by the phases of the moon, but these patterns are complex and not fully understood.','The Lunar Sovereigns hold the most power, controlling access to moonstones and advanced technology. Their rule is absolute within Nocturnis, but their influence wanes in the outer regions. The Scorch Nomads and Verdant Folk have a tense but mutually beneficial relationship, with the Nomads providing technology and the Verdant Folk providing food and shelter. The Crystal Dwarves maintain a neutral stance, focusing on their mining operations and trade agreements. The primary conflict lies between the Lunar Sovereigns and the other factions, who resent the Sovereigns'' control of resources and technology.','Religion in this world is varied and often tied to the unique aspects of each faction. The Lunar Sovereigns worship the moon and Nocturna, believing themselves to be chosen by its power. The Scorch Nomads revere the ancient pre-war technology, seeing it as a reminder of a lost golden age. The Verdant Folk practice a form of animism, believing in the spirits of nature and the interconnectedness of all living things. The Crystal Dwarves worship the mountains and the earth, seeing themselves as stewards of its resources. Religious tensions exist, particularly between the Lunar Sovereigns and the other factions, who see the Sovereigns'' beliefs as justification for their control and oppression.','Technology in this world is a blend of pre-war remnants and post-war innovations. The Great War, which ended 75 years ago, left behind advanced but often broken or dangerous technology. The current society has managed to salvage and repurpose some of this tech, but much of it remains beyond their understanding. Most people rely on steam-powered machinery and simple electronics. The city of Nocturnis has a unique technology called Lunar Compasses, which are devices that can predict the city''s nightly rearrangements with about 60% accuracy. These compasses are highly sought after but require rare moonstones to function.','2026-09-28 13:13:47.764978','Weapons in this world are a mix of traditional and advanced. Most common are steam-powered firearms, which require a special type of ammunition infused with a small amount of Nocturna to function. Swords and other melee weapons are also common, often made from salvaged pre-war materials. The most feared weapons are the remnants of the Great War, such as the Arcane Cannons, which can harness Nocturna to devastating effect. However, these weapons are rare and closely guarded by the city''s ruling council, the Lunar Sovereigns. Carrying weapons within the city limits is legal but regulated, and only licensed individuals can carry firearms.',NULL);
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
INSERT INTO characters VALUES(1000000001,34,'Weathered dark skin, a nomad''s squint, and charcoal on the side of every finger. She wears a surveyor''s coat cut down from something heavier, its pockets restitched to hold dividers, a lamp-oil bottle and a roll of waxed twine. Her hair is cropped short and going grey at one temple only.','Born in a Scorch caravan that made its living selling bearings and salvaged road-signs to a city which rearranged itself out from under them every night. She walked into Nocturnis at nineteen with a stolen theodolite and a talent nobody in the caravan had a word for: she can look at a street and tell where it has been. The Lunar Sovereigns hired her four years ago to map the nightly rearrangements, pay her badly, and deny in public that the survey exists at all. She has been behind on rent for most of that time.','Isbet Marrow wants to reach a complete map of the city''s nightly rearrangements.','2026-09-28 13:13:48.066116',0,'reach',14,'Sovereign paperwork, being told a street has always been there','The memory loss. She has started writing down what she did yesterday.','Isbet Marrow',8,0,0,1,3,'Fresh bearings, strong tea, the hour before the city moves',NULL,'keep','Iz','Methodical to the point of rudeness. She would rather finish a bearing than finish a sentence, and she trusts a measurement she took herself over anything a Sovereign puts in writing. Dry, patient, slow to anger, and incapable of forgetting a debt in either direction.',1000000004,'Isbet Marrow needs to keep her survey notes in order across every night.','female',1000000001,11,'Isbet Marrow wants to keep the one measurement that proves the Sovereigns are lying.','Isbet Marrow needs to reach the memory of what she has begun to lose.','2026-09-28 13:13:48.066116',13,NULL,NULL);
INSERT INTO characters VALUES(1000000002,61,'Heavyset, pale even for a Sovereign, silver hair gone thin on top, and a cough he blames on the plaster dust. He carries a ledger under one arm at all hours, including the hours when nobody is awake to owe him anything.','A Sovereign family''s third son, bought out of his inheritance at thirty for a sum that covered exactly one boarding house in the Larkspur Quarter. He has spent the thirty years since laying his foundation stones loose so the building survives the shifting, and reminding anyone who will hold still that he owns it outright. He does not go up to the third floor after dark.','Grenn Ollivar wants to obtain the overdue rent before the Bell of Saint Aravel rings again.','2026-09-28 13:13:48.097537',0,'obtain',7,'The Sovereign''s Circle, tenants who talk to the walls','That the house will crack after all, and that his family was right','Grenn Ollivar',6,0,0,0,1,'His own foundations, being right about the plaster, a settled ledger',1000000001,'reach','Old Grenn','Loud, aggrieved, and fundamentally decent under about two inches of grievance. He threatens eviction roughly weekly and has carried one out once in thirty years. He would very much like to be asked about the house.',1000000002,'Grenn Ollivar needs to keep his boarding-house ledger settled.','male',1000000001,9,'Grenn Ollivar wants to reach someone who will ask him about the foundations.','Grenn Ollivar needs to reach the third floor after dark and inspect his house.','2026-09-28 13:13:48.097537',12,NULL,NULL);
INSERT INTO characters VALUES(1000000003,44,'A big man gone grey all the way through, skin and coat and eyes the same lamp-lit nothing, standing straighter than a living man stands. His hands are still a ringer''s hands. He does not blink and he does not look away.','He rang the eleven o''clock for twenty-nine years, which meant twenty-nine years of standing in the one tower the city cannot move while the rest of it went past underneath him. Nobody told him what that does. The parish stopped seeing him some while before it stopped hearing him, and by then he had already forgotten his sister''s name, then his own, then what the rope was for -- though he still pulls it, on the hour, more or less.','Marek Sollen wants to keep the bell ringing at the appointed hour.','2026-09-28 13:13:48.107869',0,'keep',8,'Anything that says a name out loud in his tower','Nothing at all, and that is the whole of what is wrong with him','Marek Sollen',10,1,0,0,1,'The hour. The rope. The weight coming back up the line',1000000007,'avoid','the Ringer','Nothing decides anything up there any more. What is left is habit and a terrible steadiness: he climbs, he waits, he pulls, and anything in the chamber that still answers to a name is something he goes for without hurrying and without stopping.',1000000003,'Marek Sollen needs to attend the rope until the city hears its hour.','male',1000000001,14,'Marek Sollen wants to avoid every name spoken in the tower.','Marek Sollen needs to reach the person he was before the bell took his name.','2026-09-28 13:13:48.107869',3,NULL,NULL);
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
CREATE TABLE IF NOT EXISTS "items" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "bulk" varchar DEFAULT 'handy' NOT NULL, "character_id" integer, "combustible" boolean DEFAULT FALSE NOT NULL, "created_at" datetime(6) NOT NULL, "description" text, "disposition" varchar DEFAULT 'intact' NOT NULL, "inscription" text, "location_id" integer, "name" varchar, "playthrough_id" integer, "properties" text, "readable" boolean DEFAULT FALSE NOT NULL, "template_id" integer, "updated_at" datetime(6) NOT NULL, "use_kind" varchar DEFAULT 'ordinary' NOT NULL, "x" integer, "y" integer, "fragility" varchar DEFAULT 'sturdy' NOT NULL, "tier" varchar DEFAULT 'portable' NOT NULL, "holds" varchar, "within_id" integer, "how" varchar, "kit_key" varchar, CONSTRAINT "fk_rails_e8ed83a2e6"
FOREIGN KEY ("location_id")
  REFERENCES "locations" ("id")
, CONSTRAINT "fk_rails_35423c7ef8"
FOREIGN KEY ("character_id")
  REFERENCES "characters" ("id")
, CONSTRAINT "fk_rails_5248e92099"
FOREIGN KEY ("playthrough_id")
  REFERENCES "playthroughs" ("id")
);
INSERT INTO items VALUES(1000000001,'handy',NULL,0,'2026-09-28 13:13:48.009689','A quarto ledger swollen with damp, lying open on the boards under the south louvre where the light is best, a pencil tied to its spine with string. Every cartographer who ever climbed the tower wrote their bearing in it, because a bearing from here is the only one in Nocturnis that means the same thing tomorrow. The last entry is four years old, and the hand under it is the hand the parish register gives for Marek Sollen.','intact',unistr('From the bell, true: Celestial Spire 041. Sovereign''s Circle 118. Larkspur, tonight -- (unfinished)\u000aRung the eleven. Rung the eleven. Rung the'),1000000007,'climbers'' bearing book',NULL,'{"entries": 212, "last_entry_years": 4}',1,NULL,'2026-09-28 13:13:48.009689','ordinary',NULL,NULL,'sturdy','portable',NULL,NULL,NULL,NULL);
INSERT INTO items VALUES(1000000002,'light',1000000003,0,'2026-09-28 13:13:48.117571','A strip of hardwood the length of a forearm, notched once for every hour rung, worn smooth in the middle where a hand has held it for twenty-nine years. The notches stop about two thirds of the way along, and after that the wood is scored across in one continuous line that does not count anything.','intact',unistr('XI. XI. XI. XI. XI. XI. XI. XI. XI. XI. XI. XI. XI. XI. XI. XI. XI. XI.\u000a————————————————————————————————'),NULL,'bell-rope tally',NULL,'{"notches": 29, "counted": false}',1,NULL,'2026-09-28 13:13:48.117571','ordinary',NULL,NULL,'sturdy','portable',NULL,NULL,NULL,NULL);
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
INSERT INTO location_connections VALUES(1000000001,'open',1000000002,'2026-09-28 13:13:48.158312','adjacent',NULL,NULL,NULL,1000000001,'about 3 minutes','climbing','2026-09-28 13:13:48.158312');
INSERT INTO location_connections VALUES(1000000002,'open',1000000001,'2026-09-28 13:13:48.164198','adjacent',NULL,NULL,NULL,1000000002,'about 3 minutes','climbing','2026-09-28 13:13:48.164198');
INSERT INTO location_connections VALUES(1000000003,'open',1000000003,'2026-09-28 13:13:48.169274','adjacent',NULL,NULL,NULL,1000000001,'about a minute','walking','2026-09-28 13:13:48.169274');
INSERT INTO location_connections VALUES(1000000004,'open',1000000001,'2026-09-28 13:13:48.175612','adjacent',NULL,NULL,NULL,1000000003,'about a minute','walking','2026-09-28 13:13:48.175612');
INSERT INTO location_connections VALUES(1000000005,'open',1000000004,'2026-09-28 13:13:48.190108','a short walk',NULL,NULL,NULL,1000000001,'about 15 minutes','climbing','2026-09-28 13:13:48.190108');
INSERT INTO location_connections VALUES(1000000006,'open',1000000001,'2026-09-28 13:13:48.193789','a short walk',NULL,NULL,NULL,1000000004,'about 15 minutes','climbing','2026-09-28 13:13:48.193789');
INSERT INTO location_connections VALUES(1000000007,'open',1000000005,'2026-09-28 13:13:48.198707','across the district',NULL,NULL,NULL,1000000002,'about 20 minutes','walking','2026-09-28 13:13:48.198707');
INSERT INTO location_connections VALUES(1000000008,'open',1000000002,'2026-09-28 13:13:48.201826','across the district',NULL,NULL,NULL,1000000005,'about 20 minutes','walking','2026-09-28 13:13:48.201826');
INSERT INTO location_connections VALUES(1000000009,'open',1000000007,'2026-09-28 13:13:48.207808','a short walk',NULL,NULL,NULL,1000000003,'about 5 minutes','walking','2026-09-28 13:13:48.207808');
INSERT INTO location_connections VALUES(1000000010,'open',1000000003,'2026-09-28 13:13:48.214105','a short walk',NULL,NULL,NULL,1000000007,'about 5 minutes','walking','2026-09-28 13:13:48.214105');
INSERT INTO location_connections VALUES(1000000011,'open',1000000006,'2026-09-28 13:13:48.218089','across the district',NULL,NULL,NULL,1000000004,'about an hour','climbing','2026-09-28 13:13:48.218089');
INSERT INTO location_connections VALUES(1000000012,'open',1000000004,'2026-09-28 13:13:48.223289','across the district',NULL,NULL,NULL,1000000006,'about an hour','climbing','2026-09-28 13:13:48.223289');
CREATE TABLE IF NOT EXISTS "locations" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "created_at" datetime(6) NOT NULL, "danger" varchar DEFAULT 'safe' NOT NULL, "depth" integer, "description" text, "detail_level" varchar DEFAULT 'stub' NOT NULL, "generation_checkpoint" json, "hazard" varchar, "hazard_die" integer, "last_protagonist_visit" datetime(6), "lore" text, "mobile" boolean DEFAULT FALSE NOT NULL, "name" varchar, "parent_location_id" integer, "population" varchar, "story_id" integer NOT NULL, "teaser" text, "updated_at" datetime(6) NOT NULL, "width" integer, "x" integer, "y" integer, "z" integer, "surface" varchar, "kind" varchar, "density" varchar, CONSTRAINT "fk_rails_5bc98acf09"
FOREIGN KEY ("parent_location_id")
  REFERENCES "locations" ("id")
, CONSTRAINT "fk_rails_fedd9b21a0"
FOREIGN KEY ("story_id")
  REFERENCES "stories" ("id")
);
INSERT INTO locations VALUES(1000000001,'2026-09-28 13:13:47.852399','safe',NULL,unistr('You stand in a narrow room on the third floor of Grenn''s boarding house, where the air hangs thick with the smell of boiled cabbage and warm lamp oil. A fine dust of plaster drifts down from the ceiling, caught in the amber glow of a kerosene lamp that sits on an oak table pushed against the window. The building groans softly around you, its timbers settling into their new configuration as the city breathes. The floorboards bow in the middle, and the plaster wall is patched in three places where old cracks have come and gone with previous rearrangements.\u000a\u000aOn the table, a half-finished map of Nocturnis spreads across the scarred wood, weighted down at the corners by a chipped inkwell, a stub of charcoal, and a cold cup of tea. A leather satchel slumped on the floor beside the chair holds a compass that is not of any make you trust, along with field notes bundled with waxed twine. The window is cracked open, and through it you can see Mournwell Lane below, its cobblestones pulsing with a faint, silvery luminescence as they drift almost imperceptibly into a new alignment. Across the way, the shuttered face of a clothier''s shop has migrated closer to your window since morning, its sign turned just far enough that you cannot read it any more.'),'realized',NULL,NULL,NULL,NULL,unistr('Grenn''s boarding house has stood on this spot for as long as anyone in the Larkspur Quarter can recall, though the corner it occupies is rarely the same corner two nights running. Old Grenn, a heavyset man with a cough he blames on the dust, has run the place for thirty years and learned to lay his foundation stones loose so the building can shift without cracking apart. It is one of the few structures in the district not owned outright by the Lunar Sovereigns, which makes Grenn proud and his tenants nervous in equal measure.\u000a\u000aRoom 3 in particular has a reputation. Two previous tenants left in the night without giving notice, and a third--a sailor off a caravan--swore the walls whispered to him in the small hours. Grenn dismisses this as the same disorientation that takes anyone who lives too long under Nocturna''s glow, though he is careful never to spend more than an hour at a stretch in the room himself, and he charges less for it than for Room 2.'),1,'Grenn''s Boarding House, Room 3',NULL,NULL,1000000001,'Boiled cabbage, lamp oil, and the creak of a building that won''t sit still.','2026-09-28 13:13:47.852399',NULL,NULL,NULL,NULL,NULL,NULL,NULL);
INSERT INTO locations VALUES(1000000002,'2026-09-28 13:13:47.863422','safe',NULL,NULL,'stub',NULL,NULL,NULL,NULL,NULL,1,'Mournwell Lane',NULL,NULL,1000000001,'Step out and eavesdrop on the lane while you still know where it runs.','2026-09-28 13:13:47.863422',NULL,NULL,NULL,NULL,NULL,NULL,NULL);
INSERT INTO locations VALUES(1000000003,'2026-09-28 13:13:47.871991','safe',NULL,NULL,'stub',NULL,NULL,NULL,NULL,NULL,1,'Grenn''s Boarding House hallway',NULL,NULL,1000000001,'Wake the old man and buy yourself a little goodwill before dawn.','2026-09-28 13:13:47.871991',NULL,NULL,NULL,NULL,NULL,NULL,NULL);
INSERT INTO locations VALUES(1000000004,'2026-09-28 13:13:47.884288','safe',NULL,NULL,'stub',NULL,NULL,NULL,NULL,NULL,1,'Larkspur Quarter rooftops',NULL,NULL,1000000001,'Climb higher and watch the city read itself into a new shape tonight.','2026-09-28 13:13:47.884288',NULL,NULL,NULL,NULL,NULL,NULL,NULL);
INSERT INTO locations VALUES(1000000005,'2026-09-28 13:13:47.895701','safe',NULL,NULL,'stub',NULL,NULL,NULL,NULL,NULL,0,'Sovereign''s Circle',NULL,NULL,1000000001,'The Sovereigns'' own ground, where the streets have been made to hold still.','2026-09-28 13:13:47.895701',NULL,NULL,NULL,NULL,NULL,NULL,NULL);
INSERT INTO locations VALUES(1000000006,'2026-09-28 13:13:47.901006','safe',NULL,NULL,'stub',NULL,NULL,NULL,NULL,NULL,0,'The Celestial Spire',NULL,NULL,1000000001,'The one tower the city cannot move, and the only bearing worth trusting.','2026-09-28 13:13:47.901006',NULL,NULL,NULL,NULL,NULL,NULL,NULL);
INSERT INTO locations VALUES(1000000007,'2026-09-28 13:13:47.916758','dangerous',NULL,unistr('The stair comes out into the bell chamber and stops being a stair, and the whole city is suddenly below you and going quietly sideways. Saint Aravel''s bell hangs on a headstock older than the Great War, green-black, big enough to stand inside, and the rope comes down through a hole in the boards and lies coiled on the floor as neatly as anything you have seen in Nocturnis. There is no lamp up here. There does not need to be: the louvres are open on four sides and the Larkspur Quarter is putting out enough silver to read a bearing by.\u000a\u000aThe Ringer is standing by the rope with his back to the louvres, and he was standing there before you came up. He is grey the way a photograph is grey, all of him the same shade, and he is far too still for a man waiting for the hour. Nothing in the chamber moves except the bell, very slightly, in the wind off the roofs. The only way down is the stair behind you, and the boards on it creak on the way down exactly as loudly as they creaked on the way up.'),'realized',NULL,NULL,NULL,NULL,unistr('Saint Aravel''s is one of the three fixed points in Nocturnis. The Sovereigns will tell you the tower does not move because it was built on pre-war footings; the Crystal Dwarves will tell you it does not move because there is a seam of unworked moonstone under it and Nocturna will not cross its own supply. Either way, a bearing taken from the bell is the only bearing in this city that means the same thing tomorrow, which is why every cartographer in the quarter eventually climbs it and why the parish gave up charging for the privilege.\u000a\u000aWhat the parish does not put in the guidebook is what standing in a fixed point does to a person over twenty-nine years, while everything they can see goes past them every night. Marek Sollen rang the hour here from the age of fifteen. The register has him retiring at forty-four, which is the year the entries stop rather than the year anything happened, and it does not say where he went. The bell is still rung on the hour, more or less, and nobody in the Larkspur Quarter has gone up to ask who is ringing it.'),0,'The Bell of Saint Aravel',NULL,NULL,1000000001,'Follow the eleven o''clock chime back to the tower that rang it.','2026-09-28 13:13:47.916758',NULL,NULL,NULL,NULL,NULL,NULL,NULL);
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
INSERT INTO quest_outcomes VALUES(1000000001,NULL,'2026-09-28 13:13:48.448811',1,NULL,'the-bearing-holds',1000000001,NULL,NULL,'With the Ringer''s tally and the tower''s own bearing book in your satchel, the survey has its fixed point at last, and a measurement the Sovereigns cannot answer -- twenty-nine years counted from the one place in Nocturnis that holds still, and then no more counting at all.','2026-09-28 13:13:48.448811');
INSERT INTO quest_outcomes VALUES(1000000002,'out_of_order','2026-09-28 13:13:48.456485',0,NULL,'taken-under-his-hands',1000000001,NULL,NULL,'You took the bearing while the Ringer was still standing over the rope, and it shows. The tally is proof of what he was, but the figures you carried home beside it are the figures of somebody writing with their back to something, and they are off by the width of a flinch.','2026-09-28 13:13:48.456485');
CREATE TABLE IF NOT EXISTS "quest_steps" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "bound_at" datetime(6), "created_at" datetime(6) NOT NULL, "minutes" integer, "position" integer NOT NULL, "quest_id" integer NOT NULL, "summary" text NOT NULL, "target_id" integer, "target_name" varchar, "target_type" varchar, "teaser" text, "trigger_kind" varchar NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_ba1603d17b"
FOREIGN KEY ("quest_id")
  REFERENCES "quests" ("id")
);
INSERT INTO quest_steps VALUES(1000000001,'2026-08-31 23:00:00','2026-09-28 13:13:48.345025',NULL,1,1000000001,'Climb to the bell of Saint Aravel, the one bearing in the city that means the same thing tomorrow.',1000000007,'The Bell of Saint Aravel','Location','Every cartographer in the quarter climbs Saint Aravel''s tower sooner or later.','reach_location','2026-09-28 13:13:48.385525');
INSERT INTO quest_steps VALUES(1000000002,'2026-08-31 23:00:00','2026-09-28 13:13:48.395867',NULL,2,1000000001,'Get the Ringer''s tally, twenty-nine years of hours counted from a fixed point.',1000000002,'bell-rope tally','Item',NULL,'hold_item','2026-09-28 13:13:48.414183');
INSERT INTO quest_steps VALUES(1000000003,'2026-08-31 23:00:00','2026-09-28 13:13:48.419086',NULL,3,1000000001,'Take the climbers'' bearing book off the boards and carry the tower''s bearing home.',1000000001,'climbers'' bearing book','Item',NULL,'hold_item','2026-09-28 13:13:48.427258');
CREATE TABLE IF NOT EXISTS "quests" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "contributes" boolean DEFAULT TRUE NOT NULL, "created_at" datetime(6) NOT NULL, "origin" varchar DEFAULT 'seeded' NOT NULL, "parent_quest_id" integer, "premise" text NOT NULL, "status" varchar DEFAULT 'open' NOT NULL, "story_id" integer NOT NULL, "title" varchar NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_a64954ae79"
FOREIGN KEY ("parent_quest_id")
  REFERENCES "quests" ("id")
, CONSTRAINT "fk_rails_fa77fc496b"
FOREIGN KEY ("story_id")
  REFERENCES "stories" ("id")
);
INSERT INTO quests VALUES(1000000001,1,'2026-09-28 13:13:48.302583','seeded',NULL,'Take one bearing in Nocturnis that will still be true tomorrow, and the proof of what standing on it costs.','open',1000000001,'The Fixed Bearing','2026-09-28 13:13:48.302583');
CREATE TABLE IF NOT EXISTS "races" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "created_at" datetime(6) NOT NULL, "description" text NOT NULL, "monstrous" boolean DEFAULT FALSE NOT NULL, "name" varchar NOT NULL, "universe_id" integer NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_c25ac61605"
FOREIGN KEY ("universe_id")
  REFERENCES "universes" ("id")
);
INSERT INTO races VALUES(1000000001,'2026-09-28 13:13:47.781948','The Crystal Dwarves are a race of short, stout beings who inhabit the Crystal Peaks. They possess crystalline skin that refracts light and a natural affinity for crafting and technology. Their society is clan-based and industrious, with a strong emphasis on craftsmanship and innovation. They are the primary miners of moonstones, which they trade with the Lunar Sovereigns for other resources.',0,'Crystal Dwarves',1000000001,'2026-09-28 13:13:47.781948');
INSERT INTO races VALUES(1000000002,'2026-09-28 13:13:47.786949','The Lunar Sovereigns are humans who have adapted to harness Nocturna, granting them enhanced abilities such as night vision and heightened reflexes during nighttime. They are the ruling class of Nocturnis, often possessing pale, almost translucent skin and silver hair due to prolonged exposure to Nocturna. Their society is hierarchical and secretive, with power concentrated among a few families who control access to moonstones and advanced technology.',0,'Lunar Sovereigns',1000000001,'2026-09-28 13:13:47.786949');
INSERT INTO races VALUES(1000000003,'2026-09-28 13:13:47.792445','What is left when somebody stands too long under Nocturna''s glow and the disorientation and the memory loss run all the way to the end. A Blighted keeps the shape of a person and the habits of one -- it will still climb a stair it has no reason to climb, still turn a key in a door it does not need opened -- and it keeps nothing else. It cannot be reasoned with, because there is nobody in there to reason with, and it will not be reasoned at: a Blighted goes for anybody still carrying a name. The Sovereigns keep them out of the ledgers and out of the guidebooks and have never once said the word in public, which is why every district has a story about one and no district has a record of one.',1,'Nocturna-Blighted',1000000001,'2026-09-28 13:13:47.792445');
INSERT INTO races VALUES(1000000004,'2026-09-28 13:13:47.799659','The Scorch Nomads are a hardy people who have adapted to survive in the harsh landscape of the Scorch. They are a mix of humans and other races who were exiled or fled Nocturnis after the Great War. They possess dark, weathered skin and are known for their resilience and resourcefulness. Their society is tribal and egalitarian, with a deep distrust of technology and the Lunar Sovereigns.',0,'Scorch Nomads',1000000001,'2026-09-28 13:13:47.799659');
INSERT INTO races VALUES(1000000005,'2026-09-28 13:13:47.803146','The Verdant Folk are a race of humanoid plant beings who inhabit the Verdant Expanse. They possess bark-like skin, leaves for hair, and a deep connection to nature. Their society is communal and peaceful, with a strong emphasis on harmony and balance. They possess an innate ability to manipulate plant life, which they use to create their homes and tools.',0,'Verdant Folk',1000000001,'2026-09-28 13:13:47.803146');
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
INSERT INTO scenes VALUES(1000000001,NULL,NULL,'2026-09-28 13:13:48.588368','Your pen stops halfway through a bearing, because there is somebody breathing in the doorway. Old Grenn has come up to the third floor after dark, which in four years he has never once done, and he is holding his ledger flat against his chest like a man who means to read aloud from it. The lamp throws his shadow across the half-finished map, across the two blocks of the Larkspur Quarter you have not been able to make agree since morning. He does not say anything. Neither do you. Plaster ticks down out of the ceiling between you, and below the window Mournwell Lane has already gone silver, and somewhere under the floorboards the building begins, very slightly, to turn.',NULL,0,1,1000000001,NULL,NULL,NULL,1000000001,'2026-08-31 23:00:00','The story opens in Room 3 with Grenn Ollivar standing in the doorway after dark, the rent due by morning, and the city starting to move.',NULL,'2026-09-28 13:13:48.588368');
CREATE TABLE IF NOT EXISTS "stories" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "created_at" datetime(6) NOT NULL, "generation_snapshot" text, "genre" varchar, "preface" text, "start_time" datetime(6), "summary" text, "title" varchar, "universe_id" integer NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_2a912ea846"
FOREIGN KEY ("universe_id")
  REFERENCES "universes" ("id")
);
INSERT INTO stories VALUES(1000000001,'2026-09-28 13:13:47.824594',NULL,'Fantasy','The apartment smells of boiled cabbage and lamp oil. A fine dust of plaster drifts down from the ceiling where the building groans, settling itself into its new configuration for the night. You sit at the oak table by the window, watching motes swirl in the amber light of your kerosene lamp. Across the scarred wood, a half-finished map of Nocturnis spreads out like a rumor you can''t quite believe. The Larkspur Quarter, where you woke this morning, has already migrated two blocks north by your last bearings. The Bell of Saint Aravel chimes once in the distance—11 p.m., or close enough. Outside the open window, the cobblestones of Mournwell Lane breathe with a faint, silvery luminescence, rearranging themselves with the patience of something that has done this for seventy-five years. Your landlord, old Grenn, has left a note under your door demanding the overdue rent by morning, or he''ll have your table and your instruments out in the lane by noon. The map is worth more than the rent. Both are due before the city forgets where it put this building.','2026-08-31 23:00:00','The player is a cartographer hired by the Lunar Sovereigns to map the nightly rearrangements of the city of Nocturnis, where buildings, streets, and landmarks shift with the moon''s phases due to residual magical energy called Nocturna. As they work, they uncover conspiracies within the Sovereigns, encounter the displaced peoples of the Scorch Nomads and Verdant Folk, and grapple with the disorienting effects of prolonged exposure to Nocturna, which causes memory loss. The game''s tone is grounded, atmospheric, and focused on mystery and survival in a world that refuses to stay still.','The Lunar Cartographer (engine sweep)',1000000001,'2026-09-28 13:13:47.824594');
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
INSERT INTO world_events VALUES(1000000001,'2026-09-28 13:13:48.479171',NULL,'2026-08-31 23:00:00',NULL,'2026-08-31 23:05:00','seeded',1000000001,'The building finishes settling into its new configuration for the night, and the plaster stops falling from the ceiling of Room 3.','2026-09-28 13:13:48.479171',NULL);
INSERT INTO world_events VALUES(1000000002,'2026-09-28 13:13:48.484298',NULL,'2026-08-31 23:00:00',NULL,'2026-09-01 07:00:00','seeded',1000000001,'Grenn Ollivar comes for the overdue rent, and the table and the instruments go out into Mournwell Lane.','2026-09-28 13:13:48.484298',NULL);
CREATE TABLE IF NOT EXISTS "world_mechanics" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "cadence" varchar NOT NULL, "created_at" datetime(6) NOT NULL, "description" text, "kind" varchar NOT NULL, "last_run_at" datetime(6), "name" varchar NOT NULL, "story_id" integer NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_d7328bbbe5"
FOREIGN KEY ("story_id")
  REFERENCES "stories" ("id")
);
INSERT INTO world_mechanics VALUES(1000000001,'nightly','2026-09-28 13:13:48.238559','At midnight Nocturna floods the city and the Larkspur Quarter travels. The boarding house, its hallway, the lane below it and the rooftops above it go as one piece with their own doors intact -- old Grenn laid his foundation stones loose for exactly this -- and come to rest against a different part of Nocturnis. The fixed ground does not move: the Celestial Spire, the Sovereign''s Circle and the bell tower of Saint Aravel are where they have always been. What changes is which of them you can walk to from here, and a map of last night is a map of nowhere.','shuffle_connections',NULL,'The nightly rearrangement',1000000001,'2026-09-28 13:13:48.238559');
CREATE TABLE IF NOT EXISTS "schema_migrations" ("version" varchar NOT NULL PRIMARY KEY);
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
CREATE TABLE IF NOT EXISTS "ar_internal_metadata" ("key" varchar NOT NULL PRIMARY KEY, "value" varchar, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL);
INSERT INTO ar_internal_metadata VALUES('environment','test','2026-09-28 13:13:11.413601','2026-09-28 13:13:11.413604');
INSERT INTO ar_internal_metadata VALUES('schema_sha1','2517bbb30bb5f6cb104115c857e888515ca7fe1f','2026-09-28 13:13:11.420647','2026-09-28 13:13:11.420649');
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
INSERT INTO sqlite_sequence VALUES('characters',1000000003);
INSERT INTO sqlite_sequence VALUES('chats',1000000000);
INSERT INTO sqlite_sequence VALUES('interactions',1000000000);
INSERT INTO sqlite_sequence VALUES('items',1000000002);
INSERT INTO sqlite_sequence VALUES('lab_exits_judgements',1000000000);
INSERT INTO sqlite_sequence VALUES('lab_exits_samples',1000000000);
INSERT INTO sqlite_sequence VALUES('lab_realization_samples',1000000000);
INSERT INTO sqlite_sequence VALUES('location_connections',1000000012);
INSERT INTO sqlite_sequence VALUES('locations',1000000007);
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
INSERT INTO sqlite_sequence VALUES('races',1000000005);
INSERT INTO sqlite_sequence VALUES('relay_receipts',1000000000);
INSERT INTO sqlite_sequence VALUES('scenes',1000000001);
INSERT INTO sqlite_sequence VALUES('stories',1000000001);
INSERT INTO sqlite_sequence VALUES('system_one_receipts',1000000000);
INSERT INTO sqlite_sequence VALUES('world_events',1000000002);
INSERT INTO sqlite_sequence VALUES('world_mechanics',1000000001);
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
