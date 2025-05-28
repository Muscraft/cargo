use crate::core::{Edition, Feature, Features, Manifest, Package};
use crate::{CargoResult, GlobalContext};
use annotate_snippets::{AnnotationKind, Group, Level, Snippet};
use cargo_util_schemas::manifest::{TomlLintLevel, TomlToolLints};
use pathdiff::diff_paths;
use std::cmp::Ordering;
use std::fmt::Display;
use std::ops::Range;
use std::path::Path;

const LINT_GROUPS: &[LintGroup] = &[
    CORRECTNESS,
    NURSERY,
    PEDANTIC,
    RESTRICTION,
    STYLE,
    SUSPICIOUS,
    TEST_DUMMY_UNSTABLE,
];
pub const LINTS: &[Lint] = &[IM_A_TEAPOT, UNKNOWN_LINTS];

#[derive(Clone)]
pub struct TomlSpan {
    pub key: Range<usize>,
    pub value: Range<usize>,
}

pub fn get_key_value_span(
    document: &toml::Spanned<toml::de::DeTable<'static>>,
    path: &[&str],
) -> Option<TomlSpan> {
    let mut table = document.get_ref();
    let mut iter = path.into_iter().peekable();
    while let Some(key) = iter.next() {
        let key_s: &str = key.as_ref();
        let (key, item) = table.get_key_value(key_s)?;
        if iter.peek().is_none() {
            return Some(TomlSpan {
                key: key.span(),
                value: item.span(),
            });
        }
        if let Some(next_table) = item.get_ref().as_table() {
            table = next_table;
        }
        if iter.peek().is_some() {
            if let Some(array) = item.get_ref().as_array() {
                let next = iter.next().unwrap();
                return array.iter().find_map(|item| match item.get_ref() {
                    toml::de::DeValue::String(s) if s == next => Some(TomlSpan {
                        key: key.span(),
                        value: item.span(),
                    }),
                    _ => None,
                });
            }
        }
    }
    None
}

/// Gets the relative path to a manifest from the current working directory, or
/// the absolute path of the manifest if a relative path cannot be constructed
pub fn rel_cwd_manifest_path(path: &Path, gctx: &GlobalContext) -> String {
    diff_paths(path, gctx.cwd())
        .unwrap_or_else(|| path.to_path_buf())
        .display()
        .to_string()
}

#[derive(Copy, Clone, Debug)]
pub struct LintGroup {
    pub name: &'static str,
    pub default_level: LintLevel,
    pub desc: &'static str,
    pub feature_gate: Option<&'static Feature>,
}

const CORRECTNESS: LintGroup = LintGroup {
    name: "correctness",
    desc: "code that is outright wrong or useless",
    default_level: LintLevel::Deny,
    feature_gate: None,
};

const NURSERY: LintGroup = LintGroup {
    name: "nursery",
    desc: "new lints that are still under development",
    default_level: LintLevel::Allow,
    feature_gate: None,
};

const PEDANTIC: LintGroup = LintGroup {
    name: "pedantic",
    desc: "lints which are rather strict or have occasional false positives",
    default_level: LintLevel::Allow,
    feature_gate: None,
};

const RESTRICTION: LintGroup = LintGroup {
    name: "restriction",
    desc: "lints which prevent the use of language and library features",
    default_level: LintLevel::Allow,
    feature_gate: None,
};

const STYLE: LintGroup = LintGroup {
    name: "style",
    desc: "code that should be written in a more idiomatic wa",
    default_level: LintLevel::Warn,
    feature_gate: None,
};

const SUSPICIOUS: LintGroup = LintGroup {
    name: "suspicious",
    desc: "code that is most likely wrong or useless",
    default_level: LintLevel::Warn,
    feature_gate: None,
};

/// This lint group is only to be used for testing purposes
const TEST_DUMMY_UNSTABLE: LintGroup = LintGroup {
    name: "test_dummy_unstable",
    desc: "test_dummy_unstable is meant to only be used in tests",
    default_level: LintLevel::Allow,
    feature_gate: Some(Feature::test_dummy_unstable()),
};

