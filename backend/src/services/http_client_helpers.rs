// Shared HTTP client utilities.
//
// Provides a reusable pattern for "use provided client or create a temporary one"
// which was duplicated 4× in `genre_detection.rs`.
///
/// Either borrows an existing client or creates a new one.
/// Returns a reference that can be used for the rest of the scope.
pub fn get_or_create_client(client: Option<&reqwest::Client>) -> Result<std::borrow::Cow<'_, reqwest::Client>, String> {
    match client {
        Some(c) => Ok(std::borrow::Cow::Borrowed(c)),
        None => {
            let c = reqwest::Client::builder()
                .user_agent("MusicManager/1.0.0 (jarno.mets@gmail.com)")
                .build()
                .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
            Ok(std::borrow::Cow::Owned(c))
        }
    }
}
