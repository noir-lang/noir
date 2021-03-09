// XXX: We could alleviate a runtime check from noir
// By casting directly
// Example:
// priv z1 =  x as u32 
// priv z2 =  x as u16 
//
// The IR would see both casts and replace it with
// 
// 
// priv z1 = x as u16;
// priv z2 = x as u16;
//
// 
// Then maybe another optimisation could be done so that it transforms into
//
// priv z1 = x as u16
// priv z2 = z1
// This is what I would call a general optimisation, so it could live inside of the IR module
// A more specific optimisation would be to have z2 = z1 not use a gate (copy_from_to), this is more specific to plonk-aztec and would not live in this module