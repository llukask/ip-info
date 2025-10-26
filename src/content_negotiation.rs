//! HTTP Content Negotiation utilities for parsing Accept headers and media types.
//!
//! This module provides functionality for parsing and working with HTTP Accept headers
//! and media types (MIME types) as defined in HTTP/1.1 specifications. It supports
//! parsing media types with parameters, quality values (q-values), and wildcard matching.
//!
//! # Overview
//!
//! The main components of this module are:
//!
//! - [`MediaType`] - Represents a media type with optional parameters
//! - [`Directive`] - Represents an Accept header directive with quality value
//! - [`parse_accept_directive()`] - Parses individual Accept header directives
//! - [`parse_accept()`] - Parses complete Accept headers and sorts by preference
//!
//! # Example
//!
//! ```
//! use ip_info::content_negotiation::{parse_accept, try_parse_accept, MediaType, ParseError};
//! use std::convert::TryFrom;
//!
//! // Parse an Accept header (skips invalid directives)
//! let accept_header = "text/html;q=0.9,application/json,text/plain;q=0.8";
//! let directives = parse_accept(accept_header);
//!
//! // Results are sorted by quality value (preference)
//! assert_eq!(directives[0].q, 1.0); // application/json (default q=1.0)
//! assert_eq!(directives[1].q, 0.9); // text/html
//! assert_eq!(directives[2].q, 0.8); // text/plain
//!
//! // Strict parsing that returns errors
//! let result = try_parse_accept("text/html,invalid,application/json");
//! assert!(result.is_err());
//!
//! // Check media type matching with error handling
//! let html_type = MediaType::try_from("text/html").unwrap();
//! let wildcard = MediaType::try_from("text/*").unwrap();
//! assert!(html_type.matches(&wildcard));
//!
//! // Handle parse errors
//! let invalid = MediaType::try_from("invalid");
//! assert_eq!(invalid.unwrap_err(), ParseError::MissingSlash);
//! ```
//!
//! # References
//!
//! This implementation follows the HTTP specifications:
//! - [RFC 7231](https://tools.ietf.org/html/rfc7231) - HTTP/1.1 Semantics and Content
//! - [RFC 6838](https://tools.ietf.org/html/rfc6838) - Media Type Specifications

use thiserror::Error;

/// Errors that can occur when parsing media types and Accept headers.
#[derive(Error, Debug, PartialEq)]
pub enum ParseError {
    /// The media type string is missing a required '/' separator
    #[error("media type must contain a '/' separator")]
    MissingSlash,
    /// The quality value (q-value) is not a valid float between 0.0 and 1.0
    #[error("invalid quality value: {0}")]
    InvalidQualityValue(String),
}

/// Represents a media type (MIME type) with optional parameters.
///
/// A media type consists of a main type and sub-type separated by a slash,
/// followed by optional parameters in the form `key=value` separated by semicolons.
///
/// This follows the format defined in [RFC 6838](https://tools.ietf.org/html/rfc6838)
/// and [RFC 7231 Section 3.1.1.1](https://tools.ietf.org/html/rfc7231#section-3.1.1.1).
///
/// # Examples
///
/// ```
/// use ip_info::content_negotiation::MediaType;
/// use std::convert::TryFrom;
///
/// let media_type = MediaType::try_from("text/html;charset=utf-8").unwrap();
/// assert_eq!(media_type.main_type, "text");
/// assert_eq!(media_type.sub_type, "html");
/// assert_eq!(media_type.parameters, vec![("charset".to_string(), "utf-8".to_string())]);
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct MediaType {
    /// The main type (e.g., "text", "application", "image")
    pub main_type: String,
    /// The sub-type (e.g., "html", "json", "png")
    pub sub_type: String,
    /// Parameters as key-value pairs (e.g., charset=utf-8)
    pub parameters: Vec<(String, String)>,
}

/// Represents an Accept header directive with a media type and quality value.
///
/// This is used to parse individual parts of an HTTP Accept header, where each
/// directive specifies a media type and an optional quality value (q-value)
/// indicating the client's preference for that media type.
///
/// # Examples
///
/// ```
/// use ip_info::content_negotiation::parse_accept_directive;
///
/// let directive = parse_accept_directive("text/html;q=0.8").unwrap();
/// assert_eq!(directive.media_type.main_type, "text");
/// assert_eq!(directive.media_type.sub_type, "html");
/// assert_eq!(directive.q, 0.8);
/// ```
#[derive(Debug)]
pub struct Directive {
    /// The media type for this directive
    pub media_type: MediaType,
    /// Quality value (0.0 to 1.0), defaults to 1.0 if not specified
    pub q: f32,
}

