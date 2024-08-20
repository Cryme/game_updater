use shared::file::{
    compress_in_mem, hash_of, ServerFileInfo, ServerFolderInfo, COMPRESSED_FOLDER_NAME,
    ROOT_FOLDER_INFO_FILE_NAME,
};
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use walkdir::WalkDir;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() < 2 {
        panic!("Error: No path provided!\nUsage: preparator \"path/to/folder\"");
    }

    let current_dir = env::current_dir().unwrap();

    let path_to_folder = &args[1];

    if !Path::new(path_to_folder).exists() {
        panic!("Folder {path_to_folder} not exists!")
    }

    let root_folder_path = current_dir.join(COMPRESSED_FOLDER_NAME);

    let mut root_folder = ServerFolderInfo::default();

    if root_folder_path.is_dir() {
        std::fs::remove_dir_all(&root_folder_path).unwrap();
    }

    std::fs::create_dir(&root_folder_path).unwrap();

    let mut current_folder = &mut root_folder;
    let mut current_depth = 1;

    for entry in
        WalkDir::new(path_to_folder).sort_by(|a, b| a.path().is_dir().cmp(&b.path().is_dir()))
    {
        let Ok(entry) = entry else { continue };

        if entry.depth() == 0 {
            //this is root folder
            continue;
        }

        let rel_path = entry
            .path()
            .to_string_lossy()
            .replace(path_to_folder, "")
            .trim_matches('/')
            .to_string();

        if entry.depth() > current_depth {
            let parent = entry.path().parent().unwrap();

            println!(
                "deeper to {}",
                parent.file_name().unwrap().to_str().unwrap()
            );

            current_folder = current_folder
                .folders
                .get_mut(parent.file_name().unwrap().to_str().unwrap())
                .unwrap();
            current_depth = entry.depth();
        } else if entry.depth() < current_depth {
            println!("Relocate required");

            current_folder = &mut root_folder;

            let chain: Vec<_> = rel_path.split('/').collect();

            for folder_name in &chain[0..chain.len() - 1] {
                println!("relocating to {}", folder_name);

                current_folder = current_folder.folders.get_mut(*folder_name).unwrap();
            }
            current_depth = entry.depth();
        }

        if entry.path().is_dir() {
            println!("{} - folder", rel_path);
            current_folder.folders.insert(
                entry.file_name().to_str().unwrap().to_string(),
                ServerFolderInfo::default(),
            );

            continue;
        }

        let compressed_path = root_folder_path.join(&rel_path);
        let compressed_folder = compressed_path.parent().unwrap();

        if !compressed_folder.is_dir() {
            std::fs::create_dir_all(compressed_folder).unwrap();
        }

        let mut file = File::open(entry.path()).unwrap();

        let mut bytes = Vec::with_capacity(file.metadata().unwrap().size() as usize);

        file.read_to_end(&mut bytes).unwrap();

        let hash = hash_of(&bytes);

        let mut file = File::create(compressed_path).unwrap();

        compress_in_mem(&bytes, &mut file).unwrap();

        println!("{} - {}", rel_path, hash);

        let info = ServerFileInfo {
            hash,
            size: file.metadata().unwrap().size(),
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
            skip_hash_check: false,
            deleted: false,
        };

        current_folder
            .files
            .insert(entry.file_name().to_str().unwrap().to_string(), info);
    }

    root_folder.calc_size();

    let out_path = current_dir.join(Path::new("./database").join(ROOT_FOLDER_INFO_FILE_NAME));

    let mut file = File::create(out_path.clone()).unwrap();

    file.write_all(
        ron::ser::to_string_pretty(&root_folder, ron::ser::PrettyConfig::default())
            .unwrap()
            .as_bytes(),
    )
    .unwrap();

    println!(
        "Compressed files folder: {}",
        root_folder_path.to_str().unwrap()
    );
    println!("Generated file list: {}", out_path.to_str().unwrap());
}
