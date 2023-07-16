use download::{create_json_for_music, download_music, download_playlist};
use play::play_music_new;

use std::{
    error::Error,
    thread::{self},
};
mod download;
mod play;
//yt-dlp --version -> 2023.07.06
fn main() -> Result<(), Box<dyn Error>> {
    //download_files()?;
    play_music_new();
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
