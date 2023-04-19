use dep::std::compat;
use dep::std::ec::consts::te::baby_jubjub;
use dep::std::hash;
use dep::std::eddsa::eddsa_poseidon_verify;
use dep::std;

fn main(msg: pub Field, _priv_key_a: Field, _priv_key_b: Field) {
    // Skip this test for non-bn254 backends
    if compat::is_bn254() {
        // let bjj = baby_jubjub();

        // let pub_key_a = bjj.curve.mul(priv_key_a, bjj.curve.gen);
        let pub_key_a_x = 5958787406588418500595239545974275039455545059833263445973445578199987122248;
        let pub_key_a_y = 6291453822075498887551694851992571215511219854100590306020486222643399599966;
        // let pub_key_b = bjj.curve.mul(priv_key_b, bjj.curve.gen);
        // let pub_key_b_x = 19522885864221102065733517599414632594441624981666469001809339503157461222552;
        // let pub_key_b_y = 17652333933129211461240983013989988903130120700742832943186712581416334147739;

        // Manually computed as fields can't use modulo. Importantantly the commitment is within
        // the subgroup order. Note that choice of hash is flexible for this step.
        // let r_a = hash::pedersen([priv_key_a, msg])[0] % bjj.suborder; // modulus computed manually
        // let r_a = 161490627844314967434218537112044327531827991364156693670922373984031106453;
        // let r_b = hash::pedersen([priv_key_b, msg])[0] % bjj.suborder; // modulus computed manually
        // let r_b = 2365791308032549643506220161686150043336029213998559307390193647484103923348;

        // let r8_a = bjj.curve.mul(r_a, bjj.base8);
        let r8_a_x = 12701037151626467441858303530846040756656970729849074442899237454651294526343;
        let r8_a_y = 167159699056112791619442377873552194056131231335147805875057144366514712633;
        // let r8_b = bjj.curve.mul(r_b, bjj.base8);
        let r8_b_x = 1673844937671191161222451342456048355167330081367086809349241319413342408558;
        let r8_b_y = 6202873177260205770199785557298715091502981852957429177245493218486919053107;

        // let perm_a: [Field; 6] = hash::poseidon::bn254::perm::x5_6([
        //     0,
        //     r8_a.x,
        //     r8_a.y,
        //     pub_key_a.x,
        //     pub_key_a.y,
        //     msg,
        // ]);
        // let h_a = perm_a[0];
        // let h_a = 21371027913064755721489810138902574499031056361794273463577587174578008711057;

        // let perm_b: [Field; 6] = hash::poseidon::bn254::perm::x5_6([
        //     0,
        //     r8_b.x,
        //     r8_b.y,
        //     pub_key_b.x,
        //     pub_key_b.y,
        //     msg,
        // ]);
        // let h_b = perm_b[0];
        // let h_b = 15015879520414293281817167314697737785772315344690416596928480331452621620489;

        // let s_a = (r_a + priv_key_a * h_a) % bjj.suborder; // modulus computed manually
        let s_a = 960;
        // let s_b = (r_b + priv_key_b * h_b) % bjj.suborder; // modulus computed manually
        let s_b = 3562;

        // User A verifies their signature over the message
        constrain eddsa_poseidon_verify(pub_key_a_x, pub_key_a_y, s_a, r8_a_x, r8_a_y, msg);

        // User B's signature over the message can't be used with user A's pub key
        constrain !eddsa_poseidon_verify(pub_key_a_x, pub_key_a_y, s_b, r8_b_x, r8_b_y, msg);

        // User A's signature over the message can't be used with another message
        constrain !eddsa_poseidon_verify(pub_key_a_x, pub_key_a_y, s_a, r8_a_x, r8_a_y, msg + 1);
    }
}