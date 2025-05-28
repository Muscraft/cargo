# Lints

Note: [Cargo's linting system is unstable](unstable.md#lintscargo) and can only be used on nightly toolchains



| Group                | Description                                                      | Default level |
|----------------------|------------------------------------------------------------------|---------------|
| `cargo::correctness` | code that is outright wrong or useless                           | deny          |
| `cargo::nursery`     | new lints that are still under development                       | allow         |
| `cargo::pedantic`    | lints which are rather strict or have occasional false positives | allow         |
| `cargo::restriction` | lints which prevent the use of language and library features     | allow         |
| `cargo::style`       | code that should be written in a more idiomatic wa               | warn          |
| `cargo::suspicious`  | code that is most likely wrong or useless                        | warn          |


## Warn-by-default

These lints are all set to the `warn` by default.
- [`unknown_lints`](#unknown_lints)

## `unknown_lints`
Group: `suspicious`

Level: `warn`

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


