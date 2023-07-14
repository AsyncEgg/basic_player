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
    let handle1 = thread::spawn(|| {
        download_playlist(url1, "1.json").expect("Couldnt download playlist");
    });

    // Spawn a new thread to download playlist2
    let handle2 =
        thread::spawn(|| download_playlist(url2, "2.json").expect("Couldnt download playlist"));

    let handle3 = thread::spawn(|| download_music(url3).expect("Couldnt download playlist"));

    // Wait for both threads to complete
    handle1.join().unwrap();
    handle2.join().unwrap();
    handle3.join().unwrap();
    Ok(())
}
//TODO Change unwraps to results

fn get_info(url: Url, split: &str, info: Vec<&str>) -> io::Result<HashMap<String, String>> {
    println!("getting info {:?}, {:?}", info, url);
    let mut param = String::new();

    for (i, item) in info.iter().enumerate() {
        param.push_str("%(");
        param.push_str(item);
        param.push_str(")s");

        // Check if the current item is not the last
        if i != info.len() - 1 {
            param.push_str(split);
        }
    }

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
    println!("Downloading: {url}");
    let map = get_info(url, "<split>", vec!["id"]).expect("Couldnt grab id");
    let id = map.get("id").expect("Couldnt grab id from map");

    if fs::metadata(format!("music/{id}")).is_ok() {
        eprintln!("Music Already exists: {id}");
        return Ok(());
    }

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

    let output_saver = OutputSaver {
        filename: format!("music/.downloads/{id}.txt"),
        stdout: cmd.stdout.take().unwrap(),
    };

    let output_thread = thread::spawn(move || {
        output_saver
            .save_output()
            .unwrap_or_else(|e| eprintln!("Save output: {e}"));
    });

    output_thread
        .join()
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to join output thread"))?;

    remove_file(format!("music/.downloads/{id}.txt"))
        .unwrap_or_else(|e| eprintln!("Remove File: {e}"));

    create_json(id)
}

fn create_json(url: Url) -> io::Result<()> {
    let info = get_info(
        url,
        "<split>",
        vec!["id", "title", "duration>%H:%M:%S", "channel", "webpage_url"],
    )?;

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
    if !url.contains("playlist") {
        eprintln!("Non playlist url: {url}");
        return Ok(());
    }

    println!("Downloading: {}", url);

    let playlist_location = "music/.playlists/";
    let path = format!("{playlist_location}{path}");

    if fs::metadata(path.clone()).is_ok() {
        eprintln!("Playlist with name {path} already exists");
        return Ok(());
    }
    create_dir("music/").unwrap_or_else(|e| eprintln!("Error creating new dir: {e}"));
    create_dir(playlist_location).unwrap_or_else(|e| eprintln!("Error creating new dir: {e}"));

    //let info = get_info(url, "<split>", vec!["webpage_url"])?;
    let mut file = File::create(path)?;

    let mut playlist_info = BTreeMap::new();

    let mut handles = vec![];

    let (tx, rx) = mpsc::channel();
    let string_url = url.to_string();
    // spawn thread for running the command
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

        // iterate over stdout, sending each line down the channel
        BufReader::new(output).lines().for_each(|line| {
            if let Ok(line) = line {
                tx.send(line).expect("Could not send data over channel");
            }
        });
    });
    handles.push(t);

    // receive output on the consumer side and print each line
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

    let playlist_str = serde_json::to_string(&playlist_info)?;
    file.write_all(playlist_str.as_bytes())?;

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Finished: {}", url);
    Ok(())
}
