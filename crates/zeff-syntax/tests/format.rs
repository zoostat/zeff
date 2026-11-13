//! The formatter (Language Reference, section 3.5; release 0.7.0, step 31a): the one style
//! on every construct of the grammar, the specification's own before and after, every script
//! of the repository, and the properties that formatting twice changes nothing, that the
//! formatted script has the tree of the script given, and that no comment is lost. The last
//! two are checked by `format` itself on everything it writes, and a difference comes back
//! as `Unformatted::Internal`, so a test holds them by asking that it never does.

use proptest::prelude::*;
use std::path::{Path, PathBuf};
use zeff_syntax::format::{Unformatted, format};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A script formatted; a script that parses must come back formatted.
fn formatted(text: &str) -> String {
    match format("t.zeff", text) {
        Ok(written) => written,
        Err(Unformatted::Syntax(found)) => panic!(
            "the script does not parse: {:?}\n{text}",
            found.iter().map(|d| d.message.clone()).collect::<Vec<_>>()
        ),
        Err(Unformatted::Internal(what)) => panic!("{what}:\n{text}"),
    }
}

/// What a case of the tests shows: the script given, a rule, and the script written.
fn shown(text: &str) -> String {
    let written = formatted(text);
    // Formatting what was written changes nothing.
    assert_eq!(formatted(&written), written, "not stable:\n{written}");
    // No line ends with a space, none has a tab or a carriage return.
    for line in written.lines() {
        assert!(
            !line.ends_with(' ') && !line.contains(['\t', '\r']),
            "{line:?}"
        );
    }
    assert!(written.is_empty() || written.ends_with('\n') && !written.ends_with("\n\n"));
    format!("{text}--------\n{written}")
}

#[test]
fn the_before_and_after_of_the_specification() {
    let reference =
        std::fs::read_to_string(root().join("docs/spec/zeff_language_reference.md")).unwrap();
    let from = reference
        .find("Before and after `zeff fmt`:")
        .expect("the example of section 3.5");
    let mut blocks = reference[from..].split("```\n");
    let (_, before, _, after) = (
        blocks.next(),
        blocks.next().unwrap(),
        blocks.next(),
        blocks.next().unwrap(),
    );
    assert!(before.starts_with("let m=wages"), "{before}");
    assert_eq!(formatted(before), after);
    // The line of the example is too long for one line. A chain that fits is not broken.
    assert_eq!(
        formatted("let m=wages.where(year>=2010).reg(lwage~educ+exper,fe:firm,se:robust)\n"),
        "let m = wages.where(year >= 2010).reg(lwage ~ educ + exper, fe: firm, se: robust)\n"
    );
}

#[test]
fn the_spacing_of_the_style() {
    insta::assert_snapshot!(
        "spacing",
        shown(
            "use   functions :: clean\nlet a=1+2*3-4/5\nlet b = x^2  +  y ^ -1\nlet c=-a+!b\nlet d = a<b&&c>=d||e!=f&&g==h\nlet r=1 .. 10\nlet f = y~x+a:b+exper ^ 2\nlet g= ~z\nlet h = wages [ \"educ\" ] [ 1 ]\nlet i = {a:1,b:\"two\",..rest}\nlet j=[1,2,  3]\nlet k = [\"a\"=>1,\"b\"=>2]\nlet l = f( x ,y:2, z=3 , ..opts )\nlet m = d.add( `gross sales`=a*b ).mean( `gross sales` )\nlet n = map(xs,fn(i)i/n)\nlet o = map(xs, fn( i:Int , j ) i+j)\nlet p = (a+b)*c\nlet q = a - -b\nlet s = pkg::f(1).g ( )\nlet t = if a{1}else{2}\nlet u = case{a>1=>\"x\",_=>\"y\"}\nlet v = [for i in 1..3{i*2}]\nlet w = r\"raw\\d\" + \"esc\\n\"\nlet x = 1_000 + 1.5e3 + NA + Inf + true\n"
        )
    );
}

