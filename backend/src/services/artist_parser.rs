/// Artist parsing utilities for extracting multiple artists from artist fields and titles
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashSet;

static ARTIST_SEPARATORS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\s*(?:&|,|\s+and\s+|\s+vs\.?\s+|\s+x\s+)\s*").unwrap()
});

// Patterns for extracting featured artists from title
static FEAT_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\s*(?:\(|\[)?\s*(?:feat\.?|ft\.?|featuring)\s+([^)\]]+)(?:\)|\])?").unwrap()
});

// Pattern for remixes - extract remixer name
#[allow(dead_code)]
static REMIX_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(?:\(|\[)?\s*([^()\[\]]+?)\s+(?:remix|rmx|bootleg|edit|rework|flip|vip)\s*(?:\)|\])?").unwrap()
});

// Pattern for "prod. by" or "produced by"
static PROD_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\s*(?:\(|\[)?\s*(?:prod\.?\s*(?:by)?|produced\s+by)\s+([^)\]]+)(?:\)|\])?").unwrap()
});

/// Result of parsing artist information
#[derive(Debug, Clone)]
pub struct ParsedArtists {
    /// Main/primary artists
    pub primary: Vec<String>,
    /// Featured artists (from "feat.", "ft.", etc.)
    pub featured: Vec<String>,
    /// Remixers/producers
    pub remixers: Vec<String>,
    /// Producers (from "prod. by")
    pub producers: Vec<String>,
}

impl ParsedArtists {
    pub fn new() -> Self {
        Self {
            primary: Vec::new(),
            featured: Vec::new(),
            remixers: Vec::new(),
            producers: Vec::new(),
        }
    }
    
    /// Get all unique artists
    pub fn all_artists(&self) -> Vec<String> {
        let mut all: HashSet<String> = HashSet::new();
        
        for artist in &self.primary {
            all.insert(normalize_artist_name(artist));
        }
        for artist in &self.featured {
            all.insert(normalize_artist_name(artist));
        }
        for artist in &self.remixers {
            all.insert(normalize_artist_name(artist));
        }
        for artist in &self.producers {
            all.insert(normalize_artist_name(artist));
        }
        
        all.into_iter().collect()
    }
}

/// Normalize an artist name (trim, consistent casing for comparison)
fn normalize_artist_name(name: &str) -> String {
    name.trim().to_string()
}

