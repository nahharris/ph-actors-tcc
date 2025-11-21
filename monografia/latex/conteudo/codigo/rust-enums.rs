enum Coordinate {
    Polar { 
        theta: f32, 
        radius: f32
    },
    Cartesian(f32, f32),
    Origin,
}

fn main() {
    let origin: Coordinate = Coordinate::Origin;
    let point1: Coordinate = Coordinate::Cartesian(1.0, 1.0);
    let point2: Coordinate = Coordinate::Polar { theta: 45.0, radius: 2.0 };
}

