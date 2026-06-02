fn main(
    device_secret: Field,
    usb_serial: pub Field,
    commitment: pub Field,
    challenge: pub Field,
    user_id_hash: pub Field,
) -> pub Field {
    // The proof is bound to the hardware serial because it is a public input.
    // We can also incorporate it into the commitment check for stronger binding if needed.
    let computed_commitment = device_secret * device_secret + user_id_hash;
    assert(computed_commitment == commitment);

    device_secret * challenge + user_id_hash + usb_serial
}
