use rayon::prelude::*;
use scraper::{Html, Selector};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use url::Url;

fn get_links(url: &str) -> Vec<String> {
    let response = reqwest::blocking::get(url).expect("Failed to get response");
    let body = response.text().expect("Failed to read response body");
    let document = Html::parse_document(&body);
    let selector = Selector::parse("a[href]").expect("Failed to create selector");
    let mut links = Vec::new();
    for element in document.select(&selector) {
        if let Some(href) = element.value().attr("href") {
            links.push(href.to_string());
        }
    }
    links
}

fn download_file(url: &str, target_dir: &str) -> String {
    let response = reqwest::blocking::get(url).expect("Failed to get response");
    let headers = response.headers();

    // Retrieve the filename from the Content-Disposition header
    let filename = headers
        .get("Content-Disposition")
        .and_then(|cd| cd.to_str().ok())
        .and_then(|cd| {
            let parts: Vec<&str> = cd.split(';').collect();
            for part in parts {
                if part.trim().starts_with("filename=") {
                    return Some(
                        part.trim()
                            .split('=')
                            .nth(1)
                            .unwrap_or("nothing_to_download")
                            .trim()
                            .to_string(),
                    );
                }
            }
            None
        })
        .unwrap_or_else(|| "nothing_to_download".to_string());

    if filename != "nothing_to_download" {
        // Create the target directory if it does not exist
        fs::create_dir_all(target_dir).expect("Failed to create target directory");

        // Construct the full path for the downloaded file
        let file_path = Path::new(target_dir).join(&filename);

        // Write the content to a file
        let body = response.bytes().expect("Failed to read response body");
        let mut file = File::create(&file_path).expect("Failed to create file");
        file.write_all(&body).expect("Failed to write to file");
    }

    filename
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <url> <target_directory>", args[0]);
        std::process::exit(1);
    }

    let base_url = &args[1];
    let target_dir = &args[2];

    let links = get_links(base_url);

    links.par_iter().for_each(|link| {
        let full_link = Url::parse(base_url)
            .expect("Failed to parse base URL")
            .join(link)
            .expect("Failed to join URL");
        println!("Following link: {}", full_link);
        let first_link = get_links(&full_link.to_string());

        if let Some(first_link) = first_link.get(0) {
            let first_link_full = Url::parse(&full_link.to_string())
                .expect("Failed to parse full link")
                .join(first_link)
                .expect("Failed to join URL");
            println!("First link on {}: {}", full_link, first_link_full);
            let filename = download_file(&first_link_full.to_string(), target_dir);
            println!("Downloaded file: {}", filename);
        } else {
            println!("No links found on {}", full_link);
        }
    });
}