#[derive(Copy, Clone, Debug)]
pub struct Lint {
    pub name: &'static str,
    pub desc: &'static str,
    pub primary_group: &'static LintGroup,
    pub edition_lint_opts: Option<(Edition, LintLevel)>,
    pub feature_gate: Option<&'static Feature>,
    /// This is a markdown formatted string that will be used when generating
    /// the lint documentation. If docs is `None`, the lint will not be
    /// documented.
    pub docs: Option<&'static str>,
}

impl Lint {
    pub fn level(
        &self,
        pkg_lints: &TomlToolLints,
        edition: Edition,
        unstable_features: &Features,
    ) -> (LintLevel, LintLevelReason) {
        // We should return `Allow` if a lint is behind a feature, but it is
        // not enabled, that way the lint does not run.
        if self
            .feature_gate
            .is_some_and(|f| !unstable_features.is_enabled(f))
        {
            return (LintLevel::Allow, LintLevelReason::Default);
        }

        let lint = pkg_lints.get(self.name);

        let group = pkg_lints.get(self.primary_group.name);

        let edition_level = self
            .edition_lint_opts
            .as_ref()
            .and_then(|(e, l)| if edition >= *e { Some(l) } else { None });

        let default_level = self.primary_group.default_level;

        // Feature Gate > Forbid > Defined > Lint Edition > Group Default
        //
        // Lint vs Group comes down to priority, if they are equal the lint
        // takes precedence, as it is more specific than the group.
        match (lint, group, edition_level) {
            (Some(lint), _, _) if lint.level() == TomlLintLevel::Forbid => {
                (lint.level().into(), LintLevelReason::Package)
            }
            (_, Some(group), _) if group.level() == TomlLintLevel::Forbid => {
                (group.level().into(), LintLevelReason::Package)
            }
            (_, _, Some(edition_level)) if edition_level == &LintLevel::Forbid => {
                (*edition_level, LintLevelReason::Edition(edition))
            }
            (_, _, _) if default_level == LintLevel::Forbid => {
                (default_level, LintLevelReason::Default)
            }
            (Some(lint), Some(group), _) => {
                // If both the lint and group are defined, we compare their
                // priorities to see which one should take precedence
                let level = match lint.priority().cmp(&group.priority()) {
                    Ordering::Greater => lint.level(),
                    // In the case of equal priority, we prefer the lint itself as
                    // it is more specific than the group
                    Ordering::Equal => lint.level(),
                    Ordering::Less => group.level(),
                };
                (level.into(), LintLevelReason::Package)
            }
            (Some(lint), None, _) => (lint.level().into(), LintLevelReason::Package),
            (None, Some(group), _) => (group.level().into(), LintLevelReason::Package),
            (None, None, Some(edition_level)) => {
                (*edition_level, LintLevelReason::Edition(edition))
            }
            (None, None, None) => (default_level, LintLevelReason::Default),
        }
    }

    fn emitted_source(&self, lint_level: LintLevel, reason: LintLevelReason) -> String {
        format!("`cargo::{}` is set to `{lint_level}` {reason}", self.name,)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LintLevel {
    Allow,
    Warn,
    Deny,
    Forbid,
}

impl Display for LintLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LintLevel::Allow => write!(f, "allow"),
            LintLevel::Warn => write!(f, "warn"),
            LintLevel::Deny => write!(f, "deny"),
            LintLevel::Forbid => write!(f, "forbid"),
        }
    }
}

impl LintLevel {
    pub fn is_error(&self) -> bool {
        self == &LintLevel::Forbid || self == &LintLevel::Deny
    }

    pub fn to_diagnostic_level(self) -> Level<'static> {
        match self {
            LintLevel::Allow => unreachable!("allow does not map to a diagnostic level"),
            LintLevel::Warn => Level::WARNING,
            LintLevel::Deny => Level::ERROR,
            LintLevel::Forbid => Level::ERROR,
        }
    }

    fn force(self) -> bool {
        match self {
            Self::Allow => false,
            Self::Warn => true,
            Self::Deny => true,
            Self::Forbid => true,
        }
    }
}

impl From<TomlLintLevel> for LintLevel {
    fn from(toml_lint_level: TomlLintLevel) -> LintLevel {
        match toml_lint_level {
            TomlLintLevel::Allow => LintLevel::Allow,
            TomlLintLevel::Warn => LintLevel::Warn,
            TomlLintLevel::Deny => LintLevel::Deny,
            TomlLintLevel::Forbid => LintLevel::Forbid,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LintLevelReason {
    Default,
    Edition(Edition),
    Package,
}

impl Display for LintLevelReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LintLevelReason::Default => write!(f, "by default"),
            LintLevelReason::Edition(edition) => write!(f, "in edition {}", edition),
            LintLevelReason::Package => write!(f, "in `[lints]`"),
        }
    }
}

