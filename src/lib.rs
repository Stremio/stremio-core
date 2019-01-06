extern crate reqwest;
use reqwest::{Result,get};
use serde_json::to_string;

pub mod types;
use self::types::*;

pub mod state_types;
use self::state_types::*;

fn get_cinemeta() -> reqwest::Result<CatalogResponse> {
    Ok(reqwest::get("https://v3-cinemeta.strem.io/catalog/movie/top.json")?.json()?)
}

fn get_watchhub() -> reqwest::Result<StreamResponse> {
    Ok(reqwest::get("https://watchhub-us.strem.io/stream/movie/tt0120338.json")?.json()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let mut container = StateContainer::with_reducer(&|state, action| {
            match action {
                Action::CatalogsReceived(Ok(resp)) => {
                    return Some(Box::new(State{
                        catalog: Loadable::Ready(ItemsView::Grouped(resp.metas))
                    }));
                },
                // @TODO
                Action::CatalogsReceived(Err(err)) => {
                    return Some(Box::new(State{
                        catalog: Loadable::Message(err.to_string())
                    }));
                },
                _ => {},
            };
            // Doesn't mutate
            None
        });
        container.dispatch(match get_cinemeta() {
            Ok(resp) => { Action::CatalogsReceived(Ok(resp)) },
            Err(err) => { Action::CatalogsReceived(Err("request error")) },
        });
        println!("{:?}", &container.state);
        //println!("{}", serde_json::to_string(&state).expect("rip"));
        //assert_eq!(2 + 2, 4);
    }
}
