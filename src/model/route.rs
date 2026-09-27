//! Where a model call goes, and the one credential it carries there.
//!
//! Every call the engine makes is an OpenRouter request: the chat
//! completions (narration, the structured calls, realization) and the
//! System One decisions. A player reaches OpenRouter one of two ways, with
//! their own key, or through a relay the owner runs that holds his key under
//! a monthly cap per player. The relay mirrors OpenRouter's paths under its
//! own base, so a request body is the same on both routes and only the base
//! and the bearer differ.
//!
//! A [`Secret`] is never printed: its `Debug` says only that it is there, it
//! has no `Display`, and nothing in this module puts it anywhere but the
//! `Authorization` header of the one host its route names.

/// OpenRouter itself, where a player's own key is sent and nowhere else.
pub const OPENROUTER: &str = "https://openrouter.ai";

/// The chat completions path, on OpenRouter and under a relay's base alike.
pub const CHAT_COMPLETIONS: &str = "/api/v1/chat/completions";

/// OpenRouter's System One path (`SystemOneAgent::OPENROUTER_ENDPOINT`).
pub const DECISIONS: &str = "/api/alpha/decisions";

/// A key or a token. It can be read only inside this crate, where it becomes
/// a request header.
#[derive(Clone, PartialEq, Eq)]
pub struct Secret(String);

impl Secret {
    pub fn new(value: impl Into<String>) -> Secret {
        Secret(value.into())
    }

    pub(crate) fn expose(&self) -> &str {
        &self.0
    }

    pub fn is_blank(&self) -> bool {
        self.0.trim().is_empty()
    }
}

impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Secret(set)")
    }
}

/// How this engine reaches a model, if it does at all.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Route {
    /// Straight to OpenRouter with the player's own key.
    Direct { key: Secret },
    /// Through the owner's relay: `base_url` is the relay's OpenRouter mirror
    /// (`https://<relay>/relay/openrouter`), `token` the player's invitation.
    Relay { base_url: String, token: Secret },
    /// No model access: the engine plays its offline path only.
    None,
}

/// Which route a player asked for (`model_route` in the terminal's
/// configuration): `Auto` takes their own key before a relay invitation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Preference {
    #[default]
    Auto,
    Own,
    Relay,
}

/// One request's destination: the whole URL and the bearer sent with it.
#[derive(Debug, PartialEq, Eq)]
pub struct Endpoint<'r> {
    pub url: String,
    pub bearer: &'r Secret,
}

impl Route {
    /// The route a player's settings give. Their own key comes first under
    /// `Auto`: a player who set one means to pay for their own calls. There
    /// is no fallback from one route to the other: `Own` with no key, or
    /// `Relay` with no invitation, is no model access.
    pub fn pick(
        own_key: Option<Secret>,
        relay: Option<(String, Secret)>,
        preference: Preference,
    ) -> Route {
        let own = own_key
            .filter(|key| !key.is_blank())
            .map(|key| Route::Direct { key });
        let relayed = relay
            .filter(|(base, token)| !base.trim().is_empty() && !token.is_blank())
            .map(|(base_url, token)| Route::Relay { base_url, token });
        let chosen = match preference {
            Preference::Auto => own.or(relayed),
            Preference::Own => own,
            Preference::Relay => relayed,
        };
        chosen.unwrap_or(Route::None)
    }

    /// Whether any model can be asked.
    pub fn is_none(&self) -> bool {
        matches!(self, Route::None)
    }

    /// The base every path is appended to, with no trailing slash.
    pub fn base(&self) -> Option<&str> {
        match self {
            Route::Direct { .. } => Some(OPENROUTER),
            Route::Relay { base_url, .. } => Some(base_url.trim_end_matches('/')),
            Route::None => None,
        }
    }

    /// Where a request to `path` goes on this route, and what it carries.
    pub fn endpoint(&self, path: &str) -> Option<Endpoint<'_>> {
        let bearer = match self {
            Route::Direct { key } => key,
            Route::Relay { token, .. } => token,
            Route::None => return None,
        };
        Some(Endpoint {
            url: format!("{}{path}", self.base()?),
            bearer,
        })
    }

    /// What a status line says about the route.
    pub fn describe(&self) -> &'static str {
        match self {
            Route::Direct { .. } => "your OpenRouter key",
            Route::Relay { .. } => "the relay",
            Route::None => "no model access",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_secret_never_prints() {
        let route = Route::Relay {
            base_url: "https://relay.example/relay/openrouter".into(),
            token: Secret::new("token-that-must-not-print"),
        };
        assert!(!format!("{route:?}").contains("token-that-must-not-print"));
        let own = Route::Direct {
            key: Secret::new("key-that-must-not-print"),
        };
        assert!(!format!("{own:?}").contains("key-that-must-not-print"));
    }

    #[test]
    fn a_direct_request_goes_only_to_openrouter_with_the_key() {
        let key = Secret::new("own-key");
        let route = Route::Direct { key: key.clone() };
        let endpoint = route.endpoint(CHAT_COMPLETIONS).unwrap();
        assert_eq!(
            endpoint.url,
            "https://openrouter.ai/api/v1/chat/completions"
        );
        assert_eq!(endpoint.bearer, &key);
        assert_eq!(
            route.endpoint(DECISIONS).unwrap().url,
            "https://openrouter.ai/api/alpha/decisions"
        );
    }

    #[test]
    fn a_relay_request_goes_to_the_relay_with_the_token() {
        let token = Secret::new("player-token");
        let route = Route::Relay {
            base_url: "https://relay.example/relay/openrouter/".into(),
            token: token.clone(),
        };
        let endpoint = route.endpoint(CHAT_COMPLETIONS).unwrap();
        assert_eq!(
            endpoint.url,
            "https://relay.example/relay/openrouter/api/v1/chat/completions"
        );
        assert_eq!(endpoint.bearer, &token);
        assert_eq!(
            route.endpoint(DECISIONS).unwrap().url,
            "https://relay.example/relay/openrouter/api/alpha/decisions"
        );
    }

    #[test]
    fn the_players_own_key_comes_first_and_nothing_falls_back() {
        let key = || Some(Secret::new("own"));
        let relay = || {
            Some((
                "https://relay.example/relay/openrouter".to_string(),
                Secret::new("t"),
            ))
        };
        assert!(matches!(
            Route::pick(key(), relay(), Preference::Auto),
            Route::Direct { .. }
        ));
        assert!(matches!(
            Route::pick(None, relay(), Preference::Auto),
            Route::Relay { .. }
        ));
        assert!(matches!(
            Route::pick(key(), relay(), Preference::Relay),
            Route::Relay { .. }
        ));
        assert_eq!(Route::pick(None, relay(), Preference::Own), Route::None);
        assert_eq!(Route::pick(key(), None, Preference::Relay), Route::None);
        assert_eq!(
            Route::pick(Some(Secret::new("  ")), None, Preference::Auto),
            Route::None
        );
        assert!(Route::None.endpoint(CHAT_COMPLETIONS).is_none());
    }
}
