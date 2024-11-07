use std::time::Duration;

use super::errors::UrlError;
use content_disposition::parse_content_disposition;
use regex::Regex;
use reqwest::{
    header::{
        HeaderMap, ACCEPT_RANGES, CONNECTION, CONTENT_DISPOSITION, CONTENT_LENGTH, RANGE,
        USER_AGENT,
    },
    Client, ClientBuilder,
};

const FILENAME_RE: &str = r#"^[\w\s,-]+(\.[\w-]+)*\.[A-Za-z0-9]{2,4}$"#;
const CHROME_AGENT: &str = r#"Mozilla/5.0 (Windows; U; Windows NT 10.5; Win64; x64; en-US) AppleWebKit/537.33 (KHTML, like Gecko) Chrome/50.0.2124.268 Safari/536"#;

#[derive(Debug, Default, Clone)]
pub struct Url {
    pub link: String,
    pub filename: String,
    pub content_length: usize,
    pub range_support: bool,
}

impl Url {
    pub async fn new(link: &str) -> Result<Self, UrlError> {
        //self explanatory
        if url::Url::parse(link).is_err() {
            return Err(UrlError::InvalidUrl);
        }
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(15))
            .build()?;
        let head_request = client
            .head(link)
            .header(USER_AGENT, CHROME_AGENT)
            .header(CONNECTION, "keep-alive")
            .send()
            .await;
        let res = {
            if let Ok(r) = head_request {
                r
            } else {
                client
                    .get(link)
                    .header(USER_AGENT, CHROME_AGENT)
                    .header(CONNECTION, "keep-alive")
                    .send()
                    .await?
            }
        };
        let headers = res.headers().to_owned();
        drop(res);
        //parses content length header else content length is 0
        let content_length = headers.content_length().unwrap_or_default();
        //parse name from content disposition header else parse from url else name is empty
        let filename = headers
            .content_dispo()
            .unwrap_or(parse_name_from_url(link).unwrap_or_default());
        //if header accept ranges exist then there is range support , else manually try a request with range
        let range_support = headers
            .accept_ranges()
            .unwrap_or(manual_range_test(&client, link).await);
        let link = link.to_owned();
        Ok(Self {
            link,
            filename,
            content_length,
            range_support,
        })
    }
}

pub trait ParseHeaders {
    fn content_length(&self) -> Option<usize>;
    fn accept_ranges(&self) -> Option<bool>;
    fn content_dispo(&self) -> Option<String>;
}

impl ParseHeaders for HeaderMap {
    //parses content length if its available
    fn content_length(&self) -> Option<usize> {
        self.get(CONTENT_LENGTH)
            .and_then(|length| length.to_str().ok())
            .and_then(|s| s.parse::<usize>().ok())
    }

    fn accept_ranges(&self) -> Option<bool> {
        //checks range support through checking the header
        let range_header = self.get(ACCEPT_RANGES)?;
        if range_header.to_str().unwrap_or_default().trim() == "bytes" {
            return Some(true);
        }
        Some(false)
    }
    fn content_dispo(&self) -> Option<String> {
        //parses content disposition from header
        let content_dispo = self.get(CONTENT_DISPOSITION)?;
        let header_value = content_dispo.to_str().unwrap_or_default();
        let dis = parse_content_disposition(header_value);
        if let Some(fname) = dis.filename_full() {
            return Some(fname);
        }
        None
    }
}

async fn manual_range_test(client: &Client, link: &str) -> bool {
    //test range support through sending a ranged request (fallback for the header parse)
    match client.get(link).header(RANGE, "bytes=0-1").send().await {
        Ok(res) => res.bytes().await.map_or(false, |bytes| bytes.len() == 1),
        Err(_) => false,
    }
}

fn parse_name_from_url(link: &str) -> Option<String> {
    //self explanatory
    let last_segment = link.split('/').last()?.trim();
    let re = Regex::new(FILENAME_RE).expect("Invalid filename regex");
    if re.is_match(last_segment) {
        return Some(last_segment.to_string());
    }
    None
}
