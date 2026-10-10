PRAGMA foreign_keys=OFF;
BEGIN TRANSACTION;
CREATE TABLE IF NOT EXISTS "characters_scenes" ("character_id" integer NOT NULL, "scene_id" integer NOT NULL);
INSERT INTO characters_scenes VALUES(1000000001,1000000001);
INSERT INTO characters_scenes VALUES(1000000003,1000000001);
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
INSERT INTO universes VALUES(1000000001,'One city, one institution, and a functional caste order underneath it. The Registry employs one adult in six and files the rest. Above the clerks sit the Inspectorate, who audit; above them the Sealed Room, which nobody in this story has seen. Below everyone are the Marginalia, whose records were damaged or amended and who consequently take the work that ought not be remembered. Out past the levee the Unwritten trade fish, salt and salvage into a city that will not put their names on a receipt. Everyone in Ambry is polite, and everyone counts the days since their file was last touched.','2026-10-02 13:25:39.527963','Wages are paid in coin, but the real currency is standing in the record. A registered trade licence, a filed apprenticeship, a marriage entered in the ward book: these are the things people spend a life acquiring, because they are the things that hold. Copying is the largest industry, salt and fish the largest imports, and the Registry''s stationery budget is the single biggest line in the city accounts. Unrecorded work is cheap, quick, and done by the Marginalia, who charge more than their reputation suggests because they know exactly what they are worth and nobody can remember arguing with them.','Ambry sits in a wide river delta that floods twice a year. The Registry occupies the whole of the high ground: eleven storeys of ward offices, copy rooms and stacks, built in rings around a courtyard nobody has permission to cross. Below it the Middle Wards spread out in numbered blocks, each one a warren of tenements and copying shops. Beyond the levee are the flood plains, unsurveyed and therefore unrecorded, where the Unwritten live in houses that move with the water. The Registry''s own basements run four floors below the flood line and are dry, which nobody has ever satisfactorily explained.','The Registry began four hundred years ago as a flood survey -- somebody had to write down which fields were underwater and whose they were. When the second survey found the unrecorded fields had drifted from the recorded ones, the survey stopped being about land. The Registry has never once been overthrown, though it has twice been reorganized so thoroughly that the previous version is unrecoverable. Eighty years ago the Great Amendment closed nine thousand files in a single winter for reasons the surviving minutes describe as procedural. The Marginalia date themselves from it.','Standard physical laws hold, with one exception: in the city of Ambry and the wards around it, a thing that is not written down stops being reliably itself. The effect is slow and dull rather than dramatic. An unrecorded corridor will, over a season, start to run a few feet short. An unregistered tool loses its temper and its edge. A person whose file is closed is not struck down; they are simply harder to remember, then harder to see, then gone from the accounts of everyone who knew them, until the only evidence they existed is what somebody wrote. Nothing about this is reversible by force. It is reversible by paperwork, which is why the Registry rules here.','The Inspectorate audits the ward offices, the ward offices resent the Inspectorate, and both fear the Sealed Room, which communicates by writ and never in person. Power in Ambry is the ability to make a document exist or stop existing, so the political struggle is entirely archival: misfiling, backdating, quiet amendment, and the occasional fire in a stack room. The Unwritten have no political existence at all, which turns out to make them the only people in the delta the Registry cannot touch. Several factions inside the Registry are extremely interested in why.','The city''s faith is called the Fair Copy, and it holds that the world is a document being transcribed, that the transcription contains errors, and that a life lived legibly is the only way to be carried into the next edition. Its clergy are archivists; its sacrament is having your name read aloud. The Marginalia keep a quieter heresy: that the errors are the only honest part of the document. The Unwritten find the whole argument funny, and say so, which is one more reason nobody in the city writes down what they say.','Roughly late nineteenth century, and skewed hard toward anything that copies or preserves. Ambry has pneumatic tube post between every ward office, carbon paper, wax cylinders, a functioning telegraph and no electricity in the residential wards. Presses, seals, indelible inks and lamp-lit copy rooms are the industry the city actually runs on. There are no engines to speak of outside the pumping stations, because an engine that is not maintained on a filed schedule stops working in ways engineers find distressing rather than interesting.','2026-10-02 13:25:39.527963','Batons, seal-knives and single-shot service revolvers, all of them registered and most of them carried by people who have never drawn one. Physical violence is rarely the instrument of choice, because the Registry has a better one: a writ of closure ends a person''s file, and a closed file ends the person over the following months. Possession of an unregistered blade is a minor offence. Possession of a blank writ is not an offence at all, since officially none exist outside the Sealed Room.',NULL);
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
INSERT INTO characters VALUES(1000000001,41,'Narrow, upright, greying at the front, with ink worn permanently into the first two fingers of her right hand and spectacles she only puts on to check somebody else''s arithmetic. Her office coat is eleven years old, brushed, and mended twice at the right elbow where it meets the desk.','Fourth-generation Copyists'' Line, apprenticed at eleven, posted to Ward Office 12 at thirty and never moved because she asked not to be. She is the clerk other clerks send a queried entry to. Twice she has found an amendment in an incoming ledger that should not have been possible and twice she has filed a query about it and heard nothing back, which she has told herself is the system working slowly and has stopped believing.','Odile Vance wants to reconcile the missing hour in Ward Office 12''s daybook.','2026-10-02 13:25:39.771056',0,'keep',11,'Amendments in another hand, being told a query is still open','Signing her name to something that turns out to have closed a file','Odile Vance',8,0,0,1,3,'A ledger that balances, the smell of a fresh cylinder of wax, rain on the window',NULL,'obtain','Vance','Precise, dry, and unhurried in a way that reads as calm and is actually stubbornness. She does not raise her voice or her hopes. She will not sign something she has not read, which has cost her every promotion she was ever considered for, and she considers this a fair trade she would make again.',1000000001,'Odile Vance needs to keep every query open until its source is recorded.','female',1000000001,9,'Odile Vance wants to obtain the truth behind the impossible amendment.','Odile Vance needs to reach the sealed registry and read what it contains.','2026-10-02 13:25:39.771056',15,NULL,NULL);
INSERT INTO characters VALUES(1000000002,29,'Slight, dark-haired, with a face that will not stay in your memory for an hour after he leaves the room -- an effect people find restful and then cannot account for. He wears a copy-room apron over ordinary clothes and keeps a second pencil behind his ear he never uses.','Third generation Marginalia; his grandmother''s file was closed in the Great Amendment and the damage came down the family with the name. Hired into Ward Office 12 four years ago for the entries nobody wanted their hand on, and good at it. He sat at the desk across from Vance, drank tea she made too strong, and kept a private index of every amendment that came through the ward -- because his people keep their own archive, and because he had begun to see a pattern in whose files got touched.','Perrin Lasco wants to place his private index in the hands of a clerk whose file remains open.','2026-10-02 13:25:39.805004',1,'keep',14,'Being asked to repeat his name, doors that lock from one side','Being closed quietly, on a Tuesday, with nobody able to say what was lost','Perrin Lasco',6,0,0,0,1,'An index that catches something, other people''s strong tea, the last hour of a shift',NULL,'withhold','Perrin','Quick, funny, and careful in a way people mistake for nerves. He asks one question too many and then apologizes for it. Being forgettable has taught him to be useful fast, and to write everything down twice.',1000000003,'Perrin Lasco needs to keep two copies of every amendment.','male',1000000001,10,'Perrin Lasco wants to obtain a name that will remember him after his file closes.','Perrin Lasco needs to reach a person who will hear his own name and not use him.','2026-10-02 13:25:39.805004',8,NULL,NULL);
INSERT INTO characters VALUES(1000000003,52,'Tall, unhurried, immaculate; the sharply consistent good looks of a family nobody has ever had to remember with effort. Grey coat, Inspectorate seal on a chain, and shoes you hear coming up the hallway a long time before he arrives.','Ledger-Kept, four hundred years of unbroken record behind him, and a career built on being the man the Inspectorate sends when a ward office needs to be reminded that it is audited. He has signed eleven closures. He believes, sincerely, that each was procedurally correct, and he has stopped reading the supporting files before he signs because reading them was making the work harder to do.','Halkett Rowe wants to close Ward Office 12 with every query answered and the file clean.','2026-10-02 13:25:39.812522',0,'keep',9,'An open query, a clerk who reads the file out loud to him','That the Sealed Room has a file on him, and that it is thicker than it should be','Halkett Rowe',8,0,0,0,1,'A ward office that has nothing to hide, being met at the door, punctuality',1000000001,'obtain','Sub-Inspector Rowe','Courteous, patient, genuinely likeable, and entirely unable to imagine that the institution which made him could be the thing doing harm. He listens well. He will hear you out, agree with you, and file the writ anyway, and he will be sorry about it in a way that costs him nothing.',1000000002,'Halkett Rowe needs to keep his appointment at the next ward on time.','male',1000000001,11,'Halkett Rowe wants to obtain the Registry''s absolution for his eleven closures.','Halkett Rowe needs to reach one supporting file and read it before he signs.','2026-10-02 13:25:39.812522',13,NULL,NULL);
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
INSERT INTO items VALUES(1000000001,'light',NULL,0,'2026-10-02 13:25:39.639302','A Registry writ of closure on the heavy grey stock only the Inspectorate issues, lying in the wire basket under the pneumatic tube where it arrived without a carrier. It is ward-marked 12, sealed, and countersigned in advance; the line for the name of the file to be closed is still empty, and the countersignature is in the same careful, borrowed slant as the amendment in your daybook.','intact','WRIT OF CLOSURE — Ward 12. The file of ______________________ is closed, no query to be raised. By order of the Inspectorate. Countersigned.',1000000001,'blank closure writ',NULL,'{"registered": true, "ward": 12, "name_line": "blank"}',1,NULL,'2026-10-02 13:25:39.639302','ordinary',NULL,NULL,'sturdy','portable',NULL,NULL,NULL,NULL,NULL);
INSERT INTO items VALUES(1000000002,'immovable',NULL,0,'2026-10-02 13:25:39.656806','The ward''s copying press: a cast-iron screw press on a plinth bolted through the floorboards between the coat rail and the closet door, with a stack of damp linen sheets on the shelf beneath it. It has stood in this corner since the works of 1846 and it weighs what a small horse weighs.','intact',NULL,1000000001,'filing press',NULL,'{"registered": true, "ward": 12}',0,NULL,'2026-10-02 13:25:39.656806','ordinary',NULL,NULL,'sturdy','portable',NULL,NULL,NULL,NULL,NULL);
INSERT INTO items VALUES(1000000003,'light',NULL,0,'2026-10-02 13:25:39.670147','Her own ward stamp, lying beside the open daybook where she set it down. The ink on the pad is still wet, which is the only thing in this room that agrees with her about what time it is.','intact',NULL,1000000001,'ward stamp',NULL,'{"registered": true, "ward": 12}',0,NULL,'2026-10-02 13:25:39.670147','ordinary',NULL,NULL,'sturdy','portable',NULL,NULL,NULL,NULL,NULL);
INSERT INTO items VALUES(1000000004,'handy',NULL,0,'2026-10-02 13:25:39.697207','A slim index in a hand that is not hers: two columns, dates on the left and file numbers on the right, running to within an hour of this afternoon. Nothing in it carries a ward mark, which means no copy of it exists anywhere in the Registry.','intact',unistr('11 Frost — 0714/12 — closed, no query raised\u000a2 Thaw — 0902/12 — closed, no query raised\u000a19 Thaw — 1188/12 — QUERY RAISED (O.V.) — still open\u000aThis day, 4th hour — 1188/12 — amended. Same hand as 0714 and 0902. P.L.'),1000000002,'Perrin''s private index',NULL,'{"registered": false, "columns": 2}',1,NULL,'2026-10-02 13:25:39.697207','ordinary',NULL,NULL,'sturdy','portable',NULL,NULL,NULL,NULL,NULL);
INSERT INTO items VALUES(1000000005,'light',NULL,0,'2026-10-02 13:25:39.720221','A copy-room apron folded to the size of a book and pushed to the back of the lowest shelf, behind a stack of blank ward forms squared up too neatly to be accidental. It has been folded and refolded by somebody who did it in the dark.','intact',NULL,1000000002,'copy-room apron',NULL,'{"registered": false}',0,NULL,'2026-10-02 13:25:39.720221','ordinary',NULL,NULL,'sturdy','portable',NULL,NULL,NULL,NULL,NULL);
INSERT INTO items VALUES(1000000006,'handy',1000000001,0,'2026-10-02 13:25:39.801033','A quarter-bound ledger, eleven years of her own handwriting, with a ruled and totalled gap between four and five o''clock this afternoon that she did not write.','intact',unistr('3.40 — Levee boundary, Marchmain acre 7. Amendment received, ward-marked, entered. O.V.\u000a4.00 —\u000a5.00 —\u000a5.15 — Query 1188 (Marchmain acre 7) still open. Second application made. O.V.'),NULL,'Ward Office 12 daybook',NULL,'{"pages": 340, "sealed": false, "registered": true}',1,NULL,'2026-10-02 13:25:39.801033','ordinary',NULL,NULL,'sturdy','portable',NULL,NULL,NULL,NULL,NULL);
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
INSERT INTO location_connections VALUES(1000000001,'open',1000000002,'2026-10-02 13:25:39.852200','adjacent',NULL,NULL,NULL,1000000001,'about a minute','walking','2026-10-02 13:25:39.852200');
INSERT INTO location_connections VALUES(1000000002,'open',1000000001,'2026-10-02 13:25:39.855002','adjacent',NULL,NULL,NULL,1000000002,'about a minute','walking','2026-10-02 13:25:39.855002');
INSERT INTO location_connections VALUES(1000000003,'open',1000000003,'2026-10-02 13:25:39.856581','adjacent',NULL,NULL,NULL,1000000001,'about a minute','walking','2026-10-02 13:25:39.856581');
INSERT INTO location_connections VALUES(1000000004,'open',1000000001,'2026-10-02 13:25:39.857881','adjacent',NULL,NULL,NULL,1000000003,'about a minute','walking','2026-10-02 13:25:39.857881');
CREATE TABLE IF NOT EXISTS "locations" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "created_at" datetime(6) NOT NULL, "danger" varchar DEFAULT 'safe' NOT NULL, "depth" integer, "description" text, "detail_level" varchar DEFAULT 'stub' NOT NULL, "generation_checkpoint" json, "hazard" varchar, "hazard_die" integer, "last_protagonist_visit" datetime(6), "lore" text, "mobile" boolean DEFAULT FALSE NOT NULL, "name" varchar, "parent_location_id" integer, "population" varchar, "story_id" integer NOT NULL, "teaser" text, "updated_at" datetime(6) NOT NULL, "width" integer, "x" integer, "y" integer, "z" integer, "surface" varchar, "kind" varchar, "density" varchar, CONSTRAINT "fk_rails_5bc98acf09"
FOREIGN KEY ("parent_location_id")
  REFERENCES "locations" ("id")
