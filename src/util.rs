use crate::errors::*;
use std::fs::File;
use std::io::BufReader;
use log::{debug, info};
use serde::{Serialize, Deserialize};
use std::io::copy;

pub async fn download_mods(fmods: Vec<serde_json::Value>, mod_folder: &str, username: &str, api_token: &str) -> Result<()> {
    info!("Begining Mod Download");
    let client = reqwest::Client::new();
    for fmod in fmods {
        if let Some(releases) = fmod.get("releases") {
            match releases {
                serde_json::Value::Array(r) => {
                    let release = r.last().unwrap();
                    let download_url: String = release.get("download_url").unwrap().to_string();
                    let len = download_url.len();

                    // the download_url has double quotes ("") surrounding it. The slice grabs the middle bits
                    let request_url = format!("https://mods.factorio.com{}", &download_url[1..len-1]);
                    download_file(request_url, &mod_folder, &client, &[("username", &username), ("token", &api_token)]).await?;
                },
                _ => {}
            };
        }
    }
    Ok(())
}

pub async fn search_mods(fmods: Vec<String>) -> Result<Vec<serde_json::Value>> {
    let client = reqwest::Client::new();
    let mut jsondata: Vec<serde_json::Value> = Vec::new();
    for fmod in fmods {
        info!("Searching mod: {}", fmod);
        let requesturl = format!("https://mods.factorio.com/api/mods/{}", fmod);
        let res = client.get(requesturl).send()
            .await?
            .json::<serde_json::Value>()
            .await?;
        jsondata.push(res);
        info!("Found mod: {}", fmod);
    }
    Ok(jsondata)
}

pub async fn download_file<T: Serialize + ?Sized>(target: String, mod_folder: &str, client: &reqwest::Client, params: &T) -> Result<()> {
    debug!("Sending GET request to {}", target);
    let response = client.get(target).query(params).send().await?;

    let mut dest = {
        let fname = format!("{}/{}", mod_folder, response
            .url()
            .path_segments()
            .and_then(|segments| segments.last())
            .and_then(|name| if name.is_empty() { None } else { Some(name) })
            .unwrap_or("tmp.bin"));

        info!("Downolading file: {}", fname);
        File::create(fname)?
    };
    let content =  response.text().await?;
    copy(&mut content.as_bytes(), &mut dest)?;
    Ok(())
}

fn load_yaml(config: &str) -> Result<serde_yaml::Value> {
    let f = File::open(config)?;
    let reader = BufReader::new(f);
    Ok(serde_yaml::from_reader(reader)?)
}

#[derive(Serialize, Deserialize)]
pub struct FacModConfig {
    pub username: String,
    pub api_token: String,
    pub mod_dir: String,
    pub mod_list: Vec<String>
}

pub fn load_config(path: &str) -> Result<FacModConfig> {
    let canonpath = std::fs::canonicalize(path)?;
    let pathstr = canonpath.to_str().unwrap();
    info!("Loading Config: {}", pathstr);
    let conf = serde_yaml::from_value(load_yaml(pathstr)?)?;
    Ok(conf)
}