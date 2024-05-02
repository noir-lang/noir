mod contract_functions;
mod contracts;

use crate::{address::AztecAddress, grumpkin_point::GrumpkinPoint};

global MSG_SENDER = AztecAddress { inner: 27 };

global PUBLIC_KEY = GrumpkinPoint { x: 123456789, y: 123456789 };
