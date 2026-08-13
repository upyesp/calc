use calc_i18n::Localizer;

#[test]
fn lookup_uses_localized_catalog() {
    let l = Localizer::resolve(Some("fr"), &[]);
    assert_eq!(l.locale(), "fr");
    assert_eq!(l.lookup("app-name"), "Calculatrice");
}

#[test]
fn missing_key_falls_back_to_english_then_to_key() {
    let l = Localizer::resolve(Some("fr"), &[]);
    // "prompt" exists only in the English catalog → fallback
    assert_eq!(l.lookup("prompt"), "calc>");
    // unknown key → the key itself
    assert_eq!(l.lookup("no-such-key"), "no-such-key");
}

/// Fluent wraps interpolated values in bidi isolating characters (U+2068/69)
/// to prevent RTL text injection — a feature we keep. Strip them to assert on
/// the visible text.
fn strip_isolates(s: &str) -> String {
    s.chars()
        .filter(|c| *c != '\u{2068}' && *c != '\u{2069}')
        .collect()
}

#[test]
fn placeholders_are_filled() {
    let l = Localizer::resolve(Some("en"), &[]);
    assert_eq!(
        strip_isolates(&l.lookup_args("saved", &[("name", "fib")])),
        "saved fib"
    );
    assert_eq!(
        strip_isolates(&l.lookup_args("no-definition", &[("name", "nope")])),
        "no definition for nope in this session"
    );
}

#[test]
fn detected_locales_negotiate_with_region_matching() {
    let l = Localizer::resolve(None, &["fr-FR".to_string(), "en".to_string()]);
    assert_eq!(l.locale(), "fr");
    assert_eq!(l.lookup("app-name"), "Calculatrice");
}

#[test]
fn unsupported_detected_locale_falls_back_to_english() {
    let l = Localizer::resolve(None, &["de".to_string()]);
    assert_eq!(l.locale(), "en");
}

#[test]
fn explicit_preference_wins_over_detection() {
    let l = Localizer::resolve(Some("ar"), &["en".to_string()]);
    assert_eq!(l.locale(), "ar");
}

#[test]
fn every_supported_locale_loads() {
    for locale in calc_i18n::SUPPORTED_LOCALES {
        let l = Localizer::resolve(Some(locale), &[]);
        assert_eq!(l.locale(), *locale);
        assert_eq!(l.lookup("app-name"), l.lookup("app-name")); // loads without panic
    }
}
