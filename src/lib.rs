pub mod types;
use self::types::*;

pub mod state_types;
use self::state_types::*;
#[cfg(test)]
mod tests {
    use serde_json::to_string;
    use reqwest::{Result,get};
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
        // @TODO figure out how to do middlewares/reducers pipeline
        container.dispatch(match get_cinemeta() {
            Ok(resp) => { Action::CatalogsReceived(Ok(resp)) },
            Err(err) => { Action::CatalogsReceived(Err("request error")) },
        });
        assert_eq!(
            match &container.get_state().catalog {
                Loadable::Ready(ItemsView::Grouped(x)) => x.len(),
                _ => 0,
            },
            100,
        );
        //println!("{:?}", container.get_state());
        //println!("{}", serde_json::to_string(&state).expect("rip"));
        //assert_eq!(2 + 2, 4);
    }


    fn get_cinemeta() -> reqwest::Result<CatalogResponse> {
        Ok(reqwest::get("https://v3-cinemeta.strem.io/catalog/movie/top.json")?.json()?)
    }

    fn get_watchhub() -> reqwest::Result<StreamResponse> {
        Ok(reqwest::get("https://watchhub-us.strem.io/stream/movie/tt0120338.json")?.json()?)
    }
}
