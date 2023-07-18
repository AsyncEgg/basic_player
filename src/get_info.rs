use std::error::Error;
use std::{collections::BTreeMap, process::Command};

//Get the info from a song url like soundcloud or youtube
pub fn get_info(url: &str, info: Vec<&str>) -> Result<BTreeMap<String, String>, Box<dyn Error>> {
    println!("getting info {:?}, {:?}", info, url);
    let mut param = String::new();
    let split = "<split>";
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

    let cmd_output = String::from_utf8(cmd)?;

    // Organize info into a BTreeMap info_name: info
    let mut organized_data = BTreeMap::new();
    cmd_output.split(split).zip(info).for_each(|(c, i)| {
        organized_data.insert(i.to_string(), c.to_string());
    });

    Ok(organized_data)
}

pub fn get_mimetype_from_path(path: &str) -> Result<String, Box<dyn Error>> {
    //grab mimetype from file extention
    match path.split('.').last().unwrap() {
        "jpeg" | "jpg" => Ok("image/jpeg".to_string()),
        "svg" => Ok("image/svg+xml".to_string()),
        ext => Ok(format!("image/{ext}")),
    }
}
