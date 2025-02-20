// This program will fail when all the inputs are true
#[fuzz]
fn fuzz(a_00:bool, a_01:bool, a_02:bool, a_03:bool, a_04:bool, a_05:bool, a_06:bool, a_07:bool, a_08:bool, a_09:bool, a_10:bool, a_11:bool, a_12:bool, a_13:bool, a_14:bool, a_15:bool, a_16:bool, a_17:bool, a_18:bool, a_19:bool, a_20:bool, a_21:bool, a_22:bool, a_23:bool, a_24:bool) {
    assert(!(a_00 & a_01 & a_02 & a_03 & a_04 & a_05 & a_06 & a_07 & a_08 & a_09 & a_10 & a_11 & a_12 & a_13 & a_14 & a_15 & a_16 & a_17 & a_18 & a_19 & a_20 & a_21 & a_22 & a_23 & a_24));
}
