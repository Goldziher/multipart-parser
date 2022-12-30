use crate::parse_content_header;
use encoding_rs::{Encoding, UTF_8};
use lazy_static::lazy_static;
use percent_encoding::percent_decode;
use regex::Regex;
use serde_json::Value;
use std::borrow::Cow::{Borrowed, Owned};
use std::collections::HashMap;

lazy_static! {
    static ref CRLF_SANITIZATION_REGEX: Regex = Regex::new(r"(^\r\n|\r\n$)").unwrap();
    static ref BOUNDARY_SANITIZE_REGEX: Regex = Regex::new(r"(^-*|-*$)").unwrap();
}

#[derive(Debug, PartialEq, Eq)]
pub struct UploadFile {
    content_type: String,
    filename: String,
    headers: HashMap<String, String>,
    content: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct JsonField {
    content_type: String,
    headers: HashMap<String, String>,
    content: Value,
}

#[derive(Debug, PartialEq, Eq)]
pub struct StringField {
    content_type: String,
    headers: HashMap<String, String>,
    content: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Field {
    File(UploadFile),
    Json(JsonField),
    String(StringField),
}

#[inline]
fn extract_filename(options: HashMap<String, String>) -> String {
    match options.get("filename*") {
        Some(filename_with_asterisk) => {
            let mut parts = filename_with_asterisk.splitn(3, '\'');

            let charset = match parts.next() {
                None => "UTF-8",
                Some(charset) => charset,
            };
            // we have no use for the language component - if its sent.
            parts.next();

            let encoded_filename = match parts.next() {
                None => "",
                Some(filename) => filename,
            };

            let percent_decoded: Vec<u8> =
                percent_decode(encoded_filename.as_bytes()).collect::<Vec<u8>>();

            let (decoded, ..) = Encoding::for_label(charset.as_bytes())
                .unwrap_or(UTF_8)
                .decode(percent_decoded.as_slice());

            match decoded {
                Borrowed(value) => value.to_owned(),
                Owned(value) => value,
            }
        }
        None => match options.get("filename") {
            None => String::from(""),
            Some(filename) => filename.to_string(),
        },
    }
}

pub fn parse_multipart_form_data(
    body: &[u8],
    boundary: &[u8],
    charset: &[u8],
) -> HashMap<String, Field> {
    let mut result: HashMap<String, Field> = HashMap::new();

    let encoding = Encoding::for_label(charset).unwrap_or(UTF_8);
    let (string_body, ..) = encoding.decode(body);
    let (string_boundary, ..) = encoding.decode(boundary);

    for form_part in string_body
        .split(string_boundary.as_ref())
        .map(|el| BOUNDARY_SANITIZE_REGEX.replace_all(el, ""))
        .filter(|el| !el.is_empty())
    {
        let mut filename: String = "".to_owned();
        let mut field_name: String = "".to_owned();
        let mut headers: Vec<(String, String)> = Vec::new();
        let mut content_type: String = "text/plain".to_owned();

        match form_part.split_once("\r\n\r\n") {
            None => continue,
            Some((headers_value, content_value)) => {
                if headers_value.contains(':') {
                    for (header_key, header_value) in headers_value
                        .split("\r\n")
                        .filter_map(|part| part.split_once(':'))
                        .map(|(key, value)| (key.to_lowercase(), value.trim().to_owned()))
                    {
                        if header_key.to_lowercase() == "content-type" {
                            content_type = header_value.clone();
                        }

                        if header_key.to_lowercase() == "content-disposition" {
                            let (value, options) =
                                parse_content_header(header_value.as_str()).to_owned();

                            field_name = match options.get("name") {
                                None => String::from(""),
                                Some(value) => value.to_string(),
                            };
                            filename = extract_filename(options);

                            headers.push((header_key.to_owned(), value));
                        } else {
                            headers.push((header_key.to_owned(), header_value));
                        }
                    }
                }

                if !field_name.is_empty() {
                    let post_data = match CRLF_SANITIZATION_REGEX.replace_all(content_value, "") {
                        Borrowed(result) => result.to_owned(),
                        Owned(result) => result,
                    };

                    if !filename.is_empty() {
                        result.insert(
                            field_name.to_owned(),
                            Field::File(UploadFile {
                                content_type: content_type.to_owned(),
                                filename: filename.to_owned(),
                                headers: HashMap::from_iter(headers),
                                content: post_data,
                            }),
                        );
                    } else {
                        match serde_json::from_str(post_data.as_str()) {
                            Ok(json_value) => {
                                result.insert(
                                    field_name.to_owned(),
                                    Field::Json(JsonField {
                                        content_type: content_type.to_owned(),
                                        headers: HashMap::from_iter(headers),
                                        content: json_value,
                                    }),
                                );
                            }
                            Err(_) => {
                                result.insert(
                                    field_name.to_owned(),
                                    Field::String(StringField {
                                        content_type: content_type.to_owned(),
                                        headers: HashMap::from_iter(headers),
                                        content: post_data,
                                    }),
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_postman_multipart() {
        let body = b"----------------------------850116600781883365617864\r\nContent-Disposition: form-data; name=\"attributes\"; filename=\"test-attribute_5.tsv\"\r\nContent-Type: text/tab-separated-values\r\n\r\n\"Campaign ID\"\t\"Plate Set ID\"\t\"No\"\n\r\n----------------------------850116600781883365617864\r\nContent-Disposition: form-data; name=\"fasta\"; filename=\"test-sequence_correct_5.fasta\"\r\nContent-Type: application/octet-stream\r\n\r\n>P23G01_IgG1-1411:H:Q10C3:1/1:NID18\r\nCAGGTATTGAA\r\n\r\n----------------------------850116600781883365617864--\r\n";
        let boundary = b"----------------------------850116600781883365617864";
        let result = parse_multipart_form_data(body, boundary, b"utf-8");

        let attributes = match result.get("attributes").unwrap() {
            Field::File(field) => field,
            _ => panic!("value should be an UploadFile"),
        };
        assert_eq!(attributes.content_type, "text/tab-separated-values");
        assert_eq!(attributes.filename, "test-attribute_5.tsv");
        assert_eq!(
            attributes.content,
            "\"Campaign ID\"\t\"Plate Set ID\"\t\"No\"\n"
        );

        let fasta = match result.get("fasta").unwrap() {
            Field::File(field) => field,
            _ => panic!("value should be an UploadFile"),
        };
        assert_eq!(fasta.content_type, "application/octet-stream");
        assert_eq!(fasta.filename, "test-sequence_correct_5.fasta");
        assert_eq!(
            fasta.content,
            ">P23G01_IgG1-1411:H:Q10C3:1/1:NID18\r\nCAGGTATTGAA\r\n"
        );
    }

    #[test]
    fn test_parse_encoded_value() {
        let body = b"--20b303e711c4ab8c443184ac833ab00f\r\nContent-Disposition: form-data; name=\"value\"\r\n\r\nTransf\xc3\xa9rer\r\n--20b303e711c4ab8c443184ac833ab00f--\r\n";
        let boundary = b"20b303e711c4ab8c443184ac833ab00f";
        let result = parse_multipart_form_data(body, boundary, b"utf-8");

        let attributes = match result.get("value").unwrap() {
            Field::String(field) => field,
            _ => panic!("value should be a String"),
        };
        assert_eq!(attributes.content, "Transférer");
    }

    #[test]
    fn test_parse_asian_characters() {
        let body = b"--a7f7ac8d4e2e437c877bb7b8d7cc549c\r\nContent-Disposition: form-data; name=\"file\"; filename=\"\xe7\x94\xbb\xe5\x83\x8f.jpg\"\r\nContent-Type: image/jpeg\r\n\r\n<file content>\r\n--a7f7ac8d4e2e437c877bb7b8d7cc549c--\r\n";
        let boundary = b"a7f7ac8d4e2e437c877bb7b8d7cc549c";
        let result = parse_multipart_form_data(body, boundary, b"utf-8");

        let file = match result.get("file").unwrap() {
            Field::File(field) => field,
            _ => panic!("value should be an UploadFile"),
        };

        assert_eq!(file.content_type, "image/jpeg");
        assert_eq!(file.filename, "画像.jpg");
        assert_eq!(file.content, "<file content>");
    }

    #[test]
    fn test_parse_filename_with_extended_value() {
        let body = b"--a7f7ac8d4e2e437c877bb7b8d7cc549c\r\nContent-Disposition: form-data; name='file'; filename*=UTF-8''Na%C3%AFve%20file.jpg\r\nContent-Type: image/jpeg\r\n\r\n<file content>\r\n--a7f7ac8d4e2e437c877bb7b8d7cc549c--\r\n";
        let boundary = b"a7f7ac8d4e2e437c877bb7b8d7cc549c";
        let result = parse_multipart_form_data(body, boundary, b"utf-8");

        let file = match result.get("file").unwrap() {
            Field::File(field) => field,
            _ => panic!("value should be an UploadFile"),
        };

        assert_eq!(file.content_type, "image/jpeg");
        assert_eq!(file.filename, "Naïve file.jpg");
        assert_eq!(file.content, "<file content>");
    }

    #[test]
    fn test_parse_filename_with_extended_value_with_language_tag() {
        let body = b"--a7f7ac8d4e2e437c877bb7b8d7cc549c\r\nContent-Disposition: form-data; name='file'; filename*=UTF-8'en'Na%C3%AFve%20file.jpg\r\nContent-Type: image/jpeg\r\n\r\n<file content>\r\n--a7f7ac8d4e2e437c877bb7b8d7cc549c--\r\n";
        let boundary = b"a7f7ac8d4e2e437c877bb7b8d7cc549c";
        let result = parse_multipart_form_data(body, boundary, b"utf-8");

        let file = match result.get("file").unwrap() {
            Field::File(field) => field,
            _ => panic!("value should be an UploadFile"),
        };

        assert_eq!(file.content_type, "image/jpeg");
        assert_eq!(file.filename, "Naïve file.jpg");
        assert_eq!(file.content, "<file content>");
    }

    #[test]
    fn test_mixed_files_and_form_data() {
        let body = b"--a7f7ac8d4e2e437c877bb7b8d7cc549c\r\nContent-Disposition: form-data; name=\"field0\"\r\n\r\nvalue0\r\n--a7f7ac8d4e2e437c877bb7b8d7cc549c\r\nContent-Disposition: form-data; name=\"file\"; filename=\"file.txt\"\r\nContent-Type: text/plain\r\n\r\n<file content>\r\n--a7f7ac8d4e2e437c877bb7b8d7cc549c\r\nContent-Disposition: form-data; name=\"field1\"\r\n\r\nvalue1\r\n--a7f7ac8d4e2e437c877bb7b8d7cc549c--\r\n";
        let boundary = b"a7f7ac8d4e2e437c877bb7b8d7cc549c";
        let result = parse_multipart_form_data(body, boundary, b"utf-8");

        let file = match result.get("file").unwrap() {
            Field::File(field) => field,
            _ => panic!("value should be an UploadFile"),
        };

        assert_eq!(file.content_type, "text/plain");
        assert_eq!(file.filename, "file.txt");
        assert_eq!(file.content, "<file content>");

        let string_field_1 = match result.get("field0").unwrap() {
            Field::String(field) => field,
            _ => panic!("value should be an UploadFile"),
        };

        assert_eq!(string_field_1.content_type, "text/plain");
        assert_eq!(string_field_1.content, "value0");

        let string_field_2 = match result.get("field1").unwrap() {
            Field::String(field) => field,
            _ => panic!("value should be an UploadFile"),
        };

        assert_eq!(string_field_2.content_type, "text/plain");
        assert_eq!(string_field_2.content, "value1");
    }
}
