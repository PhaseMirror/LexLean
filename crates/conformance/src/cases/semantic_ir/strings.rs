//! Accepted source strings preserve literal UTF-8 through generated Lean.

use crate::support::{self, P};
use serde_json::json;

pub(super) fn verify() {
    let project = P::example();
    project.edit("lexlean.toml", "language = \"1.0\"", "language = \"1.1\"");
    let controls = (0..=159)
        .filter(|value| ![37, 127, 133].contains(value))
        .map(|value| char::from_u32(value).expect("ASCII and C1 scalar"))
        .collect::<String>();
    let values = [
        controls.as_str(),
        "\0a\u{1}f\u{80}a",
        r#"literal \0 \u{80} \u0080 \x00 \n " ' \"#,
        "\u{ad}\u{7ff}\u{800}\u{d7ff}\u{e000}\u{ffff}",
        "\u{10000}\u{1f600}\u{e0100}\u{10ffff}",
        "\u{301}\u{200b}",
        "comment data /- -- -/ and braces {}",
        "",
    ];
    let declarations = values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let hex = value
                .as_bytes()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>();
            json!({"kind":"definition","name":format!("literal{index}"),"parameters":[],
                "result":{"kind":"bool"},"axioms":[],
                "body":{"kind":"primitive","operation":"equal","result":{"kind":"bool"},
                    "arguments":[{"kind":"primitive","operation":"utf8_encode","result":{"kind":"bytes"},
                        "arguments":[{"kind":"string","value":value}]},
                        {"kind":"bytes","hex":hex}]}})
        })
        .collect::<Vec<_>>();
    let module = json!({"spec":"lexlean/semantic-module/1","declarations":declarations});
    project.write("src/Main.lex.tex", &format!(
        "\\begin{{lexlean}}{{Main}}\n\\useglossary{{lexlean.std.nat@1.1.0}}\n\\title{{Natural number addition}}\n\\begin{{semanticmodule}}\n\\semanticdata{{{module}}}\n\\end{{semanticmodule}}\n\\end{{lexlean}}\n"
    ));
    project.relock();
    let _ = support::verify_ok_backed("SM-19", &project);
    if support::lean_backed("SM-19") {
        let built = project.build_ok();
        let source = project
            .build_dir(&built.build_id.expect("build identity"))
            .join("modules/LexLeanExample/Main.lean");
        let lean = std::fs::read_to_string(source).expect("actual generated module");
        let evaluations = (0..values.len())
            .map(|index| format!("#eval LexLeanExample.Main.literal{index}\n"))
            .collect::<String>();
        let directory = tempfile::tempdir().expect("evaluation directory");
        let path = directory.path().join("LiteralRuntime.lean");
        std::fs::write(&path, format!("{lean}\n{evaluations}"))
            .expect("generated module and probes");
        let binary = support::real_elan_home()
            .join("toolchains")
            .join(support::mangled_toolchain_name())
            .join("bin")
            .join(if cfg!(windows) { "lean.exe" } else { "lean" });
        let output = std::process::Command::new(binary)
            .arg(path)
            .current_dir(directory.path())
            .env("LEAN_PATH", "")
            .output()
            .expect("pinned Lean evaluates every modeled literal");
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            String::from_utf8(output.stdout).expect("UTF-8 output"),
            "true\n".repeat(values.len())
        );
    }
}
