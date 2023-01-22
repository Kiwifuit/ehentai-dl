use std::{process::exit, thread::spawn};

mod gallery;
mod logger;
mod parsers;

fn main() {
    if let Err(msg) = logger::setup_logger(logger::Level::Info) {
        eprintln!("{msg}");
        exit(1)
    }

    let galleries = gallery::make_galleries("res/galleries.txt").unwrap();
    let mut threads = vec![];
    for gallery in galleries {
        threads.push(spawn(move || {
            if let Ok(total) = gallery.download_images() {
                let total = total
                    .to_string()
                    .as_bytes()
                    .rchunks(3)
                    .rev()
                    .map(std::str::from_utf8)
                    .collect::<Result<Vec<&str>, _>>()
                    .unwrap()
                    .join(",");

                println!("Downloaded {:} bytes", total);
            }
        }));
    }

    for thread in threads {
        thread        .join()
        .unwrap_or_else(|e| {
            let exit_code;

            if let Ok(res) = e.downcast::<String>() {
                eprintln!("An error occurred while downloading: {}", res);
                exit_code = 2;
            } else {
                eprintln!("An unknown error occurred while downloading");
                exit_code = -500;
            }

            eprintln!("Please check the logs for more information. If this is a development build, please set the logger to DEBUG or TRACE");
            exit(exit_code)
        });
    }
}
