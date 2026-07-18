//! Core keymap types: builder, merge/validation, lookup and description.

use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;

use crokey::{KeyCombination, KeyCombinationFormat};
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;
#[cfg(feature = "toml")]
mod toml_support;

/// One keybinding table for one scope.
///
/// A `Keymap<A>` maps [`KeyCombination`]s to the actions of a single scope
/// (one action enum per scope). Applications with several scopes compose
/// several `Keymap` instances, one per action enum.
///
/// Build it once at startup with [`Keymap::builder`], optionally apply user
/// overrides with [`Keymap::merge`], then use [`Keymap::action_for`] to
/// dispatch events and [`Keymap::describe`] to render help entries.
#[derive(Debug, Clone)]
pub struct Keymap<A> {
    /// 声明顺序保存(帮助/示例导出按此顺序渲染)。
    entries: Vec<Entry<A>>,
    lookup: HashMap<KeyCombination, A>,
}

#[derive(Debug, Clone)]
struct Entry<A> {
    action: A,
    /// serde 变体名 —— 用户配置文件里的 action 键名契约。
    name: String,
    default_keys: Vec<KeyCombination>,
    /// 当前生效键位(默认或用户覆盖后)。
    keys: Vec<KeyCombination>,
    /// 本条是否为用户覆盖(冲突消解时只有覆盖可被回退)。
    overridden: bool,
    desc: Option<String>,
}

impl<A> Entry<A> {
    fn revert_to_default(&mut self) {
        self.keys = self.default_keys.clone();
        self.overridden = false;
    }
}

/// A read-only view over one keymap entry, in declaration order.
///
/// Yielded by [`Keymap::entries`]; useful for rendering help screens.
#[derive(Debug, Clone, Copy)]
pub struct KeymapEntry<'a, A> {
    /// The action this entry binds.
    pub action: &'a A,
    /// The action's config key name (its serde variant name).
    pub name: &'a str,
    /// Currently effective key combinations (defaults or user overrides).
    pub keys: &'a [KeyCombination],
    /// Optional human-readable description, as set via [`KeymapBuilder::desc`].
    pub desc: Option<&'a str>,
}

/// A non-fatal problem found while merging user overrides.
///
/// Warnings never abort the merge: the affected entries fall back to their
/// defaults (or are ignored) and the application stays usable. Hosts decide
/// how to surface them (log, modal, status line...). All fields are public so
/// hosts can produce their own localized messages; [`std::fmt::Display`] gives
/// a reasonable English default.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum KeymapWarning {
    /// A key string of this action failed to parse (or used an unsupported
    /// multi-key chord); the action fell back to its default keys.
    #[non_exhaustive]
    ParseError {
        /// Config key name of the affected action.
        action: String,
        /// The rejected key string as written by the user.
        input: String,
        /// Parser error description.
        message: String,
    },
    /// The override's value has the wrong type (not a key string or a list of
    /// key strings); the action fell back to its default keys.
    #[non_exhaustive]
    InvalidEntry {
        /// Config key name of the affected action.
        action: String,
    },
    /// One key ended up bound to several actions; the overridden ones among
    /// them fell back to their default keys.
    #[non_exhaustive]
    Conflict {
        /// The contested key combination.
        key: KeyCombination,
        /// Config key names of all actions that were bound to `key`.
        actions: Vec<String>,
    },
    /// The override refers to no known action (typo, or a stale config for a
    /// removed action); the entry was ignored.
    #[non_exhaustive]
    UnknownAction {
        /// The unrecognized config key name.
        name: String,
    },
}

impl std::fmt::Display for KeymapWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseError {
                action,
                input,
                message,
            } => write!(
                f,
                "invalid key {input:?} for action {action:?} ({message}); using default keys"
            ),
            Self::InvalidEntry { action } => write!(
                f,
                "invalid value for action {action:?}: expected a key string or a list of key strings; using default keys"
            ),
            Self::Conflict { key, actions } => {
                write!(
                    f,
                    "key \"{key}\" bound to several actions ({}); overridden ones reverted to defaults",
                    actions.join(", ")
                )
            }
            Self::UnknownAction { name } => write!(f, "unknown action {name:?}; entry ignored"),
        }
    }
}

