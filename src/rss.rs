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
    #[serde(rename = "atom:link")]
    atom_link: RssLink,
    #[serde(rename = "$unflatten=description")]
    description: String,
    #[serde(rename = "item")]
    items: Vec<RssItem>,
}

#[derive(Serialize)]
struct RssLink {
    href: String,
    rel: String,
    #[serde(rename = "type")]
    content_type: String,
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
    feed_items: &Vec<FeedItem>,
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
    for feed_item in feed_items {
        let link = {
            if let Some(item_link) = &feed_item.link {
                server_name.clone() + "/" + &item_link
            } else {
                let id_link = full_index_link.clone() + "#" + &feed_item.file_name;
                id_link.clone()
            }
        };

        rss_items.push(RssItem {
            title: feed_item.meta.title.clone(),
            link: link.clone(),
            description: feed_item.content.clone(),
            guid: link,
            pub_date: date_to_rfc822(&feed_item.meta.date),
        });
    }
    let channel = RssChannel {
        title: feed_cfg.title.clone(),
        description: feed_cfg.description.clone(),
        link: full_feed_link.to_string(),
        atom_link: RssLink {
            href: full_feed_link.to_string(),
            rel: "self".to_string(),
            content_type: "application/rss+xml".to_string(),
        },
        items: rss_items,
    };

    let mut buffer = Vec::new();
    let writer = Writer::new_with_indent(&mut buffer, b' ', 2);
    let mut ser = Serializer::with_root(writer, Some("channel"));
    channel
        .serialize(&mut ser)
        .map_err(|err| format!("XML Serialization error: {}", err.to_string()))?;
    let rss_channel_str = String::from_utf8(buffer)
        .map_err(|err| format!("Channel string conversion error: {}", err.to_string()))?;
    return Ok(format!(
        "<rss version=\"2.0\" xmlns:atom=\"http://www.w3.org/2005/Atom\">{}</rss>",
        rss_channel_str
    ));
}
