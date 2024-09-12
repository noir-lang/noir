use crate::from_call_data::FromCallData;

#[derive(FromCallData)]
struct Point {
    x: Field,
    y: Field,
}

#[derive(FromCallData)]
struct GenericPoint<T> {
    x: T,
    y: T,
}

#[derive(FromCallData)]
struct Nothing {}
