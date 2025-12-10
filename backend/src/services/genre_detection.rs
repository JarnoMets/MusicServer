use super::{genre_cache_service, genre_label_service};
use crate::db::Database;
use serde_json::Value;
use urlencoding::encode;

#[allow(dead_code)]
pub struct GenreDetectionService;

/// Detect genre for an artist using MusicBrainz API or cache
/// Uses provided HTTP client for connection reuse, falls back to creating one if not provided
pub async fn detect_genre_for_artist(
    db: &Database,
    artist_name: String,
) -> Result<Option<String>, String> {
    detect_genre_for_artist_with_client(db, artist_name, None).await
}

/// Detect genre for an artist using provided HTTP client (more memory efficient)
pub async fn detect_genre_for_artist_with_client(
    db: &Database,
    artist_name: String,
    http_client: Option<&reqwest::Client>,
) -> Result<Option<String>, String> {
    if artist_name.trim().is_empty() {
        return Ok(None);
    }

    // Check cache first
    match genre_cache_service::get_cached_genre(db, &artist_name).await {
        Ok(Some(cached_genre)) => {
            log::debug!("Found cached genre for artist: {}", artist_name);
            return Ok(Some(cached_genre));
        }
        Ok(None) => {
            log::debug!(
                "No cached genre for artist: {}, querying MusicBrainz",
                artist_name
            );
        }
        Err(e) => {
            log::warn!("Error checking cache: {:?}", e);
        }
    }

    // If not cached, query MusicBrainz
    let genre = query_musicbrainz(&artist_name, http_client).await?;

    // Store in cache if found
    if let Some(ref g) = genre {
        // Try to canonicalize the detected tag
        match genre_label_service::canonicalize(db, g).await {
            Ok(Some(canonical)) => {
                // Cache canonical genre under artist
                match genre_cache_service::cache_genre(db, &artist_name, &canonical).await {
                    Ok(_) => log::debug!(
                        "Cached canonical genre for artist={} -> {}",
                        artist_name,
                        canonical
                    ),
                    Err(e) => log::warn!("Error caching canonical genre: {:?}", e),
                }
                return Ok(Some(canonical));
            }
            Ok(None) => {
                // No canonical mapping: store the raw detected tag
                match genre_cache_service::cache_genre(db, &artist_name, g).await {
                    Ok(_) => log::debug!(
                        "Cached raw detected genre for artist: {} -> {}",
                        artist_name,
                        g
                    ),
                    Err(e) => log::warn!("Error caching raw detected genre: {:?}", e),
                }
            }
            Err(e) => {
                log::warn!("Error canonicalizing genre {}: {:?}", g, e);
            }
        }
    }

    Ok(genre)
}

/// Query MusicBrainz API for artist genre
async fn query_musicbrainz(artist_name: &str, http_client: Option<&reqwest::Client>) -> Result<Option<String>, String> {
    let encoded_artist_name = encode(artist_name);

    let url = format!(
        "https://musicbrainz.org/ws/2/artist/?query=artist:{}&fmt=json",
        encoded_artist_name
    );

    // Use provided client or create a temporary one
    let owned_client;
    let client = match http_client {
        Some(c) => c,
        None => {
            owned_client = reqwest::Client::builder()
                .user_agent("MusicManager/1.0.0 (jarno.mets@gmail.com)")
                .build()
                .map_err(|e| format!("Failed to create client: {}", e))?;
            &owned_client
        }
    };

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let json: Value = response
        .json()
        .await
        .map_err(|e| format!("JSON parsing failed: {}", e))?;

    Ok(get_artist_top_tag_from_json(&json, artist_name))
}

/// Extract the top tag from the MusicBrainz API response
fn get_artist_top_tag_from_json(json: &Value, artist_name: &str) -> Option<String> {
    if let Some(artists) = json.get("artists").and_then(|artists| artists.as_array()) {
        let matched_artists: Vec<_> = artists
            .iter()
            .filter(|artist| artist["name"].as_str() == Some(artist_name))
            .collect();

        if let Some(first_artist) = matched_artists.first() {
            if let Some(tags) = first_artist.get("tags").and_then(|tags| tags.as_array()) {
                let mut highest_tag: Option<String> = None;
                let mut highest_count = 0;

                for tag in tags {
                    if let Some(count) = tag.get("count").and_then(|count| count.as_u64()) {
                        if count > highest_count {
                            highest_count = count;
                            if let Some(name) = tag.get("name").and_then(|name| name.as_str()) {
                                highest_tag = Some(name.to_string());
                            }
                        }
                    }
                }

                return highest_tag;
            } else if let Some(disambiguation) = first_artist
                .get("disambiguation")
                .and_then(|dis| dis.as_str())
            {
                return Some(disambiguation.to_string());
            }
        }
    }

    None
}
