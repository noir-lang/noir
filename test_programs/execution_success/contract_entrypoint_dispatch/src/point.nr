use crate::from_call_data::FromCallData;
use crate::to_call_data::ToCallData;

#[derive(FromCallData, ToCallData)]
struct Point {
    x: Field,
    y: Field,
}

#[derive(FromCallData, ToCallData)]
struct GenericPoint<T> {
    x: T,
    y: T,
}

#[derive(FromCallData, ToCallData)]
struct ThreePoints {
    points: [Point; 3]
}

#[derive(FromCallData, ToCallData)]
struct Nothing {}
