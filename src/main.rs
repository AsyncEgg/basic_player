use download::{create_json_for_music, download_music, download_playlist};

use std::{error::Error, process::Command, thread};

mod download;
//yt-dlp --version -> 2023.07.06
fn main() -> Result<(), Box<dyn Error>> {
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
    // Wait for threads to complete
    handle1.join().unwrap();
    handle2.join().unwrap();
    handle3.join().unwrap();
    handle4.join().unwrap();

    create_json_for_music()?;

    //
    //TODO FIND A WAY TO PLAY OPUS FILES MAYBE OPUS AND AUDIOPUS

    let handle4 = thread::spawn(move || {
        use rodio::{source::Source, Decoder, OutputStream, Sink};
        use std::fs::File;
        use std::io::BufReader;

        // Get a output stream handle to the default physical sound device
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        // Load a sound from a file, using a path relative to Cargo.toml
        let file = BufReader::new(File::open("music/DUT5rEU6pqM/DUT5rEU6pqM.ogg").unwrap());
        // Decode that sound file into a source
        let source = Decoder::new(file).unwrap();

        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        // Add a dummy source of the sake of the example.
        sink.append(source);

        // The sound plays in a separate thread. This call will block the current thread until the sink
        // has finished playing all its queued sounds.
        sink.sleep_until_end();
    });
    handle4.join().unwrap();
    Ok(())
}
