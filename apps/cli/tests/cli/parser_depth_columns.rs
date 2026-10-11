//! Uncaught parser stack-overflow diagnostics point at the first body or
//! operand token of the deepest nesting level, as QuickJS's `next_token()`
//! check does, and exit with status 1.

use std::process::Command;

fn run(binary: &str, options: &[&str], source: &str) -> (i32, Vec<u8>, Vec<u8>) {
    let output = Command::new(binary)
        .args(options)
        .args(["-e", source])
        .output()
        .expect("run qjs for parser-depth column case");
    (
        output.status.code().unwrap_or(-1),
        output.stdout,
        output.stderr,
    )
}

/// One uncaught overflow case: source and the exact stderr.
struct Case {
    name: &'static str,
    source: String,
    expected_stderr: &'static str,
}

fn repeat(left: &str, leaf: &str, right: &str, depth: usize) -> String {
    format!("{}{leaf}{}", left.repeat(depth), right.repeat(depth))
}

fn cases() -> Vec<Case> {
    vec![
        // Direct roots report `<cmdline>` locations.
        Case {
            name: "direct-block-3266",
            source: repeat("{", "0", "}", 3266),
            expected_stderr: "SyntaxError: stack overflow\n    at <cmdline>:1:3267\n",
        },
        Case {
            name: "direct-if-3438",
            source: repeat("if(1)", "0", "", 3438),
            expected_stderr: "SyntaxError: stack overflow\n    at <cmdline>:1:17191\n",
        },
        Case {
            name: "direct-label-3438",
            source: (0..3438).map(|i| format!("L{i}:")).collect::<String>() + "0",
            expected_stderr: "SyntaxError: stack overflow\n    at <cmdline>:1:19519\n",
        },
        // eval-wrapped sources report the synthesized `<input>` location plus
        // the `<eval>` caller frame.
        Case {
            name: "eval-new-noargs-5935",
            source: format!(
                "eval(\"function z(){{return {}}}\")",
                repeat("new ", "Object", "", 5935)
            ),
            expected_stderr: "SyntaxError: stack overflow\n    at <input>:1:23761\n    at <eval> (<cmdline>:1:5)\n",
        },
        Case {
            name: "eval-import-742",
            source: format!(
                "eval(\"function z(){{return {}}}\")",
                repeat("import(", "0", ")", 742)
            ),
            expected_stderr: "SyntaxError: stack overflow\n    at <input>:1:5215\n    at <eval> (<cmdline>:1:5)\n",
        },
    ]
}

#[test]
fn parser_stack_overflow_points_at_the_deepest_body_token() {
    let oxide = env!("CARGO_BIN_EXE_qjs");
    for case in cases() {
        let (code, stdout, stderr) = run(oxide, &[], &case.source);
        assert_eq!(code, 1, "{} must exit 1", case.name);
        assert!(stdout.is_empty(), "{} must print nothing", case.name);
        assert_eq!(
            String::from_utf8(stderr).expect("utf8 stderr"),
            case.expected_stderr,
            "{} stderr location",
            case.name,
        );
    }
}