/// User overrides for one keymap, keyed by action config name.
///
/// Deserializes from any serde format as a map of action name to either one
/// key string or a list of key strings:
///
/// ```toml
/// page_down = ["ctrl-d", "space"]
/// quit = "q"
/// ```
///
/// Keys are plain strings here so that unknown action names and invalid key
/// strings survive deserialization and can be reported as [`KeymapWarning`]s
/// by [`Keymap::merge`] instead of failing the whole file.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(transparent)]
pub struct KeymapOverrides {
    // BTreeMap 保证告警顺序确定(可测试)。
    entries: BTreeMap<String, KeyList>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum KeyList {
    One(String),
    Many(Vec<String>),
    /// 兜底吞掉任何其他类型(数字/表/混合列表...):让整份配置的反序列化
    /// 永不失败,类型错误降级为该条目的 `InvalidEntry` 告警。
    Invalid(serde::de::IgnoredAny),
}

impl KeyList {
    /// `None` 表示值类型不对(`Invalid`),由调用方报 `InvalidEntry`。
    fn as_strings(&self) -> Option<Vec<&str>> {
        match self {
            Self::One(s) => Some(vec![s.as_str()]),
            Self::Many(v) => Some(v.iter().map(String::as_str).collect()),
            Self::Invalid(_) => None,
        }
    }
}

impl<N: Into<String>, K: Into<String>, I: IntoIterator<Item = K>> FromIterator<(N, I)>
    for KeymapOverrides
{
    fn from_iter<T: IntoIterator<Item = (N, I)>>(iter: T) -> Self {
        Self {
            entries: iter
                .into_iter()
                .map(|(name, keys)| {
                    let keys = keys.into_iter().map(Into::into).collect();
                    (name.into(), KeyList::Many(keys))
                })
                .collect(),
        }
    }
}

/// Builder for [`Keymap`]; obtained via [`Keymap::builder`].
///
/// Default bindings are developer code: any invalid key string, duplicate
/// `bind` for the same action, or key conflict across actions panics in
/// [`KeymapBuilder::build`] so mistakes surface at development time. User
/// input never goes through the builder — it goes through [`Keymap::merge`],
/// which reports problems as warnings instead.
#[derive(Debug)]
pub struct KeymapBuilder<A> {
    entries: Vec<Entry<A>>,
}

impl<A> Keymap<A> {
    /// Start building a keymap. See [`KeymapBuilder`].
    pub fn builder() -> KeymapBuilder<A> {
        KeymapBuilder {
            entries: Vec::new(),
        }
    }
}

impl<A> KeymapBuilder<A>
where
    A: Copy + Eq + Hash + Serialize,
{
    /// Declare the default key combinations for `action`, in crokey syntax
    /// (`"j"`, `"ctrl-d"`, `"pagedown"`, `"f1"`...). A single uppercase
    /// letter implies shift (`"G"` ≡ `"shift-g"`). Duplicate keys within the
    /// list are deduplicated.
    ///
    /// # Panics
    ///
    /// Panics if `keys` is empty, a key string does not parse, a key is a
    /// multi-key chord (unsupported), or `action` was already bound.
    pub fn bind<I, S>(mut self, action: A, keys: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let name = variant_name(&action);
        assert!(
            !self.entries.iter().any(|e| e.action == action),
            "action {name:?} bound twice in the default keymap"
        );
        let keys_out = parse_keys(keys).unwrap_or_else(|(input, e)| {
            panic!("invalid default key {input:?} for action {name:?}: {e}")
        });
        assert!(
            !keys_out.is_empty(),
            "action {name:?} bound with no keys in the default keymap"
        );
        self.entries.push(Entry {
            action,
            name,
            default_keys: keys_out.clone(),
            keys: keys_out,
            overridden: false,
            desc: None,
        });
        self
    }

    /// Attach a human-readable description to an already-bound `action`,
    /// rendered in help screens and as a comment by
    /// [`Keymap::to_toml_example`].
    ///
    /// # Panics
    ///
    /// Panics if `action` has not been bound yet.
    pub fn desc(mut self, action: A, text: impl Into<String>) -> Self {
        let entry = self
            .entries
            .iter_mut()
            .find(|e| e.action == action)
            .unwrap_or_else(|| {
                panic!(
                    "desc() for action {:?} before bind()",
                    variant_name(&action)
                )
            });
        entry.desc = Some(text.into());
        self
    }

    /// Finish the builder.
    ///
    /// # Panics
    ///
    /// Panics if two actions share a default key: the default table must be
    /// conflict-free (this is also what makes conflict resolution in
    /// [`Keymap::merge`] always converge).
    pub fn build(self) -> Keymap<A> {
        let mut seen: HashMap<KeyCombination, &str> = HashMap::new();
        for entry in &self.entries {
            for key in &entry.keys {
                if let Some(other) = seen.insert(*key, &entry.name) {
                    panic!(
                        "default key \"{key}\" bound to both {other:?} and {:?}",
                        entry.name
                    );
                }
            }
        }
        let mut keymap = Keymap {
            entries: self.entries,
            lookup: HashMap::new(),
        };
        keymap.rebuild_lookup();
        keymap
    }
}

impl<A> Keymap<A>
where
    A: Copy + Eq + Hash,
{
    /// Merge user overrides into this keymap.
    ///
    /// Each action present in `overrides` has its key list replaced entirely;
    /// absent actions keep their current keys. An explicit empty list (`[]`)
    /// unbinds the action — deliberate, so users can free a key; hosts guarding
    /// critical actions (e.g. quit) can check [`Keymap::keys`] afterwards.
    /// Problems never fail the merge — they are returned as [`KeymapWarning`]s
    /// and the affected entries fall back to their **defaults** (even if an
    /// earlier merge had overridden them; see [`KeymapWarning`] for the rules).
    pub fn merge(&mut self, overrides: KeymapOverrides) -> Vec<KeymapWarning> {
        let mut warnings = Vec::new();

        for (name, keys) in &overrides.entries {
            let Some(entry) = self.entries.iter_mut().find(|e| &e.name == name) else {
                warnings.push(KeymapWarning::UnknownAction { name: name.clone() });
                continue;
            };
            let Some(raws) = keys.as_strings() else {
                warnings.push(KeymapWarning::InvalidEntry {
                    action: name.clone(),
                });
                entry.revert_to_default();
                continue;
            };
            match parse_keys(raws) {
                Ok(parsed) => {
                    entry.keys = parsed;
                    entry.overridden = true;
                }
                Err((input, message)) => {
                    warnings.push(KeymapWarning::ParseError {
                        action: name.clone(),
                        input,
                        message,
                    });
                    // 告警文案承诺「using default keys」——即使此前某次 merge 覆盖过,
                    // 也要回到默认而不是停留在旧覆盖上,否则帮助/告警与实际绑定不符。
                    entry.revert_to_default();
                }
            }
        }

        warnings.extend(self.resolve_conflicts());
        self.rebuild_lookup();
        warnings
    }

    /// The action bound to `key`, if any.
    ///
    /// Accepts anything convertible into a [`KeyCombination`] — in particular
    /// a `crossterm` `KeyEvent`.
    pub fn action_for<K: Into<KeyCombination>>(&self, key: K) -> Option<A> {
        self.lookup.get(&canonical(key.into())).copied()
    }

    /// Currently effective key combinations for `action` (empty slice if the
    /// action is not part of this keymap).
    pub fn keys(&self, action: A) -> &[KeyCombination] {
        self.entry(action).map_or(&[], |e| e.keys.as_slice())
    }

    /// Human-readable key names for `action`, formatted with
    /// [`crokey::STANDARD_FORMAT`]. Reflects user overrides — use this for
    /// help screens so they always show the real bindings.
    pub fn describe(&self, action: A) -> Vec<String> {
        self.describe_with(&crokey::STANDARD_FORMAT, action)
    }

    /// Like [`Keymap::describe`] with a custom [`KeyCombinationFormat`].
    pub fn describe_with(&self, format: &KeyCombinationFormat, action: A) -> Vec<String> {
        self.keys(action)
            .iter()
            .map(|k| format.to_string(*k))
            .collect()
    }

    /// The description attached to `action` via [`KeymapBuilder::desc`].
    pub fn desc(&self, action: A) -> Option<&str> {
        self.entry(action).and_then(|e| e.desc.as_deref())
    }

    /// Iterate all entries in declaration order (for help screens and
    /// config templates).
    pub fn entries(&self) -> impl Iterator<Item = KeymapEntry<'_, A>> {
        self.entries.iter().map(|e| KeymapEntry {
            action: &e.action,
            name: &e.name,
            keys: &e.keys,
            desc: e.desc.as_deref(),
        })
    }

