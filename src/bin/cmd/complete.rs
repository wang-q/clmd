use clap::{Arg, ArgMatches, Command};
use std::io::Write;

pub fn make_subcommand() -> Command {
    Command::new("complete")
        .about("Generate shell completion scripts")
        .arg(
            Arg::new("shell")
                .help("Shell type (bash, zsh, fish, powershell, elvish)")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .help("Output file (default: stdout)"),
        )
        .after_help(
            r###"Generate shell completion scripts for clmd.

Supported shells:
  bash         - Bash completions
  zsh          - Zsh completions
  fish         - Fish completions
  powershell   - PowerShell completions
  elvish       - Elvish completions

Examples:
  clmd complete bash > /etc/bash_completion.d/clmd
  clmd complete zsh > /usr/local/share/zsh/site-functions/_clmd
  clmd complete fish > ~/.config/fish/completions/clmd.fish
  clmd complete powershell -o ~/clmd-completion.ps1   # then add: . ~/clmd-completion.ps1 to $PROFILE
  clmd complete elvish | slurp
  clmd complete bash -o clmd.bash
"###,
        )
}

pub fn execute(matches: &ArgMatches, _options: &clmd::Options) -> anyhow::Result<()> {
    let shell = matches
        .get_one::<String>("shell")
        .map(|s| s.as_str())
        .unwrap();

    let completion_script = match shell {
        "bash" => generate_bash_completion(),
        "zsh" => generate_zsh_completion(),
        "fish" => generate_fish_completion(),
        "powershell" => generate_powershell_completion(),
        "elvish" => generate_elvish_completion(),
        _ => return Err(anyhow::anyhow!("Unsupported shell: {}", shell)),
    };

    let output_path = matches.get_one::<String>("output").map(|s| s.as_str());

    if let Some(path) = output_path {
        std::fs::write(path, completion_script)
            .map_err(|e| anyhow::anyhow!("Failed to write file '{}': {}", path, e))?;
    } else {
        std::io::stdout()
            .write_all(completion_script.as_bytes())
            .map_err(|e| anyhow::anyhow!("Failed to write stdout: {}", e))?;
    }

    Ok(())
}

fn generate_bash_completion() -> String {
    r#"#!/bin/bash
# clmd bash completion script
# Source this file: source <(clmd complete bash)

_clmd() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"

    # Main commands
    local commands="convert extract fmt stats toc validate complete help"

    # Global options
    local global_opts="-c --config -e --extension --safe -h --help -V --version"

    # Extension options
    local extensions="table strikethrough tasklist footnotes autolink tagfilter superscript subscript underline highlight math wikilink spoiler alerts"

    case "${COMP_CWORD}" in
        1)
            COMPREPLY=( $(compgen -W "${commands}" -- ${cur}) )
            return 0
            ;;
        *)
            case "${COMP_WORDS[1]}" in
                convert)
                    _clmd_convert
                    return 0
                    ;;
                extract)
                    _clmd_extract
                    return 0
                    ;;
                fmt)
                    _clmd_fmt
                    return 0
                    ;;
                stats)
                    _clmd_stats
                    return 0
                    ;;
                toc)
                    _clmd_toc
                    return 0
                    ;;
                validate)
                    _clmd_validate
                    return 0
                    ;;
                complete)
                    COMPREPLY=( $(compgen -W "bash zsh fish powershell elvish" -- ${cur}) )
                    return 0
                    ;;
                *)
                    COMPREPLY=( $(compgen -W "${global_opts}" -- ${cur}) )
                    return 0
                    ;;
            esac
            ;;
    esac
}

_clmd_convert() {
    local cur="${COMP_WORDS[COMP_CWORD]}"
    local subcommands="to from"
    local formats="html xml latex man typst pdf docx epub rtf commonmark"

    if [[ ${COMP_CWORD} -eq 2 ]]; then
        COMPREPLY=( $(compgen -W "${subcommands}" -- ${cur}) )
    elif [[ ${COMP_CWORD} -eq 3 ]]; then
        COMPREPLY=( $(compgen -W "${formats}" -- ${cur}) )
    else
        local opts="-o --output --full --hardbreaks --width -h --help"
        COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
        _filedir
    fi
}

_clmd_extract() {
    local cur="${COMP_WORDS[COMP_CWORD]}"
    local subcommands="links images headings code tables footnotes yaml-front-matter task-items"

    if [[ ${COMP_CWORD} -eq 2 ]]; then
        COMPREPLY=( $(compgen -W "${subcommands}" -- ${cur}) )
    else
        local opts="-o --output --format -l --level --language -k --key --checked --unchecked -h --help"
        COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
        _filedir
    fi
}

