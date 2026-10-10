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
INSERT INTO universes VALUES(1000000001,'One county, one court, and a hard division between the people of the shore and the people of the marsh. The shore lives by salvage, oysters and the wrecking trade and considers the law a weather event. The marsh lives by grazing and considers itself the county''s conscience. Above both sit the Justicars, appointed for life and travelling in circuit, and beside them the Bell-Keepers -- a guild of one family per coast, who read the water and are the only people whose word can rise a court. Everyone here can tell you how long they have until the water turns.','2026-10-02 13:25:46.806066','Oysters, salt, salvage and fees. Everything on this coast is paid at low water and settled before the turn, which makes credit almost unknown and grudges almost permanent. The Assizes are funded by the fees of the cases they hear, which means a court with nothing to try does not eat -- a fact everybody notices and nobody says. The wrecking trade is illegal, universal, and the single largest source of coin between the marsh and the sea.','A drowned coast of mud flats, salt marsh and islands that are only islands twice a day. The Assize causeway runs a mile out from the shore to Gallows Rock and is walkable for about five hours either side of low water. On the rock stand the court, the tide post and the beached hulk of the *Vestry*, a revenue cutter that came ashore ninety years ago and was never floated off. Inland the marsh runs to grazing, and beyond that the county town, which the Assizes visit once and never twice.','The Assizes were founded four hundred years ago after a wrecking case collapsed twice on perjury, when somebody noticed that the men who had sworn on the flats had not been able to lie. The court moved onto the causeway the following spring and has sat there since. Twice the Crown has tried to move it inland and twice the verdicts stopped holding. Ninety years ago the revenue cutter *Vestry* came ashore across the court''s own steps during a hearing; the court adjourned, the hearing resumed inside the wreck, and the wreck has been the vestry ever since.','Standard physical laws, with one local exception the courts are built around: on this coast an oath spoken over standing salt water binds, and an oath spoken over dry ground does not. Nobody has explained it and everybody has arranged their life around it. A sworn statement made on the causeway at low water holds a man in place -- he cannot walk away from it, cannot contradict it, and will find his own mouth refusing the lie. The same words spoken inland are only words. The binding lasts exactly as long as the water that heard it stays where it is, which on this coast is about six hours.','The Justicars answer to the Crown and are ignored by it. Real power on the coast is the ability to say what time the water turns, which is the Bell-Keeper''s, and the ability to say what was sworn, which is the court''s. The two have been quietly at odds for a century: a Justicar who wants a verdict wants more water than the Keeper will give, and a Keeper who rings the court up early has ended a hearing that a life depended on. Neither can overrule the other and both know it.','The coast keeps the Turning: that the world is water and the only sin is being where you said you would not be when it comes in. Its clergy are tide-readers, its sacrament is the count of the last hour before high water, and its heresy -- held quietly by most of the marsh -- is that the sea does not care and never did. Oaths are religious business here as much as legal, which is why a Bell-Keeper is sworn and a Justicar is merely appointed.','Sail, tar, oil lamp and iron. Roughly eighteenth century and skewed entirely toward the sea: block and tackle, chain, dredging gear, tide tables computed by hand and reprinted every spring. There is no telegraph and no engine on the coast. The most advanced instrument in the county is the Assize bell, cast to a pitch that carries over surf, and the second is the tide-slate, a ruled board of black slate on which the day''s high water is chalked and which no one is permitted to erase but the Bell-Keeper.','2026-10-02 13:25:46.806066','Boat hooks, gutting knives, and the chain. Violence on this coast is mostly the sea''s, and what people do to each other they do with the law and the tide -- a man left chained at the post through one turn of the water is a man the county has killed without touching him. Firearms exist and are ruined by salt within a season, so nobody bothers. The Assizes carry no guard beyond the Bell-Keeper, because the causeway itself is the guard: at high water there is nowhere to run to.',NULL);
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
INSERT INTO characters VALUES(1000000001,44,'Tall, salt-weathered a decade early, with cropped grey hair and a permanent squint to seaward. Deaf in the left ear, which faces the bell. Oilskin over court black, and chalk worn into the creases of her right hand so deeply that it does not wash out.','Sworn Bell-Keeper at seven, as her mother and her mother''s mother were, and keeper of this causeway alone since she was twenty-five. She has rung nineteen years of Assizes up and down and has ended two hearings early, one of which cost a man his acquittal and is the only thing she has ever apologised for. She writes the slate herself every night, in a still room, on a dry board, and has never had cause to check it in the morning.','Coraith Vell wants to keep the tide-slate true through the day''s high water.','2026-10-02 13:25:47.024103',0,'keep',10,'A figure given without a source, being asked to round, anyone standing between her and the flats','Ringing a court up and finding afterwards that the water had not turned','Coraith Vell',8,0,0,1,3,'A slate that agrees with the water, the ten minutes before a turn, the sound of a court rising on time',NULL,'reach','Vell','Exact, literal and slow to speak, with the Keeper''s inability to give a figure she is not sure of. She does not argue; she states the water and lets people arrange themselves around it. Under pressure she gets quieter and more precise, which people mistake for calm and is closer to fury.',1000000001,'Coraith Vell needs to attend the bell and ring the court on time.','female',1000000001,13,'Coraith Vell wants to reach the source of the hand that altered her figure.','Coraith Vell needs to reach the flats and measure the water herself.','2026-10-02 13:25:47.024103',14,NULL,NULL);
INSERT INTO characters VALUES(1000000002,58,'Neat, dry and out of place: gown, bands, town shoes ruined by the causeway, and a face that has been polite in bad weather for thirty years. He keeps his hands folded because they are cold.','Appointed at thirty-one and moved every season since, through forty parishes he could recite and not one he could find a friend in. He has heard eleven wrecking cases on this coast and returned nine convictions, and he is fairly sure that is the correct proportion. He has never in his life known when the water would turn, and he has spent thirty years being told.','Ammon Brace wants to reach the hearing before the tide closes the causeway.','2026-10-02 13:25:47.093179',0,'reach',9,'Adjournments, the smell of the flats, being asked to wait without a reason','Signing off on a delay that turns out to have been a sentence','Ammon Brace',6,0,0,0,1,'A court that sits when it says it will, a witness who answers the question, being given a figure',1000000001,'attend','Justicar Brace','Courteous, unhurried, genuinely fair-minded within the law and entirely unable to see outside it. He will hear anybody out. He wants the hearing to happen because a hearing not held is a man held without one, and he will not notice that the fastest way to have it is to trust a number nobody sourced.',1000000002,'Ammon Brace needs to attend every witness until the hearing is complete.','male',1000000001,8,'Ammon Brace wants to obtain certainty that the law has not failed the accused.','Ammon Brace needs to reach a judgment that accounts for the water beyond the law.','2026-10-02 13:25:47.093179',16,NULL,NULL);
INSERT INTO characters VALUES(1000000003,33,'Broad, sunburnt over salt-burn, with a wrecker''s hands and a rope-scar around the left wrist that predates the chain by ten years. Shirt, no coat; they took the coat.','Third-generation salvager out of the shore villages, chained at the tide post since the ebb for the wrecking of the *Marianne Dowe* -- a job everybody on this coast agrees took six men and which only he has been charged with. He knows all six names. He has said none of them, because five of them are his cousins and the sixth pays for his mother''s roof.','Neb Halloran wants to obtain a hearing before the water reaches the tide post.','2026-10-02 13:25:47.099075',0,'obtain',13,'Being counted with the marsh, questions about his cousins, the sound of the bell','Drowning at the post while the court is still deciding whether to hear him','Neb Halloran',10,0,0,0,1,'A tide that comes in slow, being called by his name, an argument he can win',1000000002,'avoid','Neb','Loud, funny and shrewd, and getting quieter the higher the water gets. He bargains out of habit even when he has nothing, and he tells the truth in fragments, testing what each one buys before he parts with the next.',1000000004,'Neb Halloran needs to keep his mother''s roof secure through the trial.','male',1000000001,15,'Neb Halloran wants to avoid naming the five cousins who paid for his mother''s roof.','Neb Halloran needs to reach the truth without leaving his cousins to drown in his place.','2026-10-02 13:25:47.099075',9,NULL,NULL);
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
CREATE TABLE IF NOT EXISTS "items" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "bulk" varchar DEFAULT 'handy' NOT NULL, "character_id" integer, "combustible" boolean DEFAULT FALSE NOT NULL, "created_at" datetime(6) NOT NULL, "description" text, "disposition" varchar DEFAULT 'intact' NOT NULL, "inscription" text, "location_id" integer, "name" varchar, "playthrough_id" integer, "properties" text, "readable" boolean DEFAULT FALSE NOT NULL, "template_id" integer, "updated_at" datetime(6) NOT NULL, "use_kind" varchar DEFAULT 'ordinary' NOT NULL, "x" integer, "y" integer, "fragility" varchar DEFAULT 'sturdy' NOT NULL, "tier" varchar DEFAULT 'portable' NOT NULL, "holds" varchar, "within_id" integer, "how" varchar, "kit_key" varchar, "noticed_at" datetime(6), CONSTRAINT "fk_rails_e8ed83a2e6"
FOREIGN KEY ("location_id")
  REFERENCES "locations" ("id")
