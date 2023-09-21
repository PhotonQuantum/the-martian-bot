use image_hasher::{HashAlg, Hasher, HasherConfig, Image};
use once_cell::sync::Lazy;
use url::Url;

static IMG_HASHER: Lazy<Hasher> = Lazy::new(|| {
    HasherConfig::new()
        .hash_alg(HashAlg::DoubleGradient)
        .preproc_dct()
        .to_hasher()
});

pub fn clean_url(mut url: Url) -> Url {
    const WHITELIST: &[(&str, &[&str])] = &[
        ("youtube.com", &["v"]),
        ("www.youtube.com", &["v"]),
        (
            "play.rust-lang.org",
            &["version", "mode", "edition", "gist"],
        ),
    ];

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

pub fn hash_img(photo: &impl Image) -> i64 {
    let img_hash = IMG_HASHER.hash_image(photo);
    let mut buf = [0u8; 8];
    buf[..5].copy_from_slice(img_hash.as_bytes());
    i64::from_be_bytes(buf)
}