_clmd_fmt() {
    local cur="${COMP_WORDS[COMP_CWORD]}"
    local opts="-o --output -i --in-place -b --backup --width -h --help"
    COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
    _filedir
}

_clmd_stats() {
    local cur="${COMP_WORDS[COMP_CWORD]}"
    local opts="-o --output --format -h --help"
    COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
    _filedir
}

_clmd_toc() {
    local cur="${COMP_WORDS[COMP_CWORD]}"
    local opts="-o --output -l --levels --numbered --links -h --help"
    COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
    _filedir
}

_clmd_validate() {
    local cur="${COMP_WORDS[COMP_CWORD]}"
    local opts="-o --output --format --check-links --check-external-links --check-images --check-refs --strict -h --help"
    COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
    _filedir
}

complete -F _clmd clmd
"#.to_string()
}

fn generate_zsh_completion() -> String {
    r#"#compdef clmd
# clmd zsh completion script

_clmd() {
    local curcontext="$curcontext" state line
    typeset -A opt_args

    _arguments -C \
        '(-c --config)'{-c,--config}'[Configuration file path]:config file:_files -g "*.toml"' \
        '(-e --extension)'{-e,--extension}'[Enable extensions]:extension:(table strikethrough tasklist footnotes autolink tagfilter superscript subscript underline highlight math wikilink spoiler alerts)' \
        '--safe[Enable safe mode]' \
        '(-h --help)'{-h,--help}'[Show help]' \
        '(-V --version)'{-V,--version}'[Show version]' \
        '1: :_clmd_commands' \
        '*:: :->args'

    case "$state" in
        args)
            case "$line[1]" in
                convert)
                    _clmd_convert
                    ;;
                extract)
                    _clmd_extract
                    ;;
                fmt)
                    _clmd_fmt
                    ;;
                stats)
                    _clmd_stats
                    ;;
                toc)
                    _clmd_toc
                    ;;
                validate)
                    _clmd_validate
                    ;;
                complete)
                    _clmd_complete
                    ;;
            esac
            ;;
    esac
}

_clmd_commands() {
    local commands=(
        'convert:Convert between Markdown and other formats'
        'extract:Extract specific elements from Markdown'
        'fmt:Format Markdown to canonical CommonMark/GFM'
        'stats:Show statistics about Markdown document'
        'toc:Generate table of contents'
        'validate:Validate Markdown document'
        'complete:Generate shell completion scripts'
        'help:Show help'
    )
    _describe -t commands 'clmd commands' commands
}

_clmd_convert() {
    _arguments -C \
        '1: :_clmd_convert_subcommands' \
        '*:: :->args'

    case "$state" in
        args)
            case "$line[1]" in
                to|from)
                    _arguments \
                        '(-o --output)'{-o,--output}'[Output file]:output file:_files' \
                        '--full[Generate full document]' \
                        '--hardbreaks[Convert newlines to <br>]' \
                        '--width[Line width]:width:' \
                        '1:format:(html xml latex man typst pdf docx epub rtf commonmark)' \
                        '2:input file:_files -g "*.md"'
                    ;;
            esac
            ;;
    esac
}

_clmd_convert_subcommands() {
    local subcommands=(
        'to:Convert to format'
        'from:Convert from format'
    )
    _describe -t subcommands 'convert subcommands' subcommands
}

_clmd_extract() {
    _arguments \
        '1: :_clmd_extract_subcommands' \
        '(-o --output)'{-o,--output}'[Output file]:output file:_files' \
        '--format[Output format]:format:(text json markdown csv raw)' \
        '(-l --level)'{-l,--level}'[Filter by level]:level:(1 2 3 4 5 6)' \
        '--language[Filter by language]:language:' \
        '(-k --key)'{-k,--key}'[Extract specific key]:key:' \
        '--checked[Only show checked items]' \
        '--unchecked[Only show unchecked items]' \
        '2:input file:_files -g "*.md"'
}

_clmd_extract_subcommands() {
    local subcommands=(
        'links:Extract all links'
        'images:Extract all images'
        'headings:Extract all headings'
        'code:Extract all code blocks'
        'tables:Extract all tables'
        'footnotes:Extract all footnotes'
        'yaml-front-matter:Extract YAML front matter'
        'task-items:Extract all task list items'
    )
    _describe -t subcommands 'extract subcommands' subcommands
}

