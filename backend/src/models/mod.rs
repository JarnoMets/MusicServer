pub mod access_token;
pub mod app_state;
pub mod auto_download;
pub mod genre;
pub mod music;
pub mod playlist;
pub mod stream;
pub mod youtube;
pub mod audit;
pub mod user;
pub mod metadata;
pub mod metadata_suggestion;

pub use app_state::AppState;
pub use access_token::{
    AccessToken, AccessTokenResponse, CreateAccessTokenRequest, CreateAccessTokenResponse,
    UpdateAccessTokenRequest,
};
pub use auto_download::{AutoDownloadConfig, UpdateAutoDownloadConfigRequest, AutoDownloadStatus};
pub use metadata::MetadataConfig;
pub use genre::ArtistGenre;
pub use music::{
    ArtistSummary, BulkAddToPlaylistByRegexRequest, BulkAddToPlaylistResponse,
    BulkRenameByRegexRequest, BulkRenameResponse, BulkUpdateMusicRequest,
    CreateMusicFileRequest, MusicFile, MusicQueryParams, UpdateMusicFileRequest,
};
pub use playlist::{
    Playlist, PlaylistSummary, PlaylistTrackRequest, PlaylistWithItems, UpdatePlaylistRequest,
};
pub use stream::{CreateStreamRequest, InternetStream, UpdateStreamRequest};
pub use youtube::{
    CreateYoutubePlaylistRequest, UpdateYoutubePlaylistRequest, 
    YoutubeDownloadStats, YoutubePlaylist
};
pub use audit::CreateAuditLogRequest;
pub use user::{User, UserResponse, GoogleAuthRequest, GoogleMobileAuthRequest, AuthResponse, Claims};
