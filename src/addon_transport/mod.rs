//! Fetches addon manifests and resources through the platform's environment.
//!
//! Use [`Env::addon_transport`](crate::runtime::Env::addon_transport) to select a
//! transport from an addon's URL. The default implementation uses
//! [`AddonHTTPTransport`] for HTTP and HTTPS URLs and [`UnsupportedTransport`]
//! for other schemes. An environment can override this selection.
//!
//! # Example
//!
//! Build a request for a movie catalog that has no required extras. Pass the
//! addon's manifest URL and the catalog ID it advertises.
//!
//! ```
//! use stremio_core::{
//!     runtime::{Env, TryEnvFuture},
//!     types::addon::{ResourcePath, ResourceResponse},
//! };
//! use url::Url;
//!
//! fn fetch_movie_catalog<E: Env + 'static>(
//!     manifest_url: &Url,
//!     catalog_id: &str,
//! ) -> TryEnvFuture<ResourceResponse> {
//!     let path = ResourcePath::without_extra("catalog", "movie", catalog_id);
//!     E::addon_transport(manifest_url).resource(&path)
//! }
//! ```

mod http_transport;
pub use http_transport::*;

mod addon_transport;
pub use addon_transport::*;

mod unsupported_transport;
pub use unsupported_transport::*;