, CONSTRAINT "fk_rails_35423c7ef8"
FOREIGN KEY ("character_id")
  REFERENCES "characters" ("id")
, CONSTRAINT "fk_rails_5248e92099"
FOREIGN KEY ("playthrough_id")
  REFERENCES "playthroughs" ("id")
);
INSERT INTO items VALUES(1000000001,'light',NULL,0,'2026-10-02 13:25:46.926810','The court''s bell: bronze, the size of two cupped hands, on a yew handle worn pale where nineteen years of your grip have been. It hangs on its hook at your shoulder, and the court rises and sits on it and on nothing else. Nobody but the Bell-Keeper takes it down.','intact',NULL,1000000001,'Assize hand-bell',NULL,'{"rung": false}',0,NULL,'2026-10-02 13:25:46.926810','ordinary',NULL,NULL,'sturdy','portable',NULL,NULL,NULL,NULL,NULL);
INSERT INTO items VALUES(1000000002,'heavy',1000000001,0,'2026-10-02 13:25:47.070863','A ruled board of black slate, three hands wide, on which the day''s high water is chalked. Nobody but the Bell-Keeper may erase it. Today''s figure is in a hand that has her slant and none of her pressure.','intact','HIGH WATER, THIS DAY — three hours and forty minutes after noon. Court rises on the bell. Sworn: the Bell-Keeper of the Causeway.',NULL,'Assize tide-slate',NULL,'{"ruled": true, "erasable_by": "bell-keeper", "chalk": "altered"}',1,NULL,'2026-10-02 13:25:47.070863','ordinary',NULL,NULL,'sturdy','portable',NULL,NULL,NULL,NULL,NULL);
INSERT INTO items VALUES(1000000003,'light',1000000003,0,'2026-10-02 13:25:47.105480','A length of tarred line knotted along its whole length, wound round Neb''s free fist: one knot for every wave that has reached the stain band since the ebb, the way the Shorefolk have counted water for two hundred years without writing any of it down. Read against the band, it is the tide''s own figure for this morning, and it is not the one on the slate.','intact',NULL,NULL,'Neb''s knotted cord',NULL,'{"knots": 41, "written": false}',0,NULL,'2026-10-02 13:25:47.105480','ordinary',NULL,NULL,'sturdy','portable',NULL,NULL,NULL,NULL,NULL);
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
INSERT INTO location_connections VALUES(1000000001,'open',1000000002,'2026-10-02 13:25:47.148458','adjacent',NULL,NULL,NULL,1000000001,'about a minute','walking','2026-10-02 13:25:47.148458');
INSERT INTO location_connections VALUES(1000000002,'open',1000000001,'2026-10-02 13:25:47.151921','adjacent',NULL,NULL,NULL,1000000002,'about a minute','walking','2026-10-02 13:25:47.151921');
INSERT INTO location_connections VALUES(1000000003,'open',1000000003,'2026-10-02 13:25:47.154783','adjacent','drop',4,NULL,1000000001,'about a minute','walking','2026-10-02 13:25:47.154783');
INSERT INTO location_connections VALUES(1000000004,'open',1000000001,'2026-10-02 13:25:47.157441','adjacent',NULL,NULL,NULL,1000000003,'about a minute','walking','2026-10-02 13:25:47.157441');
CREATE TABLE IF NOT EXISTS "locations" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "created_at" datetime(6) NOT NULL, "danger" varchar DEFAULT 'safe' NOT NULL, "depth" integer, "description" text, "detail_level" varchar DEFAULT 'stub' NOT NULL, "generation_checkpoint" json, "hazard" varchar, "hazard_die" integer, "last_protagonist_visit" datetime(6), "lore" text, "mobile" boolean DEFAULT FALSE NOT NULL, "name" varchar, "parent_location_id" integer, "population" varchar, "story_id" integer NOT NULL, "teaser" text, "updated_at" datetime(6) NOT NULL, "width" integer, "x" integer, "y" integer, "z" integer, "surface" varchar, "kind" varchar, "density" varchar, CONSTRAINT "fk_rails_5bc98acf09"
FOREIGN KEY ("parent_location_id")
  REFERENCES "locations" ("id")
