use core::panic;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use base64::{engine::general_purpose, Engine as _};
use clap::{arg, command};
use lava_torrent::torrent::v1::Torrent;
use log::info;
use simple_logger::SimpleLogger;
use tokio::io::BufReader;
use tokio::{fs::File, io::AsyncReadExt};
use transmission_rpc::{
    types::{BasicAuth, TorrentAddArgs},
    TransClient,
};
use url::Url;
// #[derive(Parser, Debug)]
// #[command(author, version, about, long_about = None)]
// struct Cli {
//     files: Vec<PathBuf>,
//     #[arg(short, long)]
//     search: Vec<PathBuf>,
//     #[arg(short='k', long)]
//     skip: Vec<PathBuf>,
// }

#[tokio::main]
async fn main() {
    let matchse = command!()
        .arg(
            arg!(files: <TORRENT> "Torrent files to process")
                .num_args(1..)
                .required(true)
                .index(1),
        )
        .arg(
            arg!(search: -s --search <DIR> "Directories to search for downloaded files").num_args(1..),
        )
        .arg(
            arg!(skip: -k <SKIP> "Directories to skip").num_args(1..),
        )
        .arg(arg!(url: -u <URL> "Transmission URL, e.g.: http://example.com:9091/transmission/rpc"))
        .arg(arg!(--user <USER> "Username for Transmission"))
        .arg(arg!(--password <PASSWORD> "Password for Transmission"))
        .get_matches();

    let mut directory_map: HashMap<String, &PathBuf> = HashMap::new();
    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();
    let files: Vec<_> = matchse.get_many::<String>("files").unwrap().collect();
    let search: Vec<PathBuf> = matchse
        .get_many::<String>("search")
        .unwrap()
        .map(|x| x.into())
        .collect();
    let skip: HashSet<_> = matchse.get_many::<String>("skip").unwrap().collect();
    let url: &String = matchse.get_one("url").unwrap();
    let user = matchse.get_one::<String>("user").unwrap().to_string();
    let password = matchse.get_one::<String>("password").unwrap().to_string();
    let mut client = TransClient::with_auth(Url::parse(url).unwrap(), BasicAuth { user, password });
    if files.is_empty() {
        println!("No Torrent files provided");
        return;
    }
    for search in &search {
        info!("Indexing {:?}", search);
        if !search.is_dir() {
            continue;
        }
        let mut cnt = 0;
        search.read_dir().unwrap().for_each(|r| {
            let p = r.as_ref().unwrap().path();
            let r = p.file_name().unwrap().to_string_lossy().into_owned();
            if skip.contains(&r) {
                return;
            }
            if directory_map.contains_key(&r) {
                panic!("duplicate name {:?} in {:?}", r, search);
            }
            directory_map.insert(r, search);
            cnt += 1;
        });
        info!("Indexed {} files in {:?}", cnt, search);
    }
    info!("Processing {:?} torrent files", files);
    info!("Search directories: {:?}", search);
    let mut found_cnt = 0;
    for torrent_file in &files {
        let torrent = Torrent::read_from_file(torrent_file).unwrap();
        if let Some(path) = directory_map.get(&torrent.name) {
            let path = path.to_string_lossy().into_owned();
            // let path = path.replace("mounts/Private", "/volume2/Private").replace("mounts", "/volume3");
            // info!("Found {:?} in {:?}", torrent.name, path);
            let file = File::open(torrent_file).await.unwrap();
            let mut buffer: Vec<u8> = Vec::new();
            // read file into buffer
            let mut reader = BufReader::new(file);
            reader.read_to_end(&mut buffer).await.unwrap();
            let torrent_base64 = general_purpose::STANDARD_NO_PAD.encode(buffer);
            let add: TorrentAddArgs = TorrentAddArgs {
                metainfo: Some(torrent_base64),
                download_dir: Some(path.clone()),
                ..TorrentAddArgs::default()
            };
            let res = client.torrent_add(add).await;
            // info!("{:?}", res);
            res.unwrap();
            // info!("Adding  to {:?}", path);
            found_cnt += 1;
        } else {
            info!("Could not find {:?}", torrent.name);
        }
        // info!("Torrent: {:?}", torrent.name);
        // info!("Files: {:?}", torrent.extra_fields);
        // info!("Files: {:?}", torrent.extra_info_fields);
        // panic!();
    }
    info!("Found {} torrents from {} torrents", found_cnt, files.len());
}