    fn entry(&self, action: A) -> Option<&Entry<A>> {
        self.entries.iter().find(|e| e.action == action)
    }

    fn rebuild_lookup(&mut self) {
        self.lookup = self
            .entries
            .iter()
            .flat_map(|e| e.keys.iter().map(|k| (*k, e.action)))
            .collect();
    }

    /// 反复回退「参与冲突的用户覆盖」直到无冲突。默认表自身无冲突(build 断言),
    /// 且每轮至少回退一条覆盖,故必然收敛;纯默认之间不可能相撞,循环安全终止。
    fn resolve_conflicts(&mut self) -> Vec<KeymapWarning> {
        let mut warnings = Vec::new();
        loop {
            let mut by_key: BTreeMap<KeyCombination, Vec<usize>> = BTreeMap::new();
            for (i, entry) in self.entries.iter().enumerate() {
                for key in &entry.keys {
                    by_key.entry(*key).or_default().push(i);
                }
            }
            let mut reverted = false;
            for (key, indices) in by_key {
                if indices.len() < 2 {
                    continue;
                }
                warnings.push(KeymapWarning::Conflict {
                    key,
                    actions: indices
                        .iter()
                        .map(|&i| self.entries[i].name.clone())
                        .collect(),
                });
                for i in indices {
                    let entry = &mut self.entries[i];
                    if entry.overridden {
                        entry.revert_to_default();
                        reverted = true;
                    }
                }
            }
            if !reverted {
                return warnings;
            }
        }
    }
}