, CONSTRAINT "fk_rails_fedd9b21a0"
FOREIGN KEY ("story_id")
  REFERENCES "stories" ("id")
);
INSERT INTO locations VALUES(1000000001,'2026-10-02 13:25:46.875124','safe',NULL,unistr('You are standing in the open court on Gallows Rock, which is four walls of dressed stone about waist high, a flagged floor, a bench, and no roof at all -- roofs, on this coast, are what the sea takes first. The Justicar''s bench faces seaward so that whoever is being tried has the water behind them. The day''s cases are chalked on the tide-slate propped against the wall at your left hand, with the high-water figure above them in somebody else''s writing.\u000a\u000aBehind you the causeway runs a mile back to the shore and its edges have already stopped being edges. Seaward, the rock narrows to the tide post, and you can see the chain from here. The beached hulk of the *Vestry* lies canted across the northern side of the court with its hatch propped open on a boat hook, close enough that the court''s own papers are kept in it. The bell hangs on its hook at your shoulder. Nobody in this court but you can tell how long any of it has.'),'realized',NULL,NULL,NULL,NULL,unistr('The court has sat on this rock for four hundred years because an oath sworn over standing salt water binds and an oath sworn on dry ground does not. Everything about the building follows from that: no roof, a low wall so the water can be seen, a floor that drains, and a bench positioned so the Justicar is the only person in the court who is not watching the tide.\u000a\u000aThe tide-slate is the one object here with a rule about it. Only the Bell-Keeper may erase the day''s figure, which in four hundred years has been broken twice, both times in the same case, and the case is not in the records because the record was the thing that was altered.'),0,'The Causeway Court',NULL,NULL,1000000001,'A bench, a slate, and a mile of causeway going under.','2026-10-02 13:25:46.875124',NULL,NULL,NULL,NULL,NULL,NULL,NULL);
INSERT INTO locations VALUES(1000000002,'2026-10-02 13:25:46.941332','safe',NULL,unistr('The seaward end of Gallows Rock is a shelf of granite about twelve feet across, sloping, worn smooth, with the post set into it: an iron column the thickness of a mast, sunk four hundred years and stained to the colour of the rock in a band that tells you exactly where the water gets to. Neb Halloran is chained to it by the left wrist with about six feet of play, standing where the shelf is highest, and he has arranged himself so as not to be sitting when you arrive.\u000a\u000aThere is nothing else out here. No shelter, no rail, no second way off -- the shelf runs back to the court and that is all. The surf is working at the low side of the rock in a way it was not doing an hour ago, and the stain band on the post is a hand and a half above the wet.'),'realized',NULL,'flooded',4,NULL,unistr('The post predates the court. It was a mooring for the boats that worked the rock before anybody thought of trying anyone here, and the practice of chaining the accused to it during a hearing began as a way of stopping men from walking off down the causeway mid-testimony. That it also puts them where the water will reach was noticed early and never corrected, and the county''s phrase for a hung jury is still *the post decided*.\u000a\u000aThree men have died at it in the court''s records, all three during adjournments, all three with the Bell-Keeper of the day recorded as having given the correct figure.'),0,'The Tide Post',NULL,NULL,1000000001,'A chain, a man, and about five feet of dry rock.','2026-10-02 13:25:46.941332',NULL,NULL,NULL,NULL,NULL,NULL,NULL);
INSERT INTO locations VALUES(1000000003,'2026-10-02 13:25:46.954653','safe',NULL,NULL,'stub',NULL,NULL,NULL,NULL,NULL,0,'The Vestry Hulk',NULL,NULL,1000000001,'A revenue cutter that came ashore in 1836 and is now where the papers live.','2026-10-02 13:25:46.954653',NULL,NULL,NULL,NULL,NULL,NULL,NULL);
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
CREATE TABLE IF NOT EXISTS "playthrough_npc_states" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "ceasefire" boolean DEFAULT FALSE NOT NULL, "character_id" integer NOT NULL, "created_at" datetime(6) NOT NULL, "following" boolean DEFAULT FALSE NOT NULL, "location_id" integer, "peace_after_blow_id" integer DEFAULT 0 NOT NULL, "playthrough_id" integer NOT NULL, "updated_at" datetime(6) NOT NULL, "noticed_at" datetime(6), CONSTRAINT "fk_rails_ef100d7841"
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
CREATE TABLE IF NOT EXISTS "playthrough_paragraphs" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "audit" json, "author" varchar DEFAULT 'player' NOT NULL, "created_at" datetime(6) NOT NULL, "playthrough_id" integer NOT NULL, "scene_id" integer NOT NULL, "text" text NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_db3983d32c"
FOREIGN KEY ("playthrough_id")
  REFERENCES "playthroughs" ("id")
