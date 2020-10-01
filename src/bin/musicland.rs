use clap::Clap;

extern crate musicland;
use musicland::system_tools::*;
use clemenlib::*;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::fs;
use std::collections::HashMap;

#[derive(Clap)]
#[clap(version = "1.0.2", author = "Greg Cooper")]
struct Opts {
    #[clap(default_value="musicland_conf.json")]
    config: String,
    
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    /// Prints the configuration
    PrintConfig(PrintConfig),
    /// Lists the playlists on the volumio box
    VolumioListsLs(VolumioLsPl),
    /// Lists the playlists on the volumio box
    VolumioAdd(VolumioAdd),
    /// Add a non-library based playlist to volumio, including music files
    VolumioAddRaw(VolumioAddRaw),
    /// Dumps local clementine lists out as json
    LocalListsLs(LocalLsPl),
    /// Finds links missing from view to library for all playlists on remote machine
    FindMissingLinks(FindMissingLinks),
    /// Does an rsync dry-run to find differences.
    FindLibraryDiffs(FindLibraryDiffs),
    /// This will export a playlist, copy it over, and sync the files
    PushLocalPlaylist(PushLocalPlaylist),
}

#[derive(Clap)]
struct VolumioLsPl {
//    clementine_file: String
}

#[derive(Clap)]
struct PrintConfig {
//    clementine_file: String
}
#[derive(Clap)]
struct VolumioAdd {
    clementine_file: String,
    playlist_name: String
}

#[derive(Clap)]
struct VolumioAddRaw {
    playlist_name: String
//    from_library_name: String,
//    to_library_name: String,
}

#[derive(Clap)]
struct LocalLsPl {
//    #[clap(default_value="default")]
    clementine_file: String,
    library_name: String,
}

#[derive(Clap)]
struct FindMissingLinks {
    library_name: String,
}

#[derive(Clap)]
struct FindLibraryDiffs {
    to_library_name: String,
}

#[derive(Clap)]
struct PushLocalPlaylist {
    to_library_name: String,
    playlist_name: String,
}


// Configuration structures

