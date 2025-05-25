use proptest::{
    array::uniform3,
    sample::select,
    strategy::{BoxedStrategy, Strategy},
    string::string_regex,
};

pub const DEFAULT_STRATEGY: &str = "mix";

#[rustfmt::skip]
pub const SIMPLE_STRATEGIES: &[(&str, &str)] = [
    ("whitespace",             r#"[a \t\n\r\x85\u200E\u200F\u2028\u2029]{1,8}"#),
    ("block-comment",          r#"[/*!a \n]{1,16}"#),
    ("line-comment",           r#"[/! a\n]{1,10}"#),
    ("punctuation",            r#"[-!#$%&*+,./:;<=>?@^_|~ ]{1,8}"#),
    ("identifier",             "[_#ra1£·áΩ🦀\x07\u{FFFF}. ]{1,12}"),
    ("lifetime",               "['#ra1£·🦀]{1,8}"),
    ("string-literal",         r#"[\\\n#'"rbcx _]{1,12}"#),
    ("unicode-escape",         r#""\\u\{.{0,8}[} ]""#),
    ("hashed-raw",             r#"(r|br|cr)#[\\\n#"rx _]{1,10}"#),
    ("nulls",                  "[\\\\\0cr\"#]{1,12}"),
    ("newlines",               r#"[\\"'#rbcx\n ]{1,10}"#),
    ("crs",                    r#"[\\/*!"'#rbcx\r\n ]{1,10}"#),
    ("numeric-literal",        r#"[01][-+._012389abcdefghoxABCDEYZHOX·]{1,16}"#),
    ("numeric-literal-narrow", r#"[01][-+._019aeE·]{1,12}"#),
    ("shebang",                r#"([!#\na/*]|\[!?attrlike\]){1,12}"#),
    ("delimiters",             r#"[\\[\\](){} a]{1,12}"#),
]
.as_slice();

pub(crate) fn any_char() -> BoxedStrategy<String> {
    proptest::char::any().prop_map(|c| c.to_string()).boxed()
}

/// Strategy returning sequences made from a mix of some of the simple strategies.
pub(crate) fn mix() -> BoxedStrategy<String> {
    // These are shortened from the simple strategies above
    const BLOCKS: &[&str] = [
        r#"[a \t\n\r\x85\u200E\u200F\u2028\u2029]{1,3}"#, // whitespace
        r#"[/*!a \n]{1,8}"#,                              // block-comment
        r#"[/! a\n]{1,5}"#,                               // line-comment
        r#"[-!#$%&*+,./:;<=>?@^_|~ ]{1,5}"#,              // punctuation
        "[_#ra£·áΩ🦀\x07\u{FFFF}. ]{1,3}",                // identifier
        "['#ra]{1,3}",                                    // lifetime
        r#"[\\\n"'#rbcx ]{1,8}"#,                         // string-literal
        r#"[01][-+._012389abcdefghoxABCDEYZHOX]{1,8}"#,   // numeric-literal
        "\0",                                             // just a NUL
    ]
    .as_slice();

    uniform3(select(BLOCKS))
        .prop_flat_map(|inputs| string_regex(&inputs.join("")).unwrap())
        .boxed()
}
