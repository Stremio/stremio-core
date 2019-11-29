use crate::state_types::models::common::{addon_get, Loadable};
use crate::state_types::{Effects, EnvError, Environment};
use crate::types::addons::{AggrRequest, Descriptor, ResourceRequest, ResourceResponse};
use serde_derive::Serialize;
use std::convert::TryFrom;
use std::marker::PhantomData;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", content = "content")]
pub enum CatalogError {
    EmptyContent,
    UnexpectedResp,
    Other(String),
}

pub type CatalogContent<T> = Loadable<T, CatalogError>;

#[derive(Debug, Clone, Serialize)]
pub struct Catalog<T> {
    pub request: ResourceRequest,
    pub content: CatalogContent<T>,
}

pub enum CatalogAction<'a, T, Env: Environment + 'static> {
    CatalogRequested {
        request: &'a ResourceRequest,
        env: PhantomData<Env>,
    },
    CatalogReplaced {
        catalog: Catalog<T>,
    },
    CatalogResponseReceived {
        request: &'a ResourceRequest,
        response: &'a Result<ResourceResponse, EnvError>,
        limit: Option<usize>,
    },
}

pub fn catalog_update<T, Env>(catalog: &mut Catalog<T>, action: CatalogAction<T, Env>) -> Effects
where
    T: Clone + TryFrom<ResourceResponse>,
    Env: Environment + 'static,
{
    match action {
        CatalogAction::CatalogRequested { request, .. } => {
            if request.ne(&catalog.request) {
                *catalog = Catalog {
                    request: request.to_owned(),
                    content: CatalogContent::Loading,
                };
                Effects::one(addon_get::<Env>(request.to_owned()))
            } else {
                Effects::none().unchanged()
            }
        }
        CatalogAction::CatalogReplaced {
            catalog: next_catalog,
        } => {
            if next_catalog.request.ne(&catalog.request) {
                *catalog = next_catalog;
                Effects::none()
            } else {
                Effects::none().unchanged()
            }
        }
        CatalogAction::CatalogResponseReceived {
            request, response, ..
        } => {
            if request.eq(&catalog.request) {
                catalog.content = match response {
                    Ok(response) => match T::try_from(response.to_owned()) {
                        Ok(content) => CatalogContent::Ready(content),
                        Err(_) => CatalogContent::Err(CatalogError::UnexpectedResp),
                    },
                    Err(error) => CatalogContent::Err(CatalogError::Other(error.to_string())),
                };
                Effects::none()
            } else {
                Effects::none().unchanged()
            }
        }
    }
}

pub fn catalog_update_with_vector_content<T, Env>(
    catalog: &mut Catalog<Vec<T>>,
    action: CatalogAction<Vec<T>, Env>,
) -> Effects
where
    T: Clone,
    Vec<T>: TryFrom<ResourceResponse>,
    Env: Environment + 'static,
{
    match action {
        CatalogAction::CatalogResponseReceived {
            request,
            response,
            limit,
        } => {
            if request.eq(&catalog.request) {
                catalog.content = match response {
                    Ok(response) => match <Vec<T>>::try_from(response.to_owned()) {
                        Ok(ref content) if content.is_empty() => {
                            CatalogContent::Err(CatalogError::EmptyContent)
                        }
                        Ok(content) => {
                            if let Some(limit) = limit {
                                CatalogContent::Ready(content.into_iter().take(limit).collect())
                            } else {
                                CatalogContent::Ready(content)
                            }
                        }
                        Err(_) => CatalogContent::Err(CatalogError::UnexpectedResp),
                    },
                    Err(error) => CatalogContent::Err(CatalogError::Other(error.to_string())),
                };
                Effects::none()
            } else {
                Effects::none().unchanged()
            }
        }
        _ => catalog_update::<_, Env>(catalog, action),
    }
}

pub enum CatalogsAction<'a, T, Env: Environment + 'static> {
    CatalogsRequested {
        addons: &'a [Descriptor],
        request: &'a AggrRequest<'a>,
        env: PhantomData<Env>,
    },
    CatalogsReplaced {
        catalogs: Vec<Catalog<T>>,
    },
    CatalogResponseReceived {
        request: &'a ResourceRequest,
        response: &'a Result<ResourceResponse, EnvError>,
        limit: Option<usize>,
    },
}

pub fn catalogs_update<T, Env>(
    catalogs: &mut Vec<Catalog<T>>,
    action: CatalogsAction<T, Env>,
) -> Effects
where
    T: Clone + TryFrom<ResourceResponse>,
    Env: Environment + 'static,
{
    match action {
        CatalogsAction::CatalogsRequested {
            addons, request, ..
        } => {
            let requests = request
                .plan(&addons)
                .into_iter()
                .map(|(_, request)| request)
                .collect::<Vec<ResourceRequest>>();
            if requests
                .iter()
                .ne(catalogs.iter().map(|catalog| &catalog.request))
            {
                let (next_catalogs, effects) = requests
                    .iter()
                    .map(|request| {
                        (
                            Catalog {
                                request: request.to_owned(),
                                content: CatalogContent::Loading,
                            },
                            addon_get::<Env>(request.to_owned()),
                        )
                    })
                    .unzip::<_, _, Vec<_>, Vec<_>>();
                *catalogs = next_catalogs;
                Effects::many(effects)
            } else {
                Effects::none().unchanged()
            }
        }
        CatalogsAction::CatalogsReplaced {
            catalogs: next_catalogs,
        } => {
            if next_catalogs
                .iter()
                .map(|catalog| &catalog.request)
                .ne(catalogs.iter().map(|catalog| &catalog.request))
            {
                *catalogs = next_catalogs;
                Effects::none()
            } else {
                Effects::none().unchanged()
            }
        }
        CatalogsAction::CatalogResponseReceived {
            request,
            response,
            limit,
        } => {
            let catalog_index = catalogs
                .iter()
                .position(|catalog| catalog.request.eq(request));
            if let Some(catalog_index) = catalog_index {
                catalog_update::<_, Env>(
                    &mut catalogs[catalog_index],
                    CatalogAction::CatalogResponseReceived {
                        request,
                        response,
                        limit,
                    },
                )
            } else {
                Effects::none().unchanged()
            }
        }
    }
}

pub fn catalogs_update_with_vector_content<T, Env>(
    catalogs: &mut Vec<Catalog<Vec<T>>>,
    action: CatalogsAction<Vec<T>, Env>,
) -> Effects
where
    T: Clone,
    Vec<T>: TryFrom<ResourceResponse>,
    Env: Environment + 'static,
{
    match action {
        CatalogsAction::CatalogResponseReceived {
            request,
            response,
            limit,
        } => {
            let catalog_index = catalogs
                .iter()
                .position(|catalog| catalog.request.eq(request));
            if let Some(catalog_index) = catalog_index {
                catalog_update_with_vector_content::<_, Env>(
                    &mut catalogs[catalog_index],
                    CatalogAction::CatalogResponseReceived {
                        request,
                        response,
                        limit,
                    },
                )
            } else {
                Effects::none().unchanged()
            }
        }
        _ => catalogs_update::<_, Env>(catalogs, action),
    }
}