impl MediaType {
    fn part_matches(a: &str, b: &str) -> bool {
        a == b || a == "*" || b == "*"
    }

    /// Checks if this media type matches another media type.
    ///
    /// Matching follows these rules:
    /// - Exact matches return true (e.g., "text/html" matches "text/html")
    /// - Wildcard "*" matches any value (e.g., "text/*" matches "text/html")
    /// - Full wildcard "*/*" matches any media type
    ///
    /// Parameters are not considered in matching.
    ///
    /// # Examples
    ///
    /// ```
    /// use ip_info::content_negotiation::MediaType;
    /// use std::convert::TryFrom;
    ///
    /// let html = MediaType::try_from("text/html").unwrap();
    /// let wildcard = MediaType::try_from("text/*").unwrap();
    /// let json = MediaType::try_from("application/json").unwrap();
    ///
    /// assert!(html.matches(&wildcard));
    /// assert!(wildcard.matches(&html));
    /// assert!(!html.matches(&json));
    /// ```
    pub fn matches(&self, other: &Self) -> bool {
        let main_matches = Self::part_matches(&self.main_type, &other.main_type);
        let sub_matches = Self::part_matches(&self.sub_type, &other.sub_type);

        main_matches && sub_matches
    }
}

impl TryFrom<&str> for MediaType {
    type Error = ParseError;

    /// Parses a media type string into a MediaType struct.
    ///
    /// The string should be in the format: `main_type/sub_type[;param1=value1;param2=value2;...]`
    ///
    /// # Errors
    ///
    /// Returns a `ParseError::MissingSlash` if the string doesn't contain a '/' separator.
    ///
    /// # Examples
    ///
    /// ```
    /// use ip_info::content_negotiation::MediaType;
    /// use std::convert::TryFrom;
    ///
    /// let simple = MediaType::try_from("text/html").unwrap();
    /// let with_params = MediaType::try_from("text/html;charset=utf-8;boundary=something").unwrap();
    /// let wildcard = MediaType::try_from("*/*").unwrap();
    ///
    /// // Error case
    /// assert!(MediaType::try_from("invalid").is_err());
    /// ```
    fn try_from(s: &str) -> Result<Self, ParseError> {
        let (media_main, rest) = s.split_once('/').ok_or(ParseError::MissingSlash)?;
        let parts: Vec<&str> = rest.split(';').collect();
        let sub_type = parts[0];

        let parameters = parts[1..]
            .iter()
            .filter_map(|param| param.split_once('='))
            .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
            .collect();

        Ok(MediaType {
            main_type: media_main.to_string(),
            sub_type: sub_type.to_string(),
            parameters,
        })
    }
}

/// Parses a single Accept header directive into a Directive struct.
///
/// An Accept directive has the format: `media_type[;param=value;...][;q=quality_value]`
/// The quality value (q) is separated from other parameters and defaults to 1.0 if not specified.
///
/// # Arguments
///
/// * `directive_str` - A string slice containing a single Accept directive
///
/// # Returns
///
/// A `Result<Directive, ParseError>` containing the parsed directive or an error.
///
/// # Errors
///
/// Returns a `ParseError` if:
/// - The directive doesn't contain a '/' (invalid media type)
/// - The q-value cannot be parsed as a float
///
/// # Examples
///
/// ```
/// use ip_info::content_negotiation::parse_accept_directive;
///
/// let simple = parse_accept_directive("text/html").unwrap();
/// assert_eq!(simple.q, 1.0);
///
/// let with_quality = parse_accept_directive("text/html;q=0.8").unwrap();
/// assert_eq!(with_quality.q, 0.8);
///
/// let with_params = parse_accept_directive("text/html;charset=utf-8;q=0.9").unwrap();
/// assert_eq!(with_params.q, 0.9);
/// assert_eq!(with_params.media_type.parameters.len(), 1);
///
/// // Error cases
/// assert!(parse_accept_directive("invalid").is_err());
/// assert!(parse_accept_directive("text/html;q=invalid").is_err());
/// ```
pub fn parse_accept_directive(directive_str: &str) -> Result<Directive, ParseError> {
    let (media_main, rest) = directive_str
        .split_once('/')
        .ok_or(ParseError::MissingSlash)?;

    let parts: Vec<&str> = rest.split(';').collect();
    let sub_type = parts[0];

    let mut q = 1.0;
    let mut parameters = Vec::new();

    for param_str in &parts[1..] {
        if let Some((key, value)) = param_str.split_once('=') {
            let key = key.trim();
            let value = value.trim();
            if key == "q" {
                q = value
                    .parse::<f32>()
                    .map_err(|_| ParseError::InvalidQualityValue(value.to_string()))?;
            } else {
                parameters.push((key.to_string(), value.to_string()));
            }
        }
    }

    let media_type = MediaType {
        main_type: media_main.to_string(),
        sub_type: sub_type.to_string(),
        parameters,
    };

    Ok(Directive { media_type, q })
}