#[test]
fn what_does_not_fit_is_broken_and_what_was_broken_stays_broken() {
    insta::assert_snapshot!(
        "breaking",
        shown(
            "let fits = wages.where(year >= 2010 && age >= 18).add(lwage = log(wage)).reg(lwage ~ educ + exper, fe: firm)\nlet one = report([m1, m2, m3, m4], stats: [n_obs, r_squared], labels: {educ: \"Years of schooling\", exper: \"Experience\"})\nlet short = wages\n.where(year >= 2010)\n.n_rows()\nlet spread = frame(a = 1,\n  b = 2)\nlet kept = frame(a = 1, b = 2)\nlet deep = some_function_with_a_long_name(another_function_with_a_long_name(yet_another_long_function_name(first_argument_long, second_argument_long), third_argument_also_long))\nlet sum = first_long_operand_name + second_long_operand_name + third_long_operand_name + fourth_long_operand_name + fifth\nlet sum2 = a +\n b + c\nlet text = \"a string that is far too long for one line of the page but which a formatter must never split in two, whatever its length may be\"\nlet list = [alpha_alpha_alpha, beta_beta_beta, gamma_gamma_gamma, delta_delta_delta, epsilon_epsilon, zeta_zeta_zeta]\nplot(means, x: industry, y: mean_lwage, color: female).bars(position: dodge).errorbars(low: mean_lwage - 1.96 * se, high: mean_lwage + 1.96 * se).legend(top)\nwages.add(age_group = case {\n age < 25 => \"young\",\n age < 55 => \"prime\",\n _ => \"older\",\n})\nlet nested = [for s in 1..500 {\nlet d = frame(x = dist.normal().draw(n: size, seed: s), u = dist.uniform().draw(n: size, seed: s + 10000)).add(y = case { inv_logit(0.5 * x) > u => 1, _ => 0 })\nattempt(d.logistic(y ~ x))\n}]\n"
        )
    );
}

#[test]
fn comments_blank_lines_and_line_ends() {
    insta::assert_snapshot!(
        "comments",
        shown(
            "// A header\n//   with an indented line\n\n\n\n\nlet a = 1 // why one\nlet b = some_function(with_arguments, of_some_length, and_more)   // a comment at the end\n\n  // before c, wrongly indented\nlet c = f(x, // the first\n  y, // the second\n  // before z\n  z)\nlet d = wages\n  // before the where\n  .where(year > 2010) // after the where\n  .n_rows()\n// zeff: allow(W0201) the sum skips rows by design\nlet e = d.sum(x)\nlet f = d.sum(y)  // zeff: allow(W0201) the same\nfor i in 1..3 { // on the brace\n  // inside\n  print(i)\n\n\n\n  print(i + 1)\n  // at the end of the block\n}\nlet g = [1, // one\n  2]\nfor j in 1..3{print(j)   // of the statement, not of the brace\n}\nlet h = if a {  // of the brace\n1 } else { 2 // of the two\n}\nlet p = wages.where(year >= 2010 && age >= 18).add(lwage = log(wage)).aggregate(mean_lwage = mean(lwage), by: firm) // zeff: allow(W0201) stays on the first line of its statement\nlet q = f( // on the bracket\nx)\n// the last line\n"
        )
    );
    // Carriage returns, tabs and spaces at the ends of lines go.
    assert_eq!(
        formatted("let a = 1  \r\n\tlet b = 2\t\r\n\r\n\r\n\r\n\r\nlet c = 3"),
        "let a = 1\nlet b = 2\n\n\nlet c = 3\n"
    );
    // A script of nothing, and of comments alone.
    assert_eq!(formatted(""), "");
    assert_eq!(formatted("\n\n  \n"), "");
    assert_eq!(formatted("  // only this  \n\n\n"), "// only this\n");
}

#[test]
fn blocks_functions_models_and_graphs() {
    insta::assert_snapshot!(
        "blocks",
        shown(
            "/// Three checks.\n///\n///   an indented line of the page\n/// data: The frame.\n/// > robustness(d)\nfn robustness(data:Frame,f:Formula,cluster:Var=firm)->List<RegFit>{\n[\ndata.reg(f, se: robust),\ndata.reg(f, cluster: cluster),\n]\n}\n\nfn with_many_parameters(first_parameter: Column<Float>, second_parameter: Fn(Float, ..) -> Float, third: {a: Int, b: Str}, fourth: classical | robust = classical, fifth: Float? = NA) -> Nothing {\nif first { return }\nfor x in xs {\nif x > 1 {\ncontinue\n} else if x < 0 { break } else {\nprint(x)\n}\n}\n}\nlet m = model {\nengagement =~ eng1 + eng2\nsupport =~ sup1 + sup2 + sup3\nengagement ~ support as a\nturnover ~ engagement as b + support as c\nindirect := a * b\neng1 ~~ eng2\n}\nlet g = dag {\nability->training->productivity\n[age,region] -> training\nx <-> y -- z\nlatent ability,u\nselected s\n}\nlet h = dag { a -> b }\nlet k = case {\nage < 25 => \"young\",\nage < 55 && tenure > 10 => { let y = 2\n y },\n_ => \"older\"\n}\nlet empty = if a {} else {}\n"
        )
    );
}

