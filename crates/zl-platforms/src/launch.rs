//! Building the Android intent of a launch (RF-09): a pure function from a profile and a context
//! to a fully substituted [`LaunchIntent`]; Kotlin only turns it into an `Intent`.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use zl_config::platforms::RETROARCH_CORE_DIR;

use crate::error::LaunchError;
use crate::model::{
    Emulator, ExtraTemplate, ExtraTemplateValue, IntTemplate, IntentTemplate, Package, Profile,
};
use crate::template::{self, Part, TAG_PREFIX};

/// The ROM as URIs, each built by the caller (only Android can make them).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RomUris {
    /// `file://` URI of the ROM (`{rom_uri}`).
    #[serde(default)]
    pub file: Option<String>,
    /// SAF document URI of the ROM (`{rom_saf}`).
    #[serde(default)]
    pub saf: Option<String>,
    /// SAF tree URI the user granted and that contains the ROM (`{rom_saf_tree}`).
    #[serde(default)]
    pub saf_tree: Option<String>,
    /// `content://` URI from the launcher's FileProvider (`{rom_provider}`).
    #[serde(default)]
    pub provider: Option<String>,
}

/// What the builder knows about the game being launched.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaunchContext {
    /// Filesystem path of the ROM (`{rom}`); empty for tag-driven platforms.
    #[serde(default)]
    pub rom_path: String,
    /// The ROM as URIs.
    #[serde(default)]
    pub rom_uris: RomUris,
    /// Canonical platform id (`{platform}`).
    pub platform: String,
    /// Folder of the platform on disk (`{platform_dir}`).
    #[serde(default)]
    pub platform_dir: Option<String>,
    /// Tags of the game (`{tags.NAME}`).
    #[serde(default)]
    pub tags: BTreeMap<String, String>,
    /// RetroArch core id to use instead of the platform's first.
    #[serde(default)]
    pub core: Option<String>,
    /// Installed package to launch; absent means the emulator's first package.
    #[serde(default)]
    pub package: Option<String>,
}

/// A substituted extra value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExtraValue {
    /// A string.
    String {
        /// The text.
        value: String,
    },
    /// A boolean.
    Bool {
        /// The flag.
        value: bool,
    },
    /// An integer.
    Int {
        /// The number.
        value: i64,
    },
    /// An array of strings.
    StringArray {
        /// The elements.
        value: Vec<String>,
    },
}

/// A substituted extra.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Extra {
    /// Name of the extra.
    pub key: String,
    /// Typed value.
    #[serde(flatten)]
    pub value: ExtraValue,
}

/// ClipData to attach to the intent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClipData {
    /// URI of the item.
    pub uri: String,
    /// MIME type of the item.
    pub mime: String,
}

/// A URI permission to grant to the target package before starting it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UriGrant {
    /// URI to grant.
    pub uri: String,
    /// Grant read access.
    pub read: bool,
    /// Grant write access.
    pub write: bool,
}

/// A complete intent, ready for Kotlin.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaunchIntent {
    /// Action, if any.
    #[serde(default)]
    pub action: Option<String>,
    /// Package of the emulator.
    pub package: String,
    /// Full activity class name; absent means the package's launcher activity.
    #[serde(default)]
    pub activity: Option<String>,
    /// Categories.
    #[serde(default)]
    pub categories: Vec<String>,
    /// Data URI.
    #[serde(default)]
    pub data: Option<String>,
    /// MIME type.
    #[serde(default)]
    pub mime: Option<String>,
    /// Extras, in order.
    #[serde(default)]
    pub extras: Vec<Extra>,
    /// Flags by name.
    #[serde(default)]
    pub flags: Vec<String>,
    /// ClipData, if any.
    #[serde(default)]
    pub clip_data: Option<ClipData>,
    /// URI permissions to grant to `package` before the launch.
    #[serde(default)]
    pub uri_grants: Vec<UriGrant>,
}

/// Values the variables resolve to for one launch.
struct Values<'a> {
    /// The caller's context.
    ctx: &'a LaunchContext,
    /// Package that will be launched.
    package: &'a Package,
    /// RetroArch core file name, when the profile uses one.
    core_file: Option<&'a str>,
}

