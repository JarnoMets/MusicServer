use crate::db::Database;
use crate::models::MusicFile;
use crate::services::playlist_service;
use lofty::{Accessor, Probe, TaggedFileExt, TagExt};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use uuid::Uuid;
use zip::write::FileOptions;
use zip::ZipWriter;
use tempfile::NamedTempFile;

pub async fn export_playlist_zip(
    db: &Database,
    playlist_id: Uuid,
    include_rekordbox_xml: bool,
) -> Result<(NamedTempFile, String), Box<dyn std::error::Error>> {
    let playlist = playlist_service::get_playlist_with_items(db, playlist_id)
        .await?
        .ok_or("Playlist not found")?;

    let temp_zip = NamedTempFile::new()?;
    let file = File::create(temp_zip.path())?;
    let mut zip = ZipWriter::new(file);
    let options: FileOptions<'_, ()> = FileOptions::default()
        .compression_method(zip::CompressionMethod::Stored) // Stored is faster for large audio files
        .unix_permissions(0o644);

    let mut track_entries = Vec::new();

    for (index, item) in playlist.items.iter().enumerate() {
        let original_path = Path::new(&item.file_path);
        if !original_path.exists() {
            log::warn!("File not found for export: {}", item.file_path);
            continue;
        }

        // Create a temporary file to update markers
        let temp_track_file = NamedTempFile::new()?;
        std::fs::copy(original_path, temp_track_file.path())?;

        // Update metadata using lofty
        update_metadata(temp_track_file.path(), item).await;

        // Add to ZIP
        let file_name = format!(
            "{:02} - {} - {}.{}",
            index + 1,
            item.artist.as_deref().unwrap_or("Unknown"),
            item.title,
            original_path.extension().and_then(|e| e.to_str()).unwrap_or("mp3")
        );
        let sanitized_name = sanitize_filename::sanitize(&file_name);

        zip.start_file(&sanitized_name, options)?;
        let mut f = File::open(temp_track_file.path())?;
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer)?;
        zip.write_all(&buffer)?;

        if include_rekordbox_xml {
            track_entries.push((item.clone(), sanitized_name));
        }
    }

    if include_rekordbox_xml {
        let xml_content = generate_rekordbox_xml(&playlist.name, &track_entries);
        zip.start_file("rekordbox.xml", options)?;
        zip.write_all(xml_content.as_bytes())?;
    }

    zip.finish()?;
    
    let download_name = format!("{}.zip", sanitize_filename::sanitize(&playlist.name));
    Ok((temp_zip, download_name))
}

async fn update_metadata(path: &Path, item: &MusicFile) {
    let res = (move || -> Result<(), Box<dyn std::error::Error>> {
        let mut tagged_file = Probe::open(path)?.read()?;
        
        let tag = match tagged_file.primary_tag_mut() {
            Some(t) => t,
            None => {
                if let Some(first_tag) = tagged_file.first_tag_mut() {
                    first_tag
                } else {
                    // Create a new tag if none exist
                    let tag_type = tagged_file.primary_tag_type();
                    tagged_file.insert_tag(lofty::Tag::new(tag_type));
                    tagged_file.primary_tag_mut().unwrap()
                }
            }
        };

        tag.set_artist(item.artist.clone().unwrap_or_else(|| "Unknown".to_string()));
        tag.set_title(item.title.clone());
        if let Some(album) = &item.album {
            tag.set_album(album.clone());
        }
        if let Some(genre) = &item.genre {
            tag.set_genre(genre.clone());
        } else if let Some(guessed) = &item.guessed_genre {
            tag.set_genre(guessed.clone());
        }
        
        if let Some(track_num) = item.track_number {
            tag.set_track(track_num as u32);
        }

        if let Some(bpm) = item.bpm {
            if bpm > 0.0 {
                tag.insert_text(lofty::ItemKey::BPM, format!("{:.2}", bpm));
            }
        }
        if let Some(key) = &item.initial_key {
            if !key.is_empty() && key != "NONE" {
                tag.insert_text(lofty::ItemKey::InitialKey, key.clone());
            }
        }

        tag.save_to_path(path)?;
        Ok(())
    })();

    if let Err(e) = res {
        log::error!("Failed to update metadata for {}: {}", item.file_path, e);
    }
}