, CONSTRAINT "fk_rails_52ef8100a0"
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
CREATE TABLE IF NOT EXISTS "playthroughs" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "character_id" integer, "created_at" datetime(6) NOT NULL, "current_location_id" integer, "current_scene_id" integer, "ended_at" datetime(6), "mode" varchar DEFAULT 'narrated' NOT NULL, "player_id" integer, "story_id" integer NOT NULL, "token" varchar NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_9b48509224"
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
CREATE TABLE IF NOT EXISTS "quest_outcomes" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "character_id" integer, "condition" varchar, "created_at" datetime(6) NOT NULL, "is_default" boolean DEFAULT FALSE NOT NULL, "minutes" integer, "name" varchar NOT NULL, "quest_id" integer NOT NULL, "ramification_minutes" integer, "ramification_summary" text, "step_position" integer, "summary" text NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_acf8ece7c5"
FOREIGN KEY ("quest_id")
  REFERENCES "quests" ("id")
);
INSERT INTO quest_outcomes VALUES(1000000001,NULL,NULL,'2026-10-02 13:25:47.280814',1,NULL,'the-court-rises-true',1000000001,NULL,NULL,NULL,'You ring the court up on the water''s own figure, with Neb''s cord for the count and nobody''s chalk for the hour. Justicar Brace hears him with the stain band still dry above his head, and the seventy minutes go into the record as what they were -- somebody''s hand, and not the sea''s.','2026-10-02 13:25:47.280814');
INSERT INTO quest_outcomes VALUES(1000000002,NULL,'out_of_order','2026-10-02 13:25:47.285031',0,NULL,'rung-on-the-chalk',1000000001,NULL,NULL,NULL,'You rang the court up before you had been out to the water, so it sat on the chalk. Neb Halloran is heard at last from the causeway''s edge, soaked to the chest; the hearing is lawful in every particular, and the figure it sat on is the forger''s.','2026-10-02 13:25:47.285031');
CREATE TABLE IF NOT EXISTS "quest_steps" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "bound_at" datetime(6), "created_at" datetime(6) NOT NULL, "minutes" integer, "position" integer NOT NULL, "quest_id" integer NOT NULL, "summary" text NOT NULL, "target_id" integer, "target_name" varchar, "target_type" varchar, "teaser" text, "trigger_kind" varchar NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_ba1603d17b"
FOREIGN KEY ("quest_id")
  REFERENCES "quests" ("id")
);
INSERT INTO quest_steps VALUES(1000000001,'2026-09-01 05:20:00','2026-10-02 13:25:47.202962',NULL,1,1000000001,'Go out to the tide post and read the stain band against the water yourself.',1000000002,'The Tide Post','Location','The water at the post will say whether the slate is lying.','reach_location','2026-10-02 13:25:47.222311');
INSERT INTO quest_steps VALUES(1000000002,'2026-09-01 05:20:00','2026-10-02 13:25:47.229007',NULL,2,1000000001,'Get the knotted cord Neb has been keeping since the ebb, the water''s own count.',1000000003,'Neb''s knotted cord','Item',NULL,'hold_item','2026-10-02 13:25:47.240998');
INSERT INTO quest_steps VALUES(1000000003,'2026-09-01 05:20:00','2026-10-02 13:25:47.244296',NULL,3,1000000001,'Take the hand-bell off its hook and ring the court up on the true figure.',1000000001,'Assize hand-bell','Item',NULL,'hold_item','2026-10-02 13:25:47.258200');
CREATE TABLE IF NOT EXISTS "quests" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "contributes" boolean DEFAULT TRUE NOT NULL, "created_at" datetime(6) NOT NULL, "origin" varchar DEFAULT 'seeded' NOT NULL, "parent_quest_id" integer, "premise" text NOT NULL, "status" varchar DEFAULT 'open' NOT NULL, "story_id" integer NOT NULL, "title" varchar NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_a64954ae79"
FOREIGN KEY ("parent_quest_id")
  REFERENCES "quests" ("id")
