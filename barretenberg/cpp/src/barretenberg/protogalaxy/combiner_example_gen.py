# TODO(https://github.com/AztecProtocol/barretenberg/issues/760): Delete this?
import numpy as np

# np.set_printoptions(formatter={'int': hex})

EXTENDED_RELATION_LENGTH = 12

class Row:
    # Construct a set of 'all' polynomials with a very simple structure
    def __init__(self, base_poly):
        # Constuct polys by adding increasing factors of 2 to an input poly
        self.q_m = base_poly + 2 * 0
        self.q_c = base_poly + 2 * 1
        self.q_l = base_poly + 2 * 2
        self.q_r = base_poly + 2 * 3
        self.q_o = base_poly + 2 * 4
        self.q_4 = base_poly + 2 * 5
        self.q_arith = base_poly + 2 * 6
        self.q_delta_range = base_poly + 2 * 7
        self.q_elliptic = base_poly + 2 * 8
        self.q_aux = base_poly + 2 * 9
        self.q_lookup = base_poly + 2 * 10
        self.q_poseidon2_external_1 = base_poly + 2 * 11
        self.q_poseidon2_external_2 = base_poly + 2 * 12
        self.sigma_1 = base_poly + 2 * 13
        self.sigma_2 = base_poly + 2 * 14
        self.sigma_3 = base_poly + 2 * 15
        self.sigma_4 = base_poly + 2 * 16
        self.id_1 = base_poly + 2 * 17
        self.id_2 = base_poly + 2 * 18
        self.id_3 = base_poly + 2 * 19
        self.id_4 = base_poly + 2 * 20
        self.table_1 = base_poly + 2 * 21
        self.table_2 = base_poly + 2 * 22
        self.table_3 = base_poly + 2 * 23
        self.table_4 = base_poly + 2 * 24
        self.lagrange_first = base_poly + 2 * 25
        self.lagrange_last = base_poly + 2 * 26
        self.w_l = base_poly + 2 * 27
        self.w_r = base_poly + 2 * 28
        self.w_o = base_poly + 2 * 29
        self.w_4 = base_poly + 2 * 30
        self.sorted_accum = base_poly + 2 * 31
        self.z_perm = base_poly + 2 * 32
        self.z_lookup = base_poly + 2 * 33
        self.table_1_shift = base_poly + 2 * 34
        self.table_2_shift = base_poly + 2 * 35
        self.table_3_shift = base_poly + 2 * 36
        self.table_4_shift = base_poly + 2 * 37
        self.w_l_shift = base_poly + 2 * 38
        self.w_r_shift = base_poly + 2 * 39
        self.w_o_shift = base_poly + 2 * 40
        self.w_4_shift = base_poly + 2 * 41
        self.sorted_accum_shift = base_poly + 2 * 42
        self.z_perm_shift = base_poly + 2 * 43
        self.z_lookup_shift = base_poly + 2 * 44

    def arith_relation(self):
        return self.q_m * self.w_l * self.w_r + self.q_l * self.w_l + self.q_r * self.w_r + self.q_o * self.w_o + self.q_c

def extend_one_entity(input):
    result = input
    delta = input[1]-input[0]
    for _ in range(2, EXTENDED_RELATION_LENGTH):
        result.append(delta + result[-1])
    return result

def compute_first_example():
    # Construct baseline extensions for the two rows; extentions for all polys will be computed via the Row constructor
    baseline_extension_0 = np.array(extend_one_entity([0, 128]))
    baseline_extension_1 = baseline_extension_0 + 1

    # Construct extensions for all polys for the two rows in consideration
    row_0_extended = Row(baseline_extension_0)
    row_1_extended = Row(baseline_extension_1)

    accumulator = np.array([0 for _ in range(EXTENDED_RELATION_LENGTH)])
    zeta_pow = 1
    zeta = 2
    for row in [row_0_extended, row_1_extended]:
        accumulator += zeta_pow * row.arith_relation()
        zeta_pow *= zeta
    return accumulator


def compute_second_example():
    def arith_relation(w_l, w_r, w_o, q_m, q_l, q_r, q_o, q_c):
        return q_m * w_l * w_r + q_l * w_l + q_r * w_r + q_o * w_o + q_c

    result = 0
    #                0   1   2   3   4   5   6   7   8   9  10  11  12    
    w_l = np.array([ 1,  3,  5,  7,  9, 11, 13, 15, 17, 19, 21, 23, 25])
    w_r = np.array([ 2,  4,  6,  8, 10, 12, 14, 16, 18, 20, 22, 24, 26])
    w_o = np.array([ 3,  7, 11, 15, 19, 23, 27, 31, 35, 39, 43, 47, 51])
    q_m = np.array([ 0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0])
    q_l = np.array([ 1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1])
    q_r = np.array([ 1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1,  1])
    q_o = np.array([-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1])
    q_c = np.array([ 0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0])
    # contribution is zero, but why not?
    result += arith_relation(w_l, w_r, w_o, q_m, q_l, q_r, q_o, q_c)      

    w_l = np.array([ 0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12])
    w_r = np.array([ 4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4])
    w_o = np.array([ 4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4])
    q_m = np.array([ 0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12])
    q_l = np.array([ 1,  0, -1, -2, -3, -4, -5, -6, -7, -8, -9,-10,-11])
    q_r = np.array([ 1,  0, -1, -2, -3, -4, -5, -6, -7, -8, -9,-10,-11])
    q_o = np.array([-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1])
    q_c = np.array([ 0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0])

    result += arith_relation(w_l, w_r, w_o, q_m, q_l, q_r, q_o, q_c)
    result *= 2

    return result

if __name__ == "__main__":
    print(f"First example: \n  {compute_first_example()}")
    print(f"Second example:\n  {compute_second_example()}")