/// Parses a complete HTTP Accept header value into a sorted list of directives.
///
/// The Accept header contains comma-separated media type directives, each with optional
/// parameters and quality values. This function parses all directives and sorts them
/// by quality value in descending order (highest preference first).
///
/// Invalid directives are skipped rather than causing the entire parse to fail.
///
/// # Arguments
///
/// * `header_value` - The complete Accept header value as a string slice
///
/// # Returns
///
/// A `Vec<Directive>` sorted by quality value in descending order.
/// Invalid directives are silently skipped.
///
/// # Examples
///
/// ```
/// use ip_info::content_negotiation::parse_accept;
///
/// let accept = "text/html;q=0.9,application/json;q=1.0,text/plain;q=0.8";
/// let directives = parse_accept(accept);
///
/// // Results are sorted by quality value (highest first)
/// assert_eq!(directives[0].media_type.sub_type, "json"); // q=1.0
/// assert_eq!(directives[1].media_type.sub_type, "html"); // q=0.9
/// assert_eq!(directives[2].media_type.sub_type, "plain"); // q=0.8
/// ```
pub fn parse_accept(header_value: &str) -> Vec<Directive> {
    let mut r = header_value
        .split(',')
        .filter_map(|s| parse_accept_directive(s.trim()).ok())
        .collect::<Vec<Directive>>();
    r.sort_by(|a, b| b.q.partial_cmp(&a.q).unwrap());
    r
}

