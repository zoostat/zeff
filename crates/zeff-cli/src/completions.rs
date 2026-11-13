//! `zeff completions <shell>` (Engine and Project Specification, section 2.1): the file by
//! which bash, zsh, fish or PowerShell completes a `zeff` command line: the commands, the
//! options and their values, and after `zeff help` the names of the functions, of the
//! packages and of the diagnostic codes.
//!
//! The file is written by the program from its own lists: the commands and options of the
//! tables below, which a test holds to the text of `zeff help`, and the functions, packages
//! and codes of the registry. So a file written by an installation cannot name less, or more,
//! than that installation has.

use zeff_engine::VERSION;

/// What follows a command on the command line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum After {
    Nothing,
    /// A script of the project, a `.zeff` file.
    Script,
    /// A function, a package or a diagnostic code.
    Topic,
    /// One of the four shells.
    Shell,
}

/// The commands of this release, in the order of `zeff help`, each with what follows it and
/// a line that says what it does.
pub const COMMANDS: &[(&str, After, &str)] = &[
    ("new", After::Nothing, "create a project folder"),
    (
        "run",
        After::Script,
        "run scripts/main.zeff, or the script named",
    ),
    ("check", After::Script, "check a script without running it"),
    ("test", After::Nothing, "run the tests of the project"),
    (
        "fmt",
        After::Nothing,
        "write every script in the one style of the language",
    ),
    (
        "lsp",
        After::Nothing,
        "start the language server an editor talks to",
    ),
    ("repl", After::Script, "open the console"),
    (
        "eval",
        After::Nothing,
        "run one statement and print its value",
    ),
    (
        "help",
        After::Topic,
        "a help page, the functions of a package, or a code explained",
    ),
    ("version", After::Nothing, "print the version of Zeff"),
    (
        "selftest",
        After::Nothing,
        "check this installation against the published results",
    ),
    (
        "completions",
        After::Shell,
        "write the completions of a shell",
    ),
];

/// What an option takes after it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Takes {
    Nothing,
    /// A folder.
    Folder,
    /// One of a few words.
    OneOf(&'static [&'static str]),
    /// Something the shell cannot offer: a number, a size.
    Value,
}

/// The options of this release: the long form, the short one where there is one, what the
/// option takes, and a line that says what it does.
pub const OPTIONS: &[(&str, Option<&str>, Takes, &str)] = &[
    (
        "--project",
        Some("-p"),
        Takes::Folder,
        "the project to act on",
    ),
    (
        "--format",
        None,
        Takes::OneOf(&["text", "json"]),
        "text, or JSON records for an editor",
    ),
    (
        "--color",
        None,
        Takes::OneOf(&["auto", "always", "never"]),
        "colour in messages",
    ),
    (
        "--ascii",
        None,
        Takes::Nothing,
        "rules and symbols in ASCII",
    ),
    (
        "--width",
        None,
        Takes::Value,
        "the width frames and tables wrap at",
    ),
    (
        "--memory",
        None,
        Takes::Value,
        "the memory budget of the run, as in 2GB",
    ),
    (
        "--deny-warnings",
        None,
        Takes::Nothing,
        "a warning that no pragma allows is an error",
    ),
    (
        "--check",
        None,
        Takes::Nothing,
        "for fmt: write nothing, say which scripts would change",
    ),
    (
        "--stdio",
        None,
        Takes::Nothing,
        "for lsp: talk on standard input and output",
    ),
    ("--quiet", Some("-q"), Takes::Nothing, "hide notes"),
    (
        "--verbose",
        Some("-v"),
        Takes::Nothing,
        "show timings and build details",
    ),
];

/// The shells a file can be written for.
pub const SHELLS: &[&str] = &["bash", "zsh", "fish", "powershell"];

/// The names `zeff help` takes: every function of the engine, every package, and every
/// diagnostic code with an explained page; each once, the functions first.
pub fn topics() -> Vec<String> {
    let mut functions: Vec<String> = Vec::new();
    let mut packages: Vec<String> = Vec::new();
    if let Ok(registry) = zeff_engine::registry::registry() {
        functions = registry
            .functions()
            .iter()
            .map(|f| f.spec.name.to_string())
            .collect();
        functions.sort();
        functions.dedup();
        packages = registry
            .packages_shipped()
            .into_iter()
            .map(str::to_string)
            .filter(|package| !functions.contains(package))
            .collect();
    }
    let codes = zeff_engine::explain::codes()
        .into_iter()
        .map(str::to_string);
    functions.into_iter().chain(packages).chain(codes).collect()
}