#[test]
fn a_script_that_does_not_parse_is_left_alone() {
    for text in [
        "let = 3\n",
        "let a = (1 + \n",
        "x.\n",
        "let a = 1 +\n",
        "fn f( {\n",
    ] {
        assert!(
            matches!(format("t.zeff", text), Err(Unformatted::Syntax(_))),
            "{text}"
        );
    }
}

/// The scripts of the repository: the sample script of the specification, the scripts and
/// the functions of the sample projects, and the functions of the packages written in Zeff.
fn scripts() -> Vec<(String, String)> {
    let mut found = Vec::new();
    let mut add = |path: PathBuf| {
        let text =
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let name = path
            .strip_prefix(root())
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        found.push((name, text));
    };
    add(root().join("docs/spec/zeff_samples.zeff"));
    let mut folders: Vec<PathBuf> = Vec::new();
    for sample in std::fs::read_dir(root().join("samples")).unwrap().flatten() {
        folders.push(sample.path().join("scripts"));
        folders.push(sample.path().join("functions"));
    }
    for tier in std::fs::read_dir(root().join("packages"))
        .unwrap()
        .flatten()
    {
        if tier.path().is_dir() {
            for package in std::fs::read_dir(tier.path()).unwrap().flatten() {
                folders.push(package.path().join("functions"));
            }
        }
    }
    folders.sort();
    for folder in folders {
        let Ok(entries) = std::fs::read_dir(&folder) else {
            continue;
        };
        let mut files: Vec<PathBuf> = entries
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|e| e == "zeff"))
            .collect();
        files.sort();
        for file in files {
            add(file);
        }
    }
    found
}

#[test]
fn every_script_of_the_repository_is_formatted_and_stays_so() {
    let all = scripts();
    assert!(all.len() >= 24, "{} scripts found", all.len());
    for (name, text) in &all {
        let written = match format(name, text) {
            Ok(written) => written,
            Err(why) => panic!("{name}: {why:?}"),
        };
        assert_eq!(formatted(&written), written, "{name} is not stable");
        // The documentation lines of a function written in Zeff are its help page: they
        // come through as they are.
        for line in text
            .lines()
            .filter(|line| line.trim_start().starts_with("///"))
        {
            assert!(
                written.lines().any(|kept| kept.trim() == line.trim()),
                "{name}: {line}"
            );
        }
    }
    let sample = &all[0];
    insta::assert_snapshot!("sample_script_formatted", formatted(&sample.1));
}

fn property_cases(usual: u32) -> u32 {
    std::env::var("ZEFF_PROPERTY_MULTIPLIER")
        .ok()
        .and_then(|m| m.parse::<u32>().ok())
        .map_or(usual, |m| usual.saturating_mul(m))
}

fn code_like() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-zA-Z0-9_ \\n\"`.,:=~+*/^!<>&|(){}\\[\\]-]{0,120}").unwrap()
}

/// A script with the indentation of each line, the spaces at its end and its line ending
/// changed at random: what the formatter must see through.
fn deformed(text: &str, choices: &[u8]) -> String {
    let mut out = String::new();
    for (at, line) in text.lines().enumerate() {
        let choice = choices[at % choices.len()];
        // A documentation line keeps the spaces after its slashes, which are its text; the
        // indentation before any line is free.
        for _ in 0..(choice % 7) {
            out.push(' ');
        }
        out.push_str(line.trim_start());
        for _ in 0..(choice / 7 % 3) {
            out.push(' ');
        }
        out.push_str(if choice.is_multiple_of(2) {
            "\n"
        } else {
            "\r\n"
        });
    }
    out
}

/// A script with the spaces between its words changed at random: wherever two tokens of a
/// line stand apart, they stand one to four spaces apart. No line is joined or broken, and
/// no comment is touched.
fn respaced(text: &str, choices: &[u8]) -> String {
    let tokens = zeff_syntax::lex("t.zeff", text).tokens;
    let mut out = String::new();
    let mut at = 0;
    let mut gaps = 0;
    for token in &tokens {
        if token.span.start < at {
            continue;
        }
        let gap = &text[at..token.span.start];
        if !gap.is_empty() && at > 0 && gap.chars().all(|c| c == ' ') {
            let line_start = text[..at].rfind('\n').map_or(0, |end| end + 1);
            // The spaces before the first word of a line are its indentation, not a gap.
            if text[line_start..at].trim().is_empty() {
                out.push_str(gap);
            } else {
                let width = 1 + usize::from(choices[gaps % choices.len()] % 4);
                out.push_str(&" ".repeat(width));
                gaps += 1;
            }
        } else {
            out.push_str(gap);
        }
        out.push_str(&text[token.span.start..token.span.end]);
        at = token.span.end;
    }
    out.push_str(&text[at..]);
    out
}

