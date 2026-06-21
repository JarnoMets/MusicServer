// Shared filename parsing utilities.
//
// Multiple parts of the codebase (uploads, folder sync, background sync)
// need to extract artist / title from a filename like "Artist - Title.mp3".
// This module provides a single implementation.
///
/// Parse a filename into (optional artist, title) by splitting on " - "
/// and stripping the file extension.
pub fn parse_filename(file_name: &str) -> (Option<String>, String) {
    if let Some(pos) = file_name.find(" - ") {
        let artist = file_name[..pos].trim();
        let mut title = &file_name[(pos + 3)..];
        // strip extension
        if let Some(dot) = title.rfind('.') {
            title = &title[..dot];
        }
        (Some(artist.to_string()), title.trim().to_string())
    } else {
        let mut title = file_name;
        if let Some(dot) = title.rfind('.') {
            title = &title[..dot];
        }
        (None, title.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_artist_and_title() {
        let (artist, title) = parse_filename("Some Artist - Cool Song.mp3");
        assert_eq!(artist.as_deref(), Some("Some Artist"));
        assert_eq!(title, "Cool Song");
    }

    #[test]
    fn test_no_artist() {
        let (artist, title) = parse_filename("JustATitle.flac");
        assert!(artist.is_none());
        assert_eq!(title, "JustATitle");
    }

    #[test]
    fn test_no_extension() {
        let (artist, title) = parse_filename("Artist - Title");
        assert_eq!(artist.as_deref(), Some("Artist"));
        assert_eq!(title, "Title");
    }

    #[test]
    fn test_multiple_dashes_uses_first_separator() {
        let (artist, title) = parse_filename("Artist - Album - Track Name.mp3");
        assert_eq!(artist.as_deref(), Some("Artist"));
        assert_eq!(title, "Album - Track Name");
    }

    #[test]
    fn test_trims_whitespace() {
        let (artist, title) = parse_filename("  Artist  -  Title  .wav");
        assert_eq!(artist.as_deref(), Some("Artist"));
        assert_eq!(title, "Title");
    }
}
