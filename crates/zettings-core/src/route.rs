use crate::SCHEME_NAME;

#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct CategoryId(pub String);

#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct PageId(pub String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Route {
    pub category: CategoryId,
    pub page: Option<PageId>,
    pub params: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RouteError {
    #[error("route must use the {expected} scheme")]
    WrongScheme { expected: &'static str },
    #[error("route is missing a category")]
    MissingCategory,
    #[error("route segment contains invalid characters: {segment}")]
    InvalidSegment { segment: String },
    #[error("route query component is malformed")]
    MalformedQuery,
}

impl Route {
    #[must_use]
    pub fn new(category: &str) -> Self {
        Self {
            category: CategoryId(String::from(category)),
            page: None,
            params: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_page(mut self, page: &str) -> Self {
        self.page = Some(PageId(String::from(page)));
        self
    }

    #[must_use]
    pub fn canonical(&self) -> String {
        let mut out = String::from("/");
        out.push_str(&self.category.0);
        if let Some(page) = &self.page {
            out.push('/');
            out.push_str(&page.0);
        }
        out
    }

    #[must_use]
    pub fn to_uri(&self) -> String {
        let mut uri = format!("{SCHEME_NAME}://{}", self.category.0);
        if let Some(page) = &self.page {
            uri.push('/');
            uri.push_str(&page.0);
        }
        if !self.params.is_empty() {
            uri.push('?');
            let pairs: Vec<String> = self
                .params
                .iter()
                .map(|(k, v)| format!("{}={}", percent_encode(k), percent_encode(v)))
                .collect();
            uri.push_str(&pairs.join("&"));
        }
        uri
    }

    pub fn parse_uri(input: &str) -> Result<Self, RouteError> {
        let rest = input
            .strip_prefix(SCHEME_NAME)
            .ok_or(RouteError::WrongScheme {
                expected: SCHEME_NAME,
            })?;
        let rest = rest.strip_prefix("://").ok_or(RouteError::WrongScheme {
            expected: SCHEME_NAME,
        })?;

        let (path, query) = match rest.split_once('?') {
            Some((p, q)) => (p, Some(q)),
            None => (rest, None),
        };

        let mut segments = path.split('/');
        let category = segments.next().unwrap_or_default();
        if category.is_empty() {
            return Err(RouteError::MissingCategory);
        }
        validate_segment(category)?;

        let page = match segments.next() {
            None | Some("") => None,
            Some(seg) => {
                validate_segment(seg)?;
                Some(PageId(String::from(seg)))
            }
        };

        let params = match query {
            None => Vec::new(),
            Some(q) => parse_query(q)?,
        };

        Ok(Self {
            category: CategoryId(String::from(category)),
            page,
            params,
        })
    }
}

fn validate_segment(segment: &str) -> Result<(), RouteError> {
    let valid = !segment.is_empty()
        && segment
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !segment.starts_with('-')
        && !segment.ends_with('-');
    if valid {
        Ok(())
    } else {
        Err(RouteError::InvalidSegment {
            segment: String::from(segment),
        })
    }
}

fn parse_query(query: &str) -> Result<Vec<(String, String)>, RouteError> {
    let mut params = Vec::new();
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (raw_key, raw_value) = pair.split_once('=').ok_or(RouteError::MalformedQuery)?;
        params.push((
            percent_decode(raw_key).ok_or(RouteError::MalformedQuery)?,
            percent_decode(raw_value).ok_or(RouteError::MalformedQuery)?,
        ));
    }
    Ok(params)
}

fn percent_encode(input: &str) -> String {
    use std::fmt::Write as _;

    let mut out = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            _ => {
                let _ = write!(out, "%{byte:02X}");
            }
        }
    }
    out
}

fn percent_decode(input: &str) -> Option<String> {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let hex = bytes.get(i + 1..i + 3)?;
            let value = u8::from_str_radix(std::str::from_utf8(hex).ok()?, 16).ok()?;
            out.push(value);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_and_uri_round_trip() {
        let route = Route::new("system").with_page("sound");
        assert_eq!(route.canonical(), "/system/sound");
        assert_eq!(route.to_uri(), "zyntrix-settings://system/sound");
        let parsed = Route::parse_uri(&route.to_uri()).unwrap();
        assert_eq!(parsed, route);
    }

    #[test]
    fn params_encode_and_decode() {
        let network =
            String::from_utf8(vec![0x43, 0x61, 0x66, 0xC3, 0xA9, 0x20, 0x35, 0x47]).unwrap();
        let mut route = Route::new("network").with_page("wifi");
        route.params.push((String::from("network"), network));
        let uri = route.to_uri();
        assert!(uri.contains("network=Caf%C3%A9%205G"));
        let parsed = Route::parse_uri(&uri).unwrap();
        assert_eq!(parsed.params, route.params);
    }

    #[test]
    fn rejects_wrong_scheme_missing_category_and_bad_segments() {
        assert_eq!(
            Route::parse_uri("ms-settings://system"),
            Err(RouteError::WrongScheme {
                expected: SCHEME_NAME
            })
        );
        assert_eq!(
            Route::parse_uri("zyntrix-settings://"),
            Err(RouteError::MissingCategory)
        );
        assert!(Route::parse_uri("zyntrix-settings://System/Display").is_err());
        assert!(Route::parse_uri("zyntrix-settings://system/display!").is_err());
    }

    #[test]
    fn malformed_query_is_rejected() {
        assert_eq!(
            Route::parse_uri("zyntrix-settings://system?tab"),
            Err(RouteError::MalformedQuery)
        );
        assert_eq!(
            Route::parse_uri("zyntrix-settings://system?tab=%ZZ"),
            Err(RouteError::MalformedQuery)
        );
    }
}
