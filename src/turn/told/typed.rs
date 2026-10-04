//! `Playthrough::Volition::SystemOne` inside a turn: one typed request about
//! the people a volition slot decides for, asked through the turn's models
//! before the slot's rows are written, so no transaction is held open while
//! it waits. The die decides whatever the answer does not, and every way the
//! call fails ends at the die ([`volition::Judgment::Failed`]).

use super::Turn;
use crate::model::Book;
use crate::records::Row;
use crate::volition::{self, Asked, Judgment};

impl Turn<'_, '_> {
    /// System One's judgment of `asked` in `location`, or none when System
    /// One is off or nobody is asked anything.
    pub(super) fn judged(&mut self, asked: &[Asked], location: &Row) -> Option<Judgment> {
        if asked.is_empty() || !self.models.system_one() {
            return None;
        }
        let request =
            volition::typed_request(&self.m.game(), asked, location, self.line.as_deref());
        let questions = &request["questions"];
        if questions.as_object().is_none_or(|map| map.is_empty()) {
            return None;
        }
        let filed = self.filed("volition");
        let mut book = Book {
            store: self.m.store,
            records: &mut self.m.records,
        };
        let reply = self
            .models
            .ask_questions(&mut book, &filed, &request["state"], questions);
        Some(volition::judge(asked, questions, reply.as_ref()))
    }

    /// The typed judgment of what the people the speech die lets speak in
    /// the room the player stands in say, before the step writes it.
    pub(super) fn judged_speech(&mut self, addressee: Option<i64>) -> Option<Judgment> {
        let here = self.m.here()?;
        let asked: Vec<Asked> = self
            .m
            .speakers(Some(&here), addressee)
            .into_iter()
            .map(|(who, _)| {
                let speech = volition::speech_options(&self.m.game(), &who, &here);
                Asked {
                    character: who,
                    acts: None,
                    speech: Some(speech),
                }
            })
            .collect();
        self.judged(&asked, &here)
    }

    /// The typed judgment of what the people in the room the turn began in
    /// do, before the volition step writes it.
    pub(super) fn judged_acts(&mut self, from: Option<&Row>) -> Option<Judgment> {
        let from = from?;
        let asked: Vec<Asked> = self
            .m
            .volition_cast(Some(from))
            .into_iter()
            .map(|who| {
                let acts = volition::choices(&self.m.game(), &who, from);
                Asked {
                    character: who,
                    acts: Some(acts),
                    speech: None,
                }
            })
            .collect();
        self.judged(&asked, from)
    }

    /// The typed judgment of how the people in the room the party walks
    /// into react to it, before the reactions step writes them.
    pub(super) fn judged_reactions(&mut self, destination: &Row) -> Option<Judgment> {
        let asked = self.m.arrival_asked(destination);
        self.judged(&asked, destination)
    }
}
