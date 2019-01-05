extern crate reqwest;
use reqwest::*;

pub mod types;
use types::*;

//fn get_cinemeta() -> Result<MetaResponse> {
//    reqwest::get("https://v3-cinemeta.strem.io/catalog/movies/top.json")?.json()?
//}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let resp = MetaResponse{ metas: vec![] };
        println!("{:?}", resp);
        assert_eq!(2 + 2, 4);
    }
}
