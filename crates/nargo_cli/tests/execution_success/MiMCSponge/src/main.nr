// MiMCSponge circuit takes N many input and outputs M many output where N and M is
// arbitrary choosen and each item of M and N is bn254 element. 

use dep::std;

fn main(){
    //sponge test
    let plaintext: [Field;2] = [1,2];
    let k: Field = 3;
    let res: [Field;3]  = std::hash::mimc::MiMCSponge(plaintext,k);

    assert(res[0] == 20225509322021146255705869525264566735642015554514977326536820959638320229084);
    assert(res[1] == 13871743498877225461925335509899475799121918157213219438898506786048812913771);
    assert(res[2] == 21633608428713573518356618235457250173701815120501233429160399974209848779097);

}