use rivus_axum_macro::i18n_assets;
use rivus_axum::i18n::i18n::t;

// Invoking the macro. 
// It searches for files relative to CARGO_MANIFEST_DIR (rivus-axum-macro).
i18n_assets!("tests/assets");

#[test]
fn test_i18n_assets_loading() {
    // Test static translation (English)
    let en_hello = t("en", "hello", &[]);
    assert_eq!(en_hello, "Hello, world!");

    // Test static translation (Chinese)
    let zh_hello = t("zh", "hello", &[]);
    assert_eq!(zh_hello, "你好，世界！");

    // Test placeholder translation (English)
    let en_welcome = t("en", "welcome", &[("name", "Jason")]);
    assert_eq!(en_welcome, "Welcome, Jason!");

    // Test placeholder translation (Chinese)
    let zh_welcome = t("zh", "welcome", &[("name", "Jason")]);
    assert_eq!(zh_welcome, "欢迎，Jason！");

    // Test missing key
    let missing_key = t("en", "non_existent_key", &[]);
    assert_eq!(missing_key, "[Missing Key: non_existent_key]");

    // Test missing language
    let missing_lang = t("fr", "hello", &[]);
    assert_eq!(missing_lang, "[Missing Lang: fr]");
}
