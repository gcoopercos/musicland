pub mod system_tools {
    use ssh2::Session;
    use std::path::PathBuf;
    use std::path::Path;
    use std::process::{Command,Stdio};
    use std::net::TcpStream;
    use std::io::prelude::*;
    use std::io::BufReader;
    use std::fs;
    use clemenlib::*;
    use std::collections::HashMap;
    
    pub fn ls_volumio_playlists(hostportconfig: &str,
                                username: &str) -> Result<Vec<String>, ()> {

        // Connect to the local SSH server
        // let tcp = TcpStream::connect("127.0.0.1:22").unwrap();
        let tcp = TcpStream::connect(&hostportconfig).unwrap();
        let mut sess = Session::new().unwrap();
        sess.set_tcp_stream(tcp);
        sess.handshake().unwrap();

        // Might want to use "userauth_pubkey_file(..)
        match sess.userauth_agent(username) {
            Ok(()) => println!("Authenticated."),
            Err(msg) => {
                println!("Problems with ssh authentication: {}", msg);
                return Err(())
            }
        }

        let mut channel = sess.channel_session().unwrap();
        channel.exec("ls -1 /data/playlist").unwrap();
        let mut s = String::new();
        channel.read_to_string(&mut s).unwrap();
        channel.wait_close().expect("Unable to close ssh2 stream");

        let mut playlist_names: Vec<String> = Vec::new();
        
        let lines = s.lines();
        for line in lines {
            playlist_names.push(line.to_string());
        }
        
        Ok(playlist_names)
    }


    pub fn copy_music_to_volumio(
        source_file: &str,
        target_file: &str) {
        let dest_file_parent = Path::new(target_file).parent().unwrap();
        let mut cmd = Command::new("/usr/bin/ssh");
        cmd.args(&["volumio@stonecold.local".to_string(),
                   "mkdir -p ".to_string() + dest_file_parent.to_str().unwrap()]);

        cmd.stdout(Stdio::piped());
        
        println!("CMD: {:?}", cmd);
        let child =
            cmd.spawn().expect("Problems spawing child process");

        
        let reader = BufReader::new(child.stdout.unwrap());

        for line in reader.lines() {
            println!("{}", &line.unwrap());
        }

        let mut cmd = Command::new("/usr/bin/scp");

        cmd.args(&[source_file,("volumio@stonecold.local:".to_string() + target_file).as_str()]);

        cmd.stdout(Stdio::piped());
        
        println!("CMD: {:?}", cmd);
        let child =
            cmd.spawn().unwrap();
            // Command::new("/usr/bin/rsync")
            // .args(&["--size-only","-nrthvzi",local_loc, &to_loc])
            // .stdout(Stdio::piped())
            // .spawn().unwrap();

        
        let reader = BufReader::new(child.stdout.unwrap());

        for line in reader.lines() {
            println!("{}", &line.unwrap());
        }


        
    }
            
    pub fn copy_to_volumio(playlist: &Playlist) ->Result<(), ()> {
        let mut cmd = Command::new("/usr/bin/scp");
        let plfilename = String::from(&playlist.name) + ".vl";
        cmd.args(&[plfilename,
                   "volumio@stonecold.local:/data/playlist".to_string()]);
        cmd.stdout(Stdio::piped());
        
        println!("CMD: {:?}", cmd);
        let  child =
            cmd.spawn().unwrap();
            // Command::new("/usr/bin/rsync")
            // .args(&["--size-only","-nrthvzi",local_loc, &to_loc])
            // .stdout(Stdio::piped())
            // .spawn().unwrap();

        
        let reader = BufReader::new(child.stdout.unwrap());

        for line in reader.lines() {
            println!("{}", &line.unwrap());
        }
        Ok(())
    }

    
    pub fn rsync_tomachine(host: &str,
                           username: &str,
                           remote_loc: &str,
                           local_loc: &str,
                           report_only: bool) -> Result<(), ()> {
        let to_loc = username.to_owned() + "@" + host + ":" + remote_loc;

        let mut cmd =  Command::new("/usr/bin/rsync");

        if report_only {
            cmd.args(&["--size-only","--stats", "-nrthvzi",local_loc, &to_loc]);
        } else {
            cmd.args(&["--size-only","--stats", "-rthvzi",local_loc, &to_loc]);
        }
        cmd.stdout(Stdio::piped());

        println!("CMD: {:?}", cmd);
        let child =
            cmd.spawn().unwrap();
        
        let reader = BufReader::new(child.stdout.unwrap());

        for line in reader.lines() {
            println!("{}", &line.unwrap());
        }
        
        Ok(())
    }

    
    // Location examples:
    //
    // NOTE The '/' at the end of the source but not destination
    // Source:
    // gregory@enterprise.local:/mnt/storage1/music/library/
    // Local Location:
    // /home/music/library
    // pub fn rsync_library(hostportconfig: &str,
    //                      username: &str,
    //                      remote_loc: &str,
    //                      local_loc: &str) -> Result<(), ()> {

    //     // Connect to the local SSH server
    //     let tcp = TcpStream::connect(&hostportconfig).unwrap();
    //     let mut sess = Session::new().unwrap();
    //     sess.set_tcp_stream(tcp);
    //     sess.handshake().unwrap();

    //     // Might want to use "userauth_pubkey_file(..)
    //     sess.userauth_agent(username).unwrap();

    //     let mut channel = sess.channel_session().unwrap();
        
    //     let cmd = "/usr/bin/rsync --size-only -rthvzi ".to_string()
    //         + remote_loc + " " + local_loc;

    //     println!("Rsync cmd: {}", cmd);
    //     channel.exec(&cmd).unwrap();
    //     let mut s = String::new();
    //     channel.read_to_string(&mut s).unwrap();
    //     // println!("{}", s);
    //     channel.wait_close().expect("Unable to close ssh2 stream");
    //     // println!("{}", channel.exit_status().unwrap());
        
    //     let lines = s.lines();
    //     for line in lines {
    //         println!("{}",line);
    //     }
        
    //     Ok(())
    // }

    // Location examples:
    //
    // NOTE The '/' at the end of the source but not destination
    // Source:
    // gregory@enterprise.local:/mnt/storage1/music/library/
    // Local Location:
    // /home/music/library
    // pub fn rsync_missing(hostportconfig: &str,
    //                      username: &str,
    //                      remote_loc: &str,
    //                      local_loc: &str,
    //                      report_only: bool) -> Result<(), ()> {

    //     // Connect to the local SSH server
    //     let tcp = TcpStream::connect(&hostportconfig).unwrap();
    //     let mut sess = Session::new().unwrap();
    //     sess.set_tcp_stream(tcp);
    //     sess.handshake().unwrap();

    //     // Might want to use "userauth_pubkey_file(..)
    //     sess.userauth_agent(username).unwrap();

    //     let mut channel = sess.channel_session().unwrap();

    //     // Dry-run is '-n'
    //     let cmd;
    //     if report_only {
    //         cmd = "/usr/bin/rsync --stats --size-only -nrthvzi ".to_string()
    //             + remote_loc + " " + local_loc;
    //     } else {
    //         println!("COPY FILES ACTIVE");
    //         cmd = "/usr/bin/rsync --stats --size-only -rthvzi ".to_string()
    //             + remote_loc + " " + local_loc;
    //     }

    //     println!("Rsync cmd: {}", cmd);
    //     channel.exec(&cmd).unwrap();

    //     let reader = BufReader::new(channel);

    //     for line in reader.lines() {
    //         println!("{}", &line.unwrap());
    //     }
        
    //     Ok(())
    // }

    pub fn find_missing_links(playlists: &HashMap<String, Playlist>,
                              hostportconfig: &str,
                              username: &str,
                              view_root: &str,
                              library_root: &str) {
        let tcp = TcpStream::connect(&hostportconfig).unwrap();
        let mut sess = Session::new().unwrap();
        sess.set_tcp_stream(tcp);
        sess.handshake().unwrap();

        // Might want to use "userauth_pubkey_file(..)
        sess.userauth_agent(username).unwrap();

        // let mut channel = sess.channel_session().unwrap();
        // channel.exec("ls -1 /data/playlist").unwrap();
        // let mut s = String::new();
        // channel.read_to_string(&mut s).unwrap();
        // channel.wait_close().expect("Unable to close ssh2 stream");
 
        // let mut playlist_names: Vec<String> = Vec::new();
        
        // let lines = s.lines();
        // for line in lines {
        //     playlist_names.push(line.to_string());
        // }

        for (name, plist) in playlists {
            for pl_item in &plist.songs {

                println!("{} : {}",name, pl_item.uri);

                let mut view_file = String::from(&pl_item.uri);
                let library_root_uri_len = "file://".len() + library_root.len();
                view_file.replace_range(..library_root_uri_len, view_root);
                let mut lib_file = String::from(&pl_item.uri);
                lib_file.replace_range(.."file://".len(),"");

                let cmd = "ls -1 \"".to_string() +
                    &view_file + "\"";

                let mut channel = sess.channel_session().unwrap();
                channel.exec(&cmd).unwrap();
                let mut s = String::new();
                channel.read_to_string(&mut s).unwrap();
                               channel.wait_close().expect("Unable to close ssh2 stream");
                let status = channel.exit_status().unwrap();

                if status != 0 {
                    println!("Missing");
                    println!("library:root: {}", &library_root);
                    println!("cmd: {}", cmd);
                    println!("real file: {}", lib_file);

                    let target_file_path = PathBuf::from(&view_file);
                    let target_dir = target_file_path
                        .parent().unwrap();
                    let cmd = "mkdir -p \"".to_string() + target_dir.to_str().unwrap()
                    + "\"; "  +
                    "ln \"" +
                    &lib_file
                    + "\" \"" +
                    &view_file + "\"";

                    let mut channel = sess.channel_session().unwrap();
                    channel.exec(&cmd).unwrap();
                    let mut s = String::new();
                    channel.read_to_string(&mut s).unwrap();
                    channel.wait_close().expect("Unable to close ssh2 stream");

//                    channel.exit_status().unwrap();
                    println!("linking cmd: {}", cmd);

                }
            }
        }
    }


    pub fn remote_link_view(playlist: &Playlist,
                            hostportconfig: &str,
                            username: &str,
                            target_library_root: &str,
                            target_view_root: &str
    ) {
        let tcp = TcpStream::connect(&hostportconfig).unwrap();
        let mut sess = Session::new().unwrap();
        sess.set_tcp_stream(tcp);
        sess.handshake().unwrap();

        sess.userauth_agent(username).unwrap();

        for pl_item in &playlist.songs {
            println!("Song : {}",pl_item.uri);

            let mut channel = sess.channel_session().unwrap();
            let cmd = "ls -1 \"".to_string() +
                target_view_root +
                &pl_item.uri + "\"";
            // channel.exec("ls -1 /data/playlist").unwrap();
            println!("listing cmd: {}", cmd);
            channel.exec(&cmd).unwrap();
            let mut s = String::new();
            channel.read_to_string(&mut s).unwrap();
            channel.wait_close().expect("Unable to close ssh2 stream");
            let status = channel.exit_status().unwrap();
            if status != 0 {
                println!("LINK MISSING. CREATING");

                let target_file =
                    target_view_root.to_string() +
                    &pl_item.uri;

                let target_file_path = PathBuf::from(&target_file);
                let target_dir = target_file_path
                    .parent().unwrap();

                println!("Parent: {}",target_dir.to_str().unwrap());
                let mut channel = sess.channel_session().unwrap();
                let cmd = "mkdir -p \"".to_string() + target_dir.to_str().unwrap()
                    + "\"; "  +
                    "ln \"" +
                    target_library_root +
                    &pl_item.uri + "\" \"" +
                    &target_file + "\"";

                // channel.exec("ls -1 /data/playlist").unwrap();
                println!("linking cmd: {}", cmd);
                channel.exec(&cmd).unwrap();
                let mut s = String::new();
                channel.read_to_string(&mut s).unwrap();
                channel.wait_close().expect("Unable to close ssh2 stream");
                channel.exit_status().unwrap();
            }
        }
    }


    pub fn local_link_view(playlist: &Playlist,
                           library_root: &str,
                           view_root: &str)
    {
        for pl_item in &playlist.songs {
            println!("Song : {}",pl_item.uri);

            let view_filename = view_root.to_string() + &pl_item.uri;
            let library_filename = library_root.to_string() + &pl_item.uri;
            let view_path = Path::new(&view_filename);
            let library_path = Path::new(&library_filename);
            if !view_path.exists() {
                println!("Missing view file: {}",view_filename);
                println!("Creating link to: {}", library_path.to_str().unwrap());

                fs::create_dir_all(&view_path.parent().unwrap()).expect("Unable to create directory hierarchy");
                fs::hard_link(&library_path, &view_path).expect("Unable creating hard link");
            }

        }
    }

}