pub fn analyze_cargo_lints_table(
    pkg: &Package,
    path: &Path,
    pkg_lints: &TomlToolLints,
    ws_contents: &str,
    ws_document: &toml::Spanned<toml::de::DeTable<'static>>,
    ws_path: &Path,
    gctx: &GlobalContext,
) -> CargoResult<()> {
    let mut error_count = 0;
    let manifest = pkg.manifest();
    let manifest_path = rel_cwd_manifest_path(path, gctx);
    let ws_path = rel_cwd_manifest_path(ws_path, gctx);
    let mut unknown_lints = Vec::new();
    for lint_name in pkg_lints.keys().map(|name| name) {
        let (name, feature_gate) = if let Some(lint) = LINTS.iter().find(|l| l.name == lint_name) {
            (lint.name, &lint.feature_gate)
        } else if let Some(group) = LINT_GROUPS.iter().find(|g| g.name == lint_name) {
            (group.name, &group.feature_gate)
        } else {
            unknown_lints.push(lint_name);
            continue;
        };

        // Only run this on lints that are gated by a feature
        if let Some(feature_gate) = feature_gate {
            verify_feature_enabled(
                name,
                feature_gate,
                manifest,
                &manifest_path,
                ws_contents,
                ws_document,
                &ws_path,
                &mut error_count,
                gctx,
            )?;
        }
    }

    output_unknown_lints(
        unknown_lints,
        manifest,
        &manifest_path,
        pkg_lints,
        ws_contents,
        ws_document,
        &ws_path,
        &mut error_count,
        gctx,
    )?;

    if error_count > 0 {
        Err(anyhow::anyhow!(
            "encountered {error_count} errors(s) while verifying lints",
        ))
    } else {
        Ok(())
    }
}

fn verify_feature_enabled(
    lint_name: &str,
    feature_gate: &Feature,
    manifest: &Manifest,
    manifest_path: &str,
    ws_contents: &str,
    ws_document: &toml::Spanned<toml::de::DeTable<'static>>,
    ws_path: &str,
    error_count: &mut usize,
    gctx: &GlobalContext,
) -> CargoResult<()> {
    if !manifest.unstable_features().is_enabled(feature_gate) {
        let dash_feature_name = feature_gate.name().replace("_", "-");
        let title = format!("use of unstable lint `{}`", lint_name);
        let label = format!(
            "this is behind `{}`, which is not enabled",
            dash_feature_name
        );
        let second_title = format!("`cargo::{}` was inherited", lint_name);
        let help = format!(
            "consider adding `cargo-features = [\"{}\"]` to the top of the manifest",
            dash_feature_name
        );

        let (contents, path, span) = if let Some(span) =
            get_key_value_span(manifest.document(), &["lints", "cargo", lint_name])
        {
            (manifest.contents(), manifest_path, span)
        } else if let Some(lint_span) =
            get_key_value_span(ws_document, &["workspace", "lints", "cargo", lint_name])
        {
            (ws_contents, ws_path, lint_span)
        } else {
            panic!("could not find `cargo::{lint_name}` in `[lints]`, or `[workspace.lints]` ")
        };

        let mut report = Vec::new();
        report.push(
            Group::with_title(Level::ERROR.primary_title(title))
                .element(
                    Snippet::source(contents)
                        .path(path)
                        .annotation(AnnotationKind::Primary.span(span.key).label(label)),
                )
                .element(Level::HELP.message(help)),
        );

        if let Some(inherit_span) = get_key_value_span(manifest.document(), &["lints", "workspace"])
        {
            report.push(
                Group::with_title(Level::NOTE.secondary_title(second_title)).element(
                    Snippet::source(manifest.contents())
                        .path(manifest_path)
                        .annotation(
                            AnnotationKind::Context
                                .span(inherit_span.key.start..inherit_span.value.end),
                        ),
                ),
            );
        }

        *error_count += 1;
        gctx.shell().print_report(&report, false)?;
    }
    Ok(())
}