_clmd_fmt() {
    _arguments \
        '(-o --output)'{-o,--output}'[Output file]:output file:_files' \
        '(-i --in-place)'{-i,--in-place}'[Edit file in-place]' \
        '(-b --backup)'{-b,--backup}'[Create backup file]' \
        '--width[Line width]:width:' \
        '1:input file:_files -g "*.md"'
}

_clmd_stats() {
    _arguments \
        '(-o --output)'{-o,--output}'[Output file]:output file:_files' \
        '--format[Output format]:format:(text json)' \
        '1:input file:_files -g "*.md"'
}

_clmd_toc() {
    _arguments \
        '(-o --output)'{-o,--output}'[Output file]:output file:_files' \
        '(-l --levels)'{-l,--levels}'[Heading level range]:levels:' \
        '--numbered[Add numbering to TOC entries]' \
        '--links[Generate anchor links]' \
        '1:input file:_files -g "*.md"'
}

_clmd_validate() {
    _arguments \
        '(-o --output)'{-o,--output}'[Output file]:output file:_files' \
        '--format[Output format]:format:(text json)' \
        '--check-links[Check for broken internal links]' \
        '--check-external-links[Check for broken external links]' \
        '--check-images[Check if local image files exist]' \
        '--check-refs[Check for unused reference links]' \
        '--strict[Enable all validation checks]' \
        '1:input file:_files -g "*.md"'
}

_clmd_complete() {
    _arguments \
        '(-o --output)'{-o,--output}'[Output file]:output file:_files' \
        '1:shell:(bash zsh fish powershell elvish)'
}

compdef _clmd clmd
"#.to_string()
}

