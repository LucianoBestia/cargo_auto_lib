// auto_doc_tidy_html_mod

//! make HTML pretty

// region: use statements

use crate::error_mod::ResultWithLibError;

// endregion: use statements

/// Pretty HTML for docs
///
/// The HTML generated by `cargo doc` is ugly and difficult to `git diff`.
/// Tidy HTML is a HTML checker and formatter installed on most Linuxes.
pub fn auto_doc_tidy_html() -> ResultWithLibError<()> {
    // First we check if tidy is installed on the system
    // Run a dummy command and write the std/err output to tidy_warnings.txt.
    // The command `2>` will overwrite the file and not append like `2>>`.
    crate::run_shell_command_static("tidy xxx 2> docs/tidy_warnings.txt").unwrap_or_else(|e| panic!("{e}"));
    // Check if it contains `command not found`
    let text = std::fs::read_to_string("docs/tidy_warnings.txt")?;
    // don't need this file anymore
    crate::run_shell_command_static("rm -f docs/tidy_warnings.txt").unwrap_or_else(|e| panic!("{e}"));
    if !text.contains("command not found") {
        // Use tidy HTML to format the docs/*.html files to be human readable and usable for git diff.
        // Options: -m modify file, -q quiet suppress nonessential output, -w wrap at 160, -i indent 2 spaces
        // The warnings and errors are appended to the file docs/tidy_warnings.txt
        crate::run_shell_command_static(r#"find ./docs -name '*.html' -type f -print -exec tidy -mq --tidy-mark no -w 160 -i 2 '{}' \; >> docs/tidy_warnings.txt 2>&1 "#)
            .unwrap_or_else(|e| panic!("{e}"));
    }
    Ok(())
}
