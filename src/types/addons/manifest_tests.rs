#[cfg(test)]
mod tests {
    use super::super::{
        Manifest, ManifestCatalog, ManifestExtra, ManifestExtraProp, ManifestResource, ResourceRef,
    };

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
            contact_email: None,
            background: None,
            logo: None,
            id_prefixes,
            description: None,
        }
    }

    #[test]
    fn is_supported_short() {
        // without id_prefixes - should only match on resources and types
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
        // test id_prefixes
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

    fn get_catalog(extra: ManifestExtra) -> ManifestCatalog {
        ManifestCatalog {
            type_name: "movie".into(),
            id: "top".into(),
            name: None,
            extra,
        }
    }

    #[test]
    fn is_extra_supported_short() {
        let foo = [("foo".into(), "".into())];
        let bar = [("bar".into(), "".into())];
        let catalog = get_catalog(ManifestExtra::Short {
            required: vec![],
            supported: vec!["foo".into()],
        });
        // "foo" is optional - so it's ok not to pass it
        assert!(catalog.is_extra_supported(&[]));
        assert!(catalog.is_extra_supported(&foo));
        // but "bar" isn't even supported
        assert!(!catalog.is_extra_supported(&bar));
        let catalog = get_catalog(ManifestExtra::Short {
            required: vec!["foo".into()],
            supported: vec!["foo".into()],
        });
        // now we've made "foo" required
        assert!(!catalog.is_extra_supported(&[]));
        assert!(catalog.is_extra_supported(&foo));
    }

    #[test]
    fn is_extra_supported_full() {
        let foo = [("foo".into(), "".into())];
        let bar = [("bar".into(), "".into())];
        let catalog = get_catalog(ManifestExtra::Full {
            props: vec![ManifestExtraProp {
                name: "foo".into(),
                ..Default::default()
            }],
        });
        // "foo" is optional - so it's ok not to pass it
        assert!(catalog.is_extra_supported(&[]));
        assert!(catalog.is_extra_supported(&foo));
        // but "bar" isn't even supported
        assert!(!catalog.is_extra_supported(&bar));
        let catalog = get_catalog(ManifestExtra::Full {
            props: vec![ManifestExtraProp {
                name: "foo".into(),
                is_required: true,
                ..Default::default()
            }],
        });
        // now we've made "foo" required
        assert!(!catalog.is_extra_supported(&[]));
        assert!(catalog.is_extra_supported(&foo));
    }
}
