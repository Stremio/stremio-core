#[cfg(test)]
mod tests {
    // @TODO manifest with ManifestResource::Full
    // @TODO is_extra_supported
    use super::super::{Manifest, ManifestResource, ResourceRef};

    fn sample_manifest(
        resources: Vec<ManifestResource>,
        id_prefixes: Option<Vec<String>>,
    ) -> Manifest {
        Manifest {
            id: "org.test".into(),
            name: "test".into(),
            version: semver::Version::new(1, 0, 0),
            resources,
            types: vec!["movie".into()],
            catalogs: vec![],
            background: None,
            logo: None,
            id_prefixes,
            description: None,
        }
    }

    #[test]
    fn is_supported_short() {
        let manifest = sample_manifest(vec![ManifestResource::Short("stream".into())], None);
        assert_eq!(
            manifest.is_supported(&ResourceRef::without_extra("catalog", "movie", "something")),
            false
        );
        assert_eq!(
            manifest.is_supported(&ResourceRef::without_extra("stream", "series", "something")),
            false
        );
        assert_eq!(
            manifest.is_supported(&ResourceRef::without_extra("stream", "movie", "something")),
            true
        );
        let manifest = sample_manifest(
            vec![ManifestResource::Short("stream".into())],
            Some(vec!["tt".into(), "kek".into()]),
        );
        assert_eq!(
            manifest.is_supported(&ResourceRef::without_extra("stream", "movie", "something")),
            false
        );
        assert_eq!(
            manifest.is_supported(&ResourceRef::without_extra("stream", "movie", "tt2314")),
            true
        );
        assert_eq!(
            manifest.is_supported(&ResourceRef::without_extra("stream", "movie", "keksomet")),
            true
        );
    }

    #[test]
    fn is_supported_full() {
        let manifest = sample_manifest(
            vec![
                ManifestResource::Full {
                    name: "stream".into(),
                    types: Some(vec!["tv".into()]),
                    id_prefixes: Some(vec!["tt".into()]),
                },
                ManifestResource::Full {
                    name: "meta".into(),
                    types: Some(vec!["series".into()]),
                    id_prefixes: None,
                },
            ],
            None,
        );
        // respects the id_prefixes for the particular resource
        assert_eq!(
            manifest.is_supported(&ResourceRef::without_extra("stream", "tv", "2131")),
            false
        );
        // respects the types for the particular resource
        assert_eq!(
            manifest.is_supported(&ResourceRef::without_extra("stream", "movie", "tt0231")),
            false
        );
        assert_eq!(
            manifest.is_supported(&ResourceRef::without_extra("stream", "tv", "tt0231")),
            true
        );
        assert_eq!(
            manifest.is_supported(&ResourceRef::without_extra("meta", "movie", "something")),
            false
        );
        assert_eq!(
            manifest.is_supported(&ResourceRef::without_extra("meta", "series", "something")),
            true
        );
    }

}