, CONSTRAINT "fk_rails_fedd9b21a0"
FOREIGN KEY ("story_id")
  REFERENCES "stories" ("id")
);
INSERT INTO locations VALUES(1000000001,'2026-10-02 13:25:39.591205','safe',NULL,unistr('You are standing at your own desk on the second floor of the Registry''s east ring, in a room built for four clerks and staffed, lately, by two. The gas mantle above the desks is turned low and hissing. Rain comes off the levee and goes sideways past the window, and the glass is cold enough to feel from here. Your daybook lies open under your hand with the stamp beside it, and the ruled gap between four and five o''clock sits in the middle of the page looking exactly as legitimate as every other line on it.\u000a\u000aThe desk across from yours has been cleared to the wood -- no blotter, no cylinder, no apron over the chair back -- and the chair is squared to the desk the way nobody ever leaves a chair they intend to come back to. On the wall between the windows, the pneumatic tube gapes open with no carrier in it. The door to the long hallway stands ajar on the darkening corridor, and beside the filing press, half-hidden by the coat rail, is the narrow door of the supply closet: no inventory number, no audit slip, no lock. Somewhere down the hallway a door closes, nearer than the last one.'),'realized',NULL,NULL,NULL,NULL,unistr('Ward Office 12 handles amendments for the flood-plain boundary: which unsurveyed acre the levee has moved past, and whose name goes against it. It is dull, unpopular work, which is why the office has never been reorganized and why its ledgers go back further, unbroken, than almost any other ward''s. Clerks are posted here as a slight and stay for decades.\u000a\u000aThe office is also, on paper, four clerks. The two empty desks have been carried on the establishment for nineteen years, filled and vacated a handful of times, and no query about them has ever come back answered. Vance keeps the ledgers for all four positions out of habit. It is the kind of small, unexamined inaccuracy the Registry runs on -- and the reason a person could be removed from this room without anything in the record needing to change.'),0,'Ward Office 12',NULL,NULL,1000000001,'Two desks, one hour missing, and the Inspectorate due at seven.','2026-10-02 13:25:39.591205',NULL,NULL,NULL,NULL,NULL,NULL,NULL);
INSERT INTO locations VALUES(1000000002,'2026-10-02 13:25:39.681280','safe',NULL,unistr('You pull the narrow door and step in, and the office light follows you as far as the second shelf. It is a closet: three feet by six, shelved on both sides to the ceiling, smelling of wax, dust, and the particular sourness of old ink. Reams of ward paper, boxes of nibs, a coil of tube-carrier cord, four unopened tins of sealing wax. The floor is swept. Somebody has swept this floor.\u000a\u000aOn the lowest shelf, behind a stack of blank ward forms squared up too neatly to be accidental, is a copy-room apron folded to the size of a book, and inside it a slim private index in a hand that is not yours -- entries in two columns, dates on the left and file numbers on the right, running to within an hour of this afternoon. There is no other door. There is no window. The wall at the back is plaster over brick and the shelves are screwed to it, and the only way out of here is the doorway you came through, which frames your own office and the gas mantle hissing over two desks.'),'realized',NULL,NULL,NULL,NULL,unistr('The closet has no inventory number because it was shelved out in 1846 as part of the works that put the filing press in, and the works order closed before the fittings schedule was raised. That is the whole of the mystery: a paperwork sequence that ran in the wrong order eighty years ago. The consequence is that no auditor has ever had a document that says this space exists, and so no auditor has ever opened the door.\u000a\u000aPerrin Lasco worked that out in his first month, the way Marginalia work such things out, and used it for four years. It is the only square yard of the Registry''s east ring that is, in the only sense that matters in Ambry, unrecorded -- which is why what he hid is still here, and why he is not.'),0,'The Supply Closet',NULL,NULL,1000000001,'No inventory number, no audit slip, and no lock.','2026-10-02 13:25:39.681280',NULL,NULL,NULL,NULL,NULL,NULL,NULL);
INSERT INTO locations VALUES(1000000003,'2026-10-02 13:25:39.726866','safe',NULL,NULL,'stub',NULL,NULL,NULL,NULL,NULL,0,'The Long Hallway',NULL,NULL,1000000001,'Doors closing one after another, and shoes you can already hear.','2026-10-02 13:25:39.726866',NULL,NULL,NULL,NULL,NULL,NULL,NULL);
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
CREATE TABLE IF NOT EXISTS "playthrough_paragraphs" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "audit" json, "author" varchar DEFAULT 'player' NOT NULL, "created_at" datetime(6) NOT NULL, "playthrough_id" integer NOT NULL, "scene_id" integer NOT NULL, "text" text NOT NULL, "updated_at" datetime(6) NOT NULL, "consent" varchar DEFAULT 'none' NOT NULL, "requests" json, "prompt_digest" varchar, CONSTRAINT "fk_rails_db3983d32c"
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
INSERT INTO quest_outcomes VALUES(1000000001,NULL,NULL,'2026-10-02 13:25:40.006073',1,NULL,'query-answered',1000000001,NULL,NULL,NULL,'With Perrin''s index and the unsigned writ side by side on your desk, the missing hour has a source at last -- the same hand closed 0714, 0902 and Perrin Lasco. Query 1188 stays open in your name, and this time the record says why.','2026-10-02 13:25:40.006073');
INSERT INTO quest_outcomes VALUES(1000000002,NULL,'out_of_order','2026-10-02 13:25:40.009948',0,NULL,'closed-blind',1000000001,NULL,NULL,NULL,'You had the writ before you had any reason for it, and a clerk holding a blank closure with no source to set against it is a clerk explaining herself. By the time the index was in your hand, Rowe had written the hour up as your error, and Query 1188 was closed on your own report.','2026-10-02 13:25:40.009948');
CREATE TABLE IF NOT EXISTS "quest_steps" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "bound_at" datetime(6), "created_at" datetime(6) NOT NULL, "minutes" integer, "position" integer NOT NULL, "quest_id" integer NOT NULL, "summary" text NOT NULL, "target_id" integer, "target_name" varchar, "target_type" varchar, "teaser" text, "trigger_kind" varchar NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_ba1603d17b"
FOREIGN KEY ("quest_id")
  REFERENCES "quests" ("id")
);
INSERT INTO quest_steps VALUES(1000000001,'2026-08-31 18:40:00','2026-10-02 13:25:39.902070',NULL,1,1000000001,'Open the one door in this office nobody has audited in eleven years.',1000000002,'The Supply Closet','Location','Whatever filled the missing hour was put where nobody counts.','reach_location','2026-10-02 13:25:39.928274');
INSERT INTO quest_steps VALUES(1000000002,'2026-08-31 18:40:00','2026-10-02 13:25:39.935005',NULL,2,1000000001,'Take the private index that names the hand behind the amendments.',1000000004,'Perrin''s private index','Item',NULL,'hold_item','2026-10-02 13:25:39.945720');
INSERT INTO quest_steps VALUES(1000000003,'2026-08-31 18:40:00','2026-10-02 13:25:39.947629',NULL,3,1000000001,'Take the blank closure writ out of the tube basket before anybody fills in its name.',1000000001,'blank closure writ','Item',NULL,'hold_item','2026-10-02 13:25:39.954303');
CREATE TABLE IF NOT EXISTS "quests" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "contributes" boolean DEFAULT TRUE NOT NULL, "created_at" datetime(6) NOT NULL, "origin" varchar DEFAULT 'seeded' NOT NULL, "parent_quest_id" integer, "premise" text NOT NULL, "status" varchar DEFAULT 'open' NOT NULL, "story_id" integer NOT NULL, "title" varchar NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_a64954ae79"
FOREIGN KEY ("parent_quest_id")
  REFERENCES "quests" ("id")
