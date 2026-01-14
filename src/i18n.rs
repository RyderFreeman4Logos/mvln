//! Internationalization support using Fluent.
//!
//! This module provides localized message retrieval using the Fluent localization framework.
//! Messages are loaded from embedded `.ftl` files in the `i18n/` directory at compile time.
//!
//! # Supported Locales
//!
//! - `en-US`: English (United States) - Default fallback
//! - `zh-CN`: Simplified Chinese
//!
//! # Examples
//!
//! ```no_run
//! use mvln::i18n;
//! use fluent::FluentArgs;
//!
//! let bundle = i18n::init();
//!
//! // Simple message without arguments
//! let msg = i18n::simple_msg(&bundle, "op-dry-run");
//! println!("{}", msg);
//!
//! // Message with arguments
//! let mut args = FluentArgs::new();
//! args.set("src", "file.txt");
//! args.set("dest", "/backup/file.txt");
//! let msg = i18n::msg(&bundle, "op-moving", Some(&args));
//! println!("{}", msg);
//! ```

use fluent::{FluentArgs, FluentBundle, FluentResource};
use fluent_langneg::{negotiate_languages, NegotiationStrategy};
use unic_langid::{langid, LanguageIdentifier};

// Re-export fluent-langneg's LanguageIdentifier for compatibility
use fluent_langneg::LanguageIdentifier as NegLangId;

/// English (US) locale - default fallback.
static EN_US: LanguageIdentifier = langid!("en-US");

/// English translations (embedded at compile time).
const EN_US_FTL: &str = include_str!("../i18n/en-US/main.ftl");

/// Simplified Chinese translations (embedded at compile time).
const ZH_CN_FTL: &str = include_str!("../i18n/zh-CN/main.ftl");

/// Initialize internationalization with system locale detection.
///
/// This function:
/// 1. Detects the system locale using [`sys_locale`]
/// 2. Negotiates the best matching locale from available translations
/// 3. Loads the appropriate `.ftl` resource
/// 4. Falls back to `en-US` if the system locale is not supported
///
/// # Returns
///
/// A [`FluentBundle`] configured with the negotiated locale and loaded messages.
///
/// # Panics
///
/// Panics if the embedded FTL resources are invalid or if locale parsing fails
/// for hardcoded locale strings. This should never happen with valid embedded
/// resources.
///
/// # Examples
///
/// ```no_run
/// let bundle = mvln::i18n::init();
/// ```
#[must_use]
pub fn init() -> FluentBundle<FluentResource> {
    // Detect system locale
    let system_locale = sys_locale::get_locale()
        .and_then(|locale_str| locale_str.parse::<LanguageIdentifier>().ok())
        .unwrap_or_else(|| EN_US.clone());

    // Convert unic-langid to fluent-langneg format
    let system_locale_str = system_locale.to_string();
    let en_us_neg: NegLangId = "en-US".parse().expect("en-US locale is always valid");

    let requested_neg: Vec<NegLangId> = vec![system_locale_str
        .parse()
        .unwrap_or_else(|_| en_us_neg.clone())];

    let zh_cn_neg: NegLangId = "zh-CN".parse().expect("zh-CN locale is always valid");
    let available_neg = vec![en_us_neg.clone(), zh_cn_neg.clone()];

    // Negotiate best matching locale
    let negotiated = negotiate_languages(
        &requested_neg,
        &available_neg,
        Some(&en_us_neg),
        NegotiationStrategy::Filtering,
    );

    // Use first negotiated locale or fallback to en-US
    let selected_locale_str = negotiated
        .first()
        .map_or_else(|| "en-US".to_string(), std::string::ToString::to_string);

    // Convert back to unic-langid for FluentBundle
    let selected_locale: LanguageIdentifier = selected_locale_str
        .parse()
        .unwrap_or_else(|_| EN_US.clone());

    // Load appropriate FTL resource
    let zh_cn_locale: LanguageIdentifier = "zh-CN".parse().expect("zh-CN locale is always valid");
    let ftl_source = if selected_locale == zh_cn_locale {
        ZH_CN_FTL
    } else {
        EN_US_FTL
    };

    // Parse FTL resource
    let resource = FluentResource::try_new(ftl_source.to_string())
        .expect("Failed to parse embedded FTL resource");

    // Create bundle with selected locale
    let mut bundle = FluentBundle::new(vec![selected_locale]);
    bundle
        .add_resource(resource)
        .expect("Failed to add resource to bundle");

    bundle
}

