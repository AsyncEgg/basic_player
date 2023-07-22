use download::{create_json_for_music, download_music, download_playlist};
use metadata::Metadata;
use player::play_music;

use std::{error::Error, thread};
mod download;
mod get_info;
mod metadata;
mod player;
//yt-dlp --version -> 2023.07.06
fn main() -> Result<(), Box<dyn Error>> {
    //download_files()?;

    // download_playlist("https://soundcloud.com/emiliano-gonzalez-312485981/sets/i-probably-dont-have-autism?si=cc82f79544f743c988d4bbb7435cf880&utm_source=clipboard&utm_medium=text&utm_campaign=social_sharing",
    // "soundcloud.json")?;
    let url = "https://soundcloud.com/king-gizzard-the-lizard-wizard/dragon?in=king-gizzard-the-lizard-wizard/sets/petrodragonic-apocalypse-or&si=4222b03b82574859bc39f0fae5cced3c&utm_source=clipboard&utm_medium=text&utm_campaign=social_sharing";
    Metadata::new("songs/Dragon [1526029687].mp3")
        .url(url)
        .create_metadata_from_link()?;

    //play_music();
    Ok(())
}

fn download_files() -> Result<(), Box<dyn Error>> {
    let url1 = "https://youtube.com/playlist?list=OLAK5uy_l2T3pMQk8o2vwT1ekRgrbzUkWEPfY8Iao";
    let url2 = "https://youtube.com/playlist?list=OLAK5uy_nPFRFEwf39Xzib7AWl_exn2sqExrfFJwc";
    let url3 = "https://www.youtube.com/watch?v=VZ-gmdcWWZs&t=144s";
    let url4 = "https://www.youtube.com/watch?v=DUT5rEU6pqM";
    // Spawn a new threads to download playlists at the same time
    let handle1 = thread::spawn(|| {
        download_playlist(url1, "1.json").expect("Couldnt download playlist");
    });

    let handle2 =
        thread::spawn(|| download_playlist(url2, "2.json").expect("Couldnt download playlist"));

    let handle3 = thread::spawn(|| download_music(url3).expect("Couldnt download playlist"));
    let handle4 = thread::spawn(|| download_music(url4).expect("Couldnt download playlist"));
    let handle5 = thread::spawn(|| {
        download_music("https://youtu.be/Q_9VMaX61nI").expect("Couldnt download playlist")
    });
    // Wait for threads to complete
    handle1.join().unwrap();
    handle2.join().unwrap();
    handle3.join().unwrap();
    handle4.join().unwrap();
    handle5.join().unwrap();

    create_json_for_music()?;
    Ok(())
}
