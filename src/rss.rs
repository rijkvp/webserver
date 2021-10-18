use crate::{
    config::{FeedConfig, FeedOutput},
    generator::FeedItem,
};
use chrono::NaiveDate;
use quick_xml::{se::Serializer, Writer};
use serde::Serialize;

#[derive(Serialize)]
struct RssChannel {
    #[serde(rename = "$unflatten=title")]
    title: String,
    #[serde(rename = "$unflatten=link")]
    link: String,
    #[serde(rename = "$unflatten=description")]
    description: String,
    #[serde(rename = "item")]
    items: Vec<RssItem>,
}

#[derive(Serialize)]
struct RssItem {
    #[serde(rename = "$unflatten=title")]
    title: String,
    #[serde(rename = "$unflatten=link")]
    link: String,
    #[serde(rename = "$unflatten=description")]
    description: String,

    #[serde(rename = "$unflatten=guid")]
    guid: String,
    #[serde(rename = "$unflatten=pubDate")]
    pub_date: String,
}

pub fn date_to_rfc822(date: &NaiveDate) -> String {
    date.format("%a, %d %b %Y 00:00:00 +0000").to_string()
}

pub fn generate_rss_xml(
    items: &Vec<(String, FeedItem)>,
    feed_cfg: &FeedConfig,
    server_name: &String,
    index_output: &FeedOutput,
    feed_link: &String,
) -> Result<String, String> {
    let full_feed_link = server_name.clone() + "/" + feed_link;

    let full_index_link = server_name.clone()
        + "/"
        + index_output
            .link
            .to_str()
            .ok_or("Index link string conversion failed!")?;

    let mut rss_items = Vec::new();
    for (url, item) in items {
        let link_opt = {
            if let Some(content_output) = &feed_cfg.content_output {
                let path = content_output.link.join(url);
                Some(server_name.clone() + "/" + &path.to_string_lossy().to_string())
            } else {
                None
            }
        };

        let link = link_opt
            .clone()
            .unwrap_or_else(|| full_index_link.clone() + "#" + &url);
        let guid = link_opt.unwrap_or_else(|| url.to_string());
        rss_items.push(RssItem {
            title: item.title.clone(),
            link,
            description: item.content.render()?.clone(),
            guid,
            pub_date: date_to_rfc822(&item.date),
        });
    }
    let channel = RssChannel {
        title: feed_cfg.title.clone(),
        description: feed_cfg.description.clone(),
        link: full_feed_link.to_string(),
        items: rss_items,
    };

    let mut buffer = Vec::new();
    let writer = Writer::new_with_indent(&mut buffer, b' ', 2);
    let mut ser = Serializer::with_root(writer, Some("channel"));
    channel.serialize(&mut ser).map_err(|err| format!("XML Serialization error: {}", err.to_string()))?;
    let rss_channel_str =
        String::from_utf8(buffer).map_err(|err| format!("Channel string conversion error: {}", err.to_string()))?;
    return Ok(format!("<rss version=\"2.0\">{}</rss>", rss_channel_str));
}
