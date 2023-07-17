use chrono::Timelike;
use device_query::{DeviceQuery, DeviceState, Keycode};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    error::Error,
    fs::{self, File},
    io::BufReader,
    thread::sleep,
    time::{Duration, Instant},
};

const VOLUME: f32 = 0.5;

//todo make this faster :p
fn setup_sink(
    stream_handle: &OutputStreamHandle,
    audio_file: &str,
    skip: Option<Duration>,
    volume: f32,
) -> Sink {
    let sink = Sink::try_new(stream_handle).expect("Couldnt unwrap sink");
    println!("{audio_file}");
    let file = File::open(audio_file).expect("Error opening file");
    let source = Decoder::new(BufReader::new(file)).expect("Error decoding file");

    match skip {
        Some(d) => sink.append(source.skip_duration(d)),
        None => sink.append(source),
    }

    sink.pause(); // start out paused //todo change to sink.play
    sink.set_volume(volume);
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
    skip: Option<Duration>,
) -> Result<Sink, Box<dyn Error>> {
    let sink = setup_sink(stream_handle, audio_file, skip, VOLUME);
    sink.play();
    Ok(sink)
}

#[derive(Debug, Deserialize, Serialize)]
struct SongIds {
    #[serde(flatten)]
    inner: BTreeMap<String, String>,
}

fn get_files_from_json(json_path: &str) -> Vec<String> {
    let file = File::open(json_path).expect("File should open read only");
    let reader = BufReader::new(file);
    let videos: SongIds = serde_json::from_reader(reader).expect("File should be proper JSON");

    videos
        .inner
        .values()
        .cloned()
        .map(|id| format!("music/{id}/{id}.ogg"))
        .collect()
}
//TODO FIX it so that when music is paused no other song plays whent he duration of the song finishes
pub fn play_music() {
    println!("playing");
    const KEY_DEBOUNCE: Duration = Duration::from_millis(200);
    const SLEEP_DURATION: Duration = Duration::from_millis(20);

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let device_state = DeviceState::new();
    let audio_files_bind = get_files_from_json("music/.playlists/1.json");
    let audio_files: Vec<&str> = audio_files_bind.iter().map(|c| c.as_str()).collect();

    let mut current_audio_file_index = 0;
    let mut sink = setup_sink(
        &stream_handle,
        audio_files[current_audio_file_index],
        None,
        VOLUME,
    );
    let mut last_space_press_time = Instant::now() - KEY_DEBOUNCE;
    let mut last_page_end_press_time = Instant::now() - KEY_DEBOUNCE;
    let mut last_page_up_press_time = Instant::now() - KEY_DEBOUNCE;
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
            sink = play_audio_file(
                &stream_handle,
                audio_files[current_audio_file_index],
                Some(Duration::from_secs(skip_amount)),
            )
            .expect("Err playing audio file");
            now = Instant::now();
            saved_time = Duration::from_secs(0); // Reset saved_time when a new song starts
        }
        println!(
            "Elapsed: {:?}\nWhere to goto: {skip_amount}\nDuration: {song_duration}",
            now.elapsed().as_secs()
        );
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
        if keys.contains(&Keycode::Home) {
            if skip_amount > song_duration {
                skip_amount = song_duration
            }
            skip_amount += 1;
        }
        if keys.contains(&Keycode::End) {
            if 0 < skip_amount as i64 {
                skip_amount -= 1;
            } else {
                skip_amount = 0
            }
        }

        if keys.contains(&Keycode::Enter) && last_enter_press_time.elapsed() >= KEY_DEBOUNCE {
            let skip = Duration::from_secs(skip_amount);

            sink = play_audio_file(
                &stream_handle,
                audio_files[current_audio_file_index],
                Some(Duration::from_secs(skip_amount)),
            )
            .expect("Err playing audio file");
            now = Instant::now() - skip;
            last_enter_press_time = Instant::now();
        }

        if keys.contains(&Keycode::PageDown) && last_page_end_press_time.elapsed() >= KEY_DEBOUNCE {
            current_audio_file_index = (current_audio_file_index + 1) % audio_files.len();
            println!("{}", current_audio_file_index);
            skip_amount = 0;
            sink = play_audio_file(
                &stream_handle,
                audio_files[current_audio_file_index],
                Some(Duration::from_secs(skip_amount)),
            )
            .expect("Err playing audio file");

            last_page_end_press_time = Instant::now();
            now = Instant::now();
            saved_time = Duration::from_secs(0); // Reset saved_time when a new song starts
        }

        if keys.contains(&Keycode::PageUp) && last_page_up_press_time.elapsed() >= KEY_DEBOUNCE {
            if current_audio_file_index as isize - 1 < 0 {
                current_audio_file_index = audio_files.len()
            }
            current_audio_file_index = current_audio_file_index.saturating_sub(1);
            println!("{}", current_audio_file_index);
            skip_amount = 0;
            sink = play_audio_file(
                &stream_handle,
                audio_files[current_audio_file_index],
                Some(Duration::from_secs(skip_amount)),
            )
            .expect("Err playing audio file");

            last_page_up_press_time = Instant::now();
            now = Instant::now();
            saved_time = Duration::from_secs(0); // Reset saved_time when a new song starts
        }

        if keys.contains(&Keycode::Escape) {
            break;
        }
        sleep(SLEEP_DURATION);
    }
}
