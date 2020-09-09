use crate::constants::{
    LIBRARY_RECENT_STORAGE_KEY, LIBRARY_STORAGE_KEY, PROFILE_STORAGE_KEY, SCHEMA_VERSION,
    SCHEMA_VERSION_STORAGE_KEY,
};
use crate::runtime::{EnvError, Environment};
use futures::future::Either;
use futures::{future, Future, TryFutureExt};

fn migrate_storage_schema_v1<Env: Environment + 'static>(
) -> impl Future<Output = Result<(), EnvError>> {
    future::try_join_all(vec![
        Env::set_storage(SCHEMA_VERSION_STORAGE_KEY, Some(&1)),
        Env::set_storage::<()>(PROFILE_STORAGE_KEY, None),
        Env::set_storage::<()>(LIBRARY_RECENT_STORAGE_KEY, None),
        Env::set_storage::<()>(LIBRARY_STORAGE_KEY, None),
    ])
    .map_ok(|_| ())
}

pub fn migrate_storage_schema<Env: Environment + 'static>(
) -> impl Future<Output = Result<(), EnvError>> {
    Env::get_storage::<usize>(SCHEMA_VERSION_STORAGE_KEY).and_then(|schema_version| {
        match schema_version {
            Some(schema_version) if schema_version > SCHEMA_VERSION => Either::Left(future::err(
                EnvError::StorageSchemaVersionDowngrade(schema_version, SCHEMA_VERSION),
            )),
            None => Either::Right(migrate_storage_schema_v1::<Env>()),
            // TODO Some(1) => Either::Right(migrate_storage_schema_v2::<Env>()),
            _ => Either::Left(future::ok(())),
        }
    })
}