/// Split a potentially compound artist string into individual artists
#[allow(dead_code)]
fn split_artists(artist_str: &str) -> Vec<String> {
    ARTIST_SEPARATORS
        .split(artist_str)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Parse all artists from an artist field and title
pub fn parse_artists(artist: Option<&str>, title: Option<&str>) -> ParsedArtists {
    let mut result = ParsedArtists::new();
    
    // Parse artist field first
    if let Some(artist_str) = artist {
        // Extract featured artists from string
        let mut featured_vec = Vec::new();
        let remaining = extract_featured_from_string(artist_str, &mut featured_vec);
        
        for feat in featured_vec {
             result.featured.push(feat);
        }
        
        // Split remaining by separators
        for artist in ARTIST_SEPARATORS.split(&remaining) {
            let cleaned = clean_artist(artist);
            if !cleaned.is_empty() {
                result.primary.push(cleaned);
            }
        }
    }
    
    // Parse title for additional artist info
    if let Some(title_str) = title {
        let mut featured_vec = Vec::new();
        extract_featured_from_string(title_str, &mut featured_vec);
        
        for feat in featured_vec {
            if !result.featured.contains(&feat) && !result.primary.contains(&feat) {
                 result.featured.push(feat);
            }
        }
    }
    
    result
}

/// Extract featured artists from a string and return the remaining string
fn extract_featured_from_string(s: &str, featured: &mut Vec<String>) -> String {
    let mut result = s.to_string();
    
    // Use captures_iter to find all matches
    for cap in FEAT_PATTERN.captures_iter(s) {
        if let Some(feat_match) = cap.get(1) {
            // Processing featured artists string
            let feat_str = feat_match.as_str();
            
            // Recursively split featured artists too
            for artist in ARTIST_SEPARATORS.split(feat_str) {
                 let cleaned = clean_artist(artist);
                 if !featured.contains(&cleaned) && !cleaned.is_empty() {
                     featured.push(cleaned);
                 }
            }
        }
        // Remove the matched portion from result
        if let Some(full_match) = cap.get(0) {
            result = result.replace(full_match.as_str(), "");
        }
    }
    
    result.trim().to_string()
}

/// Check if a string looks like a version descriptor rather than an artist name
#[allow(dead_code)]
fn looks_like_version(s: &str) -> bool {
    let lower = s.to_lowercase();
    let version_words = [
        "radio", "extended", "original", "club", "dub", "instrumental", 
        "vocal", "acoustic", "live", "demo", "clean", "explicit",
        "mix", "version", "remaster", "remastered", "single"
    ];
    
    version_words.iter().any(|w| lower.contains(w))
}

/// Clean a title by removing artist information that's embedded in it
/// This removes feat., remix attribution, etc. to get a cleaner title
pub fn clean_title(title: &str) -> String {
    let mut cleaned = title.to_string();
    
    // Remove feat. sections
    cleaned = FEAT_PATTERN.replace_all(&cleaned, "").to_string();
    
    // Remove prod. by sections  
    cleaned = PROD_PATTERN.replace_all(&cleaned, "").to_string();
    
    // Clean up any leftover empty parentheses or brackets
    cleaned = cleaned.replace("()", "").replace("[]", "");
    
    // Clean up multiple spaces
    cleaned = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    
    cleaned.trim().to_string()
}

pub fn clean_artist(artist: &str) -> String {
    let mut cleaned = artist.to_string();
    
    // Remove common suffixes like " - Topic"
    if cleaned.ends_with(" - Topic") {
        cleaned = cleaned.replace(" - Topic", "");
    }
    
    // Remove "VEVO"
    if cleaned.ends_with("VEVO") {
        cleaned = cleaned.replace("VEVO", "");
    }
    
    cleaned.trim().to_string()
}

/// Parse artist string into canonical artist name and list of featured artists
#[allow(dead_code)]
pub fn parse_artist_string(artist_str: &str) -> (String, Vec<String>) {
    let mut primary_artists = Vec::new();
    let mut featured_artists = Vec::new();
    
    // Split on separators (feat., vs., &, etc)
    let artists: Vec<&str> = ARTIST_SEPARATORS.split(artist_str).collect();
    
    // The first part is the primary artist
    if !artists.is_empty() {
        let first = artists[0].trim();
        if !first.is_empty() {
             primary_artists.push(clean_artist(first));
        }
    }
    
    // Other parts might contain featured artists or other primary artists
    for artist in artists.iter().skip(1) {
        let trimmed = artist.trim();
        if trimmed.is_empty() { continue; }

        let mut featured_vec = Vec::new();
        // Extract featured artists using the feat. pattern
        let remaining = extract_featured_from_string(trimmed, &mut featured_vec);
        
        // Add extracted featured artists
        for feat in featured_vec {
             featured_artists.push(feat);
        }
        
        // If anything remains after extracting feat., it's another primary artist
        if !remaining.trim().is_empty() {
             // Check if it's "Remix" or similar non-artist string
             if !REMIX_PATTERN.is_match(&remaining) {
                 primary_artists.push(clean_artist(&remaining));
             }
        }
    }
    
    (primary_artists.join(" & "), featured_artists)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_artists() {
        assert_eq!(split_artists("Artist 1 & Artist 2"), vec!["Artist 1", "Artist 2"]);
        assert_eq!(split_artists("A, B, C"), vec!["A", "B", "C"]);
        assert_eq!(split_artists("DJ X x Producer Y"), vec!["DJ X", "Producer Y"]);
    }

    #[test]
    fn test_parse_featured() {
        let result = parse_artists(Some("Main Artist"), Some("Song Title feat. Guest"));
        assert_eq!(result.primary, vec!["Main Artist"]);
        assert_eq!(result.featured, vec!["Guest"]);
    }

    #[test]
    fn test_parse_remix() {
        let result = parse_artists(Some("Original Artist"), Some("Track Name (DJ Remix)"));
        assert_eq!(result.primary, vec!["Original Artist"]);
        assert_eq!(result.remixers, vec!["DJ"]);
    }

    #[test]
    fn test_compound_artist() {
        let result = parse_artists(Some("Shy FX & UK Apache"), Some("Original Nuttah"));
        assert_eq!(result.primary, vec!["Shy FX", "UK Apache"]);
    }
    
    #[test]
    fn test_macky_gee_remix() {
        // Test case: "Levela - Skatta ( Macky Gee Remix ) [JUMP UP]"
        let result = parse_artists(Some("Levela"), Some("Skatta ( Macky Gee Remix ) [JUMP UP]"));
        assert_eq!(result.primary, vec!["Levela"]);
        assert!(result.remixers.contains(&"Macky Gee".to_string()), "Remixers: {:?}", result.remixers);
    }
}
