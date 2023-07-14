use std::collections::{BTreeMap, HashMap};
use std::fs::{self, create_dir, remove_file, File};
use std::io::{self, BufRead, BufReader, Error, Write};
use std::process::{ChildStdout, Command, Stdio};
use std::sync::mpsc;
use std::thread::{self};

use serde::{Deserialize, Serialize};

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

pub fn get_info(url: Url, split: &str, info: Vec<&str>) -> io::Result<HashMap<String, String>> {
    println!("getting info {:?}, {:?}", info, url);
    let mut param = String::new();
    // Turn info to a usable format that yt-dlp can read
    for (i, item) in info.iter().enumerate() {
        param.push_str("%(");
        param.push_str(item);
        param.push_str(")s");

        // Check if the current item is not the last
        if i != info.len() - 1 {
            param.push_str(split);
        }
    }
    // Grab info
    let cmd = Command::new("yt-dlp")
        .args(vec![url, "--print", &param])
        .output()?
        .stdout;

    let mut map = HashMap::new();

    let data = String::from_utf8(cmd).expect("Could not grab string");
    data.split(split).zip(info).for_each(|(c, i)| {
        map.insert(i.to_string(), c.to_string());
    });

    Ok(map)
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

pub fn download_music(url: Url) -> io::Result<()> {
    // Get info
    println!("Downloading: {url}");
    let map = get_info(url, "<split>", vec!["id", "title"]).expect("Couldnt grab info");
    let id = map.get("id").expect("Couldnt grab id from map").trim_end();
    let title = map
        .get("title")
        .expect("Couldnt grab title from map")
        .trim_end();
    // Check if music already exists
    //I would just use json here but im lazy
    if fs::metadata(format!("music/{}/{title}.mp3", id)).is_ok() {
        eprintln!("Music Already exists: {id}");
        return Ok(());
    }

    // Grab music and thumbnail
    let mut cmd = Command::new("yt-dlp")
        .args(vec![
            url,
            "-x",
            "-o",
            "music/%(id)s/%(title)s",
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

    // Generate json info for the song
    create_json(id)
}

pub fn create_json(url: Url) -> io::Result<()> {
    // Grab info
    let info = get_info(
        url,
        "<split>",
        vec!["id", "title", "duration>%H:%M:%S", "channel", "webpage_url"],
    )?;

    // Write info to file
    let id = info
        .get("id")
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Id not found"))?;
    let path = format!("music/{id}/info.json");

    let mut file = File::create(path)?;
    let info_data = serde_json::to_string_pretty(&info)?;
    file.write_all(info_data.as_bytes())?;
    Ok(())
}

pub fn download_playlist(url: Url, path: &str) -> io::Result<()> {
    // Check if link is a playlist
    if !url.contains("playlist") {
        eprintln!("Non playlist url: {url}");
        return Ok(());
    }

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

    let mut playlist_info = BTreeMap::new();

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
    for (i, v) in rx.iter().enumerate() {
        let map = get_info(&v, "<split>", vec!["id"]).expect("Couldnt grab id");

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
