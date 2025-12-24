use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;
use tokio::task_local;
use tracing::{error, info};

task_local! {
    pub static CURRENT_LANG: String;
}

pub static I18N_STORE: OnceLock<HashMap<String, HashMap<String, String>>> = OnceLock::new();

fn load_locale_file(path: &Path) -> Option<(String, HashMap<String, String>)> {
    if path.extension()? != "toml" {
        return None;
    }

    let lang = path.file_stem()?.to_str()?.to_string();

    let content = fs::read_to_string(path)
        .inspect_err(|e| error!("Failed to read i18n file {}: {}", path.display(), e))
        .ok()?;

    let map = toml::from_str(&content)
        .inspect_err(|e| error!("Failed to parse i18n file {}: {}", path.display(), e))
        .ok()?;

    info!("Loaded i18n for lang: {}", lang);
    Some((lang, map))
}

pub fn init(dir: &str) {
    let path = Path::new(dir);
    if !path.exists() {
        error!("i18n directory not found: {}", dir);
        return;
    }

    let Ok(entries) = fs::read_dir(path).inspect_err(|e| {
        error!("Failed to read i18n directory {}: {}", path.display(), e);
    }) else {
        return;
    };

    let store = entries
        .filter_map(Result::ok)
        .filter_map(|entry| load_locale_file(&entry.path()))
        .collect();

    if I18N_STORE.set(store).is_err() {
        error!("I18N_STORE already initialized");
    }
}

pub fn translate(lang: &str, key: &str) -> Option<String> {
    I18N_STORE
        .get()
        .and_then(|store| store.get(lang))
        .and_then(|map| map.get(key))
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_and_translate() {
        let dir = std::env::temp_dir().join(format!(
            "rivus_axum_i18n_test_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("en.toml"), "200 = \"Ok\"\n").unwrap();
        fs::write(dir.join("zh.toml"), "200 = \"成功\"\n").unwrap();
        fs::write(dir.join("ignore.txt"), "200 = \"Ignored\"\n").unwrap();
        fs::write(dir.join("bad.toml"), "200 = ").unwrap();

        init("/this/path/should/not/exist");
        init(dir.to_str().unwrap());

        assert_eq!(translate("en", "200").as_deref(), Some("Ok"));
        assert_eq!(translate("zh", "200").as_deref(), Some("成功"));
        assert_eq!(translate("en", "404"), None);
        assert_eq!(translate("fr", "200"), None);

        let dir2 = std::env::temp_dir().join(format!(
            "rivus_axum_i18n_test_{}_{}_2",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir2).unwrap();
        fs::write(dir2.join("en.toml"), "200 = \"Overridden\"\n").unwrap();
        init(dir2.to_str().unwrap());
        assert_eq!(translate("en", "200").as_deref(), Some("Ok"));

        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&dir2);
    }
}