/// This lint is only to be used for testing purposes
const IM_A_TEAPOT: Lint = Lint {
    name: "im_a_teapot",
    desc: "`im_a_teapot` is specified",
    primary_group: &TEST_DUMMY_UNSTABLE,
    edition_lint_opts: None,
    feature_gate: Some(Feature::test_dummy_unstable()),
    docs: None,
};

pub fn check_im_a_teapot(
    pkg: &Package,
    path: &Path,
    pkg_lints: &TomlToolLints,
    error_count: &mut usize,
    gctx: &GlobalContext,
) -> CargoResult<()> {
    let manifest = pkg.manifest();
    let (lint_level, reason) =
        IM_A_TEAPOT.level(pkg_lints, manifest.edition(), manifest.unstable_features());

    if lint_level == LintLevel::Allow {
        return Ok(());
    }

    if manifest
        .normalized_toml()
        .package()
        .is_some_and(|p| p.im_a_teapot.is_some())
    {
        if lint_level.is_error() {
            *error_count += 1;
        }
        let level = lint_level.to_diagnostic_level();
        let manifest_path = rel_cwd_manifest_path(path, gctx);
        let emitted_reason = IM_A_TEAPOT.emitted_source(lint_level, reason);

        let span = get_key_value_span(manifest.document(), &["package", "im-a-teapot"]).unwrap();

        let report = &[Group::with_title(level.primary_title(IM_A_TEAPOT.desc))
            .element(
                Snippet::source(manifest.contents())
                    .path(&manifest_path)
                    .annotation(AnnotationKind::Primary.span(span.key.start..span.value.end)),
            )
            .element(Level::NOTE.message(&emitted_reason))];

        gctx.shell().print_report(report, lint_level.force())?;
    }
    Ok(())
}

const UNKNOWN_LINTS: Lint = Lint {
    name: "unknown_lints",
    desc: "unknown lint",
    primary_group: &SUSPICIOUS,
    edition_lint_opts: None,
    feature_gate: None,
    docs: Some(
        r#"
### What it does
Checks for unknown lints in the `[lints.cargo]` table

### Why it is bad
- The lint name could be misspelled, leading to confusion as to why it is
  not working as expected
- The unknown lint could end up causing an error if `cargo` decides to make
  a lint with the same name in the future

### Example
```toml
[lints.cargo]
this-lint-does-not-exist = "warn"
```
"#,
    ),
};