fn generate_rekordbox_xml(playlist_name: &str, tracks: &[(MusicFile, String)]) -> String {
    use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
    use quick_xml::Writer;
    use std::io::Cursor;

    let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);
    let _ = writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)));

    let mut root = BytesStart::new("DJ_PLAYLISTS");
    root.push_attribute(("Version", "1.0.0"));
    let _ = writer.write_event(Event::Start(root));

    let mut product = BytesStart::new("PRODUCT");
    product.push_attribute(("Name", "rekordbox"));
    product.push_attribute(("Version", "5.4.3"));
    product.push_attribute(("Company", "Pioneer DJ"));
    let _ = writer.write_event(Event::Empty(product));

    let mut collection = BytesStart::new("COLLECTION");
    collection.push_attribute(("Entries", tracks.len().to_string().as_str()));
    let _ = writer.write_event(Event::Start(collection));

    for (item, zip_path) in tracks {
        let mut track = BytesStart::new("TRACK");
        track.push_attribute(("TrackID", item.id.to_string().as_str()));
        track.push_attribute(("Name", item.title.as_str()));
        track.push_attribute(("Artist", item.artist.as_deref().unwrap_or("")));
        track.push_attribute(("Album", item.album.as_deref().unwrap_or("")));
        track.push_attribute(("Genre", item.genre.as_deref().or(item.guessed_genre.as_deref()).unwrap_or("")));
        track.push_attribute(("Kind", "Audio File"));
        track.push_attribute(("Size", "0")); // Optional
        track.push_attribute(("TotalTime", (item.duration.unwrap_or(0) / 1000).to_string().as_str()));
        
        if let Some(bpm) = item.bpm {
            track.push_attribute(("AverageBpm", format!("{:.2}", bpm).as_str()));
        }

        if let Some(key) = &item.initial_key {
            track.push_attribute(("InitialKey", key.as_str()));
        }

        // Location is tricky for ZIP. Usually rekordbox expects absolute file:// URLs.
        // We'll use a placeholder or relative if possible, but keep in mind this is within a ZIP.
        // For now let's just use the filename.
        track.push_attribute(("Location", format!("file://localhost/{}", zip_path).as_str()));
        
        let _ = writer.write_event(Event::Empty(track));
    }
    let _ = writer.write_event(Event::End(BytesEnd::new("COLLECTION")));

    let _ = writer.write_event(Event::Start(BytesStart::new("PLAYLISTS")));
    let mut root_node = BytesStart::new("NODE");
    root_node.push_attribute(("Name", "ROOT"));
    root_node.push_attribute(("Type", "0"));
    let _ = writer.write_event(Event::Start(root_node));

    let mut playlist_node = BytesStart::new("NODE");
    playlist_node.push_attribute(("Name", playlist_name));
    playlist_node.push_attribute(("Type", "1"));
    playlist_node.push_attribute(("KeyType", "0"));
    playlist_node.push_attribute(("Entries", tracks.len().to_string().as_str()));
    let _ = writer.write_event(Event::Start(playlist_node));

    for (item, _) in tracks {
        let mut track_ref = BytesStart::new("TRACK");
        track_ref.push_attribute(("Key", item.id.to_string().as_str()));
        let _ = writer.write_event(Event::Empty(track_ref));
    }

    let _ = writer.write_event(Event::End(BytesEnd::new("NODE")));
    let _ = writer.write_event(Event::End(BytesEnd::new("NODE")));
    let _ = writer.write_event(Event::End(BytesEnd::new("PLAYLISTS")));
    let _ = writer.write_event(Event::End(BytesEnd::new("DJ_PLAYLISTS")));

    String::from_utf8(writer.into_inner().into_inner()).unwrap_or_default()
}
