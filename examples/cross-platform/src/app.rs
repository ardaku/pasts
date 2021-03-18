use devout::{Tag, log};

const INFO: Tag = Tag::new("Info").show(true);

pub async fn run() {
    log!(INFO, "Hello, world!");
}