, CONSTRAINT "fk_rails_fa77fc496b"
FOREIGN KEY ("story_id")
  REFERENCES "stories" ("id")
);
INSERT INTO quests VALUES(1000000001,1,'2026-10-02 13:25:47.176360','seeded',NULL,'Prove the high water on the tide-slate is not yours before the court sits on it and the post decides.','open',1000000001,'The Altered Figure','2026-10-02 13:25:47.176360');
CREATE TABLE IF NOT EXISTS "races" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "created_at" datetime(6) NOT NULL, "description" text NOT NULL, "monstrous" boolean DEFAULT FALSE NOT NULL, "name" varchar NOT NULL, "universe_id" integer NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_c25ac61605"
FOREIGN KEY ("universe_id")
  REFERENCES "universes" ("id")
);
INSERT INTO races VALUES(1000000001,'2026-10-02 13:25:46.812166','One family to a coast, sworn at seven and never released. They are bred and trained to read water, and they do it the way other people hear music: a Keeper standing on the causeway can tell you the minute of the turn without a table, and cannot stop doing it. They are tall, weathered early, deaf in the ear that faces the bell, and constitutionally unable to give a number they are not sure of, which makes them useless liars and excellent witnesses.',0,'Bell-Keepers',1000000001,'2026-10-02 13:25:46.812166');
INSERT INTO races VALUES(1000000002,'2026-10-02 13:25:46.816100','Not a bloodline but effectively a caste: appointed for life, moved every season, and forbidden to hear the same county twice. Decades of never staying leave them precise, courteous and profoundly disconnected -- a Justicar knows the law of forty parishes and the name of nobody in any of them. They are the only people on the coast who cannot tell you when the water turns.',0,'Circuit Justicars',1000000001,'2026-10-02 13:25:46.816100');
INSERT INTO races VALUES(1000000003,'2026-10-02 13:25:46.827488','Graziers and dyke-menders from the inland levels, drier and stiffer than the shore in every sense. They keep records, keep grudges, and supply most of the county''s witnesses and all of its informers. A Marshborn on the causeway is somebody who came a long way to see a thing done properly.',0,'Marshborn',1000000001,'2026-10-02 13:25:46.827488');
INSERT INTO races VALUES(1000000004,'2026-10-02 13:25:46.829620','The people of the flats: oystermen, salvagers, wreckers. Broad, salt-cured and cheerfully fatalistic, with an oral memory of every hull lost on this coast for two hundred years and no written record of anything at all. They regard the Assizes as a kind of weather that occasionally takes somebody, and they turn out for every hearing.',0,'Shorefolk',1000000001,'2026-10-02 13:25:46.829620');
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
INSERT INTO scenes VALUES(1000000001,NULL,NULL,'2026-10-02 13:25:47.418642','The chalk is still under your thumb and it is still not your handwriting. You are standing at the slate in the open court on Gallows Rock with the mile of causeway behind you already darkening at its edges, and Justicar Brace is on the bench in his gown with his hands folded, waiting to be told how long he has. Out past the low wall the flats are silvering over the way they do an hour before they stop being flats. The bell hangs on its hook at your shoulder, cold, and the figure chalked above the day''s cases says ten minutes past eight in a hand that has your slant and none of your pressure. From the post at the seaward end of the rock, faint over the surf, somebody is calling a name, and it is not yours.',NULL,0,1,1000000001,NULL,NULL,NULL,1000000001,'2026-09-01 05:20:00','The story opens in the Causeway Court with the day''s high water altered on the tide-slate by seventy minutes, the Justicar waiting on the bench for a number, and the chained man audible from the tide post.',NULL,'2026-10-02 13:25:47.418642');
CREATE TABLE IF NOT EXISTS "stories" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "created_at" datetime(6) NOT NULL, "generation_snapshot" text, "genre" varchar, "preface" text, "start_time" datetime(6), "summary" text, "title" varchar, "universe_id" integer NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_2a912ea846"
FOREIGN KEY ("universe_id")
  REFERENCES "universes" ("id")
);
INSERT INTO stories VALUES(1000000001,'2026-10-02 13:25:46.849793',NULL,'tidal legal mystery','You have rung this court up and down for nineteen years and you have never once been wrong about the water, which is why the chalk on the tide-slate is wrong in somebody else''s hand. High water is written as ten minutes past eight. High water is at seven. You wrote seven yourself, last night, on a dry board in a still room, and the figure on the slate now is not yours and is close enough to yours to pass. Out at the post, Neb Halloran has been chained since the ebb for a wrecking nobody on this coast believes he did alone, and his hearing is set for the last hour before the turn -- an hour that, on that chalk, does not exist. The Justicar is already in the court with his gown on. The bell is on its hook. Whoever moved seventy minutes has given the water time to do what the court would have had to sign for.','2026-09-01 05:20:00','The player is Coraith Vell, Bell-Keeper of the Assize causeway, who finds the day''s high water altered by seventy minutes in a hand that is almost her own. The chained man at the tide post has a hearing in an hour the false chalk has erased, so working out who changed it means reading the court''s own papers against the water: the Justicar who needs a verdict, the wreck that serves as the vestry, and the question of who benefits from a drowning the record will call a delay. The tone is cold, procedural and out of doors, and the clock is the sea.','The Salt Assizes (engine sweep)',1000000001,'2026-10-02 13:25:46.849793');
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
CREATE TABLE IF NOT EXISTS "ar_internal_metadata" ("key" varchar NOT NULL PRIMARY KEY, "value" varchar, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL);
INSERT INTO ar_internal_metadata VALUES('environment','test','2026-10-02 13:25:33.143313','2026-10-02 13:25:33.143316');
INSERT INTO ar_internal_metadata VALUES('schema_sha1','072abc2bfb80d111a1808df270e9210911f5e81d','2026-10-02 13:25:33.148224','2026-10-02 13:25:33.148228');
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
INSERT INTO sqlite_sequence VALUES('items',1000000003);
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