/// 统一归一化:在 crokey `normalized()`(字母大小写与 SHIFT 对齐)之上,再把
/// 单字符**非字母**键的 SHIFT 剥掉 —— Windows 终端后端给 shift+标点(如 `?`)
/// 的事件附带 SHIFT 而 Unix 不带,解析侧的 `"?"` 也不带。入表(parse_key)与
/// 查询(action_for)两侧对称调用后,查表恒为单次命中,无需查询侧兜底重查。
fn canonical(key: KeyCombination) -> KeyCombination {
    use crokey::crossterm::event::{KeyCode, KeyModifiers};
    let key = key.normalized();
    if key.modifiers.contains(KeyModifiers::SHIFT)
        && let crokey::OneToThree::One(KeyCode::Char(c)) = key.codes
        && !c.is_ascii_alphabetic()
    {
        return KeyCombination::new(key.codes, key.modifiers - KeyModifiers::SHIFT);
    }
    key
}

/// 解析一个键位字符串。crokey parse 会整体转小写、静默丢失大小写信息,故
/// 大写字母习惯写法(`"G"`、`"ctrl-G"`)按 shift 意图在结构层补回 SHIFT 位;
/// 多键 chord(`"g-g"`)因事件侧 `From<KeyEvent>` 只产生单码组合、永远无法
/// 命中,直接拒绝。
fn parse_key(raw: &str) -> Result<KeyCombination, String> {
    use crokey::crossterm::event::KeyModifiers;
    let key = crokey::parse(raw).map_err(|e| e.to_string())?;
    if !key.is_ansi_compatible() {
        return Err(format!(
            "multi-key combination {raw:?} is not supported (bind a single key, optionally with modifiers)"
        ));
    }
    // 末段是单个大写字母 → shift 意图;canonical 里的 normalized 会把
    // 「小写 char + SHIFT」对齐为大写形态,与显式 "shift-g" 完全一致。
    let implies_shift = raw
        .rsplit('-')
        .next()
        .is_some_and(|last| last.len() == 1 && last.chars().all(|c| c.is_ascii_uppercase()));
    let key = if implies_shift {
        KeyCombination::new(key.codes, key.modifiers | KeyModifiers::SHIFT)
    } else {
        key
    };
    Ok(canonical(key))
}

/// 解析一组键位字符串(去重保序);任一失败即整组失败,带回 `(input, message)`。
fn parse_keys<I, S>(raws: I) -> Result<Vec<KeyCombination>, (String, String)>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut out = Vec::new();
    for raw in raws {
        let raw = raw.as_ref();
        match parse_key(raw) {
            Ok(key) => {
                if !out.contains(&key) {
                    out.push(key);
                }
            }
            Err(message) => return Err((raw.to_string(), message)),
        }
    }
    Ok(out)
}