/// The options that take something after them, in both forms, for a shell to step over the
/// word that follows one.
fn with_a_value() -> Vec<&'static str> {
    OPTIONS
        .iter()
        .filter(|(_, _, takes, _)| *takes != Takes::Nothing)
        .flat_map(|(long, short, _, _)| short.iter().copied().chain([*long]))
        .collect()
}

fn commands_after(after: After) -> Vec<&'static str> {
    COMMANDS
        .iter()
        .filter(|(_, what, _)| *what == after)
        .map(|(name, _, _)| *name)
        .collect()
}

fn heading(shell: &str, called: &str, load: &str) -> String {
    format!(
        "# Completions of the zeff command line for {called}.\n# Written by `zeff completions {shell}` of Zeff {VERSION}: write it again after an upgrade.\n# {load}\n\n"
    )
}

/// The file for bash. It keeps to what bash 3.2 has, the bash of macOS.
fn bash(topics: &[String]) -> String {
    let stepped = with_a_value().join("|");
    let commands: Vec<&str> = COMMANDS.iter().map(|(name, _, _)| *name).collect();
    let options: Vec<&str> = OPTIONS
        .iter()
        .flat_map(|(long, short, _, _)| [*long].into_iter().chain(short.iter().copied()))
        .collect();
    let mut after_an_option = String::new();
    for (long, short, takes, _) in OPTIONS {
        let names = match short {
            Some(short) => format!("{short}|{long}"),
            None => (*long).to_string(),
        };
        match takes {
            Takes::Nothing => {}
            Takes::Folder => after_an_option.push_str(&format!(
                "        {names}) COMPREPLY=($(compgen -d -- \"$cur\")); return ;;\n"
            )),
            Takes::OneOf(words) => after_an_option.push_str(&format!(
                "        {names}) COMPREPLY=($(compgen -W \"{}\" -- \"$cur\")); return ;;\n",
                words.join(" ")
            )),
            Takes::Value => {
                after_an_option.push_str(&format!("        {names}) COMPREPLY=(); return ;;\n"));
            }
        }
    }
    let mut out = heading(
        "bash",
        "bash",
        "Load it with:  eval \"$(zeff completions bash)\"",
    );
    out.push_str(&format!(
        "_zeff() {{\n    local cur prev command i\n    cur=\"${{COMP_WORDS[COMP_CWORD]}}\"\n    prev=\"\"\n    if [ \"$COMP_CWORD\" -gt 0 ]; then prev=\"${{COMP_WORDS[COMP_CWORD-1]}}\"; fi\n    local commands=\"{}\"\n    local options=\"{}\"\n    local shells=\"{}\"\n    local topics=\"{}\"\n",
        commands.join(" "),
        options.join(" "),
        SHELLS.join(" "),
        topics.join(" ")
    ));
    out.push_str(&format!(
        "    # The command: the first word that is neither an option nor what an option takes.\n    command=\"\"\n    i=1\n    while [ \"$i\" -lt \"$COMP_CWORD\" ]; do\n        case \"${{COMP_WORDS[i]}}\" in\n            {stepped}) i=$((i + 1)) ;;\n            -*) ;;\n            *) command=\"${{COMP_WORDS[i]}}\"; break ;;\n        esac\n        i=$((i + 1))\n    done\n    case \"$prev\" in\n{after_an_option}    esac\n    case \"$cur\" in\n        -*) COMPREPLY=($(compgen -W \"$options\" -- \"$cur\")); return ;;\n    esac\n    case \"$command\" in\n        \"\") COMPREPLY=($(compgen -W \"$commands\" -- \"$cur\")) ;;\n        {}) COMPREPLY=($(compgen -W \"$topics\" -- \"$cur\")) ;;\n        {}) COMPREPLY=($(compgen -W \"$shells\" -- \"$cur\")) ;;\n        {}) COMPREPLY=($(compgen -f -X '!*.zeff' -- \"$cur\") $(compgen -d -- \"$cur\")) ;;\n        *) COMPREPLY=() ;;\n    esac\n}}\ncomplete -F _zeff zeff\n",
        commands_after(After::Topic).join("|"),
        commands_after(After::Shell).join("|"),
        commands_after(After::Script).join("|"),
    ));
    out
}

