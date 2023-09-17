use std::sync::OnceLock;

use url::Url;

const URL_FILTER: OnceLock<Vec<Box<dyn UrlFilter>>> = OnceLock::new();
const MSG_FILTER: OnceLock<Vec<Box<dyn MsgFilter>>> = OnceLock::new();

// TODO: config based filter

pub fn whitelist_msg(msg: &str) -> bool {
    // add your filter here
    // if you want to ignore a message, please judge it true
    let eval_bot = |s: &str| s.starts_with("/eval ");

    MSG_FILTER
        .get_or_init(|| vec![Box::new(eval_bot) as Box<_>])
        .iter()
        .any(|f| f.judge(msg))
}

pub fn whitelist_url(url: &Url) -> bool {
    // add your filter here
    // if you want to ignore a message, please judge it true
    let rust_playground = |lnk: &Url| lnk.host_str() == Some("play.rust-lang.org");

    URL_FILTER
        .get_or_init(|| vec![Box::new(rust_playground) as Box<_>])
        .iter()
        .any(|f| f.judge(url))
}

pub trait MsgFilter {
    fn judge(&self, url: &str) -> bool;
}

impl<F> MsgFilter for F
where
    F: Fn(&str) -> bool,
{
    fn judge(&self, url: &str) -> bool {
        self(url)
    }
}

pub trait UrlFilter {
    fn judge(&self, url: &Url) -> bool;
}

impl<F> UrlFilter for F
where
    F: Fn(&Url) -> bool,
{
    fn judge(&self, url: &Url) -> bool {
        self(url)
    }
}