/// 经 serde 序列化取 unit variant 名 —— action 名的单一真源,自动尊重
/// `#[serde(rename_all)]` 等属性。非 unit-variant 枚举在开发期即 panic。
fn variant_name<A: Serialize>(action: &A) -> String {
    action
        .serialize(VariantNameSerializer)
        .expect("keymap actions must be fieldless enum variants (unit variants)")
        .to_string()
}

struct VariantNameSerializer;

/// 只认 `serialize_unit_variant` 的最小 serializer,其余一律报错。
mod variant_name_impl {
    use serde::ser::{Error as _, Impossible, Serializer};

    #[derive(Debug)]
    pub struct NotUnitVariant(String);

    impl std::fmt::Display for NotUnitVariant {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.0.fmt(f)
        }
    }
    impl std::error::Error for NotUnitVariant {}
    impl serde::ser::Error for NotUnitVariant {
        fn custom<T: std::fmt::Display>(msg: T) -> Self {
            Self(msg.to_string())
        }
    }

    // 拒绝一个 serializer 方法;`-> Type` 可选,缺省返回 Self::Ok。
    macro_rules! reject {
        ($($f:ident($($arg:ty),*) $(-> $ret:ty)?;)*) => {
            $(reject!(@one $f($($arg),*) $(-> $ret)?);)*
        };
        (@one $f:ident($($arg:ty),*)) => {
            reject!(@one $f($($arg),*) -> Self::Ok);
        };
        (@one $f:ident($($arg:ty),*) -> $ret:ty) => {
            fn $f(self, $(_: $arg),*) -> Result<$ret, Self::Error> {
                Err(Self::Error::custom(concat!("not a unit variant: ", stringify!($f))))
            }
        };
    }

    impl Serializer for super::VariantNameSerializer {
        type Ok = &'static str;
        type Error = NotUnitVariant;
        type SerializeSeq = Impossible<Self::Ok, Self::Error>;
        type SerializeTuple = Impossible<Self::Ok, Self::Error>;
        type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
        type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
        type SerializeMap = Impossible<Self::Ok, Self::Error>;
        type SerializeStruct = Impossible<Self::Ok, Self::Error>;
        type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

        fn serialize_unit_variant(
            self,
            _name: &'static str,
            _index: u32,
            variant: &'static str,
        ) -> Result<Self::Ok, Self::Error> {
            Ok(variant)
        }

        reject! {
            serialize_bool(bool);
            serialize_i8(i8);
            serialize_i16(i16);
            serialize_i32(i32);
            serialize_i64(i64);
            serialize_u8(u8);
            serialize_u16(u16);
            serialize_u32(u32);
            serialize_u64(u64);
            serialize_f32(f32);
            serialize_f64(f64);
            serialize_char(char);
            serialize_str(&str);
            serialize_bytes(&[u8]);
            serialize_unit();
            serialize_unit_struct(&'static str);
            serialize_none();
            serialize_seq(Option<usize>) -> Self::SerializeSeq;
            serialize_tuple(usize) -> Self::SerializeTuple;
            serialize_tuple_struct(&'static str, usize) -> Self::SerializeTupleStruct;
            serialize_tuple_variant(&'static str, u32, &'static str, usize) -> Self::SerializeTupleVariant;
            serialize_map(Option<usize>) -> Self::SerializeMap;
            serialize_struct(&'static str, usize) -> Self::SerializeStruct;
            serialize_struct_variant(&'static str, u32, &'static str, usize) -> Self::SerializeStructVariant;
        }

        // 带泛型参数的三个方法宏收不进去,保留手写。
        fn serialize_some<T: ?Sized + serde::Serialize>(
            self,
            _: &T,
        ) -> Result<Self::Ok, Self::Error> {
            Err(Self::Error::custom("not a unit variant: serialize_some"))
        }
        fn serialize_newtype_struct<T: ?Sized + serde::Serialize>(
            self,
            _: &'static str,
            _: &T,
        ) -> Result<Self::Ok, Self::Error> {
            Err(Self::Error::custom(
                "not a unit variant: serialize_newtype_struct",
            ))
        }
        fn serialize_newtype_variant<T: ?Sized + serde::Serialize>(
            self,
            _: &'static str,
            _: u32,
            _: &'static str,
            _: &T,
        ) -> Result<Self::Ok, Self::Error> {
            Err(Self::Error::custom(
                "not a unit variant: serialize_newtype_variant",
            ))
        }
    }
}