/// A text inside single quotes of a shell that ends them with `'` and has no escape in them
/// (zsh, fish, PowerShell written `''`): none of the texts here holds one, and a test says so.
fn quoted(text: &str) -> String {
    format!("'{text}'")
}

/// The file for zsh: a completion function, loaded from `fpath` or by `source`.
fn zsh(topics: &[String]) -> String {
    let stepped = with_a_value().join("|");
    let described = |pairs: Vec<(String, &str)>| -> String {
        pairs
            .iter()
            .map(|(name, says)| format!("        {}\n", quoted(&format!("{name}:{says}"))))
            .collect()
    };
    let commands = described(
        COMMANDS
            .iter()
            .map(|(name, _, says)| ((*name).to_string(), *says))
            .collect(),
    );
    let options = described(
        OPTIONS
            .iter()
            .flat_map(|(long, short, _, says)| {
                [*long]
                    .into_iter()
                    .chain(short.iter().copied())
                    .map(|name| (name.to_string(), *says))
            })
            .collect(),
    );
    let mut after_an_option = String::new();
    for (long, short, takes, _) in OPTIONS {
        let names = match short {
            Some(short) => format!("{short}|{long}"),
            None => (*long).to_string(),
        };
        match takes {
            Takes::Nothing => {}
            Takes::Folder => {
                after_an_option.push_str(&format!("        {names}) _files -/; return ;;\n"));
            }
            Takes::OneOf(words) => after_an_option.push_str(&format!(
                "        {names}) compadd {}; return ;;\n",
                words.join(" ")
            )),
            Takes::Value => after_an_option.push_str(&format!("        {names}) return ;;\n")),
        }
    }
    let mut out = String::from("#compdef zeff\n");
    out.push_str(&heading(
        "zsh",
        "zsh",
        "Load it with:  source <(zeff completions zsh)   (after compinit), or save it as _zeff in a folder of $fpath.",
    ));
    out.push_str(&format!(
        "_zeff() {{\n    local -a commands options shells topics\n    commands=(\n{commands}    )\n    options=(\n{options}    )\n    shells=({})\n    topics=({})\n",
        SHELLS.join(" "),
        topics.join(" ")
    ));
    out.push_str(&format!(
        "    # The command: the first word that is neither an option nor what an option takes.\n    local command=\"\"\n    local -i i=2\n    while (( i < CURRENT )); do\n        case \"${{words[i]}}\" in\n            {stepped}) (( i++ )) ;;\n            -*) ;;\n            *) command=\"${{words[i]}}\"; break ;;\n        esac\n        (( i++ ))\n    done\n    case \"${{words[CURRENT-1]}}\" in\n{after_an_option}    esac\n    if [[ \"${{words[CURRENT]}}\" == -* ]]; then\n        _describe 'option' options\n        return\n    fi\n    case \"$command\" in\n        \"\") _describe 'command' commands ;;\n        {}) compadd -a topics ;;\n        {}) compadd -a shells ;;\n        {}) _files -g '*.zeff' ;;\n    esac\n}}\n# Run where zsh loads this file as the function itself; registered where it is sourced.\nif [[ \"${{funcstack[1]}}\" == \"_zeff\" ]]; then\n    _zeff \"$@\"\nelse\n    compdef _zeff zeff\nfi\n",
        commands_after(After::Topic).join("|"),
        commands_after(After::Shell).join("|"),
        commands_after(After::Script).join("|"),
    ));
    out
}

