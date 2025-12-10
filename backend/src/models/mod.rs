pub mod app_state;
pub mod auto_download;
pub mod genre;
pub mod music;
pub mod playlist;
pub mod stream;
pub mod youtube;

pub use app_state::AppState;
pub use auto_download::{AutoDownloadConfig, UpdateAutoDownloadConfigRequest};
pub use genre::{ArtistGenre, DetectGenreRequest, DetectGenreResponse};
pub use music::{ArtistSummary, CreateMusicFileRequest, MusicFile, MusicQueryParams, UpdateMusicFileRequest};
pub use playlist::{
    CreatePlaylistRequest, Playlist, PlaylistSummary, PlaylistTrackRequest, PlaylistWithItems, UpdatePlaylistRequest,
};
pub use stream::{CreateStreamRequest, InternetStream, UpdateStreamRequest};
pub use youtube::{
    CreateYoutubePlaylistRequest, UpdateYoutubePlaylistRequest, YoutubeDownloadRequest, 
    YoutubeDownloadResponse, YoutubeDownloadStats, YoutubePlaylist
};
