// auto_version_from_date_mod

//! new version as date is written toCargo.toml and service_worker.js

//region: use statements
use chrono::DateTime;
use chrono::Timelike;
use chrono::{Datelike, Utc};
use filetime::FileTime;
use serde_derive::{Deserialize, Serialize};
use std::{fs, path::Path};
use unwrap::unwrap;

use crate::auto_helper_functions_mod::*;
//endregion

///file metadata
#[derive(Serialize, Deserialize)]
struct FileMetaData {
    //filename with path from Cargo.toml folder
    filename: String,
    //filedate from file
    filedate: String,
}

///the struct that is in the file auto_version_from_date.json
#[derive(Serialize, Deserialize)]
struct AutoVersionFromDate {
    ///vector of file metadata
    vec_file_metadata: Vec<FileMetaData>,
}

/// new version as date is written toCargo.toml and service_worker.js
/// In Cargo.toml writes the version as the date `yyyy.mmdd.HHMM` ex. `2019.1221.2359`.  
/// For non-library projects, the semver specification is not really useful.  
/// Having the version as the date is just fine for executables and much more human readable.  
/// The function must be executed in the root project folder where is the Cargo.toml.  
///
/// ## service_worker.js
///
/// Inside the PWA service worker javascript file is also needed to change the version.  
/// The program searches for `service_worker.js` and modify the version.  
///
/// ## no need to change version if no files changed
///
/// If src/*.rs or Cargo.toml files are not changed from last compile,
/// than no need to change version.  
/// It does not support workspaces yet.  
/// The dates of the files will be stored in target folder as auto_version_from_date.json.
/// Warning: I don't check if the service worker has changed because it rarely does.  
pub fn auto_version_from_date() {
    let mut is_files_equal = true;

    //find auto_version_from_date.json
    let json_filepath = "target/auto_version_from_date.json";
    let js_struct: AutoVersionFromDate;
    let f = fs::read_to_string(json_filepath);
    match f {
        Ok(x) => {
            println!("reading from {}", json_filepath);
            //read struct from file
            js_struct = unwrap!(serde_json::from_str(x.as_str()));
        }
        Err(_error) => {
            println!("{}file does not exist: {}{}", *RED, json_filepath, *RESET);
            //create empty struct
            js_struct = AutoVersionFromDate {
                vec_file_metadata: Vec::new(),
            }
        }
    };
    //make a vector of files
    let mut vec_of_metadata: Vec<FileMetaData> = Vec::new();
    //the Cargo.toml and in the folder rs
    let filename = "Cargo.toml".to_string();
    let metadata = unwrap!(fs::metadata(filename.as_str()));
    let mtime = FileTime::from_last_modification_time(&metadata);
    let filedate = format!("{}", mtime);
    vec_of_metadata.push(FileMetaData { filename, filedate });

    //println!("fs: {}", serde_json::to_string(&v).unwrap());

    for entry in unwrap!(fs::read_dir("src")) {
        let entry = unwrap!(entry);
        let path = entry.file_name();

        let filename = format!("src/{:?}", path);
        let filename = filename.replace("\"", "");
        //println!("filename: {}", &filename);
        let metadata = unwrap!(fs::metadata(filename.as_str()));
        let mtime = FileTime::from_last_modification_time(&metadata);
        let filedate = format!("{}", mtime);
        vec_of_metadata.push(FileMetaData { filename, filedate });
    }

    //println!("fs: {}", serde_json::to_string(&v).unwrap());

    //if files are added or deleted, other files must be also changed
    //I need to check if the files on the filesystem are the same as in the json
    for x in &vec_of_metadata {
        //search in json file
        let mut is_equal = false;
        for y in &js_struct.vec_file_metadata {
            if x.filename == y.filename && x.filedate == y.filedate {
                is_equal = true;
                break;
            } else {
                //println!("{} {}\n", y.filename, y.filedate);
            }
        }
        if !is_equal {
            println!("{} {}", x.filename, x.filedate);
            is_files_equal = false;
            break;
        }
    }

    println!("is_files_equal: {}", is_files_equal);

    if !is_files_equal {
        let date = Utc::now();
        let new_version = version_from_date(date);
        //region: write version in Cargo.toml
        {
            println!("{}write version to Cargo.toml{}", *GREEN, *RESET);
            //find version in Cargo.toml
            let cargo_filename = "Cargo.toml";
            let mut cargo_content = unwrap!(fs::read_to_string(cargo_filename));
            let delimiter = r#"version = ""#;
            let delimiter_len = delimiter.len();
            let option_location = cargo_content.find(delimiter);
            if let Some(location) = option_location {
                let start_version = location + delimiter_len;
                let option_end_quote = find_from(cargo_content.as_str(), start_version, r#"""#);
                if let Some(end_version) = option_end_quote {
                    //delete all the characters in between the markers
                    let old_version: String =
                        cargo_content.drain(start_version..end_version).collect();
                    println!(r#"old version: "{}""#, old_version.as_str());
                    if new_version != old_version {
                        println!("new_version {}", new_version);
                        cargo_content.insert_str(start_version, new_version.as_str());
                        println!("{}write file: {}{}", *YELLOW, cargo_filename, *RESET);
                        let _x = fs::write(cargo_filename, cargo_content);
                        //the Cargo.toml is now different

                        //correct the vector
                        let filename = "Cargo.toml".to_string();
                        let metadata = unwrap!(fs::metadata(filename.as_str()));
                        let mtime = FileTime::from_last_modification_time(&metadata);
                        let filedate = format!("{}", mtime);
                        unwrap!(vec_of_metadata.get_mut(0)).filedate = filedate;

                        println!("save the new file metadata");
                        let x = AutoVersionFromDate {
                            vec_file_metadata: vec_of_metadata,
                        };
                        let y = unwrap!(serde_json::to_string(&x));
                        let _f = fs::write(json_filepath, y);
                    }
                } else {
                    panic!("no end quote for version");
                }
            } else {
                panic!("Cargo.toml has no version");
            }
        }
        // endregion

        //region: write version in service_worker.js

        // search for file service_worker.js
        // if the parent folder has Cargo.toml, than search there
        // because it is a workspace with members
        // else search here

        let cargo_filename = "../Cargo.toml";
        let start_dir = if Path::new(cargo_filename).exists() {
            Path::new("../")
        } else {
            Path::new("./")
        };
        println!("start_dir: {:?}", start_dir,);

        // fill a vector of files
        for js_filename in &unwrap!(crate::utils_mod::traverse_dir_with_exclude_dir(
            start_dir,
            "/service_worker.js",
            &vec!["/.git".to_string(), "/target".to_string()]
        )) {
            println!("{}write version in {}{}", *GREEN, js_filename, *RESET);
            let mut js_content = unwrap!(fs::read_to_string(js_filename));
            let delimiter = r#"const CACHE_NAME = '"#;
            let delimiter_len = delimiter.len();
            let option_location = js_content.find(delimiter);
            if let Some(location) = option_location {
                let start_version = location + delimiter_len;
                let option_end_quote = find_from(js_content.as_str(), start_version, r#"';"#);
                if let Some(end_version) = option_end_quote {
                    //delete all the characters in between the markers
                    let old_version: String =
                        js_content.drain(start_version..end_version).collect();
                    println!(r#"old version: "{}""#, old_version.as_str());
                    if new_version != old_version {
                        println!("new_version {}", new_version);
                        js_content.insert_str(start_version, new_version.as_str());
                        println!("{}write file: {}{}", *YELLOW, js_filename, *RESET);
                        let _x = fs::write(js_filename, js_content);
                    }
                } else {
                    panic!("no end quote for version");
                }
            } else {
                panic!("service_worker.js has no version");
            }
        }
        //endregion
    }
}

/// converts a date to a version
fn version_from_date(date: DateTime<Utc>) -> String {
    // in Rust the version must not begin with zero.
    // There is an exceptional situation where is midnight 00.
    let new_version = if date.hour() == 0 {
        format!(
            "{:04}.{}{:02}.{}",
            date.year(),
            date.month(),
            date.day(),
            date.minute()
        )
    } else {
        format!(
            "{:04}.{}{:02}.{}{:02}",
            date.year(),
            date.month(),
            date.day(),
            date.hour(),
            date.minute()
        )
    };
    //return
    new_version
}

/// in string find from position
fn find_from(rs_content: &str, from: usize, find: &str) -> Option<usize> {
    let slice01 = rs_content.get(from..).unwrap();
    let option_location = slice01.find(find);
    if let Some(location) = option_location {
        //return Option with usize
        Some(from + location)
    } else {
        //return Option with none
        option_location
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_date_to_version() {
        let date_time = chrono::TimeZone::ymd(&Utc, 2020, 5, 22).and_hms(00, 34, 0);

        let version = version_from_date(date_time);
        assert_eq!(version, "2020.522.34");
    }
}