fn output_unknown_lints(
    unknown_lints: Vec<&String>,
    manifest: &Manifest,
    manifest_path: &str,
    pkg_lints: &TomlToolLints,
    ws_contents: &str,
    ws_document: &toml::Spanned<toml::de::DeTable<'static>>,
    ws_path: &str,
    error_count: &mut usize,
    gctx: &GlobalContext,
) -> CargoResult<()> {
    let (lint_level, reason) =
        UNKNOWN_LINTS.level(pkg_lints, manifest.edition(), manifest.unstable_features());
    if lint_level == LintLevel::Allow {
        return Ok(());
    }

    let level = lint_level.to_diagnostic_level();
    let mut emitted_source = None;
    for lint_name in unknown_lints {
        if lint_level.is_error() {
            *error_count += 1;
        }
        let title = format!("{}: `{lint_name}`", UNKNOWN_LINTS.desc);
        let second_title = format!("`cargo::{}` was inherited", lint_name);
        let underscore_lint_name = lint_name.replace("-", "_");
        let matching = if let Some(lint) = LINTS.iter().find(|l| l.name == underscore_lint_name) {
            Some((lint.name, "lint"))
        } else if let Some(group) = LINT_GROUPS.iter().find(|g| g.name == underscore_lint_name) {
            Some((group.name, "group"))
        } else {
            None
        };
        let help =
            matching.map(|(name, kind)| format!("there is a {kind} with a similar name: `{name}`"));

        let (contents, path, span) = if let Some(span) =
            get_key_value_span(manifest.document(), &["lints", "cargo", lint_name])
        {
            (manifest.contents(), manifest_path, span)
        } else if let Some(lint_span) =
            get_key_value_span(ws_document, &["workspace", "lints", "cargo", lint_name])
        {
            (ws_contents, ws_path, lint_span)
        } else {
            panic!("could not find `cargo::{lint_name}` in `[lints]`, or `[workspace.lints]` ")
        };

        let mut report = Vec::new();
        let mut group = Group::with_title(level.clone().primary_title(title)).element(
            Snippet::source(contents)
                .path(path)
                .annotation(AnnotationKind::Primary.span(span.key)),
        );
        if emitted_source.is_none() {
            emitted_source = Some(UNKNOWN_LINTS.emitted_source(lint_level, reason));
            group = group.element(Level::NOTE.message(emitted_source.as_ref().unwrap()));
        }
        if let Some(help) = help.as_ref() {
            group = group.element(Level::HELP.message(help));
        }
        report.push(group);

        if let Some(inherit_span) = get_key_value_span(manifest.document(), &["lints", "workspace"])
        {
            report.push(
                Group::with_title(Level::NOTE.secondary_title(second_title)).element(
                    Snippet::source(manifest.contents())
                        .path(manifest_path)
                        .annotation(
                            AnnotationKind::Context
                                .span(inherit_span.key.start..inherit_span.value.end),
                        ),
                ),
            );
        }

        gctx.shell().print_report(&report, lint_level.force())?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use snapbox::ToDebug;
    use std::collections::HashSet;

    #[test]
    fn ensure_sorted_lints() {
        // This will be printed out if the fields are not sorted.
        let location = std::panic::Location::caller();
        println!("\nTo fix this test, sort `LINTS` in {}\n", location.file(),);

        let actual = super::LINTS
            .iter()
            .map(|l| l.name.to_uppercase())
            .collect::<Vec<_>>();

        let mut expected = actual.clone();
        expected.sort();
        snapbox::assert_data_eq!(actual.to_debug(), expected.to_debug());
    }

    #[test]
    fn ensure_sorted_lint_groups() {
        // This will be printed out if the fields are not sorted.
        let location = std::panic::Location::caller();
        println!(
            "\nTo fix this test, sort `LINT_GROUPS` in {}\n",
            location.file(),
        );
        let actual = super::LINT_GROUPS
            .iter()
            .map(|l| l.name.to_uppercase())
            .collect::<Vec<_>>();

        let mut expected = actual.clone();
        expected.sort();
        snapbox::assert_data_eq!(actual.to_debug(), expected.to_debug());
    }

    #[test]
    fn ensure_updated_lints() {
        let path = snapbox::utils::current_rs!();
        let expected = std::fs::read_to_string(&path).unwrap();
        let expected = expected
            .lines()
            .filter_map(|l| {
                if l.ends_with(": Lint = Lint {") {
                    Some(
                        l.chars()
                            .skip(6)
                            .take_while(|c| *c != ':')
                            .collect::<String>(),
                    )
                } else {
                    None
                }
            })
            .collect::<HashSet<_>>();
        let actual = super::LINTS
            .iter()
            .map(|l| l.name.to_uppercase())
            .collect::<HashSet<_>>();
        let diff = expected.difference(&actual).sorted().collect::<Vec<_>>();

        let mut need_added = String::new();
        for name in &diff {
            need_added.push_str(&format!("{}\n", name));
        }
        assert!(
            diff.is_empty(),
            "\n`LINTS` did not contain all `Lint`s found in {}\n\
            Please add the following to `LINTS`:\n\
            {}",
            path.display(),
            need_added
        );
    }

    #[test]
    fn ensure_updated_lint_groups() {
        let path = snapbox::utils::current_rs!();
        let expected = std::fs::read_to_string(&path).unwrap();
        let expected = expected
            .lines()
            .filter_map(|l| {
                if l.ends_with(": LintGroup = LintGroup {") {
                    Some(
                        l.chars()
                            .skip(6)
                            .take_while(|c| *c != ':')
                            .collect::<String>(),
                    )
                } else {
                    None
                }
            })
            .collect::<HashSet<_>>();
        let actual = super::LINT_GROUPS
            .iter()
            .map(|l| l.name.to_uppercase())
            .collect::<HashSet<_>>();
        let diff = expected.difference(&actual).sorted().collect::<Vec<_>>();

        let mut need_added = String::new();
        for name in &diff {
            need_added.push_str(&format!("{}\n", name));
        }
        assert!(
            diff.is_empty(),
            "\n`LINT_GROUPS` did not contain all `LintGroup`s found in {}\n\
            Please add the following to `LINT_GROUPS`:\n\
            {}",
            path.display(),
            need_added
        );
    }
}
