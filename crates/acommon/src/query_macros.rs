#[macro_export]
macro_rules! query_fallback_get {
    ($query:expr $(, $entity:expr)+ $(,)?) => {{
        let mut result = None;
        $(
            if result.is_none() {
                result = match $entity {
                    Some(entity) => $query.get(entity).ok(),
                    None => None,
                };
            }
        )+
        result
    }};
}
