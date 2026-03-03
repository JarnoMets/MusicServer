use super::{genre_cache_service, genre_label_service, discogs_service::DiscogsService};
use crate::db::Database;
use crate::models::MetadataConfig;
use crate::services::http_client_helpers::get_or_create_client;
use serde_json::Value;
use urlencoding::encode;

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
            log::debug!("Found cached genre for artist: {} -> {}", artist_name, cached_genre);
            // Treat an explicit cached value of "Unknown" as no detection result
            // so callers can mark the artist as NotFound and avoid retrying
            // repeatedly. This prevents the scheduler from repeatedly picking
            // up artists that have an explicit 'Unknown' cached value.
            if cached_genre.eq_ignore_ascii_case("unknown") {
                return Ok(None);
            }

            return Ok(Some(cached_genre));
        }
        Ok(None) => {
            log::debug!(
                "No cached genre for artist: {}, querying metadata source",
                artist_name
            );
        }
        Err(e) => {
            log::warn!("Error checking cache: {:?}", e);
        }
    }

    // Load metadata configuration to see which source to use
    let config = MetadataConfig::get_config(&db.pool).await
        .map_err(|e| format!("Failed to load metadata config: {}", e))?;

    // Use provided client or create a temporary one
    let client = get_or_create_client(http_client)?;

    // Query configured source (Default: MusicBrainz)
    let genre: Option<String> = if config.metadata_source == "discogs" {
        log::debug!("Querying Discogs for genre for: {}", artist_name);
        DiscogsService::lookup_genre(&client, &config, &artist_name).await.map_err(|e| e.to_string())?
    } else {
        log::debug!("Querying MusicBrainz for artist: {}", artist_name);
        query_musicbrainz(&artist_name, Some(&client)).await?
    };

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

/// Detect genre for a specific track (recording) using MusicBrainz recording search
/// If a matching track is found and a genre tag is detected, the function will attempt to
/// canonicalize it and write it to the `music_files.guessed_genre` column for matching
/// rows (case-insensitive match on artist and title).
#[allow(dead_code)]
pub async fn detect_genre_for_track(
    db: &Database,
    artist_name: String,
    track_title: String,
) -> Result<Option<String>, String> {
    detect_genre_for_track_with_client(db, artist_name, track_title, None).await
}

#[allow(dead_code)]
pub async fn detect_genre_for_track_with_client(
    db: &Database,
    artist_name: String,
    track_title: String,
    http_client: Option<&reqwest::Client>,
) -> Result<Option<String>, String> {
    if artist_name.trim().is_empty() || track_title.trim().is_empty() {
        return Ok(None);
    }

    // Load config to check source
    let config = MetadataConfig::get_config(&db.pool).await
        .map_err(|e| format!("Failed to load metadata config: {}", e))?;

    // Use provided client or create a temporary one
    let client = get_or_create_client(http_client)?;

    // Query configured source
    let genre = if config.metadata_source == "discogs" {
        match DiscogsService::lookup_release_date(&client, &config, &track_title, &artist_name).await {
            Ok(Some((_date, _album, style, _conf))) => {
                if let Some(s) = style {
                     Some(s)
                } else {
                    query_musicbrainz_recording(&artist_name, &track_title, Some(&client)).await?
                }
            },
            _ => query_musicbrainz_recording(&artist_name, &track_title, Some(&client)).await?
        }
    } else {
        query_musicbrainz_recording(&artist_name, &track_title, Some(&client)).await?
    };

    if let Some(ref g) = genre {
        // Try to canonicalize the detected tag
        match genre_label_service::canonicalize(db, g).await {
            Ok(Some(canonical)) => {
                // Update guessed_genre for matching tracks (title + artist)
                let _ = sqlx::query("UPDATE music_files SET guessed_genre = $1, updated_at = NOW() WHERE lower(artist) = lower($2) AND lower(title) = lower($3)")
                    .bind(&canonical)
                    .bind(&artist_name)
                    .bind(&track_title)
                    .execute(&db.pool)
                    .await;

                return Ok(Some(canonical));
            }
            Ok(None) => {
                // No canonical mapping: write raw detected tag into guessed_genre
                let _ = sqlx::query("UPDATE music_files SET guessed_genre = $1, updated_at = NOW() WHERE lower(artist) = lower($2) AND lower(title) = lower($3)")
                    .bind(g)
                    .bind(&artist_name)
                    .bind(&track_title)
                    .execute(&db.pool)
                    .await;
            }
            Err(e) => {
                log::warn!("Error canonicalizing genre {}: {:?}", g, e);
            }
        }
    }

    Ok(genre)
}

