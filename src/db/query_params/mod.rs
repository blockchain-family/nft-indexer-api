pub mod collection;
pub mod nft;

pub enum OrderDirection {
    Asc,
    Desc,
}

impl std::fmt::Display for OrderDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderDirection::Asc => write!(f, "asc"),
            OrderDirection::Desc => write!(f, "desc"),
        }
    }
}

pub struct Ordering<T> {
    pub direction: OrderDirection,
    pub field: T,
}

impl<T, U: From<T>> From<crate::handlers::requests::Ordering<T>> for Ordering<U> {
    fn from(value: crate::handlers::requests::Ordering<T>) -> Self {
        Self {
            direction: match value.direction {
                crate::handlers::requests::OrderDirection::Asc => OrderDirection::Asc,
                crate::handlers::requests::OrderDirection::Desc => OrderDirection::Desc,
            },
            field: value.field.into(),
        }
    }
}
