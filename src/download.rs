use std::collections::{BTreeMap, HashMap};
use std::fs::{self, create_dir, remove_file, File};
use std::io::{self, BufRead, BufReader, Error, Write};
use std::process::{ChildStdout, Command, Stdio};
use std::sync::mpsc;
use std::thread::{self};

use serde::{Deserialize, Serialize};

use crate::get_info::get_info;
type Url<'a> = &'a str;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Songs {
    all_songs: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Playlists {
    all_playlists: Vec<String>,
}

pub fn get_all_from_path(path: &str) -> Result<Vec<String>, Error> {
    let cmd = Command::new("ls").arg(path).output()?.stdout;

    let data = String::from_utf8(cmd).expect("Could not grab string");
    Ok(data
        .trim_end()
        .split('\n')
        .map(|c| c.to_string())
        .collect::<Vec<String>>())
}

pub fn create_json_for_music() -> io::Result<()> {
    let all_songs = get_all_from_path("music/")?;
    let mut all_songs_json = File::create("music/all_songs.json")?;
    all_songs_json.write_all(serde_json::to_string_pretty(&all_songs)?.as_bytes())?;

    let all_playlists = get_all_from_path("music/.playlists/")?;
    let mut all_playlists_json = File::create("music/all_playlists.json")?;
    all_playlists_json.write_all(serde_json::to_string_pretty(&all_playlists)?.as_bytes())?;

    Ok(())
}

struct OutputSaver {
    filename: String,
    stdout: ChildStdout,
}
impl OutputSaver {
    pub fn save_output(self) -> io::Result<()> {
        //Save the output to filename to be used in a progress bar
        create_dir(".temp/").unwrap_or_else(|e| eprintln!("Error creating new dir: {e}"));

        let mut file = File::create(&self.filename)?;

        let reader = BufReader::new(self.stdout);

        for line in reader.lines() {
            let line = line?;
            file.write_all(line.as_bytes())?;
            file.write_all(b"\n")?;
        }

        Ok(())
    }
}

fn check_extension_in_dir(dir: &str, ext: &str) -> io::Result<bool> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;

        if format!("{entry:?}").contains(ext) {
            return Ok(true);
        }
    }

    Ok(false)
}

fn convert_opus(original: &str, new: &str) -> io::Result<()> {
    Command::new("ffmpeg")
        .args(vec![
            "-hide_banner",
            "-loglevel",
            "error",
            "-i",
            original,
            "-acodec",
            "libmp3lame",
            new,
        ])
        .output()?;

    Command::new("rm").arg(original).output()?;
    println!("ffmpeg finish {original}");
    Ok(())
}

pub fn download_music(url: Url) -> io::Result<()> {
    // Get info
    println!("Downloading: {url}");
    let map = get_info(url, vec!["id"]).expect("Couldnt grab info");
    let id = map.get("id").expect("Couldnt grab id from map").trim_end();
    // Check if music already exists
    //I would just use json here but im lazy
    if check_extension_in_dir(&format!("music/{id}/"), ".ogg").is_ok() {
        eprintln!("Music Already exists: {id}");
        return Ok(());
    }

    // Grab music and thumbnail
    let mut cmd = Command::new("yt-dlp")
        .args(vec![
            url,
            "-x",
            "-o",
            "music/%(id)s/%(id)s",
            "--progress",
            "--newline",
            "--write-thumbnail",
        ])
        .stdout(Stdio::piped())
        .spawn()?;
    // Save output as it is being writen
    let output_saver = OutputSaver {
        filename: format!(".temp/{id}.temp"),
        stdout: cmd.stdout.take().unwrap(),
    };

    output_saver
        .save_output()
        .unwrap_or_else(|e| eprintln!("Save output: {e}"));

    // Delete output file
    remove_file(format!(".temp/{id}.temp")).unwrap_or_else(|e| eprintln!("Remove File: {e}"));

    //TODO Convert audio from opus to ogg
    let path = format!("music/{id}/{id}");
    thread::spawn(move || convert_opus(&path, &format!("{path}.mp3")));
    // Generate json info for the song
    println!("{id}");
    println!("result: {:?}", create_json(id));
    Ok(())
}

pub fn create_json(id: Url) -> io::Result<()> {
    // Grab info
    let info = get_info(id,vec!["id", "title", "duration>%H:%M:%S", "channel", "webpage_url"]).unwrap();

    // Write info to file
    let path = format!("music/{id}/info.json");

    let mut file = File::create(path)?;
    let info_data = serde_json::to_string_pretty(&info)?;
    file.write_all(info_data.as_bytes())?;
    Ok(())
}

pub fn download_playlist(url: Url, path: &str) -> io::Result<()> {
    // Check if link is a playlist
    // if !url.contains("playlist") {
    //     eprintln!("Non playlist url: {url}");
    //     return Ok(());
    // }

    println!("Downloading: {}", url);

    let playlist_location = "music/.playlists/";
    let path = format!("{playlist_location}{path}");
    //Check if playlist exists
    //I would just use json here but im lazy
    if fs::metadata(path.clone()).is_ok() {
        eprintln!("Playlist with name {path} already exists");
        return Ok(());
    }
    //Create playlist and playlists location
    create_dir("music/").unwrap_or_else(|e| eprintln!("Error creating new dir: {e}"));
    create_dir(playlist_location).unwrap_or_else(|e| eprintln!("Error creating new dir: {e}"));

    // Tx sends each new line (url) to rx to be processed so instead of waiting for the whole thing it downloads quicker

    let mut handles = vec![];

    let (tx, rx) = mpsc::channel();
    let string_url = url.to_string();
    // Spawn thread for running the command
    let t = thread::spawn(move || {
        let process = Command::new("yt-dlp")
            .arg(string_url)
            .arg("--newline")
            .arg("--print")
            .arg("%(webpage_url)s")
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to execute process");

        let output = process.stdout.expect("Failed to open process stdout");

        // Iterate over stdout, sending each line down the channel
        BufReader::new(output).lines().for_each(|line| {
            if let Ok(line) = line {
                tx.send(line).expect("Could not send data over channel");
            }
        });
    });
    handles.push(t);

    // Receive output on the consumer side and print each line
    let mut playlist_info = BTreeMap::new();
    for (i, v) in rx.iter().enumerate() {
        let map = get_info(&v, vec!["id"]).expect("Couldnt grab id");

        let id = map
            .get("id")
            .expect("Couldnt grab id from map")
            .trim_end()
            .to_string();

        playlist_info.insert(i, id);
        let handle = thread::spawn(move || {
            download_music(&v).unwrap_or_else(|_| eprintln!("Problem Downloading: {v}"));
        });
        handles.push(handle)
    }
    //Write playlist info
    let playlist_str = serde_json::to_string_pretty(&playlist_info)?;
    let mut file = File::create(path)?;
    file.write_all(playlist_str.as_bytes())?;
    //Wait for threads
    for handle in handles {
        handle.join().unwrap();
    }

    println!("Finished: {}", url);
    Ok(())
}
