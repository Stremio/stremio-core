// reqwest = {version = "0.11", features = ["blocking", "json"]}
// mod addons {
//     use lazy_static::lazy_static;
//     use url::Url;

//     use crate::types::addon::Descriptor;

//     const LEGACY_REQUEST_PARAM: &str =
//         "q.json?b=eyJwYXJhbXMiOltdLCJtZXRob2QiOiJtZXRhIiwiaWQiOjEsImpzb25ycGMiOiIyLjAifQ==";

//     lazy_static! {
//         /// The local addon is protected as well as an official addon.
//         pub static ref LOCAL_ADDON_URL: Url = "http://127.0.0.1:11470/local-addon/manifest.json".parse().unwrap();

//         pub static ref PROTECTED_URLS: [Url; 2] = [
//             "https://v3-cinemeta.strem.io/manifest.json".parse().unwrap(),
//             LOCAL_ADDON_URL.clone(),
//         ];
//         pub static ref OFFICIAL_URLS: [Url; 6] = [
//             "https://v3-cinemeta.strem.io/manifest.json"
//                 .parse()
//                 .unwrap(),
//             "https://v3-channels.strem.io/manifest.json"
//                 .parse()
//                 .unwrap(),
//             "https://watchhub.strem.io/manifest.json".parse().unwrap(),
//             "https://caching.stremio.net/publicdomainmovies.now.sh/manifest.json"
//                 .parse()
//                 .unwrap(),
//             "https://opensubtitles.strem.io/stremio/v1/".parse().unwrap(),
//             LOCAL_ADDON_URL.clone(),
//         ];
//     }

//     // function getManifest(transportUrl) {
//     //     if (transportUrl === 'http://127.0.0.1:11470/local-addon/manifest.json') {
//     //         return Promise.resolve(localAddonManifest);
//     //     }

//     //     const legacy = transportUrl.endsWith('stremio/v1');
//     //     return fetch(legacy ? `${transportUrl}${LEGACY_REQUEST_PARAM}` : transportUrl)
//     //         .then((resp) => resp.json())
//     //         .then((resp) => legacy ? legacyManifestMapper.mapManifest(resp.result) : resp);
//     // }

//     pub enum Error {
//         Url(url::ParseError),
//         Request(reqwest::Error),
//     }

//     #[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
//     pub struct LegacyDescriptor {}

//     impl From<LegacyDescriptor> for Descriptor {
//         fn from(legacy: LegacyDescriptor) -> Self {
//             Descriptor {
//                 manifest: todo!(),
//                 transport_url: todo!(),
//                 flags: todo!(),
//             }
//         }
//     }

//     fn get_manifest(transport_url: Url) -> Result<Descriptor, Error> {
//         if transport_url == *LOCAL_ADDON_URL {
//             todo!("call localAddonManifest")
//         }

//         // todo: add a forward slash!
//         let is_legacy = transport_url.path().ends_with("stremio/v1");
//         let manifest = if is_legacy {
//             // will append the
//             let manifest_url = transport_url.join(LEGACY_REQUEST_PARAM).unwrap();

//             reqwest::blocking::get(manifest_url)
//                 .expect("Should fetch Manifest")
//                 .json::<LegacyDescriptor>()
//                 .expect("Should deserialize manifest")
//                 .into()
//         } else {
//             reqwest::blocking::get(transport_url)
//                 .expect("Should fetch Manifest")
//                 .json::<Descriptor>()
//                 .expect("Should deserialize manifest")
//         };

//         Ok(manifest)
//     }

//     pub fn official_addons() -> Vec<Descriptor> {
//         todo!()
//     }
// }
