//! The output switches on disk (D-162): `switches.toml` in the config
//! directory keeps the `r` and `m` switches between launches. The app writes
//! it each time a switch flips, so `config.toml` stays the user's own file
//! and is never rewritten.
//!
//! A missing file gives the defaults (repair on, markdown off). A file that
//! cannot be read, is not TOML, holds a value that is not a boolean or turns
//! both switches off gives the default for what it touched and a warning
//! line for the debug log, never an error.

use std::fs;
use std::io::{self, Write};
use std::path::Path;

use crate::jobs::Outputs;

/// The file's name inside the config directory.
pub(crate) const FILE: &str = "switches.toml";

const REPAIR: &str = "repair";
const MARKDOWN: &str = "markdown";

/// The switches in `path`, and a warning for each part that fell back to its
/// default.
pub(crate) fn load(path: &Path) -> (Outputs, Vec<String>) {
    let defaults = Outputs::default();
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return (defaults, Vec::new()),
        Err(e) => {
            let why = format!(
                "switches: can't read {}: {e}; using the defaults",
                path.display()
            );
            return (defaults, vec![why]);
        }
    };
    let table = match text.parse::<toml::Table>() {
        Ok(table) => table,
        Err(e) => {
            let first = e.to_string();
            let first = first.lines().next().unwrap_or_default().to_owned();
            let why = format!(
                "switches: {} is not TOML ({first}); using the defaults",
                path.display()
            );
            return (defaults, vec![why]);
        }
    };
    let mut warnings = Vec::new();
    let mut flag = |key: &str, default: bool| match table.get(key) {
        None => default,
        Some(toml::Value::Boolean(b)) => *b,
        Some(other) => {
            warnings.push(format!(
                "switches: `{key}` is a {}, not true or false; using {default}",
                other.type_str()
            ));
            default
        }
    };
    let mut outputs = Outputs {
        repaired_pdf: flag(REPAIR, defaults.repaired_pdf),
        markdown: flag(MARKDOWN, defaults.markdown),
    };
    for key in table
        .keys()
        .filter(|k| ![REPAIR, MARKDOWN].contains(&k.as_str()))
    {
        warnings.push(format!("switches: unknown key `{key}` ignored"));
    }
    if !outputs.repaired_pdf && !outputs.markdown {
        warnings.push("switches: both are off, so a drop would make nothing; repair is on".into());
        outputs.repaired_pdf = true;
    }
    (outputs, warnings)
}

/// Writes `outputs` to `path`, creating its directory. The text goes to a
/// temporary file beside it, is flushed to disk, then replaces the old file,
/// so a crash mid-write leaves the old switches. A crash between the two can
/// leave the temporary file behind; nothing reads it.
pub(crate) fn save(path: &Path, outputs: Outputs) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let text = format!(
        "# PDFPundit's r and m switches. The app rewrites this file when one flips;\n\
         # settings you edit by hand belong in config.toml.\n\
         {REPAIR} = {}\n\
         {MARKDOWN} = {}\n",
        outputs.repaired_pdf, outputs.markdown
    );
    let temp = path.with_extension(format!("toml.{}.tmp", std::process::id()));
    let written = fs::File::create(&temp).and_then(|mut file| {
        file.write_all(text.as_bytes())?;
        file.sync_all()
    });
    written
        .and_then(|()| fs::rename(&temp, path))
        .inspect_err(|_| {
            let _ = fs::remove_file(&temp);
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::place::ScratchDir;

    const BOTH: Outputs = Outputs {
        repaired_pdf: true,
        markdown: true,
    };
    const MARKDOWN_ONLY: Outputs = Outputs {
        repaired_pdf: false,
        markdown: true,
    };

    #[test]
    fn a_missing_file_gives_the_defaults_quietly() {
        let dir = ScratchDir::new("switches-missing");
        let (o, warnings) = load(&dir.join(FILE));
        assert_eq!(o, Outputs::default());
        assert!(warnings.is_empty(), "{warnings:?}");
    }

    #[test]
    fn what_is_saved_loads_back() {
        let dir = ScratchDir::new("switches-round-trip");
        let path = dir.join("config").join(FILE);
        for o in [BOTH, MARKDOWN_ONLY, Outputs::default()] {
            save(&path, o).expect("save");
            assert_eq!(load(&path), (o, Vec::new()));
        }
        let left: Vec<_> = fs::read_dir(dir.join("config"))
            .expect("dir")
            .map(|e| e.expect("entry").file_name())
            .collect();
        assert_eq!(left, [FILE], "no temp file left");
    }

    #[test]
    fn a_bad_file_falls_back_and_says_why() {
        let dir = ScratchDir::new("switches-bad");
        let path = dir.join(FILE);
        let cases: [(&str, Outputs, &str); 4] = [
            ("repair = [", Outputs::default(), "is not TOML"),
            ("repair = \"no\"\nmarkdown = true", BOTH, "is a string"),
            ("markdown = true\nmood = 3", BOTH, "unknown key `mood`"),
            (
                "repair = false\nmarkdown = false",
                Outputs::default(),
                "both are off",
            ),
        ];
        for (text, want, why) in cases {
            fs::write(&path, text).expect("write");
            let (o, warnings) = load(&path);
            assert_eq!(o, want, "{text:?}");
            assert!(
                warnings.iter().any(|w| w.contains(why)),
                "{text:?}: {warnings:?}"
            );
        }
    }

    #[test]
    fn a_partial_file_keeps_the_default_for_the_missing_key() {
        let dir = ScratchDir::new("switches-partial");
        let path = dir.join(FILE);
        fs::write(&path, "markdown = true\n").expect("write");
        assert_eq!(load(&path), (BOTH, Vec::new()));
    }
}