/// A script with comments written into it at random: after some of its tokens the line is
/// ended with a comment or with a pragma, numbered, and the rest of the line goes to the
/// next. Where the script still parses, the formatter must keep every one of them.
fn commented(text: &str, choices: &[u8]) -> String {
    let tokens = zeff_syntax::lex("t.zeff", text).tokens;
    let mut out = String::new();
    let mut at = 0;
    for (count, token) in tokens.iter().enumerate() {
        if token.span.start < at || token.span.end == token.span.start {
            continue;
        }
        out.push_str(&text[at..token.span.end]);
        at = token.span.end;
        let choice = choices[count % choices.len()];
        let rest = &text[at..];
        let line = rest.find('\n').map_or(rest, |end| &rest[..end]);
        // One token in eight or so, and never where the line already ends with a comment
        // or inside a documentation line.
        if choice.is_multiple_of(8)
            && !line.contains("//")
            && !text[token.span.start..at].contains("//")
        {
            if choice.is_multiple_of(16) {
                out.push_str(&format!(" // c{count}\n"));
            } else {
                // A pragma: where it belongs to a statement the script still parses, and it
                // must stay on that statement.
                out.push_str(&format!(" // zeff: allow(W0201) c{count}\n"));
            }
        }
    }
    out.push_str(&text[at..]);
    out
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(64)))]

    /// Comments written anywhere into a script of the repository: where it still parses,
    /// every comment is kept, in its order, the tree is the tree of the script (or `format`
    /// would say that they are not), and formatting again changes nothing.
    #[test]
    fn comments_written_anywhere_are_kept(
        which in 0usize..64,
        choices in prop::collection::vec(any::<u8>(), 1..40),
    ) {
        let all = scripts();
        let (name, text) = &all[which % all.len()];
        let bent = commented(text, &choices);
        match format(name, &bent) {
            Err(Unformatted::Syntax(_)) => {}
            Err(Unformatted::Internal(what)) => prop_assert!(false, "{}: {}:\n{}", name, what, bent),
            Ok(written) => {
                for mark in ["// c", "// zeff: allow(W0201) c"] {
                    prop_assert_eq!(
                        written.matches(mark).count(),
                        bent.matches(mark).count(),
                        "{}", name
                    );
                }
                match format(name, &written) {
                    Ok(again) => prop_assert_eq!(again, written, "{}:\n{}", name, bent),
                    Err(why) => prop_assert!(false, "{}: {:?}", name, why),
                }
            }
        }
    }

    /// Whatever the spaces between the words of a script of the repository, it is
    /// formatted to what the script itself is formatted to.
    #[test]
    fn the_spaces_between_the_words_of_a_script_change_nothing(
        which in 0usize..64,
        choices in prop::collection::vec(any::<u8>(), 1..40),
    ) {
        let all = scripts();
        let (name, text) = &all[which % all.len()];
        let straight = formatted(text);
        let bent = respaced(text, &choices);
        match format(name, &bent) {
            Ok(written) => prop_assert_eq!(written, straight, "{}", name),
            Err(why) => prop_assert!(false, "{}: {:?}", name, why),
        }
    }

    /// Whatever the indentation, the spaces at the ends of the lines and the line endings
    /// of a script of the repository: it is formatted to what the script itself is
    /// formatted to.
    #[test]
    fn the_indentation_and_the_line_ends_of_a_script_change_nothing(
        which in 0usize..64,
        choices in prop::collection::vec(any::<u8>(), 1..40),
    ) {
        let all = scripts();
        let (name, text) = &all[which % all.len()];
        let straight = formatted(text);
        let bent = deformed(text, &choices);
        match format(name, &bent) {
            Ok(written) => prop_assert_eq!(written, straight, "{}", name),
            Err(why) => prop_assert!(false, "{}: {:?}", name, why),
        }
    }

    /// Whatever the text: where it parses it is formatted, the tree and the comments kept
    /// (or `format` would say that they are not), and formatting it again changes nothing.
    #[test]
    fn any_script_that_parses_is_formatted_and_stays_so(text in code_like()) {
        match format("t.zeff", &text) {
            Err(Unformatted::Syntax(_)) => {}
            Err(Unformatted::Internal(what)) => prop_assert!(false, "{}:\n{}", what, text),
            Ok(written) => {
                match format("t.zeff", &written) {
                    Ok(again) => prop_assert_eq!(again, written.clone(), "{}", text),
                    Err(why) => prop_assert!(false, "{:?}:\n{}", why, written),
                }
                for line in written.lines() {
                    prop_assert!(!line.ends_with(' '), "{:?}", line);
                }
            }
        }
    }
}
