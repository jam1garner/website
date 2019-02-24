use rocket::response::content::Content;
use rocket::http::ContentType;
use crate::blog_data;

fn get_rss_feed_xml() -> Option<String> {
    let channel = rss::ChannelBuilder::default()
        .title("jam1garner")
        .link("https://jam1.re")
        .description("Writeups, Hacking, and Hacky Software")
        .items(
            blog_data::get_posts()?["posts"]
                .as_array()?
                .iter()
                .filter_map(|post|
                rss::ItemBuilder::default()    
                    .title(post["title"].as_str()?.to_string())
                    .link(format!("https://jam1.re/blog/{}", post["name"].as_str()?))
                    .guid(
                        rss::GuidBuilder::default()
                        .value(format!("https://jam1.re/blog/{}", post["name"].as_str()?))
                        .permalink(false)
                        .build()
                        .ok()?)
                    .pub_date(post["date"].as_str()?.to_string())
                    .build()
                    .ok())
                .collect::<Vec<rss::Item>>()
        )
        .build()
        .ok()?;
    
    Some(channel.to_string())
}

#[get("/")]
pub fn rss_feed() -> Option<Content<String>> {
    Some(Content(ContentType::XML, get_rss_feed_xml()?))
}