#[derive(Serialize, Deserialize, Debug)]
struct MusicLibrary {
    library_root: String,
    view_root: String,
    clementine_db_file: String,
    username: String,
    network_host: String,
    network_sshport: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct MusiclandConfig {
    local_library_ref : String,
    music_libraries: HashMap<String, MusicLibrary>,
}

fn main() {
    let opts: Opts = Opts::parse();

    let ml_config_result = read_config(&opts.config); //.unwrap();
    let ml_config;
    match ml_config_result {
        Err(err) => {
            println!("Badly formed json config: {}", err);
            std::process::exit(1);
        },
        Ok(cfg) => {
            println!("Configuration read");
            ml_config = cfg;
        }
    }
    
    match opts.subcmd {
        SubCommand::PrintConfig(_params) => {
            println!("{}", serde_json::to_string_pretty(&ml_config).unwrap());
        },
        SubCommand::VolumioAdd(params) => {
            println!("Add playlist to volumio: {:?}", &params.playlist_name);
            let plists = read_playlists("/mnt/storage1/music/library/",
                                        &params.clementine_file,
                                        Option::None).unwrap(); //Option::Some("music-library/USB/mediadrive/")).unwrap();

            let orig_playlist = &plists[&params.playlist_name];
            local_link_view(orig_playlist, //&plists[&params.playlist_name],
                            "/mnt/storage1/music/library/", "/mnt/storage1/music/view_stonecold/");

            match rsync_tomachine("stonecold.local",
                                  "volumio",
                                  "/mnt/USB/mediadrive/",
                                  "/mnt/storage1/music/view_stonecold/",
                                  false) {
                Ok(()) => {
                    println!("Rsync complete.");
                },
                Err(()) => {
                    println!("Rsync had problems.");
                }
            }

            let mut prefixed_playlist = Playlist{
                name: String::from(&orig_playlist.name),
                songs: Vec::new()
            };
            for song in &orig_playlist.songs {
                let mut dest_file = String::from(&song.uri);
                dest_file.insert_str(0, "music-library/USB/mediadrive/");
                let new_song = PlaylistItem {
                    playlist: String::from(&song.playlist),
                    service: String::from(&song.service),
                    title: String::from(&song.title),
                    artist: String::from(&song.artist),
                    uri: dest_file,
                    length: song.length
                };
                
                prefixed_playlist.songs.push(new_song);
            }
            
            export_volumio(&prefixed_playlist).expect("Problem exporting to volumio");
            copy_to_volumio(&prefixed_playlist).expect("Problem copying to volumio");
            
        },
        SubCommand::VolumioAddRaw(params) => {
            let local_clem_file = get_local_clementine().expect("Problems");
            //.to_str().unwrap();
            println!("Add playlist to volumio: {:?}", &params.playlist_name);
            let plist = read_raw_playlist(local_clem_file.to_str().unwrap(), &params.playlist_name);
            println!("{}", serde_json::to_string_pretty(&plist).unwrap());

            match plist {
                Some(playlist_data) => {
                    for song in playlist_data.songs {
                        let mut dest_file = String::from(&song.uri);
                        //                        let song_offset = dest_file.find(
                        dest_file.replace_range(.."file:///mnt/storage1/music/library".len(), "/mnt/USB/mediadrive");

                        let mut source_file = String::from(&song.uri);
                        source_file.replace_range(.."file://".len(),"");

                        println!("source_file: {}", &source_file);
                        println!("target_file: {}", &dest_file);
                        copy_music_to_volumio(&source_file, &dest_file);
                        //let dest_file_path = Path::new(&dest_file);
                        
                    }
                },
                None => {
                    println!("No playlist?");
                }
            }
            // export_volumio(&plists[&params.playlist_name]);
            // copy_to_volumio(&plists[&params.playlist_name]);
            
        },
        SubCommand::VolumioListsLs(_plt) => {
            // println!("Clementine file: {:?}", plt.clementine_file);
            match ls_volumio_playlists("stonecold.local:22","volumio") {
                Ok(playlists) => {
                    for playlist in playlists {
                        println!("Playlist: {}", playlist);
                    }
                },
                Err(()) => {
                    println!("Problems, my friend.");
                },
            }
        },
        SubCommand::LocalListsLs(plt) => {
            println!("Clementine file: {:?}", plt.clementine_file);
            let music_library = &ml_config.music_libraries[&plt.library_name];

            let local_clem_file;
            
            if "default" == plt.clementine_file {
                local_clem_file = get_local_clementine().expect("Problems");
            } else {
                local_clem_file = PathBuf::from(plt.clementine_file); //.as_str();
            }
//            let local_clem_file = get_local_clementine().expect("Problems");
            match read_playlists(&music_library.view_root,
                                 local_clem_file.to_str().unwrap(),
                                 Option::None) { //"/tmp/clemtest.db") {
                None => {},
                Some(playlists) => {
                    for (name, plist) in playlists {
                        println!("Playlist: {}", name);
                        println!("  Songs: {:?}", plist);
                    }
                },
            }
        },
        SubCommand::FindMissingLinks(cmd) => {
            let music_library = &ml_config.music_libraries[&cmd.library_name];

            let host_port = music_library.network_host.to_string() + ":" +
                &music_library.network_sshport;

            println!("Grabbing remote clementine db from host: {}", host_port);
            
            copy_from_remote_clem(&host_port,
                                  &music_library.username,
                                  "/tmp/clemlib.db").unwrap();
            match read_playlists(&music_library.view_root,"/tmp/clemlib.db",Option::None) {
                None => {},
                Some(playlists) => {
                    find_missing_links(&playlists,
                                       &host_port,
                                       &music_library.username,
                                       &music_library.view_root,
                                       &music_library.library_root);
                },
            }
        },
        // SubCommand::ExportLocalPlaylist(cmd) => {
        //     let  from_music_library = &ml_config.music_libraries[&cmd.from_library_name];
        //     let  to_music_library = &ml_config.music_libraries[&cmd.to_library_name];

        //     println!("Reading clementine db...");
        //     let plists = read_playlists(&from_music_library.library_root,
        //                                 &from_music_library.clementine_db_file,
        //                                 Option::None).unwrap();
            
        //     export_m3u(&plists[&cmd.playlist_name], &to_music_library.library_root).unwrap();
        //     println!("Playlist(s) created.");
        //     // 1. read_playlists(clemdbfile)
        //     // 2. export_m3u(playlist, "/dest/lib/prefix")
        // },
        SubCommand::PushLocalPlaylist(cmd) => {
            // 1. read_playlists(clemdbfile)
            // 2. export_m3u(playlist, "/dest/lib/prefix")
            // 3. rsync_libraryrsync library
            // 4. Copy playlists to destination machine view_root
            // 5. fix view links for exported playlist

            // A superset of export local playlist
            let  from_music_library = &ml_config.music_libraries[&ml_config.local_library_ref].library_root;

            let  to_music_library = &ml_config.music_libraries[&cmd.to_library_name];
            println!("Reading clementine db...");
            let plists = read_playlists(from_music_library,
                                        &get_local_clementine().unwrap().into_os_string().into_string().unwrap(),
                                        Option::None).unwrap();
            let playlist_found;
            if !plists.contains_key(&cmd.playlist_name) {
                println!("Playlist not found, or is empty (file based?): {}", &cmd.playlist_name);
                playlist_found = false;
            } else {
                export_m3u(&plists[&cmd.playlist_name], &to_music_library.view_root).unwrap();
                playlist_found = true;
                println!("Playlist(s) created.");
            }
            println!("Rsyncing {} to {} on {} at {}", &from_music_library,
                     &to_music_library.library_root, &to_music_library.username, &to_music_library.network_host);
            // match rsync_library(&host_port,
            //                     &to_music_library.username,
            //                     &remote_loc,
            //                     &to_music_library.library_root) {

            match rsync_tomachine(&to_music_library.network_host,
                                  &to_music_library.username,
                                  &to_music_library.library_root,
                                  &from_music_library,
                                  false) {
                                                            
                Ok(()) => {
                    println!("Rsync complete.");
                },
                Err(()) => {
                    println!("Rsync had problems.");
                }
            }

            let plist_m3uname = String::from(&cmd.playlist_name) + ".m3u";
            let dest_filename = String::from(&to_music_library.view_root)
                + &plist_m3uname;

            if playlist_found {
                let host_port = String::from(&to_music_library.network_host) +
                    ":" + &to_music_library.network_sshport;
                copy_playlist_to_remote(&host_port,
                                        &to_music_library.username,
                                        &plist_m3uname,
                                        &dest_filename).unwrap();

                println!("Playlist {} copied to {}", &plist_m3uname, &dest_filename);
                println!("Linking view files");
                remote_link_view(&plists[&cmd.playlist_name],
                                 &host_port,
                                 &to_music_library.username,
                                 &to_music_library.library_root,
                                 &to_music_library.view_root);
            } else {
                println!("Playlist not found. Skipping that part");
            }
        },
        SubCommand::FindLibraryDiffs(cmd) => {
            let  from_music_library = &ml_config.music_libraries[&ml_config.local_library_ref].library_root;
            let  to_music_library = &ml_config.music_libraries[&cmd.to_library_name];

            match rsync_tomachine(&to_music_library.network_host,
                                  &to_music_library.username,
                                  &to_music_library.library_root,
                                  &from_music_library,
                                  true) {
                                                            
                Ok(()) => {
                    println!("Rsync complete.");
                },
                Err(()) => {
                    println!("Rsync had problems.");
                }
            }
        },
    }
}

fn read_config(config_filename: &str) -> Result<MusiclandConfig> {
    let config_json = fs::read_to_string(config_filename)
        .expect("Something went wrong reading the config file");

    let config: MusiclandConfig = serde_json::from_str(&config_json)?;

    Ok(config)
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_configreader() {
        let test_config = r#"
{
    "music_libraries" : {
        "Saturn": {
            "library_root": "/home/music/library/"
        },
        "Enterprise": 
        {
            "library_root": "/mnt/storage1/music/library/"
        }
    }
}
"#;

        let config: MusiclandConfig = serde_json::from_str(test_config).unwrap();
        assert_eq!(2, config.music_libraries.len());
        for (name, library) in config.music_libraries {
            println!("{}", library.library_root);
        }
    }
}