fn generate_fish_completion() -> String {
    r#"# clmd fish completion script
# Save to: ~/.config/fish/completions/clmd.fish

# Main command
complete -c clmd -f

# Global options
complete -c clmd -s c -l config -d "Configuration file path" -r
complete -c clmd -s e -l extension -d "Enable extensions" -xa "table strikethrough tasklist footnotes autolink tagfilter superscript subscript underline highlight math wikilink spoiler alerts"
complete -c clmd -l safe -d "Enable safe mode"
complete -c clmd -s h -l help -d "Show help"
complete -c clmd -s V -l version -d "Show version"

# Subcommands
complete -c clmd -n "__fish_use_subcommand" -a "convert" -d "Convert between formats"
complete -c clmd -n "__fish_use_subcommand" -a "extract" -d "Extract elements"
complete -c clmd -n "__fish_use_subcommand" -a "fmt" -d "Format Markdown"
complete -c clmd -n "__fish_use_subcommand" -a "stats" -d "Show statistics"
complete -c clmd -n "__fish_use_subcommand" -a "toc" -d "Generate table of contents"
complete -c clmd -n "__fish_use_subcommand" -a "validate" -d "Validate document"
complete -c clmd -n "__fish_use_subcommand" -a "complete" -d "Generate shell completions"

# Convert subcommand
complete -c clmd -n "__fish_seen_subcommand_from convert; and not __fish_seen_subcommand_from to from" -a "to" -d "Convert to format"
complete -c clmd -n "__fish_seen_subcommand_from convert; and not __fish_seen_subcommand_from to from" -a "from" -d "Convert from format"
complete -c clmd -n "__fish_seen_subcommand_from convert; and __fish_seen_subcommand_from to" -a "html xml latex man typst pdf docx epub rtf commonmark" -d "Output format"
complete -c clmd -n "__fish_seen_subcommand_from convert; and __fish_seen_subcommand_from from" -a "html latex bibtex" -d "Input format"
complete -c clmd -n "__fish_seen_subcommand_from convert" -s o -l output -d "Output file" -r
complete -c clmd -n "__fish_seen_subcommand_from convert" -l full -d "Generate full document"
complete -c clmd -n "__fish_seen_subcommand_from convert" -l hardbreaks -d "Convert newlines to <br>"
complete -c clmd -n "__fish_seen_subcommand_from convert" -l width -d "Line width" -r

# Extract subcommand
complete -c clmd -n "__fish_seen_subcommand_from extract; and not __fish_seen_subcommand_from links images headings code tables footnotes yaml-front-matter task-items" -a "links" -d "Extract links"
complete -c clmd -n "__fish_seen_subcommand_from extract; and not __fish_seen_subcommand_from links images headings code tables footnotes yaml-front-matter task-items" -a "images" -d "Extract images"
complete -c clmd -n "__fish_seen_subcommand_from extract; and not __fish_seen_subcommand_from links images headings code tables footnotes yaml-front-matter task-items" -a "headings" -d "Extract headings"
complete -c clmd -n "__fish_seen_subcommand_from extract; and not __fish_seen_subcommand_from links images headings code tables footnotes yaml-front-matter task-items" -a "code" -d "Extract code blocks"
complete -c clmd -n "__fish_seen_subcommand_from extract; and not __fish_seen_subcommand_from links images headings code tables footnotes yaml-front-matter task-items" -a "tables" -d "Extract tables"
complete -c clmd -n "__fish_seen_subcommand_from extract; and not __fish_seen_subcommand_from links images headings code tables footnotes yaml-front-matter task-items" -a "footnotes" -d "Extract footnotes"
complete -c clmd -n "__fish_seen_subcommand_from extract; and not __fish_seen_subcommand_from links images headings code tables footnotes yaml-front-matter task-items" -a "yaml-front-matter" -d "Extract YAML front matter"
complete -c clmd -n "__fish_seen_subcommand_from extract; and not __fish_seen_subcommand_from links images headings code tables footnotes yaml-front-matter task-items" -a "task-items" -d "Extract task items"
complete -c clmd -n "__fish_seen_subcommand_from extract" -s o -l output -d "Output file" -r
complete -c clmd -n "__fish_seen_subcommand_from extract" -l format -d "Output format" -xa "text json markdown csv raw"
complete -c clmd -n "__fish_seen_subcommand_from extract" -s l -l level -d "Filter by level" -xa "1 2 3 4 5 6"
complete -c clmd -n "__fish_seen_subcommand_from extract" -l language -d "Filter by language" -r
complete -c clmd -n "__fish_seen_subcommand_from extract" -s k -l key -d "Extract specific key" -r
complete -c clmd -n "__fish_seen_subcommand_from extract" -l checked -d "Only show checked items"
complete -c clmd -n "__fish_seen_subcommand_from extract" -l unchecked -d "Only show unchecked items"

# Fmt subcommand
complete -c clmd -n "__fish_seen_subcommand_from fmt" -s o -l output -d "Output file" -r
complete -c clmd -n "__fish_seen_subcommand_from fmt" -s i -l in-place -d "Edit file in-place"
complete -c clmd -n "__fish_seen_subcommand_from fmt" -s b -l backup -d "Create backup"
complete -c clmd -n "__fish_seen_subcommand_from fmt" -l width -d "Line width" -r

# Stats subcommand
complete -c clmd -n "__fish_seen_subcommand_from stats" -s o -l output -d "Output file" -r
complete -c clmd -n "__fish_seen_subcommand_from stats" -l format -d "Output format" -xa "text json"

# Toc subcommand
complete -c clmd -n "__fish_seen_subcommand_from toc" -s o -l output -d "Output file" -r
complete -c clmd -n "__fish_seen_subcommand_from toc" -s l -l levels -d "Heading level range" -r
complete -c clmd -n "__fish_seen_subcommand_from toc" -l numbered -d "Add numbering"
complete -c clmd -n "__fish_seen_subcommand_from toc" -l links -d "Generate anchor links"

# Validate subcommand
complete -c clmd -n "__fish_seen_subcommand_from validate" -s o -l output -d "Output file" -r
complete -c clmd -n "__fish_seen_subcommand_from validate" -l format -d "Output format" -xa "text json"
complete -c clmd -n "__fish_seen_subcommand_from validate" -l check-links -d "Check internal links"
complete -c clmd -n "__fish_seen_subcommand_from validate" -l check-external-links -d "Check external links"
complete -c clmd -n "__fish_seen_subcommand_from validate" -l check-images -d "Check images"
complete -c clmd -n "__fish_seen_subcommand_from validate" -l check-refs -d "Check references"
complete -c clmd -n "__fish_seen_subcommand_from validate" -l strict -d "Enable all checks"

# Complete subcommand
complete -c clmd -n "__fish_seen_subcommand_from complete" -s o -l output -d "Output file" -r
complete -c clmd -n "__fish_seen_subcommand_from complete" -a "bash zsh fish powershell elvish" -d "Shell type"
"#.to_string()
}

