use bilibili::{BilibiliAdapter, FROZEN_BVID};
use site_adapter_api::{ResolveContext, SiteAdapter};
use std::env;
use std::fs;

fn main() {
    let path = env::args().nth(1).expect("HTML path argument");
    let html = fs::read_to_string(path).expect("read public page HTML");
    let adapter = BilibiliAdapter;
    let locator = adapter
        .recognize(&format!("https://www.bilibili.com/video/{FROZEN_BVID}/"))
        .expect("recognize frozen public URL")
        .locator
        .expect("frozen public URL must match");
    let media = adapter
        .resolve_with_context(
            &locator,
            ResolveContext {
                public_document: Some(&html),
                upstream_status: Some(200),
            },
        )
        .expect("resolve frozen public page");
    let navigation = adapter
        .navigation_from_html(&locator, &html)
        .expect("read public navigation");
    let protocol = media
        .streams
        .first()
        .map(|stream| format!("{:?}", stream.protocol))
        .unwrap_or_else(|| "none".into());
    println!("site=bilibili plugin=bilibili-public locator_version=1");
    println!("bvid={FROZEN_BVID} protocol={protocol}");
    println!("expires_at_unix={:?}", media.expires_at_unix);
    println!(
        "navigation current_index={:?} has_previous={} has_next={}",
        navigation.current_index,
        navigation.previous.is_some(),
        navigation.next.is_some()
    );
}
