use crate::core::Edition;
use crate::util::config::Config;
use crate::util::restricted_names;
use crate::CargoResult;
use std::fmt::Write;

const DEFAULT_VERSION: &str = "0.0.0";
const DEFAULT_PUBLISH: bool = false;

pub fn extract_manifest(s: &str, path: &std::path::Path, config: &Config) -> CargoResult<String> {
    let file = syn::parse_file(&s)?;
    let mut lits = Vec::new();
    for attr in &file.attrs {
        if attr.meta.path().is_ident("doc") {
            let syn::Meta::NameValue(nv) = &attr.meta else {
                anyhow::bail!("unsupported attr meta for {:?}", attr.meta.path())
            };
            let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(lit), .. }) = &nv.value else {
                anyhow::bail!("only string literals are supported")
            };
            lits.push(lit);
        }
    }

    let mut embedded_manifest = String::new();
    // /*! (Inner block doc comment) are treated as a single line
    if lits.len() == 1 {
        let lit = lits.first().unwrap();
        // split it up so we can process each line
        lit.value().split("\n").for_each(|s| {
            // Remove leading comment section and surrounding whitespace
            let s = s.trim_start_matches(" * ").trim();
            // Skip markdown code fences and empty strings
            if !s.contains("```") && !s.is_empty() {
                writeln!(&mut embedded_manifest, "{s}").unwrap();
            }
        });
    } else {
        // //! (inner line doc comment) are treated as separate lines
        for lit in lits {
            let s = lit.value();
            let s = s.trim();
            // Skip markdown code fences and empty strings
            if !s.contains("```") && !s.is_empty() {
                writeln!(&mut embedded_manifest, "{s}").unwrap();
            }
        }
    }

    let expanded = expand_manifest(embedded_manifest, path, config)?;
    let manifest = toml::to_string_pretty(&expanded)?;
    Ok(manifest)
}

fn expand_manifest(
    manifest: String,
    path: &std::path::Path,
    config: &Config,
) -> CargoResult<toml::Table> {
    let mut manifest: toml::Table = toml::from_str(&manifest)?;

    for key in ["workspace", "lib", "bin", "example", "test", "bench"] {
        if manifest.contains_key(key) {
            anyhow::bail!("`{key}` is not allowed in embedded manifests")
        }
    }

    // Prevent looking for a workspace
    manifest.insert("workspace".to_owned(), toml::Table::new().into());

    let package = manifest
        .entry("package".to_owned())
        .or_insert_with(|| toml::Table::new().into())
        .as_table_mut()
        .ok_or_else(|| anyhow::format_err!("`package` must be a table"))?;

    for key in [
        "workspace",
        "build",
        "links",
        "autobins",
        "autoexamples",
        "autotests",
        "autobenches",
    ] {
        if package.contains_key(key) {
            anyhow::bail!("`package.{key}` is not allowed in embedded manifests")
        }
    }

    let name = package_name(path)?;
    package
        .entry("name".to_owned())
        .or_insert(toml::Value::String(name.clone()));
    package
        .entry("version".to_owned())
        .or_insert_with(|| toml::Value::String(DEFAULT_VERSION.to_owned()));
    package.entry("edition".to_owned()).or_insert_with(|| {
        let _ = config.shell().warn(format_args!(
            "`package.edition` is unspecified, defaulting to `{}`",
            Edition::LATEST_STABLE
        ));
        toml::Value::String(Edition::LATEST_STABLE.to_string())
    });
    // Avoid accidental publishes
    package
        .entry("publish".to_owned())
        .or_insert_with(|| toml::Value::Boolean(DEFAULT_PUBLISH));

    // Since there is no `main.rs`, so we must add the name of the file to `[[bins]]`
    let mut bin = toml::Table::new();
    bin.insert("name".to_owned(), toml::Value::String(name));
    bin.insert(
        "path".to_owned(),
        toml::Value::String(
            path.to_str()
                .ok_or_else(|| anyhow::format_err!("path is not valid UTF-8"))?
                .into(),
        ),
    );
    manifest.insert(
        "bin".to_owned(),
        toml::Value::Array(vec![toml::Value::Table(bin)]),
    );

    let release = manifest
        .entry("profile".to_owned())
        .or_insert_with(|| toml::Value::Table(Default::default()))
        .as_table_mut()
        .ok_or_else(|| anyhow::format_err!("`profile` must be a table"))?
        .entry("release".to_owned())
        .or_insert_with(|| toml::Value::Table(Default::default()))
        .as_table_mut()
        .ok_or_else(|| anyhow::format_err!("`profile.release` must be a table"))?;
    release
        .entry("strip".to_owned())
        .or_insert_with(|| toml::Value::Boolean(true));

    Ok(manifest)
}

/// Gets a valid package name from a single-file package path
fn package_name(path: &std::path::Path) -> CargoResult<String> {
    let name = path
        .file_stem()
        .ok_or_else(|| anyhow::format_err!("no file name"))?
        .to_string_lossy();
    let mut slug = String::new();
    for (i, c) in name.chars().enumerate() {
        match (i, c) {
            (0, '0'..='9') => {
                slug.push('_');
                slug.push(c);
            }
            (_, '0'..='9') | (_, 'a'..='z') | (_, '_') | (_, '-') => {
                slug.push(c);
            }
            (_, 'A'..='Z') => {
                // Convert uppercase characters to lowercase to avoid `non_snake_case` warnings.
                slug.push(c.to_ascii_lowercase());
            }
            (_, _) => {
                slug.push('_');
            }
        }
    }

    // This copies the logic from `cargo new` to ensure that the name is valid,
    // but it just modifies the name instead of throwing an error.
    if restricted_names::is_keyword(&slug)
        || restricted_names::is_conflicting_artifact_name(&slug)
        || (cfg!(windows) && restricted_names::is_windows_reserved(&slug))
        || &slug == "test"
        || ["core", "std", "alloc", "proc_macro", "proc-macro"].contains(&slug.as_str())
    {
        slug.insert(0, '_');
    }
    Ok(slug)
}