fn generate_powershell_completion() -> String {
    r#"# clmd PowerShell completion script
# One-time setup (persists across sessions):
#   clmd complete powershell -o $PROFILE/../clmd-completion.ps1; Add-Content -Path $PROFILE -Value '. "$PSCommandPath/../clmd-completion.ps1"'
# Or for current session only:
#   clmd complete powershell | Out-String | Invoke-Expression

$script:clmdCommands = @('convert', 'extract', 'fmt', 'stats', 'toc', 'validate', 'complete')
$script:clmdExtensions = @('table', 'strikethrough', 'tasklist', 'footnotes', 'autolink', 'tagfilter', 'superscript', 'subscript', 'underline', 'highlight', 'math', 'wikilink', 'spoiler', 'alerts')
$script:clmdFormats = @('html', 'xml', 'latex', 'man', 'typst', 'pdf', 'docx', 'epub', 'rtf', 'commonmark')

Register-ArgumentCompleter -Native -CommandName clmd -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commands = $commandAst.CommandElements | Select-Object -Skip 1 | ForEach-Object { $_.ToString() }
    $command = $commands[0]

    switch ($command) {
        $null {
            $script:clmdCommands | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
                [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
            }
        }
        'convert' {
            if ($commands.Count -eq 1) {
                @('to', 'from') | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
                    [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                }
            } elseif ($commands.Count -eq 2) {
                $script:clmdFormats | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
                    [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                }
            }
        }
        'extract' {
            if ($commands.Count -eq 1) {
                @('links', 'images', 'headings', 'code', 'tables', 'footnotes', 'yaml-front-matter', 'task-items') | 
                    Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
                        [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
                    }
            }
        }
        'complete' {
            @('bash', 'zsh', 'fish', 'powershell', 'elvish') | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
                [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
            }
        }
    }

    # Global options
    @('-c', '--config', '-e', '--extension', '--safe', '-h', '--help', '-V', '--version') | 
        Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
            [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterName', $_)
        }
}
"#.to_string()
}

fn generate_elvish_completion() -> String {
    r#"# clmd elvish completion script
# Add to your rc.elv: eval (clmd complete elvish | slurp)

set edit:completion:arg-completer[clmd] = {|@args|
    var commands = [convert extract fmt stats toc validate complete]
    var extensions = [table strikethrough tasklist footnotes autolink tagfilter superscript subscript underline highlight math wikilink spoiler alerts]
    var formats = [html xml latex man typst pdf docx epub rtf commonmark]

    var n = (count $args)
    if (== $n 2) {
        put $@commands
    } else {
        var cmd = $args[1]
        if (== $n 3) {
            if (eq $cmd convert) {
                put to from
            } elif (eq $cmd extract) {
                put links images headings code tables footnotes yaml-front-matter task-items
            } elif (eq $cmd complete) {
                put bash zsh fish powershell elvish
            }
        } elif (== $n 4) {
            if (eq $cmd convert) {
                put $@formats
            }
        }
    }
}
"#.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_subcommand() {
        let cmd = make_subcommand();
        assert_eq!(cmd.get_name(), "complete");
    }

    #[test]
    fn test_generate_bash_completion() {
        let script = generate_bash_completion();
        assert!(script.contains("#!/bin/bash"));
        assert!(script.contains("_clmd()"));
        assert!(script.contains("complete -F _clmd clmd"));
        assert!(script.contains("_clmd_convert"));
        assert!(script.contains("_clmd_extract"));
        assert!(script.contains("_clmd_fmt"));
    }

    #[test]
    fn test_generate_zsh_completion() {
        let script = generate_zsh_completion();
        assert!(script.contains("#compdef clmd"));
        assert!(script.contains("_clmd()"));
        assert!(script.contains("_clmd_commands"));
        assert!(script.contains("compdef _clmd clmd"));
    }

    #[test]
    fn test_generate_fish_completion() {
        let script = generate_fish_completion();
        assert!(script.contains("# clmd fish completion script"));
        assert!(script.contains("complete -c clmd"));
        assert!(script.contains("convert"));
        assert!(script.contains("extract"));
    }

    #[test]
    fn test_generate_powershell_completion() {
        let script = generate_powershell_completion();
        assert!(script.contains("# clmd PowerShell completion script"));
        assert!(script.contains("Register-ArgumentCompleter"));
        assert!(script.contains("$script:clmdCommands"));
    }

    #[test]
    fn test_generate_elvish_completion() {
        let script = generate_elvish_completion();
        assert!(script.contains("# clmd elvish completion script"));
        assert!(script.contains("set edit:completion:arg-completer[clmd]"));
    }
}