/// Query MusicBrainz recording search endpoint for a recording's top tag
#[allow(dead_code)]
async fn query_musicbrainz_recording(
    artist_name: &str,
    track_title: &str,
    http_client: Option<&reqwest::Client>,
) -> Result<Option<String>, String> {
    let encoded_artist = encode(artist_name);
    let encoded_title = encode(track_title);

    // search by recording and artist to improve precision
    let url = format!(
        "https://musicbrainz.org/ws/2/recording/?query=recording:\"{}\"%%20AND%%20artist:\"{}\"&fmt=json&limit=10",
        encoded_title, encoded_artist
    );

    // Use provided client or create a temporary one
    let client = get_or_create_client(http_client)?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let json: Value = response
        .json()
        .await
        .map_err(|e| format!("JSON parsing failed: {}", e))?;

    Ok(get_recording_top_tag_from_json(&json, artist_name, track_title))
}

/// Extract the top tag from a MusicBrainz recording search response
#[allow(dead_code)]
fn get_recording_top_tag_from_json(json: &Value, artist_name: &str, track_title: &str) -> Option<String> {
    if let Some(recordings) = json.get("recordings").and_then(|r| r.as_array()) {
        // Prefer exact title + artist matches
        let mut candidates: Vec<&Value> = recordings
            .iter()
            .filter(|rec| rec.get("title").and_then(|t| t.as_str()).map(|s| s.eq_ignore_ascii_case(track_title)).unwrap_or(false))
            .collect();

        if candidates.is_empty() {
            // Fallback to any recording
            candidates = recordings.iter().collect();
        }

        for rec in candidates {
            // Check artist-credit for artist name match
            let artist_match = rec
                .get("artist-credit")
                .and_then(|ac| ac.as_array())
                .map(|arr| {
                    arr.iter().any(|entry| {
                        entry.get("name").and_then(|n| n.as_str()).map(|n| n.eq_ignore_ascii_case(artist_name)).unwrap_or(false)
                    })
                })
                .unwrap_or(false);

            if !artist_match {
                // try next candidate
                continue;
            }

            if let Some(tags) = rec.get("tags").and_then(|t| t.as_array()) {
                let mut highest_tag: Option<String> = None;
                let mut highest_count = 0;

                for tag in tags {
                    if let Some(count) = tag.get("count").and_then(|c| c.as_u64()) {
                        if count > highest_count {
                            highest_count = count;
                            if let Some(name) = tag.get("name").and_then(|n| n.as_str()) {
                                highest_tag = Some(name.to_string());
                            }
                        }
                    }
                }

                if highest_tag.is_some() {
                    return highest_tag;
                }
            }

            // If no tags, try recording's "disambiguation" as a fallback
            if let Some(disamb) = rec.get("disambiguation").and_then(|d| d.as_str()) {
                return Some(disamb.to_string());
            }
        }
    }

    None
}

/// Query MusicBrainz API for artist genre
async fn query_musicbrainz(artist_name: &str, http_client: Option<&reqwest::Client>) -> Result<Option<String>, String> {
    let encoded_artist_name = encode(artist_name);

    let url = format!(
        "https://musicbrainz.org/ws/2/artist/?query=artist:{}&fmt=json",
        encoded_artist_name
    );

    // Use provided client or create a temporary one
    let client = get_or_create_client(http_client)?;

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
