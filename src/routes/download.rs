use actix_files::NamedFile;
use actix_web::web;

pub async fn download(checksum: web::Path<String>) -> std::io::Result<NamedFile> {
    let first = checksum.chars().next().unwrap();
    let second = {
        let mut x = first.to_string();
        x.push(checksum.chars().nth(1).unwrap());
        x
    };

    let mod_checksum_path = format!("./files/{}/{}/{}.zip", first, second, checksum);

    NamedFile::open(&mod_checksum_path)
}
