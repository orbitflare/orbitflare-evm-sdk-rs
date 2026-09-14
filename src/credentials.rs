pub(crate) fn apply_api_key(url: &str, api_key: &str) -> String {
    if api_key.is_empty() || url.contains("api_key=") {
        return url.to_string();
    }
    let (base, sep) = if url.contains('?') {
        (url.to_string(), "&")
    } else if has_path(url) {
        (url.to_string(), "?")
    } else {
        (format!("{url}/"), "?")
    };
    format!("{base}{sep}api_key={api_key}")
}

fn has_path(url: &str) -> bool {
    let after_scheme = url.split_once("://").map(|(_, rest)| rest).unwrap_or(url);
    after_scheme.contains('/')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appends_key() {
        let url = apply_api_key("https://robinhood.rpc.orbitflare.com", "ORBIT-123");
        assert_eq!(
            url,
            "https://robinhood.rpc.orbitflare.com/?api_key=ORBIT-123"
        );
    }

    #[test]
    fn keeps_existing_path() {
        let url = apply_api_key("https://robinhood.rpc.orbitflare.com/v1", "ORBIT-123");
        assert_eq!(
            url,
            "https://robinhood.rpc.orbitflare.com/v1?api_key=ORBIT-123"
        );
    }

    #[test]
    fn appends_with_existing_query() {
        let url = apply_api_key("https://robinhood.rpc.orbitflare.com?foo=bar", "ORBIT-123");
        assert_eq!(
            url,
            "https://robinhood.rpc.orbitflare.com?foo=bar&api_key=ORBIT-123"
        );
    }

    #[test]
    fn skips_if_already_present() {
        let url = "https://robinhood.rpc.orbitflare.com?api_key=ORBIT-ABC";
        assert_eq!(apply_api_key(url, "ORBIT-123"), url);
    }

    #[test]
    fn skips_if_empty_key() {
        let url = "https://robinhood.rpc.orbitflare.com";
        assert_eq!(apply_api_key(url, ""), url);
    }
}
