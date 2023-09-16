use url::Url;

pub fn clean_url(mut url: Url) -> Url {
    const WHITELIST: &[(&str, &[&str])] = &[("youtube.com", &["v"])];

    if url.query().is_none() {
        return url;
    }

    let new_queries: Vec<_> = url
        .query_pairs()
        .filter_map(|(k, v)| {
            WHITELIST
                .iter()
                .find(|(host, _)| url.host_str() == Some(*host))
                .and_then(|(_, keys)| {
                    if keys.contains(&k.as_ref()) {
                        Some((k.into_owned(), v.into_owned()))
                    } else {
                        None
                    }
                })
        })
        .collect();

    if new_queries.is_empty() {
        url.set_query(None);
    } else {
        url.query_pairs_mut().clear().extend_pairs(new_queries);
    }
    url
}