/// Part of `path` after the last `/`.
fn file_of(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

impl Values<'_> {
    /// Value of the variable `name`.
    fn lookup(&self, name: &str) -> Result<String, LaunchError> {
        let ctx = self.ctx;
        let missing = || LaunchError::MissingVariable(name.to_string());
        let opt = |v: &Option<String>| v.clone().ok_or_else(missing);
        if let Some(tag) = name.strip_prefix(TAG_PREFIX) {
            return ctx
                .tags
                .get(tag)
                .cloned()
                .ok_or_else(|| LaunchError::MissingTag(tag.to_string()));
        }
        let file = file_of(&ctx.rom_path);
        match name {
            "rom" => Ok(ctx.rom_path.clone()),
            "rom_uri" => opt(&ctx.rom_uris.file),
            "rom_saf" => opt(&ctx.rom_uris.saf),
            "rom_saf_tree" => opt(&ctx.rom_uris.saf_tree),
            "rom_provider" => opt(&ctx.rom_uris.provider),
            "rom_file" => Ok(file.to_string()),
            "rom_name" => Ok(file
                .rsplit_once('.')
                .map_or(file, |(stem, _)| stem)
                .to_string()),
            "rom_dir" => Ok(ctx
                .rom_path
                .rsplit_once('/')
                .map_or("", |(dir, _)| dir)
                .to_string()),
            "platform" => Ok(ctx.platform.clone()),
            "platform_dir" => opt(&ctx.platform_dir),
            "package" => Ok(self.package.package.clone()),
            "core" => self.core_file.map(str::to_string).ok_or_else(missing),
            "core_path" => self
                .core_file
                .map(|f| self.core_path(f))
                .ok_or_else(missing),
            _ => Err(LaunchError::UnknownVariable(name.to_string())),
        }
    }

    /// Absolute path of a core file inside the installed RetroArch.
    fn core_path(&self, file: &str) -> String {
        let dir = RETROARCH_CORE_DIR.replace("{package}", &self.package.package);
        format!("{dir}/{file}")
    }

    /// Substitutes every variable of `text`.
    fn render(&self, text: &str) -> Result<String, LaunchError> {
        let mut out = String::new();
        for part in template::parse(text)? {
            match part {
                Part::Text(t) => out.push_str(t),
                Part::Var(name) => out.push_str(&self.lookup(name)?),
            }
        }
        Ok(out)
    }

    /// Substitutes an optional template.
    fn render_opt(&self, text: &Option<String>) -> Result<Option<String>, LaunchError> {
        text.as_deref().map(|t| self.render(t)).transpose()
    }

    /// Substitutes one extra.
    fn extra(&self, extra: &ExtraTemplate) -> Result<Extra, LaunchError> {
        let value = match &extra.value {
            ExtraTemplateValue::String { value } => ExtraValue::String {
                value: self.render(value)?,
            },
            ExtraTemplateValue::Bool { value } => ExtraValue::Bool { value: *value },
            ExtraTemplateValue::Int {
                value: IntTemplate::Literal(n),
            } => ExtraValue::Int { value: *n },
            ExtraTemplateValue::Int {
                value: IntTemplate::Template(t),
            } => {
                let text = self.render(t)?;
                let n = text
                    .trim()
                    .parse()
                    .map_err(|_| LaunchError::InvalidInteger(extra.key.clone()))?;
                ExtraValue::Int { value: n }
            }
            ExtraTemplateValue::StringArray { value } => ExtraValue::StringArray {
                value: value
                    .iter()
                    .map(|v| self.render(v))
                    .collect::<Result<_, _>>()?,
            },
        };
        Ok(Extra {
            key: extra.key.clone(),
            value,
        })
    }

    /// Substitutes the grants; an optional grant whose variable is missing is dropped.
    fn grants(&self, it: &IntentTemplate) -> Result<Vec<UriGrant>, LaunchError> {
        let mut out = Vec::new();
        for g in &it.uri_grants {
            match self.render(&g.uri) {
                Ok(uri) => out.push(UriGrant {
                    uri,
                    read: g.read,
                    write: g.write,
                }),
                Err(LaunchError::MissingVariable(_)) if g.optional => {}
                Err(e) => return Err(e),
            }
        }
        Ok(out)
    }
}

/// The package to launch: the requested one or the emulator's first.
fn pick_package<'a>(
    emulator: &'a Emulator,
    wanted: Option<&str>,
) -> Result<&'a Package, LaunchError> {
    match wanted {
        Some(w) => emulator
            .packages
            .iter()
            .find(|p| p.package == w)
            .ok_or_else(|| LaunchError::UnknownPackage(w.to_string())),
        None => emulator
            .packages
            .first()
            .ok_or_else(|| LaunchError::UnknownPackage(emulator.id.clone())),
    }
}

/// The core file to use: the requested core id or the platform's first.
fn pick_core<'a>(emulator: &'a Emulator, ctx: &LaunchContext) -> Result<&'a str, LaunchError> {
    let cores = emulator
        .cores
        .get(&ctx.platform)
        .map_or(&[][..], Vec::as_slice);
    let core = match &ctx.core {
        Some(id) => cores
            .iter()
            .find(|c| &c.id == id)
            .ok_or_else(|| LaunchError::UnknownCore(id.clone()))?,
        None => cores
            .first()
            .ok_or_else(|| LaunchError::NoCore(ctx.platform.clone()))?,
    };
    Ok(&core.file)
}

/// Builds the intent that launches a game with `profile` of `emulator`. Every `{variable}` is
/// substituted; an unknown one, a missing tag, or a ROM form the caller did not supply is an
/// error, never an empty string.
pub fn build_launch(
    emulator: &Emulator,
    profile: &Profile,
    ctx: &LaunchContext,
) -> Result<LaunchIntent, LaunchError> {
    if !profile.platforms.contains(&ctx.platform) {
        return Err(LaunchError::ProfilePlatformMismatch {
            profile: profile.id.clone(),
            platform: ctx.platform.clone(),
        });
    }
    let package = pick_package(emulator, ctx.package.as_deref())?;
    let core_file = if profile.uses_core {
        Some(pick_core(emulator, ctx)?)
    } else {
        None
    };
    let values = Values {
        ctx,
        package,
        core_file,
    };
    let it = &profile.intent;
    Ok(LaunchIntent {
        action: values.render_opt(&it.action)?,
        package: package.package.clone(),
        activity: package.activity.clone(),
        categories: it.category.clone(),
        data: values.render_opt(&it.data)?,
        mime: it.mime.clone(),
        extras: it
            .extras
            .iter()
            .map(|x| values.extra(x))
            .collect::<Result<_, _>>()?,
        flags: it.flags.clone(),
        clip_data: it
            .clip_data
            .as_ref()
            .map(|c| {
                Ok::<_, LaunchError>(ClipData {
                    uri: values.render(&c.uri)?,
                    mime: c.mime.clone(),
                })
            })
            .transpose()?,
        uri_grants: values.grants(it)?,
    })
}
