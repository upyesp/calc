//! calc-i18n — shared UI localization (ADR-0008).
//!
//! Fluent catalogs are embedded at build time; locale negotiation picks the best
//! match for the device's languages with English as the always-complete fallback.
//! The scripting language is never localized (ADR-0007).

/// Locales shipped in v1 (ADR-0008): the five most widely spoken languages plus
/// Arabic for right-to-left support.
pub const SUPPORTED_LOCALES: &[&str] = &["en", "zh-CN", "hi", "es", "fr", "ar"];

/// The default and always-complete fallback locale.
pub const DEFAULT_LOCALE: &str = "en";
