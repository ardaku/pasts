use devout::{log, Tag};

const INFO: Tag = Tag::new("Info").show(true);

async fn main() {
    log!(INFO, "Hello, world!");
}
