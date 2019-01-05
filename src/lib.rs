mod types;
use types::meta_item::MetaItem;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let meta = MetaItem{ id: "test".to_string(), item_type: "series".to_string(), name: "foobar".to_string() };
        println!("{:?}", meta);
        assert_eq!(2 + 2, 4);
    }
}