, CONSTRAINT "fk_rails_fa77fc496b"
FOREIGN KEY ("story_id")
  REFERENCES "stories" ("id")
);
INSERT INTO quests VALUES(1000000001,1,'2026-10-02 13:25:39.875797','seeded',NULL,'Find out who cut an hour out of Ward Office 12''s daybook and what it was spent on, before the Inspectorate closes the ward.','open',1000000001,'Query 1188','2026-10-02 13:25:39.875797');
CREATE TABLE IF NOT EXISTS "races" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "created_at" datetime(6) NOT NULL, "description" text NOT NULL, "monstrous" boolean DEFAULT FALSE NOT NULL, "name" varchar NOT NULL, "universe_id" integer NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_c25ac61605"
FOREIGN KEY ("universe_id")
  REFERENCES "universes" ("id")
);
INSERT INTO races VALUES(1000000001,'2026-10-02 13:25:39.533109','Nine generations of clerks bred, schooled and married into the Registry''s copy rooms. They are unremarkable to look at and extraordinary to watch work: a steady hand, an eye that catches an altered digit at arm''s length, and a memory for filing systems that arrives before they can read. They are the most thoroughly recorded people in Ambry and the least able to leave, since a Copyist outside the Registry has a trade nobody else can use.',0,'Copyists'' Line',1000000001,'2026-10-02 13:25:39.533109');
INSERT INTO races VALUES(1000000002,'2026-10-02 13:25:39.536660','The old registered families, continuously recorded for four hundred years without a single closed file. Continuity has left them sharply and consistently themselves -- their portraits at twenty look like them at sixty, and a Ledger-Kept in a crowd is the one your eye keeps finding. They hold the Inspectorate, most of the Registry''s senior posts, and an unexamined conviction that their advantages are personal rather than clerical.',0,'Ledger-Kept',1000000001,'2026-10-02 13:25:39.536660');
INSERT INTO races VALUES(1000000003,'2026-10-02 13:25:39.547348','People whose records were damaged, amended or partially closed, usually in the Great Amendment and usually not on purpose. They are hard to look at directly and harder to recall afterwards; a stranger will forget a Marginalia''s face inside an hour and their name inside a day. They take the work the city needs done and does not want remembered, and they keep their own private archive, which is said to be better than the Registry''s.',0,'Marginalia',1000000001,'2026-10-02 13:25:39.547348');
INSERT INTO races VALUES(1000000004,'2026-10-02 13:25:39.549069','Born past the levee, in the flood plains that were never surveyed, and therefore never entered in any book. Because they were never recorded they cannot be unrecorded, and they do not blur or fade the way city people do when a file closes. They are weathered, loud, and cheerfully unimpressed by paperwork. The Registry categorizes them as an absence rather than a people, and cannot decide whether that is a loophole or a threat.',0,'Unwritten',1000000001,'2026-10-02 13:25:39.549069');
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
INSERT INTO scenes VALUES(1000000001,NULL,NULL,'2026-10-02 13:25:40.103705','The gap in your daybook is still under your hand when the sound in the hallway stops being a sound and becomes a person. Sub-Inspector Rowe fills the doorway with the dark corridor behind him -- grey coat, Inspectorate seal on its chain, forty minutes early and entirely unhurried about being it. He looks at your desk, and at the stamp lying beside the daybook, and then for rather longer at the desk across from yours, cleared to the wood. Nothing in his face changes, which is worse than if something had. The mantle hisses over the two of you, the rain goes sideways past the glass, and behind you, close enough to touch, the narrow door of the supply closet stands exactly as unlocked as it has stood for eleven years.',NULL,0,1,1000000001,NULL,NULL,NULL,1000000001,'2026-08-31 18:40:00','The story opens in Ward Office 12 with Sub-Inspector Rowe arriving forty minutes early, the missing hour still open on the daybook, and the uninventoried supply closet at the player''s back.',NULL,'2026-10-02 13:25:40.103705');
CREATE TABLE IF NOT EXISTS "stories" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "created_at" datetime(6) NOT NULL, "generation_snapshot" text, "genre" varchar, "preface" text, "start_time" datetime(6), "summary" text, "title" varchar, "universe_id" integer NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_2a912ea846"
FOREIGN KEY ("universe_id")
  REFERENCES "universes" ("id")
);
INSERT INTO stories VALUES(1000000001,'2026-10-02 13:25:39.568268',NULL,'bureaucratic mystery','The stamp is still in your hand and the ink is still wet, and there is an hour missing out of the middle of your own daybook. Not a blank line: a clean, ruled gap between four and five o''clock, in your handwriting, with the running total carried straight across it as though nothing needed to be entered at all. You have worked Ward Office 12 for eleven years and you have never carried a total across a gap in your life. Rain is coming off the levee and going sideways past the window. Somewhere down the long hallway a door closes, and then another, nearer -- the Inspectorate does its rounds at seven and it is twenty past six. Perrin''s desk across the room has been cleared to the wood, and you cannot remember when, or by whom, or -- and this is the part that has your hands cold -- whether there was ever anybody sitting at it. There is one thing in this office nobody has audited in eleven years, and its door is behind you.','2026-08-31 18:40:00','The player is a ward clerk in Ambry who finds an hour cut out of their own daybook and the desk opposite theirs cleared of a colleague they can no longer quite remember. Working out what happened in that hour means going through the Registry''s own paperwork against it: the supply closet nobody has inventoried, the long hallway with the Inspectorate coming up it, and eventually the question of who has blank writs and why one was spent on a Marginalia clerk in Ward Office 12. The tone is dry, close and procedural, and the horror is entirely administrative.','The Unrecorded Hour (engine sweep)',1000000001,'2026-10-02 13:25:39.568268');
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
INSERT INTO sqlite_sequence VALUES('items',1000000006);
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
