//! Texts of the Rust core (tray menu, notifications) from the same Fluent catalogs
//! as the interface. English is the base and always ends the fallback chain.

use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource};
use unic_langid::LanguageIdentifier;

include!(concat!(env!("OUT_DIR"), "/locales.rs"));

const FALLBACK: &str = "en";

pub struct Texts {
    chain: Vec<FluentBundle<FluentResource>>,
}

fn bundle(locale: &str) -> Option<FluentBundle<FluentResource>> {
    let id: LanguageIdentifier = locale.parse().ok()?;
    let mut bundle = FluentBundle::new_concurrent(vec![id]);
    // Tray menus and notifications are plain text: no bidi isolation marks.
    bundle.set_use_isolating(false);
    let mut any = false;
    for (_, source) in CATALOGS.iter().filter(|(name, _)| *name == locale) {
        if let Ok(resource) = FluentResource::try_new((*source).to_owned()) {
            // Duplicate IDs keep the first definition; the interface checks catalogs.
            let _ = bundle.add_resource(resource);
            any = true;
        }
    }
    any.then_some(bundle)
}

/// "de-DE" or "de_DE.UTF-8" → "de", if a catalog for it exists.
fn available(requested: &str) -> Option<&'static str> {
    let language = requested
        .split(['-', '_', '.'])
        .next()?
        .to_ascii_lowercase();
    CATALOGS
        .iter()
        .map(|(locale, _)| *locale)
        .find(|locale| *locale == language)
}

impl Texts {
    /// `language` is the user's choice from the settings; `None` follows the system.
    pub fn new(language: Option<&str>) -> Self {
        let requested = language
            .map(str::to_owned)
            .or_else(sys_locale::get_locale)
            .unwrap_or_else(|| FALLBACK.to_owned());
        let mut locales = Vec::new();
        if let Some(locale) = available(&requested) {
            locales.push(locale);
        }
        if !locales.contains(&FALLBACK) {
            locales.push(FALLBACK);
        }
        Self {
            chain: locales.into_iter().filter_map(bundle).collect(),
        }
    }

    /// The message `id` in the first locale that has it; the ID itself otherwise.
    pub fn get(&self, id: &str, args: Option<&FluentArgs>) -> String {
        for bundle in &self.chain {
            if let Some(pattern) = bundle.get_message(id).and_then(|message| message.value()) {
                let mut errors = Vec::new();
                return bundle
                    .format_pattern(pattern, args, &mut errors)
                    .into_owned();
            }
        }
        id.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_the_language_and_falls_back_to_english() {
        let german = Texts::new(Some("de-DE"));
        assert_eq!(german.get("power-reboot", None), "Neu starten");
        let unknown = Texts::new(Some("xx"));
        assert_eq!(unknown.get("power-reboot", None), "Restart");
        assert_eq!(unknown.get("no-such-message", None), "no-such-message");
    }

    #[test]
    fn formats_arguments_without_isolation_marks() {
        let mut args = FluentArgs::new();
        args.set("number", 2);
        assert_eq!(
            Texts::new(Some("en")).get("volume-name", Some(&args)),
            "Volume 2"
        );
    }
}
