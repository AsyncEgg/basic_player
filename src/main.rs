use std::collections::{BTreeMap, HashMap};
use std::fs::{self, create_dir, remove_file, File};
use std::io::{self, BufRead, BufReader, Error, Write};
use std::process::{ChildStdout, Command, Stdio};
use std::sync::mpsc;
use std::thread::{self};
//yt-dlp --version -> 2023.07.06
type Url<'a> = &'a str;
fn main() -> Result<(), Box<Error>> {
    let url1 = "https://youtube.com/playlist?list=OLAK5uy_l2T3pMQk8o2vwT1ekRgrbzUkWEPfY8Iao";
    let url2 = "https://youtube.com/playlist?list=OLAK5uy_nPFRFEwf39Xzib7AWl_exn2sqExrfFJwc";
    let url3 = "https://www.youtube.com/watch?v=VZ-gmdcWWZs&t=144s";
    // Spawn a new threads to download playlists at the same time
    let handle1 = thread::spawn(|| {
        download_playlist(url1, "1.json").expect("Couldnt download playlist");
    });

    let handle2 =
        thread::spawn(|| download_playlist(url2, "2.json").expect("Couldnt download playlist"));

    let handle3 = thread::spawn(|| download_music(url3).expect("Couldnt download playlist"));

    // Wait for threads to complete
    handle1.join().unwrap();
    handle2.join().unwrap();
    handle3.join().unwrap();
    Ok(())
}
//TODO Change unwraps to results

fn get_info(url: Url, split: &str, info: Vec<&str>) -> io::Result<HashMap<String, String>> {
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
    fn save_output(self) -> io::Result<()> {
        //Save the output to filename to be used in a progress bar
        create_dir("music/.downloads/")
            .unwrap_or_else(|e| eprintln!("Error creating new dir: {e}"));

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

fn download_music(url: Url) -> io::Result<()> {
    // Get info
    println!("Downloading: {url}");
    let map = get_info(url, "<split>", vec!["id"]).expect("Couldnt grab id");
    let id = map.get("id").expect("Couldnt grab id from map");
    // Check if music already exists
    if fs::metadata(format!("music/{id}")).is_ok() {
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
        filename: format!("music/.downloads/{id}.dat"),
        stdout: cmd.stdout.take().unwrap(),
    };

    output_saver
        .save_output()
        .unwrap_or_else(|e| eprintln!("Save output: {e}"));

    // Delete output file
    remove_file(format!("music/.downloads/{id}.dat"))
        .unwrap_or_else(|e| eprintln!("Remove File: {e}"));

    // Generate json info for the song
    create_json(id)
}

fn create_json(url: Url) -> io::Result<()> {
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
    let info_data = serde_json::to_string(&info)?;
    file.write_all(info_data.as_bytes())?;
    Ok(())
}

fn download_playlist(url: Url, path: &str) -> io::Result<()> {
    // Check if link is a playlist
    if !url.contains("playlist") {
        eprintln!("Non playlist url: {url}");
        return Ok(());
    }

    println!("Downloading: {}", url);

    let playlist_location = "music/.playlists/";
    let path = format!("{playlist_location}{path}");
    //Check if playlist exists
    if fs::metadata(path.clone()).is_ok() {
        eprintln!("Playlist with name {path} already exists");
        return Ok(());
    }
    //Create playlist and playlists location
    create_dir("music/").unwrap_or_else(|e| eprintln!("Error creating new dir: {e}"));
    create_dir(playlist_location).unwrap_or_else(|e| eprintln!("Error creating new dir: {e}"));

    // Tx sends each new line (url) to rx to be processed so instead of waiting for the whole thing it downloads quicker
    let mut file = File::create(path)?;

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
    let playlist_str = serde_json::to_string(&playlist_info)?;
    file.write_all(playlist_str.as_bytes())?;
    //Wait for threads
    for handle in handles {
        handle.join().unwrap();
    }

    println!("Finished: {}", url);
    Ok(())
}