/// The file for fish.
fn fish(topics: &[String]) -> String {
    let stepped = with_a_value().join(" ");
    let mut out = heading(
        "fish",
        "fish",
        "Load it with:  zeff completions fish | source   or save it as ~/.config/fish/completions/zeff.fish",
    );
    out.push_str(&format!(
        "# The command on the line: the first word that is neither an option nor what an option takes.\nfunction __zeff_command\n    set -l tokens (commandline -opc)\n    set -e tokens[1]\n    set -l skip 0\n    for token in $tokens\n        if test $skip -eq 1\n            set skip 0\n            continue\n        end\n        switch $token\n            case {stepped}\n                set skip 1\n            case '-*'\n            case '*'\n                echo $token\n                return 0\n        end\n    end\n    return 1\nend\n\nfunction __zeff_no_command\n    not __zeff_command >/dev/null\nend\n\nfunction __zeff_after\n    set -l command (__zeff_command)\n    or return 1\n    contains -- $command $argv\nend\n\n# No file names unless a command takes a script.\ncomplete -c zeff -f\n\n"
    ));
    for (name, _, says) in COMMANDS {
        out.push_str(&format!(
            "complete -c zeff -n __zeff_no_command -a {name} -d {}\n",
            quoted(says)
        ));
    }
    out.push('\n');
    for (long, short, takes, says) in OPTIONS {
        let mut line = format!("complete -c zeff -l {}", &long[2..]);
        if let Some(short) = short {
            line.push_str(&format!(" -s {}", &short[1..]));
        }
        match takes {
            Takes::Nothing => {}
            Takes::Folder => line.push_str(" -x -a '(__fish_complete_directories)'"),
            Takes::OneOf(words) => line.push_str(&format!(" -x -a {}", quoted(&words.join(" ")))),
            Takes::Value => line.push_str(" -x"),
        }
        out.push_str(&format!("{line} -d {}\n", quoted(says)));
    }
    out.push_str(&format!(
        "\ncomplete -c zeff -n '__zeff_after {}' -a {}\ncomplete -c zeff -n '__zeff_after {}' -a {}\ncomplete -c zeff -n '__zeff_after {}' -k -a '(__fish_complete_suffix .zeff)'\n",
        commands_after(After::Topic).join(" "),
        quoted(&topics.join(" ")),
        commands_after(After::Shell).join(" "),
        quoted(&SHELLS.join(" ")),
        commands_after(After::Script).join(" "),
    ));
    out
}

