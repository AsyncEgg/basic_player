use std::{error::Error, fs::File, io::Write, thread, time::Duration};

use download::download_music;
use id3::{Tag, TagLike};
use metadata::create_metadata_from_link;

use crate::metadata::create_metadata;

mod download;
mod get_info;
mod metadata;
fn main() -> Result<(), Box<dyn Error>> {
    // let url = "https://soundcloud.com/sexballs/12-altars-of-apostasy-incl?in=chris-a-920974636/sets/ultrakill&si=e3d1f5114c684946934641675487830e&utm_source=clipboard&utm_medium=text&utm_campaign=social_sharing";
    // download_music(url)?;
    let url = "https://soundcloud.com/pinegroveband/need-2-1?si=f14cd3a326094f0f90786529da571165&utm_source=clipboard&utm_medium=text&utm_campaign=social_sharing";
    thread::sleep(Duration::from_secs(3));
    let path = "music/873257758/873257758.mp3";
    create_metadata_from_link(url, path)?;
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
