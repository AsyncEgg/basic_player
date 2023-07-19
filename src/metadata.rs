//todo read metadata functions

//todo write metadata functions

//todo create metadata from link

use std::{error::Error, fs::File, io::Read, path::Path};

use id3::{
    frame::{Picture, PictureType},
    Content, Frame, Tag, TagLike, Version,
};

use crate::get_info::{download_image_from_url, get_info, get_mimetype_from_path};

//todo create metadata from user input / random data
#[derive(Clone)]
pub struct Metadata {
    path: String,
    title: Option<String>,
    upload_date: Option<i32>,
    uploader: Option<String>,
    image_path: Option<String>,
    url: Option<String>,
    duration: u32,
}

impl Metadata {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
            title: None,
            upload_date: None,
            uploader: None,
            image_path: None,
            url: None,
            duration: mp3_duration::from_path(Path::new(path)).unwrap().as_secs() as u32,
        }
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    pub fn upload_date(mut self, upload_date: i32) -> Self {
        self.upload_date = Some(upload_date);
        self
    }

    pub fn uploader(mut self, uploader: &str) -> Self {
        self.uploader = Some(uploader.to_string());
        self
    }

    pub fn image_path(mut self, image_path: &str) -> Self {
        self.image_path = Some(image_path.to_string());
        self
    }

    pub fn url(mut self, url: &str) -> Self {
        self.url = Some(url.to_string());
        self
    }

    pub fn create_metadata_from_link(mut self) -> Result<(), Box<dyn Error>> {
        match &self.url {
            Some(_) => (),
            None => return Err("No url in metadata".into()),
        }

        //Check if these exist
        if let Ok(existing_tag) = Tag::read_from_path(&self.path) {
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

        let url = &self.url.clone().unwrap();
        println!("Creating metadata from: {url}");
        if self.url.is_some() {
            let info_res = get_info(
                url,
                vec!["title", "upload_date>%Y", "uploader", "duration", "id"],
            );

            let info = info_res?;

            if self.title.is_none() {
                self.title = info.get("title").map(|s| s.to_string());
            }

            if self.uploader.is_none() {
                self.uploader = info.get("uploader").map(|s| s.to_string());
            }

            if self.upload_date.is_none() {
                self.upload_date = info
                    .get("upload_date>%Y")
                    .and_then(|s| s.parse::<i32>().ok());
            }

            if self.image_path.is_none() {
                // Downloading the image and setting the path in metadata
                let id = info.get("id").ok_or("ID not found")?.trim();
                let image_path = &format!(".temp/{}.jpg", id); //todo Improve this temporary image path name
                download_image_from_url(url, image_path)?;
                self.image_path = Some(image_path.to_string());
            }
        }

        //write_metadata
        self.write_metadata()?;

        Ok(())
    }

    pub fn write_metadata(self) -> Result<(), Box<dyn Error>> {
        // Get duration from path
        let duration = mp3_duration::from_path(Path::new(&self.path))?.as_secs() as u32;

        let placeholder_path = "assets/placeholder.png";
        let (mut file, mime_type) = match &self.image_path {
            // Check if there is image path
            Some(image_path) => {
                //Check if image can be opend
                match File::open(image_path) {
                    Ok(file) => (file, get_mimetype_from_path(image_path)),
                    Err(_) => (
                        File::open(placeholder_path)?, // Error opening return placeholder
                        get_mimetype_from_path(placeholder_path),
                    ),
                }
            }
            None => (
                // No image path return placeholder
                File::open(placeholder_path)?,
                get_mimetype_from_path(placeholder_path),
            ),
        };
        let mut buffer: Vec<u8> = Vec::new();
        file.read_to_end(&mut buffer)?;

        let picture = Picture {
            mime_type,
            picture_type: PictureType::CoverFront,
            description: "Album cover".to_string(),
            data: buffer,
        };

        let mut tag = Tag::read_from_path(&self.path)?;

        if let Some(title) = &self.title {
            tag.set_title(title);
        }
        if let Some(upload_date) = self.upload_date {
            tag.set_year(upload_date);
        }
        if let Some(uploader) = &self.uploader {
            tag.set_artist(uploader);
        }

        tag.set_duration(duration); //Set duration since it is requred

        if self.image_path.is_some() {
            tag.add_frame(Frame::with_content("APIC", Content::Picture(picture)));
        }

        tag.write_to_path(&self.path, Version::Id3v24)?;

        Ok(())
    }
}

//todo function to allow people to change, titles, picture, etc