/// Parses a complete HTTP Accept header value into a sorted list of directives.
///
/// This is the strict version that returns an error if any directive fails to parse.
///
/// # Arguments
///
/// * `header_value` - The complete Accept header value as a string slice
///
/// # Returns
///
/// A `Result<Vec<Directive>, ParseError>` containing all parsed directives sorted by
/// quality value in descending order, or an error if any directive is invalid.
///
/// # Errors
///
/// Returns the first `ParseError` encountered when parsing directives.
///
/// # Examples
///
/// ```
/// use ip_info::content_negotiation::try_parse_accept;
///
/// let accept = "text/html;q=0.9,application/json;q=1.0,text/plain;q=0.8";
/// let directives = try_parse_accept(accept).unwrap();
///
/// // Results are sorted by quality value (highest first)
/// assert_eq!(directives[0].media_type.sub_type, "json"); // q=1.0
/// assert_eq!(directives[1].media_type.sub_type, "html"); // q=0.9
/// assert_eq!(directives[2].media_type.sub_type, "plain"); // q=0.8
///
/// // Error case
/// assert!(try_parse_accept("text/html,invalid,application/json").is_err());
/// ```
pub fn try_parse_accept(header_value: &str) -> Result<Vec<Directive>, ParseError> {
    let mut r = header_value
        .split(',')
        .map(|s| parse_accept_directive(s.trim()))
        .collect::<Result<Vec<Directive>, ParseError>>()?;
    r.sort_by(|a, b| b.q.partial_cmp(&a.q).unwrap());
    Ok(r)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn test_media_type_try_from_basic() {
        let media_type = MediaType::try_from("text/html").unwrap();
        assert_eq!(media_type.main_type, "text");
        assert_eq!(media_type.sub_type, "html");
        assert_eq!(media_type.parameters, vec![]);
    }

    #[test]
    fn test_media_type_try_from_with_parameter() {
        let media_type = MediaType::try_from("text/html;charset=utf-8").unwrap();
        assert_eq!(media_type.main_type, "text");
        assert_eq!(media_type.sub_type, "html");
        assert_eq!(
            media_type.parameters,
            vec![("charset".to_string(), "utf-8".to_string())]
        );
    }

    #[test]
    fn test_media_type_try_from_wildcard() {
        let media_type = MediaType::try_from("*/*").unwrap();
        assert_eq!(media_type.main_type, "*");
        assert_eq!(media_type.sub_type, "*");
        assert_eq!(media_type.parameters, vec![]);
    }

    #[test]
    fn test_media_type_try_from_partial_wildcard() {
        let media_type = MediaType::try_from("text/*").unwrap();
        assert_eq!(media_type.main_type, "text");
        assert_eq!(media_type.sub_type, "*");
        assert_eq!(media_type.parameters, vec![]);
    }

    #[test]
    fn test_media_type_try_from_invalid() {
        let result = MediaType::try_from("invalid");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ParseError::MissingSlash);
    }

    #[test]
    fn test_media_type_matches_exact() {
        let mt1 = MediaType::try_from("text/html").unwrap();
        let mt2 = MediaType::try_from("text/html").unwrap();
        assert!(mt1.matches(&mt2));
        assert!(mt2.matches(&mt1));
    }

    #[test]
    fn test_media_type_matches_different() {
        let mt1 = MediaType::try_from("text/html").unwrap();
        let mt2 = MediaType::try_from("text/plain").unwrap();
        assert!(!mt1.matches(&mt2));
        assert!(!mt2.matches(&mt1));
    }

    #[test]
    fn test_media_type_matches_wildcard_main() {
        let mt1 = MediaType::try_from("*/html").unwrap();
        let mt2 = MediaType::try_from("text/html").unwrap();
        assert!(mt1.matches(&mt2));
        assert!(mt2.matches(&mt1));
    }

    #[test]
    fn test_media_type_matches_wildcard_sub() {
        let mt1 = MediaType::try_from("text/*").unwrap();
        let mt2 = MediaType::try_from("text/html").unwrap();
        assert!(mt1.matches(&mt2));
        assert!(mt2.matches(&mt1));
    }

    #[test]
    fn test_media_type_matches_full_wildcard() {
        let mt1 = MediaType::try_from("*/*").unwrap();
        let mt2 = MediaType::try_from("text/html").unwrap();
        assert!(mt1.matches(&mt2));
        assert!(mt2.matches(&mt1));
    }

    #[test]
    fn test_media_type_matches_different_main() {
        let mt1 = MediaType::try_from("text/html").unwrap();
        let mt2 = MediaType::try_from("application/json").unwrap();
        assert!(!mt1.matches(&mt2));
        assert!(!mt2.matches(&mt1));
    }

    #[test]
    fn test_parse_accept_directive_basic() {
        let directive = parse_accept_directive("text/html").unwrap();
        assert_eq!(directive.media_type.main_type, "text");
        assert_eq!(directive.media_type.sub_type, "html");
        assert_eq!(directive.media_type.parameters, vec![]);
        assert_eq!(directive.q, 1.0);
    }

    #[test]
    fn test_parse_accept_directive_with_q() {
        let directive = parse_accept_directive("text/html;q=0.8").unwrap();
        assert_eq!(directive.media_type.main_type, "text");
        assert_eq!(directive.media_type.sub_type, "html");
        assert_eq!(directive.media_type.parameters, vec![]);
        assert_eq!(directive.q, 0.8);
    }

    #[test]
    fn test_parse_accept_directive_with_parameter_and_q() {
        let directive = parse_accept_directive("text/html;charset=utf-8;q=0.9").unwrap();
        assert_eq!(directive.media_type.main_type, "text");
        assert_eq!(directive.media_type.sub_type, "html");
        assert_eq!(
            directive.media_type.parameters,
            vec![("charset".to_string(), "utf-8".to_string())]
        );
        assert_eq!(directive.q, 0.9);
    }

    #[test]
    fn test_parse_accept_directive_wildcard() {
        let directive = parse_accept_directive("*/*;q=0.5").unwrap();
        assert_eq!(directive.media_type.main_type, "*");
        assert_eq!(directive.media_type.sub_type, "*");
        assert_eq!(directive.q, 0.5);
    }

    #[test]
    fn test_parse_accept_directive_invalid() {
        let result = parse_accept_directive("invalid");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ParseError::MissingSlash);
    }

    #[test]
    fn test_parse_accept_directive_invalid_q() {
        let result = parse_accept_directive("text/html;q=invalid");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ParseError::InvalidQualityValue("invalid".to_string())
        );
    }

    #[test]
    fn test_parse_accept_single() {
        let directives = parse_accept("text/html");
        assert_eq!(directives.len(), 1);
        assert_eq!(directives[0].media_type.main_type, "text");
        assert_eq!(directives[0].media_type.sub_type, "html");
        assert_eq!(directives[0].q, 1.0);
    }

    #[test]
    fn test_parse_accept_multiple_sorted() {
        let directives = parse_accept("text/html;q=0.8,application/json;q=0.9,text/plain;q=0.7");
        assert_eq!(directives.len(), 3);

        // Should be sorted by q-value in descending order
        assert_eq!(directives[0].media_type.main_type, "application");
        assert_eq!(directives[0].media_type.sub_type, "json");
        assert_eq!(directives[0].q, 0.9);

        assert_eq!(directives[1].media_type.main_type, "text");
        assert_eq!(directives[1].media_type.sub_type, "html");
        assert_eq!(directives[1].q, 0.8);

        assert_eq!(directives[2].media_type.main_type, "text");
        assert_eq!(directives[2].media_type.sub_type, "plain");
        assert_eq!(directives[2].q, 0.7);
    }

    #[test]
    fn test_parse_accept_default_q_values() {
        let directives = parse_accept("text/html,application/json;q=0.8");
        assert_eq!(directives.len(), 2);

        // text/html should have default q=1.0 and be first
        assert_eq!(directives[0].media_type.main_type, "text");
        assert_eq!(directives[0].media_type.sub_type, "html");
        assert_eq!(directives[0].q, 1.0);

        assert_eq!(directives[1].media_type.main_type, "application");
        assert_eq!(directives[1].media_type.sub_type, "json");
        assert_eq!(directives[1].q, 0.8);
    }

    #[test]
    fn test_parse_accept_complex_with_parameters() {
        let directives =
            parse_accept("text/html;charset=utf-8;q=0.9,application/json;q=1.0,*/*;q=0.1");
        assert_eq!(directives.len(), 3);

        // Should be sorted by q-value
        assert_eq!(directives[0].media_type.main_type, "application");
        assert_eq!(directives[0].q, 1.0);

        assert_eq!(directives[1].media_type.main_type, "text");
        assert_eq!(
            directives[1].media_type.parameters,
            vec![("charset".to_string(), "utf-8".to_string())]
        );
        assert_eq!(directives[1].q, 0.9);

        assert_eq!(directives[2].media_type.main_type, "*");
        assert_eq!(directives[2].media_type.sub_type, "*");
        assert_eq!(directives[2].q, 0.1);
    }

    #[test]
    fn test_parse_accept_no_whitespace() {
        let directives = parse_accept("text/html;q=0.8,application/json;q=0.9");
        assert_eq!(directives.len(), 2);

        // Should be sorted by q-value
        assert_eq!(directives[0].media_type.main_type, "application");
        assert_eq!(directives[0].q, 0.9);

        assert_eq!(directives[1].media_type.main_type, "text");
        assert_eq!(directives[1].q, 0.8);
    }

    #[test]
    fn test_media_type_equality() {
        let mt1 = MediaType::try_from("text/html").unwrap();
        let mt2 = MediaType::try_from("text/html").unwrap();
        let mt3 = MediaType::try_from("text/plain").unwrap();

        assert_eq!(mt1, mt2);
        assert_ne!(mt1, mt3);
    }

    #[test]
    fn test_media_type_with_multiple_parameters() {
        let media_type = MediaType::try_from("text/html;charset=utf-8;boundary=something").unwrap();
        assert_eq!(media_type.main_type, "text");
        assert_eq!(media_type.sub_type, "html");
        // Now supports multiple parameters
        assert_eq!(
            media_type.parameters,
            vec![
                ("charset".to_string(), "utf-8".to_string()),
                ("boundary".to_string(), "something".to_string()),
            ]
        );
    }

    #[test]
    fn test_parse_accept_single_wildcard() {
        // Test with just wildcards
        let directives = parse_accept("*/*");
        assert_eq!(directives.len(), 1);
        assert_eq!(directives[0].media_type.main_type, "*");
        assert_eq!(directives[0].media_type.sub_type, "*");
        assert_eq!(directives[0].q, 1.0);
    }

    #[test]
    fn test_media_type_part_matches() {
        assert!(MediaType::part_matches("text", "text"));
        assert!(MediaType::part_matches("*", "text"));
        assert!(MediaType::part_matches("text", "*"));
        assert!(MediaType::part_matches("*", "*"));
        assert!(!MediaType::part_matches("text", "application"));
    }

    #[test]
    fn test_q_value_precision() {
        let directive = parse_accept_directive("text/html;q=0.123").unwrap();
        assert!((directive.q - 0.123).abs() < f32::EPSILON);
    }

    #[test]
    fn test_q_value_edge_cases() {
        let directive1 = parse_accept_directive("text/html;q=0.0").unwrap();
        assert_eq!(directive1.q, 0.0);

        let directive2 = parse_accept_directive("text/html;q=1.0").unwrap();
        assert_eq!(directive2.q, 1.0);

        let directive3 = parse_accept_directive("text/html;q=0.001").unwrap();
        assert_eq!(directive3.q, 0.001);
    }

    #[test]
    fn test_media_type_with_three_parameters() {
        let media_type =
            MediaType::try_from("text/html;charset=utf-8;boundary=something;version=1.0").unwrap();
        assert_eq!(media_type.main_type, "text");
        assert_eq!(media_type.sub_type, "html");
        assert_eq!(
            media_type.parameters,
            vec![
                ("charset".to_string(), "utf-8".to_string()),
                ("boundary".to_string(), "something".to_string()),
                ("version".to_string(), "1.0".to_string())
            ]
        );
    }

    #[test]
    fn test_parse_accept_directive_multiple_params_with_q() {
        let directive =
            parse_accept_directive("text/html;charset=utf-8;boundary=test;q=0.7").unwrap();
        assert_eq!(directive.media_type.main_type, "text");
        assert_eq!(directive.media_type.sub_type, "html");
        assert_eq!(
            directive.media_type.parameters,
            vec![
                ("charset".to_string(), "utf-8".to_string()),
                ("boundary".to_string(), "test".to_string())
            ]
        );
        assert_eq!(directive.q, 0.7);
    }

    #[test]
    fn test_parse_accept_directive_q_in_middle() {
        let directive =
            parse_accept_directive("text/html;charset=utf-8;q=0.8;boundary=test").unwrap();
        assert_eq!(directive.media_type.main_type, "text");
        assert_eq!(directive.media_type.sub_type, "html");
        assert_eq!(
            directive.media_type.parameters,
            vec![
                ("charset".to_string(), "utf-8".to_string()),
                ("boundary".to_string(), "test".to_string())
            ]
        );
        assert_eq!(directive.q, 0.8);
    }

    #[test]
    fn test_parameter_whitespace_trimming() {
        let media_type =
            MediaType::try_from("text/html; charset = utf-8 ; boundary = something").unwrap();
        assert_eq!(media_type.main_type, "text");
        assert_eq!(media_type.sub_type, "html");
        assert_eq!(
            media_type.parameters,
            vec![
                ("charset".to_string(), "utf-8".to_string()),
                ("boundary".to_string(), "something".to_string())
            ]
        );
    }

    #[test]
    fn test_empty_parameters() {
        let media_type = MediaType::try_from("text/html;;").unwrap();
        assert_eq!(media_type.main_type, "text");
        assert_eq!(media_type.sub_type, "html");
        assert_eq!(media_type.parameters, vec![]);
    }

    #[test]
    fn test_parameter_without_value() {
        let media_type = MediaType::try_from("text/html;charset").unwrap();
        assert_eq!(media_type.main_type, "text");
        assert_eq!(media_type.sub_type, "html");
        // Parameters without = are filtered out
        assert_eq!(media_type.parameters, vec![]);
    }
}
