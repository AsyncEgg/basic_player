<<<<<<< HEAD
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
=======
use chrono::Timelike;
use device_query::{DeviceQuery, DeviceState, Keycode};
use download::{create_json_for_music, download_music, download_playlist};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use serde::{Deserialize, Serialize};

use std::{
    error::Error,
    fs::{self, File},
    io::BufReader,
    thread::{self, sleep},
    time::{Duration, Instant},
};

mod download;
//yt-dlp --version -> 2023.07.06
fn main() -> Result<(), Box<dyn Error>> {
    download_files()?;
    download_music("https://youtu.be/MFF-diLFhtQ")?;
    play_music();
    Ok(())
}
//todo make this faster :p
fn setup_sink(
    stream_handle: &OutputStreamHandle,
    audio_file: &str,
    skip: Option<Duration>,
) -> Sink {
    let sink = Sink::try_new(stream_handle).expect("Couldnt unwrap sink");

    let file = File::open(audio_file).expect("Error opening file");
    let source = Decoder::new(BufReader::new(file)).expect("Error decoding file");

    match skip {
        Some(d) => sink.append(source.skip_duration(d)),
        None => sink.append(source),
    }

    sink.pause(); // start out paused //todo change to sink.play

    sink
}
#[inline]
fn toggle_playback(sink: &mut Sink) {
    if sink.is_paused() {
        sink.play();
    } else {
        sink.pause();
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct VideoInfo {
    channel: String,
    webpage_url: String,
    id: String,
    title: String,
    #[serde(rename = "duration>%H:%M:%S")]
    duration: String,
}

fn get_info_from_id(id: &str) -> VideoInfo {
    let path = &format!("music/{id}/info.json");
    let content = fs::read_to_string(path).expect("Couldnt read file");

    serde_json::from_str(&content).expect("Couldnt json tbh")
}

fn duation_from_song_duration(song_duration: &str) -> u64 {
    let naive_time = chrono::NaiveTime::parse_from_str(song_duration, "%H:%M:%S")
        .expect("Error parsing song duration");
    let hours = naive_time.hour() as u64;
    let minutes = naive_time.minute() as u64;
    let seconds = naive_time.second() as u64;
    hours * 3600 + minutes * 60 + seconds
}

fn play_audio_file(
    stream_handle: &OutputStreamHandle,
    audio_file: &str,
) -> Result<Sink, Box<dyn Error>> {
    let sink = setup_sink(stream_handle, audio_file, None);
    sink.play();
    Ok(sink)
}

fn play_music() {
    println!("playing");
    const KEY_DEBOUNCE: Duration = Duration::from_millis(200);
    const SLEEP_DURATION: Duration = Duration::from_millis(20);

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let device_state = DeviceState::new();
    let audio_files = vec![
        // "music/Q_9VMaX61nI/Q_9VMaX61nI.ogg",
        "music/VZ-gmdcWWZs/VZ-gmdcWWZs.ogg",
        "music/DUT5rEU6pqM/DUT5rEU6pqM.ogg",
        // "music/Q_9VMaX61nI/Q_9VMaX61nI.ogg",
        "music/MFF-diLFhtQ/MFF-diLFhtQ.ogg",
    ];

    let mut current_audio_file_index = 0;
    let mut sink = setup_sink(&stream_handle, audio_files[current_audio_file_index], None);
    let mut last_space_press_time = Instant::now() - KEY_DEBOUNCE;
    let mut last_right_press_time = Instant::now() - KEY_DEBOUNCE;
    let mut last_left_press_time = Instant::now() - KEY_DEBOUNCE;
    let mut last_enter_press_time = Instant::now() - KEY_DEBOUNCE;
    let mut now = Instant::now();
    let mut saved_time = Duration::from_secs(0);
    let mut pause_start_time: Option<Instant> = None;
    let mut skip_amount = 0;
    sink.play();
    loop {
        let id = audio_files[current_audio_file_index]
            .split('/')
            .collect::<Vec<&str>>()[1];
        let song_duration = duation_from_song_duration(&get_info_from_id(id).duration);
        let converted_song_duration = Duration::from_secs(song_duration);
        // let elapsed = format!("{:?}", now.elapsed());
        // println!("{elapsed}");
        if now.elapsed() > converted_song_duration {
            current_audio_file_index = (current_audio_file_index + 1) % audio_files.len();
            println!("{}", current_audio_file_index);
            skip_amount = 0;
            sink = play_audio_file(&stream_handle, audio_files[current_audio_file_index])
                .expect("Err playing audio file");
            now = Instant::now();
            saved_time = Duration::from_secs(0); // Reset saved_time when a new song starts
        }
        // println!(
        //     "Elapsed: {:?}\nWhere to goto: {skip_amount}\nDuration: {song_duration}",
        //     now.elapsed().as_secs()
        // );
        let keys: Vec<Keycode> = device_state.get_keys();
        if keys.contains(&Keycode::F9) && last_space_press_time.elapsed() >= KEY_DEBOUNCE {
            toggle_playback(&mut sink);
            last_space_press_time = Instant::now();

            if sink.is_paused() {
                pause_start_time = Some(Instant::now());
            } else {
                if let Some(pause_time) = pause_start_time {
                    saved_time += Instant::now() - pause_time;
                    pause_start_time = None;
                }
                now = Instant::now() - saved_time - Duration::from_secs(2);
            }
        }
        if keys.contains(&Keycode::Up) {
            if skip_amount > song_duration {
                skip_amount = song_duration
            }
            skip_amount += 1;
        }
        if keys.contains(&Keycode::Down) {
            if 0 < skip_amount as i64 {
                skip_amount -= 1;
            } else {
                skip_amount = 0
            }
        }

        if keys.contains(&Keycode::Enter) && last_enter_press_time.elapsed() >= KEY_DEBOUNCE {
            let skip = Duration::from_secs(skip_amount);

            sink = play_audio_file(&stream_handle, audio_files[current_audio_file_index])
                .expect("Err playing audio file");
            now = Instant::now() - skip;
            last_enter_press_time = Instant::now();
        }

        if keys.contains(&Keycode::Right) && last_right_press_time.elapsed() >= KEY_DEBOUNCE {
            current_audio_file_index = (current_audio_file_index + 1) % audio_files.len();
            println!("{}", current_audio_file_index);
            skip_amount = 0;
            sink = play_audio_file(&stream_handle, audio_files[current_audio_file_index])
                .expect("Err playing audio file");

            last_right_press_time = Instant::now();
            now = Instant::now();
            saved_time = Duration::from_secs(0); // Reset saved_time when a new song starts
        }

        if keys.contains(&Keycode::Left) && last_left_press_time.elapsed() >= KEY_DEBOUNCE {
            if current_audio_file_index as isize - 1 < 0 {
                current_audio_file_index = audio_files.len()
            }
            current_audio_file_index = current_audio_file_index.saturating_sub(1);
            println!("{}", current_audio_file_index);
            skip_amount = 0;
            sink = play_audio_file(&stream_handle, audio_files[current_audio_file_index])
                .expect("Err playing audio file");

            last_left_press_time = Instant::now();
            now = Instant::now();
            saved_time = Duration::from_secs(0); // Reset saved_time when a new song starts
        }

        if keys.contains(&Keycode::Escape) {
            break;
        }
        sleep(SLEEP_DURATION);
    }
}
>>>>>>> 79c6e27 (quick git fix)

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