/// Get a localized message by ID with optional arguments.
///
/// This function retrieves a message from the Fluent bundle and formats it
/// with the provided arguments (if any). If the message is not found,
/// returns the message ID itself as a fallback.
///
/// # Parameters
///
/// - `bundle`: The Fluent bundle containing loaded messages
/// - `id`: The message identifier (e.g., "op-moving", "err-source-not-found")
/// - `args`: Optional arguments for message interpolation
///
/// # Returns
///
/// The formatted localized message, or the message ID if not found.
///
/// # Examples
///
/// ```no_run
/// use mvln::i18n;
/// use fluent::FluentArgs;
///
/// let bundle = i18n::init();
///
/// // With arguments
/// let mut args = FluentArgs::new();
/// args.set("path", "/tmp/file.txt");
/// let msg = i18n::msg(&bundle, "err-source-not-found", Some(&args));
///
/// // Without arguments
/// let msg = i18n::msg(&bundle, "op-dry-run", None);
/// ```
#[must_use]
pub fn msg(bundle: &FluentBundle<FluentResource>, id: &str, args: Option<&FluentArgs>) -> String {
    let Some(message) = bundle.get_message(id) else {
        // Fallback: return message ID if not found
        return id.to_string();
    };

    let Some(pattern) = message.value() else {
        // Fallback: return message ID if no value
        return id.to_string();
    };

    let mut errors = vec![];
    let formatted = bundle.format_pattern(pattern, args, &mut errors);

    // Log errors in debug builds but still return the formatted message
    #[cfg(debug_assertions)]
    if !errors.is_empty() {
        eprintln!("Fluent formatting errors for '{id}': {errors:?}");
    }

    formatted.to_string()
}

/// Convenience function for retrieving messages without arguments.
///
/// This is a simplified version of [`msg`] for messages that don't require
/// any parameter interpolation.
///
/// # Parameters
///
/// - `bundle`: The Fluent bundle containing loaded messages
/// - `id`: The message identifier
///
/// # Returns
///
/// The formatted localized message, or the message ID if not found.
///
/// # Examples
///
/// ```no_run
/// use mvln::i18n;
///
/// let bundle = i18n::init();
/// let msg = i18n::simple_msg(&bundle, "op-dry-run");
/// println!("{}", msg);
/// ```
#[must_use]
pub fn simple_msg(bundle: &FluentBundle<FluentResource>, id: &str) -> String {
    msg(bundle, id, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_creates_valid_bundle() {
        let bundle = init();
        // Should not panic and should return a valid bundle
        assert!(!bundle.locales.is_empty());
    }

    #[test]
    fn test_simple_msg_en_us() {
        let bundle = init();
        let msg = simple_msg(&bundle, "op-dry-run");
        // Should contain expected English text
        assert!(msg.contains("DRY-RUN") || msg.contains("预览模式"));
    }

    #[test]
    fn test_msg_with_args() {
        let bundle = init();
        let mut args = FluentArgs::new();
        args.set("src", "test.txt");
        args.set("dest", "/backup/test.txt");

        let message = msg(&bundle, "op-moving", Some(&args));
        // Should contain the interpolated values
        assert!(message.contains("test.txt"));
        assert!(message.contains("/backup/test.txt"));
    }

    #[test]
    fn test_missing_message_returns_id() {
        let bundle = init();
        let msg = simple_msg(&bundle, "nonexistent-message-id");
        // Should return the ID itself as fallback
        assert_eq!(msg, "nonexistent-message-id");
    }

    #[test]
    fn test_msg_without_args() {
        let bundle = init();
        let msg = msg(&bundle, "op-dry-run", None);
        // Should work the same as simple_msg
        assert!(msg.contains("DRY-RUN") || msg.contains("预览模式"));
    }

    #[test]
    fn test_error_message_with_attribute() {
        let bundle = init();
        let mut args = FluentArgs::new();
        args.set("path", "/tmp/test.txt");

        // Test main message
        let message = msg(&bundle, "err-dest-exists", Some(&args));
        assert!(message.contains("/tmp/test.txt"));

        // Note: Attributes (.hint) need to be retrieved separately in Fluent
        // The msg() function only retrieves the main message value
    }
}
