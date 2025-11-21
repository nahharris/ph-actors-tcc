use std::fmt::{Display, Formatter, Error};

// Display é um trait usado para formatação de string com {}
impl Display for Coordinate {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Coordinate::Origin => write!(f, "Origin"),
            Coordinate::Cartesian(x, y) => write!(f, "({}, {})", x, y),
            Coordinate::Polar { theta, radius } => 
                write!(f, "{} ({})", radius, theta)
        }
    }
}

fn main() {
    let coord = Coordinate::Polar { radius: 10.0, theta: 75.0 };
    println!("A coordenada é {}", coord); // -> 10 (75)
}

