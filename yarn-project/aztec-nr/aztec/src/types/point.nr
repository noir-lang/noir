use crate::types::type_serialisation::TypeSerialisationInterface;

struct Point {
    x: Field,
    y: Field,
}

impl Point {
    fn new(x: Field, y: Field) -> Self {
        Point { x, y }
    } 
}

global POINT_SERIALISED_LEN: Field = 2;

fn deserialisePoint(fields: [Field; POINT_SERIALISED_LEN]) -> Point {
    Point {
        x: fields[0],
        y: fields[1],
    }
}

fn serialisePoint(point: Point) -> [Field; POINT_SERIALISED_LEN] {
    [point.x, point.y]
}

global PointSerialisationMethods = TypeSerialisationInterface {
    deserialise: deserialisePoint,
    serialise: serialisePoint,
};