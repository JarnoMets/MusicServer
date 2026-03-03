use serde_json::Value;
use std::error::Error;
use crate::models::metadata::MetadataConfig;
use crate::services::artist_parser::{clean_artist, clean_title};
use urlencoding::encode;

pub struct DiscogsService;

#[derive(Debug)]
#[allow(dead_code)]
struct Candidate {
    year: Option<u32>,
    year_str: Option<String>,
    resource_url: Option<String>,
    album: Option<String>,
    genre: Option<String>,
    confidence: f64,
    has_track: bool,
    artist_match: bool,
    is_compilation: bool,
}

impl DiscogsService {
    async fn perform_search(client: &reqwest::Client, url: &str, config: &MetadataConfig) -> Result<Value, String> {
        let mut attempts = 0;
        let max_attempts = 3; 
        let mut delay = tokio::time::Duration::from_secs(2);

        loop {
            match Self::fetch_discogs(client, url, config).await {
                Ok(json) => return Ok(json),
                Err(e) if e.contains("authentication failed") => return Err(e),
                Err(e) => {
                    attempts += 1;
                    if attempts >= max_attempts {
                        log::error!("Discogs fetch failed after {} attempts. Last error: {}", max_attempts, e);
                        return Err(e);
                    }

                    if e.contains("429") || e.contains("Too Many Requests") {
                        log::warn!("Discogs rate limit hit (429). Waiting {}s before retry {}/{}...", delay.as_secs(), attempts, max_attempts);
                        tokio::time::sleep(delay).await;
                        delay = std::cmp::min(delay * 2, tokio::time::Duration::from_secs(60));
                    } else {
                        log::warn!("Discogs fetch failed: {}. Retrying in 1s...", e);
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    }
                }
            }
        }
    }
    
    /// Search for a release on Discogs and Extract release date/album/styles
    pub async fn lookup_release_date(
        client: &reqwest::Client,
        config: &MetadataConfig,
        title: &str,
        artist: &str,
    ) -> Result<Option<(String, Option<String>, Option<String>, f64)>, Box<dyn Error + Send + Sync>> {
        if config.metadata_source == "musicbrainz" {
            return Ok(None);
        }
        Self::lookup_with_base_url(client, config, title, artist, "https://api.discogs.com").await
    }

    // Add lookup_genre function back
    pub async fn lookup_genre(
        client: &reqwest::Client,
        config: &MetadataConfig,
        artist: &str,
    ) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
        let clean_artist_str = clean_artist(artist);
        let url = format!("https://api.discogs.com/database/search?type=artist&q={}", encode(&clean_artist_str));
        
        let data = Self::perform_search(client, &url, config).await?;
        
        if let Some(results) = data["results"].as_array() {
            for result in results {
                 // Try to verify name match
                 if let Some(name) = result["title"].as_str() {
                     let clean_res = clean_artist(name);
                     if clean_res.eq_ignore_ascii_case(&clean_artist_str) {
                         // Check resource_url for details
                         if let Some(res_url) = result["resource_url"].as_str() {
                             if let Ok(_details) = Self::fetch_discogs(client, res_url, config).await {
                                 // Look for profile or other genre indicators? 
                                 // Discogs artists don't always have a direct genre field 
                                 // We might need to check their releases, but that's expensive.
                                 // For now return None as Discogs isn't great for direct artist -> genre
                                 return Ok(None);
                             }
                         }
                     }
                 }
            }
        }
        
