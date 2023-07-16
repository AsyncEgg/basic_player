use chrono::Timelike;
use device_query::{DeviceQuery, DeviceState, Keycode};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use serde::{Deserialize, Serialize};

use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{self, BufReader},
    path::PathBuf,
    thread::sleep,
    time::{Duration, Instant},
};

fn setup_song_queue(ids: Vec<&str>) -> io::Result<BTreeMap<usize, (String, Vec<PathBuf>)>> {
    let mut b = BTreeMap::new();

    for (index, id) in ids.iter().enumerate() {
        let dir = format!("music/{id}/parts/");
        let mut parts = vec![];
        for (index, entry) in fs::read_dir(dir)?.enumerate() {
            let entry = entry?.path();
            parts.push(entry);
        }
        parts.sort();
        b.insert(index, (id.to_string(), parts));
    }

    Ok(b)
}

fn setup_sink(
    stream_handle: &OutputStreamHandle,
    audio_files: Vec<PathBuf>,
    id: String,
    skip: Option<Duration>,
) -> Sink {
    let sink = Sink::try_new(stream_handle).unwrap();

    for path in audio_files {
        //println!("Current File: {path:?}");
        let file = File::open(format!("music/{id}/parts/000.ogg")).expect("Couldnt open path");
        let source = Decoder::new(BufReader::new(file)).expect("Erm");
        match skip {
            Some(d) => sink.append(source.skip_duration(d)),
            None => sink.append(source),
        }
    }

    sink.pause(); // start out paused //todo change to sink.play

    sink
}

pub fn play_music_new() {
    println!("Setting up music");
    let ids = vec![
        "U0AcBryPUxY",
        "U8gKLveIvuk",
        "U8gKLveIvuk",
        "C4WzPHaOS2M",
        "YQX2CsMCB9M",
        "IpUKO-WKaqo",
        "wPKmEUnj3IE",
    ];
    let ids = vec!["IvmtRiQUbjA", "2uarU_OtnFU"];
    let audio_files = setup_song_queue(ids).expect("Error setting up queue");
    println!("{:#?}", audio_files);
    println!("playing");
    const KEY_DEBOUNCE: Duration = Duration::from_millis(200);
    const SLEEP_DURATION: Duration = Duration::from_millis(20);

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let device_state = DeviceState::new();

    let mut current_audio_file_index = 0;
    let (id, current_audio_file) = &audio_files[&current_audio_file_index];
    let mut sink = setup_sink(
        &stream_handle,
        current_audio_file.to_vec(),
        id.to_string(),
        None,
    );
    let mut last_space_press_time = Instant::now() - KEY_DEBOUNCE;
    let mut last_right_press_time = Instant::now() - KEY_DEBOUNCE;
    let mut last_enter_press_time = Instant::now() - KEY_DEBOUNCE;
    let mut now = Instant::now();
    let mut saved_time = Duration::from_secs(0);
    let mut pause_start_time: Option<Instant> = None;
    let mut skip_amount = 0;
    sink.play();
    loop {
        let (id, current_audio_file) = &audio_files[&current_audio_file_index];

        let song_duration = duation_from_song_duration(&get_info_from_id(id).duration);
        let converted_song_duration = Duration::from_secs(song_duration);
        // let elapsed = format!("{:?}", now.elapsed());
        // println!("{elapsed}");
        if now.elapsed() > converted_song_duration {
            current_audio_file_index = (current_audio_file_index + 1) % audio_files.len();
            println!("{}", current_audio_file_index);
            sink = setup_sink(&stream_handle, current_audio_file.clone(), id.clone(), None);
            skip_amount = 0;
            sink.play();
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

            sink = setup_sink(
                &stream_handle,
                current_audio_file.clone(),
                id.clone(),
                Some(skip),
            );
            sink.play();
            now = Instant::now() - skip;
            last_enter_press_time = Instant::now();
        }

        if keys.contains(&Keycode::Right) && last_right_press_time.elapsed() >= KEY_DEBOUNCE {
            current_audio_file_index = (current_audio_file_index + 1) % audio_files.len();
            println!("{}", current_audio_file_index);
            sink = setup_sink(&stream_handle, current_audio_file.clone(), id.clone(), None);
            skip_amount = 0;
            sink.play();

            last_right_press_time = Instant::now();
            now = Instant::now();
            saved_time = Duration::from_secs(0); // Reset saved_time when a new song starts
        }

        if keys.contains(&Keycode::Escape) {
            break;
        }
        sleep(SLEEP_DURATION);
    }
}
//todo make this faster :p
fn setup_sink_old(
    stream_handle: &OutputStreamHandle,
    audio_file: &str,
    skip: Option<Duration>,
) -> Sink {
    let sink = Sink::try_new(stream_handle).unwrap();

    let file = File::open(audio_file).unwrap();
    let source = Decoder::new(BufReader::new(file)).unwrap();

    match skip {
        Some(d) => sink.append(source.skip_duration(d)),
        None => sink.append(source),
    }

    sink.pause(); // start out paused //todo change to sink.play

    sink
}

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
    let naive_time = chrono::NaiveTime::parse_from_str(song_duration, "%H:%M:%S").unwrap();
    let hours = naive_time.hour() as u64;
    let minutes = naive_time.minute() as u64;
    let seconds = naive_time.second() as u64;
    hours * 3600 + minutes * 60 + seconds
}

