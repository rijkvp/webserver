use crate::{config::FeedConfig, generator::FeedItem};
use chrono::NaiveDate;
use quick_xml::{se::Serializer, Writer};
use serde::Serialize;

#[derive(Serialize)]
struct RssChannel {
    #[serde(rename = "$unflatten=title")]
    title: String,
    #[serde(rename = "$unflatten=description")]
    description: String,
    #[serde(rename = "$unflatten=link")]
    link: String,
    #[serde(rename = "item")]
    items: Vec<RssItem>,
}

#[derive(Serialize)]
struct RssItem {
    #[serde(rename = "$unflatten=title")]
    title: String,
    #[serde(rename = "$unflatten=guid")]
    guid: String,
    #[serde(rename = "$unflatten=link")]
    link: String,
    #[serde(rename = "$unflatten=pubDate")]
    pub_date: String,
    #[serde(rename = "$unflatten=description")]
    description: String,
}

pub fn date_to_rfc822(date: &NaiveDate) -> String {
    date.format("%a, %d %b %Y 00:00:00 +0000").to_string()
}

pub fn feed_items_to_rss(
    items: &Vec<(String, FeedItem)>,
    feed_cfg: &FeedConfig,
    full_url: &str,
) -> String {
    let mut rss_items = Vec::new();
    for (url, item) in items {
        rss_items.push(RssItem {
            title: item.title.clone(),
            guid: url.to_string(),
            link: url.to_string(),
            pub_date: date_to_rfc822(&item.date),
            description: item.content.clone(),
        });
    }
    let channel = RssChannel {
        title: feed_cfg.title.clone(),
        description: feed_cfg.description.clone(),
        link: full_url.to_string(),
        items: rss_items,
    };

    let mut buffer = Vec::new();
    let writer = Writer::new_with_indent(&mut buffer, b' ', 2);
    let mut ser = Serializer::with_root(writer, Some("channel"));
    channel.serialize(&mut ser).unwrap();
    let rss_channel_str = String::from_utf8(buffer).unwrap();
    return format!("<rss version=\"2.0\">{}</rss>", rss_channel_str);
}