/// The file for PowerShell.
fn powershell(topics: &[String]) -> String {
    let list = |words: &[String]| -> String {
        words
            .iter()
            .map(|word| quoted(word))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let owned =
        |words: &[&str]| -> Vec<String> { words.iter().map(|w| (*w).to_string()).collect() };
    let commands: Vec<&str> = COMMANDS.iter().map(|(name, _, _)| *name).collect();
    let options: Vec<&str> = OPTIONS
        .iter()
        .flat_map(|(long, short, _, _)| [*long].into_iter().chain(short.iter().copied()))
        .collect();
    let mut after_an_option = String::new();
    for (long, short, takes, _) in OPTIONS {
        let names: Vec<String> = short.iter().copied().chain([*long]).map(quoted).collect();
        let offered = match takes {
            Takes::Nothing => continue,
            Takes::OneOf(words) => format!("@({})", list(&owned(words))),
            // A folder is left to PowerShell's own completion of paths; a number or a size
            // has nothing to offer.
            Takes::Folder | Takes::Value => "@()".to_string(),
        };
        after_an_option.push_str(&format!(
            "        {{ $_ -in @({}) }} {{ $offered = {offered}; break }}\n",
            names.join(", ")
        ));
    }
    let mut out = heading(
        "powershell",
        "PowerShell",
        "Load it with:  zeff completions powershell | Out-String | Invoke-Expression   or add that line to $PROFILE",
    );
    out.push_str(&format!(
        "Register-ArgumentCompleter -Native -CommandName zeff -ScriptBlock {{\n    param($wordToComplete, $commandAst, $cursorPosition)\n    $commands = @({})\n    $options = @({})\n    $shells = @({})\n    $topics = @({})\n    $takesAValue = @({})\n",
        list(&owned(&commands)),
        list(&owned(&options)),
        list(&owned(SHELLS)),
        list(topics),
        list(&owned(&with_a_value())),
    ));
    out.push_str(&format!(
        "    # The words before the one being completed, without the name of the program.\n    $before = @($commandAst.CommandElements | Where-Object {{ $_.Extent.EndOffset -lt $cursorPosition -or ($_.Extent.EndOffset -eq $cursorPosition -and $wordToComplete -eq '') }} | ForEach-Object {{ $_.ToString() }})\n    if ($before.Count -gt 0) {{ $before = @($before | Select-Object -Skip 1) }}\n    # The command: the first word that is neither an option nor what an option takes.\n    $command = ''\n    for ($i = 0; $i -lt $before.Count; $i++) {{\n        $word = $before[$i]\n        if ($takesAValue -contains $word) {{ $i++; continue }}\n        if ($word.StartsWith('-')) {{ continue }}\n        $command = $word\n        break\n    }}\n    $previous = if ($before.Count -gt 0) {{ $before[$before.Count - 1] }} else {{ '' }}\n    $offered = $null\n    switch ($previous) {{\n{after_an_option}    }}\n    if ($null -eq $offered) {{\n        if ($wordToComplete.StartsWith('-')) {{ $offered = $options }}\n        elseif ($command -eq '') {{ $offered = $commands }}\n        elseif (@({}) -contains $command) {{ $offered = $topics }}\n        elseif (@({}) -contains $command) {{ $offered = $shells }}\n        else {{ $offered = @() }}\n    }}\n    $offered | Where-Object {{ $_ -like \"$wordToComplete*\" }} | ForEach-Object {{\n        [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)\n    }}\n}}\n",
        list(&owned(&commands_after(After::Topic))),
        list(&owned(&commands_after(After::Shell))),
    ));
    out
}

/// The completion file of a shell; `None` for a name that is none of the four.
pub fn file(shell: &str) -> Option<String> {
    let topics = topics();
    match shell {
        "bash" => Some(bash(&topics)),
        "zsh" => Some(zsh(&topics)),
        "fish" => Some(fish(&topics)),
        "powershell" => Some(powershell(&topics)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tables_are_those_of_zeff_help() {
        let usage = crate::USAGE;
        let commands = &usage[usage.find("Commands:").unwrap()..usage.find("Options:").unwrap()];
        let options = &usage[usage.find("Options:").unwrap()..];
        // Every command of the table is a line of `zeff help`, and every line that names a
        // command is in the table; `help` stands twice there, with and without a name.
        let mut written: Vec<&str> = commands
            .lines()
            .filter(|line| line.starts_with("  ") && !line.starts_with("   "))
            .filter_map(|line| line.split_whitespace().next())
            .collect();
        written.dedup();
        let mut table: Vec<&str> = COMMANDS.iter().map(|(name, _, _)| *name).collect();
        let mut sorted = written.clone();
        sorted.sort_unstable();
        sorted.dedup();
        table.sort_unstable();
        assert_eq!(table, sorted);
        // Every option of the table is written there with its short form, and every option
        // written there is in the table.
        for (long, short, _, _) in OPTIONS {
            let line = options
                .lines()
                .find(|line| line.split([' ', ',']).any(|word| word == *long))
                .unwrap_or_else(|| panic!("{long} is not in `zeff help`"));
            if let Some(short) = short {
                assert!(line.contains(&format!("{short}, {long}")), "{line}");
            }
        }
        for word in options.split_whitespace() {
            let word = word.trim_matches(|c: char| !(c.is_ascii_alphanumeric() || c == '-'));
            if word.starts_with("--") && word.len() > 2 {
                assert!(
                    OPTIONS.iter().any(|(long, _, _, _)| *long == word),
                    "{word} is in `zeff help` and not in the table"
                );
            }
        }
    }

    #[test]
    fn nothing_written_into_a_file_needs_an_escape() {
        // The lines that say what a command or an option does stand between single quotes
        // in three of the files, and after a colon in zsh's.
        for says in COMMANDS
            .iter()
            .map(|(_, _, says)| *says)
            .chain(OPTIONS.iter().map(|(_, _, _, says)| *says))
        {
            assert!(
                !says.contains(['\'', '"', '`', '$', '\\']),
                "`{says}` would need an escape"
            );
        }
        // The names offered are words: letters, digits, `_` and the `.` of a namespace.
        for topic in topics() {
            assert!(
                topic
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.'),
                "{topic}"
            );
        }
    }

    #[test]
    fn the_topics_are_the_functions_the_packages_and_the_codes() {
        let topics = topics();
        for name in [
            "mean",
            "reg",
            "dist.normal",
            "regress",
            "data",
            "Z0101",
            "W0201",
        ] {
            assert_eq!(topics.iter().filter(|t| *t == name).count(), 1, "{name}");
        }
        assert!(topics.len() > 300, "{}", topics.len());
        assert_eq!(file("ksh"), None);
        for shell in SHELLS {
            assert!(
                file(shell).is_some_and(|text| text.contains("dist.normal")),
                "{shell}"
            );
        }
    }
}
