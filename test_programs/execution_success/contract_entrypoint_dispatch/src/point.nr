use crate::from_call_data::FromCallData;

#[derive(FromCallData)]
struct Point {
    x: Field,
    y: Field,
}

#[derive(FromCallData)]
struct Nothing {}