//  fn play_music_rodio() {
//     println!("playing");
//     const KEY_DEBOUNCE: Duration = Duration::from_millis(200);
//     const SLEEP_DURATION: Duration = Duration::from_millis(20);

//     let (_stream, stream_handle) = OutputStream::try_default().unwrap();
//     let device_state = DeviceState::new();
//     let audio_files = vec![
//         // "music/Q_9VMaX61nI/Q_9VMaX61nI.ogg",
//         // "music/VZ-gmdcWWZs/VZ-gmdcWWZs.ogg",
//         // "music/DUT5rEU6pqM/DUT5rEU6pqM.ogg",
//         // "music/Q_9VMaX61nI/Q_9VMaX61nI.ogg",
//         "music/MFF-diLFhtQ/MFF-diLFhtQ.ogg",
//     ];

//     let mut current_audio_file_index = 0;
//     let mut sink = setup_sink(&stream_handle, audio_files[current_audio_file_index], None);
//     let mut last_space_press_time = Instant::now() - KEY_DEBOUNCE;
//     let mut last_right_press_time = Instant::now() - KEY_DEBOUNCE;
//     let mut last_enter_press_time = Instant::now() - KEY_DEBOUNCE;
//     let mut now = Instant::now();
//     let mut saved_time = Duration::from_secs(0);
//     let mut pause_start_time: Option<Instant> = None;
//     let mut skip_amount = 0;
//     sink.play();
//     loop {
//         let id = audio_files[current_audio_file_index]
//             .split('/')
//             .collect::<Vec<&str>>()[1];
//         let song_duration = duation_from_song_duration(&get_info_from_id(id).duration);
//         let converted_song_duration = Duration::from_secs(song_duration);
//         // let elapsed = format!("{:?}", now.elapsed());
//         // println!("{elapsed}");
//         if now.elapsed() > converted_song_duration {
//             current_audio_file_index = (current_audio_file_index + 1) % audio_files.len();
//             println!("{}", current_audio_file_index);
//             sink = setup_sink(&stream_handle, audio_files[current_audio_file_index], None);
//             skip_amount = 0;
//             sink.play();
//             now = Instant::now();
//             saved_time = Duration::from_secs(0); // Reset saved_time when a new song starts
//         }
//         println!(
//             "Elapsed: {:?}\nWhere to goto: {skip_amount}\nDuration: {song_duration}",
//             now.elapsed().as_secs()
//         );
//         let keys: Vec<Keycode> = device_state.get_keys();
//         if keys.contains(&Keycode::F9) && last_space_press_time.elapsed() >= KEY_DEBOUNCE {
//             toggle_playback(&mut sink);
//             last_space_press_time = Instant::now();

//             if sink.is_paused() {
//                 pause_start_time = Some(Instant::now());
//             } else {
//                 if let Some(pause_time) = pause_start_time {
//                     saved_time += Instant::now() - pause_time;
//                     pause_start_time = None;
//                 }
//                 now = Instant::now() - saved_time - Duration::from_secs(2);
//             }
//         }
//         if keys.contains(&Keycode::Up) {
//             if skip_amount > song_duration {
//                 skip_amount = song_duration
//             }
//             skip_amount += 1;
//         }
//         if keys.contains(&Keycode::Down) {
//             if 0 < skip_amount as i64 {
//                 skip_amount -= 1;
//             } else {
//                 skip_amount = 0
//             }
//         }

//         if keys.contains(&Keycode::Enter) && last_enter_press_time.elapsed() >= KEY_DEBOUNCE {
//             let skip = Duration::from_secs(skip_amount);

//             sink = setup_sink(
//                 &stream_handle,
//                 audio_files[current_audio_file_index],
//                 Some(skip),
//             );
//             sink.play();
//             now = Instant::now() - skip;
//             last_enter_press_time = Instant::now();
//         }

//         if keys.contains(&Keycode::Right) && last_right_press_time.elapsed() >= KEY_DEBOUNCE {
//             current_audio_file_index = (current_audio_file_index + 1) % audio_files.len();
//             println!("{}", current_audio_file_index);
//             sink = setup_sink(&stream_handle, audio_files[current_audio_file_index], None);
//             skip_amount = 0;
//             sink.play();

//             last_right_press_time = Instant::now();
//             now = Instant::now();
//             saved_time = Duration::from_secs(0); // Reset saved_time when a new song starts
//         }

//         if keys.contains(&Keycode::Escape) {
//             break;
//         }
//         sleep(SLEEP_DURATION);
//     }
// }
