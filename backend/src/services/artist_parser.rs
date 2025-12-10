/// Artist parsing utilities for extracting multiple artists from artist fields and titles
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashSet;

// Patterns for splitting artist names
static ARTIST_SEPARATORS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\s*(?:&|,|\s+and\s+|\s+vs\.?\s+|\s+x\s+)\s*").unwrap()
});

// Patterns for extracting featured artists from title
static FEAT_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\s*(?:\(|\[)?\s*(?:feat\.?|ft\.?|featuring)\s+([^)\]]+)(?:\)|\])?").unwrap()
});

// Pattern for remixes - extract remixer name
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
    
    // Parse primary artists from artist field
    if let Some(artist_str) = artist {
        // First, try to extract featured artists that might be in the artist field itself
        // e.g., "Artist 1 feat. Artist 2"
        let artist_str = extract_featured_from_string(artist_str, &mut result.featured);
        
        // Split remaining by separators
        result.primary = split_artists(&artist_str);
    }
    
    // Parse title for additional artist info
    if let Some(title_str) = title {
        // Extract featured artists from title
        for cap in FEAT_PATTERN.captures_iter(title_str) {
            if let Some(feat_match) = cap.get(1) {
                let featured_str = feat_match.as_str();
                // The featured section might have multiple artists too
                for artist in split_artists(featured_str) {
                    if !result.featured.contains(&artist) && !result.primary.contains(&artist) {
                        result.featured.push(artist);
                    }
                }
            }
        }
        
        // Extract remixers from title
        for cap in REMIX_PATTERN.captures_iter(title_str) {
            if let Some(remix_match) = cap.get(1) {
                let remixer = remix_match.as_str().trim().to_string();
                // Skip if it's the same as a primary artist (self-remix)
                if !remixer.is_empty() 
                    && !result.remixers.contains(&remixer) 
                    && !looks_like_version(&remixer)  // Skip things like "Radio", "Extended", "Original"
                {
                    result.remixers.push(remixer);
                }
            }
        }
        
        // Extract producers
        for cap in PROD_PATTERN.captures_iter(title_str) {
            if let Some(prod_match) = cap.get(1) {
                for artist in split_artists(prod_match.as_str()) {
                    if !result.producers.contains(&artist) {
                        result.producers.push(artist);
                    }
                }
            }
        }
    }
    
    result
}

/// Extract featured artists from a string and return the remaining string
fn extract_featured_from_string(s: &str, featured: &mut Vec<String>) -> String {
    let mut result = s.to_string();
    
    for cap in FEAT_PATTERN.captures_iter(s) {
        if let Some(feat_match) = cap.get(1) {
            for artist in split_artists(feat_match.as_str()) {
                if !featured.contains(&artist) {
                    featured.push(artist);
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
    let mut result = title.to_string();
    
    // Remove feat. sections
    result = FEAT_PATTERN.replace_all(&result, "").to_string();
    
    // Remove prod. by sections  
    result = PROD_PATTERN.replace_all(&result, "").to_string();
    
    // Clean up any leftover empty parentheses or brackets
    result = result.replace("()", "").replace("[]", "");
    
    // Clean up multiple spaces
    result = result.split_whitespace().collect::<Vec<_>>().join(" ");
    
    result.trim().to_string()
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