        Ok(None)
    }

    pub async fn lookup_with_base_url(
        client: &reqwest::Client,
        config: &MetadataConfig,
        title: &str,
        artist: &str,
        base_url: &str,
    ) -> Result<Option<(String, Option<String>, Option<String>, f64)>, Box<dyn Error + Send + Sync>> {
        let token = config.discogs_token.as_deref().unwrap_or("");
        
        let clean_title_str = clean_title(title);
        let clean_artist_str = clean_artist(artist);

        let build_candidate = |result: &Value, details: Option<&Value>, _: &str, _: &str| -> Option<Candidate> {
            let year = result["year"].as_str()
                .and_then(|y| y.split('-').next())
                .and_then(|y| y.parse::<u32>().ok());

            let year_str = year.map(|y| y.to_string());
            let album = result["title"].as_str().map(|s| s.to_string());
            let resource_url = result["resource_url"].as_str().map(|s| s.to_string());

            let genre = details.and_then(|d| {
                d["genre"].as_array().and_then(|g| g.first()).and_then(|g| g.as_str()).map(|s| s.to_string())
            }).or_else(|| {
                details.and_then(|d| {
                    d["style"].as_array().and_then(|s| s.first()).and_then(|s| s.as_str()).map(|s| s.to_string())
                })
            }).or_else(|| {
                // Fallback to result style/genre if details missing
                result["style"].as_array().and_then(|s| s.first()).and_then(|s| s.as_str()).map(|s| s.to_string())
            });

            // Check if it's a compilation
            let is_compilation = result["format"].as_array()
                .map(|formats| formats.iter().any(|f| {
                    let s = f.as_str().unwrap_or("").to_lowercase();
                    s.contains("compilation") || s.contains("promo") || s.contains("mixed") || s.contains("unofficial")
                }))
                .unwrap_or(false);
            
            // Also check title for compilation indicators
            let title_compilation = album.as_ref().map(|t| {
                let t_lower = t.to_lowercase();
                t_lower.contains("best of") || t_lower.contains("greatest hits") || t_lower.contains("anthology")
            }).unwrap_or(false);

            let is_compilation = is_compilation || title_compilation;

            let confidence = if result["id"].is_i64() { 1.0 } else { 0.0 };

            Some(Candidate {
                year,
                year_str,
                resource_url,
                album,
                genre,
                confidence,
                has_track: false,
                artist_match: false,
                is_compilation,
            })
        };

        let try_search = |params: Vec<(&str, &str)>| -> String {
            let mut url = format!("{}/database/search", base_url);
            let mut first = true;
            for (k, v) in params {
                if first { url.push('?'); first = false; } else { url.push('&'); }
                url.push_str(&format!("{}={}", k, encode(v)));
            }
            if !token.is_empty() {
                if first { url.push('?'); } else { url.push('&'); }
                url.push_str(&format!("token={}", token));
            }
            url
        };

        // 1. Try "artist - release_title" general query 
        // First try "master" release type which usually has the original year
        let query = format!("{} - {}", clean_artist_str, clean_title_str);
        
        let mut data = match Self::perform_search(client, &try_search(vec![("type", "master"), ("q", &query)]), config).await {
            Ok(data) => data,
            Err(e) => return Err(e.into()),
        };

        // If no master implementation found, try "release"
        if data["results"].as_array().is_none_or(|arr| arr.is_empty()) {
             let url = try_search(vec![("type", "release"), ("q", &query)]);
             if let Ok(res) = Self::perform_search(client, &url, config).await {
                 data = res;
             }
        }

        // 2. Fallback: If no results, try just the title as a track search (if the artist is featured or diff name)
        if data["results"].as_array().is_none_or(|arr| arr.is_empty()) {
            log::debug!("No results for primary search, trying track search: {}", clean_title_str);
            let url = try_search(vec![("type", "release"), ("track", &clean_title_str), ("artist", &clean_artist_str)]);
            if let Ok(res) = Self::perform_search(client, &url, config).await {
                 data = res;
            }
        }

        // 3. Fallback: If no results, try simple query
        if data["results"].as_array().is_none_or(|arr| arr.is_empty()) {
            let simplified_query = format!("{} {}", clean_artist_str, clean_title_str);
            log::debug!("No results for track search, trying simple query: {}", simplified_query);
            let url = try_search(vec![("type", "release"), ("q", &simplified_query)]);
            if let Ok(res) = Self::perform_search(client, &url, config).await {
                 data = res;
            }
        }

        if data["results"].as_array().is_none_or(|arr| arr.is_empty()) {
            log::warn!("Discogs search returned no results for: {} - {}", artist, title);
            return Ok(None);
        }

        let mut candidates: Vec<Candidate> = Vec::new();

        if let Some(results) = data["results"].as_array() {
            // Check top 10 results to have a better chance of finding the original release
            for result in results.iter().take(10) { 
                // Skip detailed lookup to save requests unless absolutely necessary
                // Most basic info (year, album name) is in the search result
                if let Some(c) = build_candidate(result, None, &clean_title_str, &clean_artist_str) {
                    candidates.push(c);
                }
            }
        }
        
        // Score and sort candidates
        // Priority: 
        // 1. Not a compilation
        // 2. Earliest year
        // 3. Highest confidence (if ID exists)
        candidates.sort_by(|a, b| {
            // First compare compilation status (false is better than true)
            match a.is_compilation.cmp(&b.is_compilation) {
                std::cmp::Ordering::Equal => {
                    // Then compare years (earliest is better)
                    // If one has year and other doesn't, the one with year is better
                    let a_year = a.year.unwrap_or(9999);
                    let b_year = b.year.unwrap_or(9999);
                    match a_year.cmp(&b_year) {
                        std::cmp::Ordering::Equal => {
                            // Finally confidence
                            b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal) 
                        },
                        ord => ord,
                    }
                },
                ord => ord,
            }
        });

        if let Some(best) = candidates.first() {
            if best.confidence > 0.0 {
                // Try to fetch the release details for a more precise release date if we have a resource URL
                let mut date_str = best.year_str.clone().unwrap_or_default();
                if let Some(ref url) = best.resource_url {
                    if let Ok(details) = Self::fetch_discogs(client, url, config).await {
                        // Discogs often exposes 'released' (YYYY-MM-DD) or 'released_formatted'
                        if let Some(released) = details["released"].as_str().or_else(|| details["released_formatted"].as_str()) {
                            // Prefer the detailed released date when present
                            if !released.is_empty() {
                                date_str = released.to_string();
                            }
                        }
                    }
                }

                return Ok(Some((
                    date_str,
                    best.album.clone(),
                    best.genre.clone(),
                    best.confidence
                )));
            }
        }

        Ok(None)
    }

    async fn fetch_discogs(client: &reqwest::Client, url: &str, config: &MetadataConfig) -> Result<Value, String> {
        log::debug!("Fetching from Discogs: {}", url);
        
        // Add User-Agent (required by Discogs API)
        let mut request = client.get(url)
            .header("User-Agent", "MusicManager/1.0");
            
        // Add auth token if available
        if let Some(token) = config.discogs_token.as_ref().filter(|s| !s.is_empty()) {
            request = request.header("Authorization", format!("Discogs token={}", token));
        }

        let response = request.send().await.map_err(|e| e.to_string())?;

        let status = response.status();
        if status == 401 {
            let error_text = response.text().await.unwrap_or_default();
            log::error!("Discogs authentication failed ({}). Check your DISCOGS_TOKEN. Body: {}", status, error_text);
            return Err(format!("Discogs authentication failed ({}). Is DISCOGS_TOKEN set correctly?", status));
        }

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            log::warn!("Discogs API returned error status: {}. Body: {}", status, error_text);
            return Err(format!("Discogs API error: {}", status));
        }

        let json: Value = response
            .json()
            .await
            .map_err(|e| format!("Discogs JSON parse failed: {}", e))?;
        
        Ok(json)
    }

    #[allow(dead_code)]
    fn is_empty(json: &Value) -> bool {
        json.get("results")
            .and_then(|r| r.as_array())
            .map(|a| a.is_empty())
            .unwrap_or(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito;
    use serde_json::json;
    use crate::models::MetadataConfig;
    use uuid::Uuid;
    use chrono::Utc;
    use reqwest::Client;

    async fn setup_test() -> (mockito::ServerGuard, Client, MetadataConfig) {
        let server = mockito::Server::new_async().await;
        let client = Client::new();
        let config = MetadataConfig {
            id: Uuid::new_v4(),
            metadata_source: "discogs".to_string(),
            discogs_token: Some("test_token".to_string()),
            updated_at: Utc::now(),
        };
        (server, client, config)
    }

    #[tokio::test]
    async fn test_lookup_fallback_tiers() {
        let (mut server, client, config) = setup_test().await;
        let base_url = server.url();

        // 0. Mock Tier 0: Master search -> Empty
        let _tier0 = server.mock("GET", "/database/search")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("type".into(), "master".into()),
                mockito::Matcher::UrlEncoded("q".into(), "AC Slater - Bass Inside".into()),
                mockito::Matcher::UrlEncoded("token".into(), "test_token".into())
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json!({"results": []}).to_string())
            .create_async()
            .await;

        // 1. Mock Tier 1: Release title search -> Empty
        let _tier1 = server.mock("GET", "/database/search")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("type".into(), "release".into()),
                mockito::Matcher::UrlEncoded("q".into(), "AC Slater - Bass Inside".into()),
                mockito::Matcher::UrlEncoded("token".into(), "test_token".into())
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json!({"results": []}).to_string())
            .create_async()
            .await;

        // 2. Mock Tier 2: Track search -> Empty
        let _tier2 = server.mock("GET", "/database/search")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("type".into(), "release".into()),
                mockito::Matcher::UrlEncoded("track".into(), "Bass Inside".into()),
                mockito::Matcher::UrlEncoded("artist".into(), "AC Slater".into()),
                mockito::Matcher::UrlEncoded("token".into(), "test_token".into())
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json!({"results": []}).to_string())
            .create_async()
            .await;

        // 3. Mock Tier 3: Simple query search -> Success!
        let _tier3 = server.mock("GET", "/database/search")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("type".into(), "release".into()),
                mockito::Matcher::UrlEncoded("q".into(), "AC Slater Bass Inside".into()),
                mockito::Matcher::UrlEncoded("token".into(), "test_token".into())
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json!({
                "results": [{
                    "year": "2015",
                    "title": "AC Slater - Bass Inside EP",
                    "style": ["Bassline"],
                    "id": 12345
                }]
            }).to_string())
            .create_async()
            .await;

        let result = DiscogsService::lookup_with_base_url(&client, &config, "Bass Inside", "AC Slater", &base_url).await;
        
        assert!(result.is_ok(), "Request failed: {:?}", result.err());
        let (year, album, style, confidence) = result.unwrap().expect("Should find result");
        assert_eq!(year, "2015");
        // Update expectation to match mock response which is "AC Slater - Bass Inside EP"
        assert_eq!(album.unwrap(), "AC Slater - Bass Inside EP");
        assert_eq!(style.unwrap(), "Bassline");
        assert!(confidence > 0.0);
    }
}
