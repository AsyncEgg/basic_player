//todo read metadata functions

//todo write metadata functions

//todo create metadata from link

use std::{error::Error, fs::File, io::Read};

use id3::{
    frame::{Picture, PictureType},
    Content, ErrorKind, Frame, Tag, TagLike, Version,
};

use crate::get_info::{get_info, get_mimetype_from_path};

pub fn create_metadata_from_link(url: &str, path: &str) -> Result<(), Box<dyn Error>> {
    //Check if these exist
    if let Ok(existing_tag) = Tag::read_from_path(path) {
        if existing_tag.title().is_some()
            && existing_tag.year().is_some()
            && existing_tag.artist().is_some()
            && existing_tag.duration().is_some()
            && existing_tag.pictures().next().is_some()
        {
            println!("Metadata Exists");
            return Ok(());
        }
    }

    println!("Creating metadata from: {url}");
    //Get data from url
    let info_res = get_info(
        url,
        vec!["title", "upload_date>%Y", "uploader", "duration", "id"],
    );

    let info = match info_res {
        Ok(info) => info,
        Err(err) => return Err(err),
    };
    // Get data from map
    let id = info.get("id").ok_or("ID not found")?.trim();
    let title = info.get("title").ok_or("Title not found")?;
    let uploader = info.get("uploader").ok_or("Uploader not found")?;
    let duration_txt = info.get("duration").ok_or("Duration not found")?;
    let duration = duration_txt // Convert String to u32
        .parse::<f32>()
        .map_err(|_| "Duration not in correct format")?
        .ceil() as u32;
    let upload_date = info // Get year
        .get("upload_date>%Y")
        .ok_or("Upload date not found")?
        .parse::<i32>()
        .map_err(|_| "Invalid upload date format")?;

    // Get path for art
    let image_path = format!("music/{id}/{id}.png",);
    println!("{image_path}");
    let mime_type = get_mimetype_from_path(&image_path)?;

    let mut file = File::open(image_path)?;
    let mut buffer: Vec<u8> = Vec::new();
    file.read_to_end(&mut buffer)?;

    //todo add functionallity for png jpg webp etc
    println!("pix");
    let picture = Picture {
        mime_type,
        picture_type: PictureType::CoverFront,
        description: "Album cover".to_string(),
        data: buffer,
    };

    // Try to read existing tag from file, or create a new one if it doesn't exist
    let mut tag = match Tag::read_from_path(path) {
        Ok(existing_tag) => existing_tag,
        Err(tag_error) => match tag_error.kind {
            ErrorKind::NoTag => Tag::new(),
            _ => return Err(Box::new(tag_error)),
        },
    };

    // Only change the metadata if it doesn't exist
    if tag.title().is_none() {
        tag.set_title(title);
    }

    if tag.year().is_none() {
        tag.set_year(upload_date);
    }

    if tag.artist().is_none() {
        tag.set_artist(uploader);
    }

    if tag.duration().is_none() {
        tag.set_duration(duration);
    }

    if tag.pictures().count() == 0 {
        tag.add_frame(Frame::with_content("APIC", Content::Picture(picture)));
    }

    // Write tags back to file
    tag.write_to_path(path, Version::Id3v24)?;

    Ok(())
}

//todo create metadata from user input / random data

pub fn create_metadata(path: &str) -> Result<(), Box<dyn Error>> {
    println!("{path}");
    let mut tag = Tag::new();
    let title = path.split('/').last().ok_or("Item not found")?;
    tag.set_title(title);
    // tag.set_year(upload_date);
    // tag.set_artist(uploader);
    // tag.set_duration(duration);
    // tag.add_frame(Frame::with_content("APIC", Content::Picture(picture)));

    tag.write_to_path(path, Version::Id3v24)?;

    Ok(())
}

//todo function to allow people to change, titles, picture, etc
