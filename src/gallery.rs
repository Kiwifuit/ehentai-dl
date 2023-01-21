// This one is for when we start communicating in channels
#![allow(unused_must_use)]

use std::io::prelude::*;
use std::path::Path;
use std::thread::ThreadId;
use std::{fs, io, string, sync, thread};

use crate::parsers::{get_images, get_tags, Tags};

use reqwest::blocking::Client;
use std::collections;

#[derive(Debug)]
pub enum GalleryError {
    OpenError(io::ErrorKind),
    ReadError(io::ErrorKind),
    DecodeError(string::FromUtf8Error),

    DownloadError(reqwest::Error),
}

pub struct Gallery {
    url: String,
    images: Vec<String>,
    tags: Tags,
    client: Client,
}

impl Gallery {
    pub fn new(url: String) -> Result<Self, GalleryError> {
        Ok(Self {
            url,
            tags: Tags::new(),
            images: vec![],
            client: Client::new(),
        })
    }
}

enum ChannelPacketType {
    Error,
    Image,
    Tag,
    End,
    Start,
}
struct ChannelPacket {
    // We have to track which thread this packet is from
    // because multiple packets from separate threads might arrive
    // to `rx`
    id: thread::ThreadId,
    ptype: ChannelPacketType,
    content: String,
}

pub fn make_galleries<StrPath>(path: &StrPath) -> Result<Vec<Gallery>, String>
where
    StrPath: AsRef<Path> + ?Sized,
{
    let mut file = fs::OpenOptions::new()
        .read(true)
        .open(path)
        .map_err(|err| err.kind().to_string())?;

    let mut buf = vec![];
    file.read(&mut buf).map_err(|err| err.kind().to_string())?;

    let galleries = String::from_utf8(buf)
        .map_err(|err| err.to_string())?
        .split('\n')
        .map(|i| i.to_string())
        .collect::<Vec<String>>();

    let mut spawned_threads: u8 = 0;
    let client = Client::new();
    let (tx, rx) = sync::mpsc::channel::<ChannelPacket>();
    for gallery in galleries {
        let client = client.clone();
        let tx = tx.clone();

        thread::spawn(move || {
            tx.send(ChannelPacket {
                id: thread::current().id(),
                ptype: ChannelPacketType::Start,
                content: gallery.clone(),
            });

            let gallery_content = client
                .get(gallery.clone())
                .send()
                .map_err(|e| {
                    tx.send(ChannelPacket {
                        id: thread::current().id(),
                        ptype: ChannelPacketType::Error,
                        content: format!("Error while getting gallery {}: {}", gallery, e),
                    });
                })
                .unwrap();

            let html = gallery_content
                .text()
                .map_err(|e| {
                    tx.send(ChannelPacket {
                        id: thread::current().id(),
                        ptype: ChannelPacketType::Error,
                        content: format!(
                            "Error while getting text from gallery {}: {}",
                            gallery, e
                        ),
                    });
                })
                .unwrap();

            let tags = get_tags(&html)
                .map_err(|e| {
                    tx.send(ChannelPacket {
                        id: thread::current().id(),
                        ptype: ChannelPacketType::Error,
                        content: format!(
                            "Error while extracting tags from gallery {}: {}",
                            gallery, e
                        ),
                    });
                })
                .unwrap();

            for tag in tags.keys() {
                let val = tags.get(tag).unwrap();

                tx.send(ChannelPacket {
                    id: thread::current().id(),
                    ptype: ChannelPacketType::Tag,
                    content: format!("{}:{}", tag, val),
                });
            }

            let images = get_images(&html)
                .map_err(|e| {
                    tx.send(ChannelPacket {
                        id: thread::current().id(),
                        ptype: ChannelPacketType::Error,
                        content: format!(
                            "Error while extracting images from gallery {}: {}",
                            gallery, e
                        ),
                    });
                })
                .unwrap();

            for image in images {
                tx.send(ChannelPacket {
                    id: thread::current().id(),
                    ptype: ChannelPacketType::Image,
                    content: image,
                });
            }

            tx.send(ChannelPacket {
                id: thread::current().id(),
                ptype: ChannelPacketType::End,
                content: String::from(""),
            });
        });

        spawned_threads += 1;
    }

    let mut tracked_threads = collections::HashMap::<ThreadId, Gallery>::new();
    while let Ok(packet) = rx.recv() {
        match packet.ptype {
            ChannelPacketType::Start => {
                tracked_threads.insert(packet.id, Gallery::new(packet.content).unwrap());
            }
            ChannelPacketType::End => {
                tracked_threads.remove(&packet.id);
            }
            ChannelPacketType::Error => {
                tracked_threads.remove(&packet.id);
                return Err(format!(
                    "Received error from thread {:?}: {}",
                    packet.id, packet.content
                ));
            }
            ChannelPacketType::Image => {
                tracked_threads
                    .get_mut(&packet.id)
                    .unwrap()
                    .images
                    .push(packet.content);
            }
            ChannelPacketType::Tag => {
                let tag: Vec<String> = packet.content.split(':').map(|t| t.to_string()).collect();
                tracked_threads
                    .get_mut(&packet.id)
                    .unwrap()
                    .tags
                    .add_tag(tag[0].clone(), tag[1].clone())
            }
        }
    }

    Ok(vec![])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gallery_works() {
        let gallery = make_galleries("./res/galleries.txt");

        assert!(gallery.is_ok())
    }
}
