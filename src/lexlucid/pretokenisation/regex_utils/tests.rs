use regex::Captures;

use crate::lexlucid::pretokenisation::regex_utils::pretokeniser_regex;

use super::constrained_captures;

#[test]
fn suffixless() {
    #[rustfmt::skip]
    let re = pretokeniser_regex(r##"\A
        (?<ats_1>
          @ {1,255}
        )
          A [A@]*?
        (?<ats_2>
          @ {1,255}
        )
        \z"##);

    fn constraint(captures: &Captures) -> bool {
        captures.name("ats_1").unwrap().as_str() == captures.name("ats_2").unwrap().as_str()
    }

    let captures = constrained_captures(&re, constraint, "@@@AA@A@@A@@@@AAA@AAA ").unwrap();
    assert_eq!(&captures[0], "@@@AA@A@@A@@@");
}

#[test]
fn raw_literal() {
    #[rustfmt::skip]
    let re = pretokeniser_regex(r##"\A
        (?<prefix>
          r | br | cr
        )
        (?<hashes_1>
          \# {1,255}
        )
        " .*? "
        (?<hashes_2>
          \# {1,255}
        )
        (?<suffix>
          (?:
            # <identifier>
            [ \p{XID_Start} ]
            \p{XID_Continue} *
          ) ?
        )
        \z"##);

    fn constraint(captures: &Captures) -> bool {
        captures.name("hashes_1").unwrap().as_str() == captures.name("hashes_2").unwrap().as_str()
    }

    let captures = constrained_captures(&re, constraint, r###"r#"a£)"#suff "###).unwrap();
    assert_eq!(&captures[0], r###"r#"a£)"#suff"###);

    let captures = constrained_captures(&re, constraint, r###"r#"a£)"#suff"###).unwrap();
    assert_eq!(&captures[0], r###"r#"a£)"#suff"###);

    let captures = constrained_captures(&re, constraint, r###"r#"a£)"#"###).unwrap();
    assert_eq!(&captures[0], r###"r#"a£)"#"###);

    let captures = constrained_captures(&re, constraint, r###"r##"a£)" "# "##suff "###).unwrap();
    assert_eq!(&captures[0], r###"r##"a£)" "# "##suff"###);

    let captures = constrained_captures(&re, constraint, "");
    assert!(captures.is_none());
}
