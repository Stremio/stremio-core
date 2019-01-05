extern crate reqwest;
use reqwest::*;

pub mod types;
use self::types::*;

fn get_cinemeta() -> Result<MetaResponse> {
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
        println!("{:?} {:?}", get_cinemeta(), get_watchhub());
        assert_eq!(2 + 2, 4);
    }
}
