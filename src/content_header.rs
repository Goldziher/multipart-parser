use lazy_static::lazy_static;
use regex::Regex;
use std::borrow::{Borrow, Cow};
use std::collections::HashMap;

lazy_static! {
    static ref PARENTHESES_RE: Regex = Regex::new(r#"\s*"\s*"#).unwrap();
    static ref ESCAPE_RE: Regex = Regex::new(r#"\\"\s*"#).unwrap();
}

#[inline]
fn unescape(value: &str) -> Cow<str> {
    ESCAPE_RE.replace_all(value, "")
}

#[inline]
fn trim_parentheses(value: &str) -> Cow<str> {
    PARENTHESES_RE.replace_all(value, "")
}

pub fn parse_content_header(header: &str) -> (String, HashMap<String, String>) {
    let unescaped = unescape(header);
    let normalized_header = trim_parentheses(unescaped.borrow());
    match normalized_header.split_once("; ") {
        Some(split_result) => {
            let (header_value, options_str) = split_result;
            let options = HashMap::from_iter(
                options_str
                    .split("; ")
                    .filter_map(|value| value.split_once('='))
                    .map(|value| (value.0.trim().to_owned(), value.1.trim().to_owned())),
            );
            return (header_value.trim().to_owned(), options);
        }
        None => (String::from(""), HashMap::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unescape() {
        assert_eq!("", unescape("\\\"    "));
        assert_eq!("", unescape("\\\""));
        assert_eq!("form-data;", unescape("form-data\\\"          ;"));
    }

    #[test]
    fn test_trim_parentheses() {
        assert_eq!("", trim_parentheses("\"    "));
        assert_eq!("", trim_parentheses("      \"    "));
        assert_eq!("", trim_parentheses("\"\""));
        assert_eq!("", trim_parentheses("\""));
    }

    #[test]
    fn test_parse_content_header_regular_header() {
        let (header_value, options) =
            parse_content_header(r#"form-data; name="attributes"; filename="test-attribute_5.tsv"#);

        assert_eq!(header_value, String::from("form-data"));
        assert_eq!(
            options,
            HashMap::from([
                (String::from("name"), String::from("attributes")),
                (
                    String::from("filename"),
                    String::from("test-attribute_5.tsv")
                )
            ])
        )
    }

    #[test]
    fn test_parse_content_header_escaped_non_ascii_1() {
        let (header_value, options) =
            parse_content_header(r#"form-data; name=\"你好\"; filename=\"file abc.txt\""#);

        assert_eq!(header_value, String::from("form-data"));
        assert_eq!(
            options,
            HashMap::from([
                (String::from("name"), String::from("你好")),
                (String::from("filename"), String::from("file abc.txt"))
            ])
        );

        let (header_value, options) =
            parse_content_header(r#"form-data; name=\"কখগ\"; filename=\"你好.txt\""#);

        assert_eq!(header_value, String::from("form-data"));
        assert_eq!(
            options,
            HashMap::from([
                (String::from("name"), String::from("কখগ")),
                (String::from("filename"), String::from("你好.txt"))
            ])
        );

        let (header_value, options) =
            parse_content_header(r#"form-data; name=\"কখগ-你好\"; filename=\"কখগ-你好.txt\""#);

        assert_eq!(header_value, String::from("form-data"));
        assert_eq!(
            options,
            HashMap::from([
                (String::from("name"), String::from("কখগ-你好")),
                (String::from("filename"), String::from("কখগ-你好.txt"))
            ])
        )
    }

    #[test]
    fn test_parse_content_header_unquoted() {
        let (header_value, options) =
            parse_content_header(r#"form-data; name=my_field; filename=file-name.txt"#);

        assert_eq!(header_value, String::from("form-data"));
        assert_eq!(
            options,
            HashMap::from([
                (String::from("name"), String::from("my_field")),
                (String::from("filename"), String::from("file-name.txt"))
            ])
        )
    }

    #[test]
    fn test_parse_content_header_quoted() {
        let (header_value, options) = parse_content_header(r#"form-data; name="my;f;ield""#);

        assert_eq!(header_value, String::from("form-data"));
        assert_eq!(
            options,
            HashMap::from([(String::from("name"), String::from("my;f;ield")),])
        );

        let (header_value, options) =
            parse_content_header(r#"form-data; name=my_field; filename="file;name.txt""#);

        assert_eq!(header_value, String::from("form-data"));
        assert_eq!(
            options,
            HashMap::from([
                (String::from("name"), String::from("my_field")),
                (String::from("filename"), String::from("file;name.txt"))
            ])
        );

        let (header_value, options) =
            parse_content_header(r#"form-data; name=; filename=filename.txt"#);

        assert_eq!(header_value, String::from("form-data"));
        assert_eq!(
            options,
            HashMap::from([
                (String::from("name"), String::from("")),
                (String::from("filename"), String::from("filename.txt"))
            ])
        );

        let (header_value, options) = parse_content_header(r#"form-data; name=";"; filename=";""#);

        assert_eq!(header_value, String::from("form-data"));
        assert_eq!(
            options,
            HashMap::from([
                (String::from("name"), String::from(";")),
                (String::from("filename"), String::from(";"))
            ])
        );

        let (header_value, options) = parse_content_header(r#"form-data; name="my\"field\"name""#);

        assert_eq!(header_value, String::from("form-data"));
        assert_eq!(
            options,
            HashMap::from([(String::from("name"), String::from("myfieldname")),])
        )
    }
}
