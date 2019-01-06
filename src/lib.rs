extern crate reqwest;
use reqwest::*;
use serde_json::to_string;

pub mod types;
use self::types::*;

pub mod state_types;
use self::state_types::*;

fn get_cinemeta() -> Result<CatalogResponse> {
    Ok(reqwest::get("https://v3-cinemeta.strem.io/catalog/movie/top.json")?.json()?)
}

fn get_watchhub() -> Result<StreamResponse> {
    Ok(reqwest::get("https://watchhub-us.strem.io/stream/movie/tt0120338.json")?.json()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        //let resp = MetaResponse{ metas: vec![] };
        let state = State{
            catalog: Loadable::Ready(ItemsView::Filtered(
                get_cinemeta().expect("rip").metas
            ))
        };
        //println!("{:?}", state);
        println!("{}", serde_json::to_string(&state).expect("rip"));
        assert_eq!(2 + 2, 4);
    }
}
