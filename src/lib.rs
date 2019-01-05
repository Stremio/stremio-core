extern crate reqwest;
use reqwest::*;

pub mod types;
use self::types::*;

fn get_cinemeta() -> Result<MetaResponse> {
    Ok(reqwest::get("https://v3-cinemeta.strem.io/catalog/movie/top.json")?.json()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        //let resp = MetaResponse{ metas: vec![] };
        let resp = get_cinemeta();
        println!("{:?}", resp);
        assert_eq!(2 + 2, 4);
    }
}
