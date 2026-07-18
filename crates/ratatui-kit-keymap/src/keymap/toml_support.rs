//! TOML convenience layer (`toml` feature): merge straight from TOML input.

use std::hash::Hash;

use serde::Deserialize;

use super::{Keymap, KeymapOverrides, KeymapWarning};

impl<A> Keymap<A>
where
    A: Copy + Eq + Hash,
{
    /// Parse `input` as a TOML table of overrides and [`Keymap::merge`] it.
    ///
    /// `Err` only for actual TOML **syntax** errors (that is the host's
    /// decision to handle — typically "fall back to defaults + warn").
    /// Everything entry-level — wrong value types included — comes back as
    /// [`KeymapWarning`]s from a successful merge.
    pub fn merge_toml_str(&mut self, input: &str) -> Result<Vec<KeymapWarning>, toml::de::Error> {
        let overrides: KeymapOverrides = toml::from_str(input)?;
        Ok(self.merge(overrides))
    }

    /// [`Keymap::merge`] overrides already extracted as a [`toml::Table`]
    /// (e.g. one scope's sub-table of a larger config file). The `toml` crate
    /// is re-exported at the crate root so hosts don't need their own
    /// version-matched dependency.
    pub fn merge_toml_table(&mut self, table: toml::Table) -> Vec<KeymapWarning> {
        // TOML 表的键恒为字符串,值经 KeyList 的 Invalid 兜底全部可吞
        // —— 这里的反序列化不可能失败,类型错误已降级为条目级告警。
        let overrides = KeymapOverrides::deserialize(table)
            .expect("KeymapOverrides deserialization from a TOML table is infallible");
        self.merge(overrides)
    }

    /// Render a complete, commented config template from the **default**
    /// bindings: one entry per action with its description as a comment. The
    /// output is a TOML table body (no `[section]` header — hosts nest it
    /// under their own scope table) and parses back through
    /// [`Keymap::merge`] without warnings.
    pub fn to_toml_example(&self) -> String {
        let mut out = String::new();
        for entry in &self.entries {
            if let Some(desc) = &entry.desc {
                for line in desc.lines() {
                    out.push_str("# ");
                    out.push_str(line);
                    out.push('\n');
                }
            }
            // 转义交给 toml crate:Value::String 的 Display 即合法 TOML 字符串。
            let keys = entry
                .default_keys
                .iter()
                .map(|k| toml::Value::String(k.to_string()).to_string())
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!("{} = [{keys}]\n\n", toml_key(&entry.name)));
        }
        out
    }
}

/// TOML 裸键仅允许 `[A-Za-z0-9_-]`,其余(serde rename 产生的怪名)按
/// TOML 字符串规则加引号转义。
fn toml_key(name: &str) -> String {
    let bare = !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    if bare {
        name.to_string()
    } else {
        toml::Value::String(name.to_string()).to_string()
    }
}

#[cfg(test)]
mod tests {
    use crokey::key;
    use serde::{Deserialize, Serialize};

    use crate::{Keymap, KeymapWarning};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    enum Action {
        PageDown,
        Quit,
    }

    fn keymap() -> Keymap<Action> {
        Keymap::builder()
            .bind(Action::PageDown, ["pagedown"])
            .desc(Action::PageDown, "scroll one page down")
            .bind(Action::Quit, ["q"])
            .build()
    }

    #[test]
    fn merges_list_and_single_string_forms() {
        let mut km = keymap();
        let warnings = km
            .merge_toml_str("page_down = [\"ctrl-d\", \"space\"]\nquit = \"esc\"\n")
            .unwrap();
        assert!(warnings.is_empty());
        assert_eq!(km.action_for(key!(ctrl - d)), Some(Action::PageDown));
        assert_eq!(km.action_for(key!(esc)), Some(Action::Quit));
        assert_eq!(km.action_for(key!(q)), None);
    }

    #[test]
    fn broken_toml_is_an_error() {
        assert!(keymap().merge_toml_str("page_down = [").is_err());
    }

    #[test]
    fn wrong_typed_entry_is_entry_level_warning() {
        let mut km = keymap();
        // 单条类型错误只废那一条(降级为 InvalidEntry 告警),
        // 其余合法覆盖照常生效 —— 不允许整份文件报废。
        let warnings = km
            .merge_toml_str("page_down = \"ctrl-d\"\nquit = 5\n")
            .unwrap();
        assert!(
            matches!(&warnings[..], [KeymapWarning::InvalidEntry { action }]
            if action == "quit")
        );
        assert_eq!(km.action_for(key!(ctrl - d)), Some(Action::PageDown));
        assert_eq!(km.action_for(key!(q)), Some(Action::Quit));
    }

    #[test]
    fn wrong_typed_entry_in_table_form() {
        let mut km = keymap();
        let table: crate::toml::Table =
            crate::toml::from_str("page_down = \"ctrl-d\"\nquit = 5\n").unwrap();
        let warnings = km.merge_toml_table(table);
        assert_eq!(warnings.len(), 1);
        assert_eq!(km.action_for(key!(ctrl - d)), Some(Action::PageDown));
    }

    #[test]
    fn entry_level_problems_become_warnings() {
        let mut km = keymap();
        let warnings = km
            .merge_toml_str("page_down = [\"ctrl-\"]\ntypo_action = \"x\"\n")
            .unwrap();
        assert_eq!(warnings.len(), 2);
        assert!(
            warnings
                .iter()
                .any(|w| matches!(w, KeymapWarning::ParseError { .. }))
        );
        assert!(
            warnings
                .iter()
                .any(|w| matches!(w, KeymapWarning::UnknownAction { .. }))
        );
        assert_eq!(km.action_for(key!(pagedown)), Some(Action::PageDown));
    }

    #[test]
    fn example_lists_all_actions_with_desc() {
        let example = keymap().to_toml_example();
        for name in ["page_down", "quit"] {
            assert!(
                example.contains(&format!("{name} = [")),
                "missing {name} in:\n{example}"
            );
        }
        assert!(example.contains("# scroll one page down"));
        // 导出的是默认键。
        assert!(example.contains("\"PageDown\"") || example.contains("\"pagedown\""));
    }

    #[test]
    fn example_escapes_toml_special_keys() {
        let km = Keymap::<Action>::builder()
            .bind(Action::Quit, ["\""])
            .bind(Action::PageDown, ["pagedown"])
            .build();
        let example = km.to_toml_example();
        // 双引号键必须转义,否则导出的模板不是合法 TOML;转义后应能无警回读。
        let mut km2 = keymap();
        assert!(
            km2.merge_toml_str(&example).unwrap().is_empty(),
            "{example}"
        );
    }

    #[test]
    fn example_round_trips_without_warnings() {
        let example = keymap().to_toml_example();
        let mut km = keymap();
        let warnings = km.merge_toml_str(&example).unwrap();
        assert!(
            warnings.is_empty(),
            "example must round-trip cleanly, got: {warnings:?}\n{example}"
        );
        // 模板即默认:合并后行为不变。
        assert_eq!(km.action_for(key!(pagedown)), Some(Action::PageDown));
        assert_eq!(km.action_for(key!(q)), Some(Action::Quit));
    }
}
