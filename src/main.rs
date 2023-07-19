use std::{env, error::Error, fs::File, io::Write, thread, time::Duration};

use download::download_music;
use id3::{Tag, TagLike};

use crate::metadata::Metadata;

mod download;
mod get_info;
mod metadata;
fn main() -> Result<(), Box<dyn Error>> {
    env::set_var("RUST_BACKTRACE", "1");
    // let url = "https://soundcloud.com/sexballs/12-altars-of-apostasy-incl?in=chris-a-920974636/sets/ultrakill&si=e3d1f5114c684946934641675487830e&utm_source=clipboard&utm_medium=text&utm_campaign=social_sharing";
    let url = "https://soundcloud.com/aaklkrkppyj4/miley-cyrus-party-in-the-usa?si=68d1a8542c0e41aeb766829b3f318785&utm_source=clipboard&utm_medium=text&utm_campaign=social_sharing";
    download_music(url)?;
    thread::sleep(Duration::from_secs(3));
    let path = "music/267678153/267678153.mp3";

    Metadata::new(path).url(url).create_metadata_from_link()?;
    // create_metadata(metadata)?;
    //write_metadata(metadata)?;
    //create_metadata(path)?;
    let tag = Tag::read_from_path(path).unwrap();
    dbg!(tag.title());
    dbg!(tag.year());
    dbg!(tag.artist());
    dbg!(tag.duration());

    if let Some(apic_frame) = tag.get("APIC") {
        if let Some(picture) = apic_frame.content().picture() {
            let image_data = &picture.data;
            let mut file = File::create("cover.jpg").unwrap();
            file.write_all(image_data).unwrap();
        }
    }
    Ok(())
}
