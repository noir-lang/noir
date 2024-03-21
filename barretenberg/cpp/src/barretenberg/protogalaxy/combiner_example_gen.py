# TODO(https://github.com/AztecProtocol/barretenberg/issues/760): Delete this?
import numpy as np

# np.set_printoptions(formatter={'int': hex})

EXTENDED_RELATION_LENGTH = 13

class Row:
    def __init__(self, start):

        self.q_c = start + 2 * 0
        self.q_l = start + 2 * 1
        self.q_r = start + 2 * 2
        self.q_o = start + 2 * 3
        self.q_4 = start + 2 * 4
        self.q_m = start + 2 * 5
        self.q_arith = start + 2 * 6
        self.q_delta_range = start + 2 * 7
        self.q_elliptic = start + 2 * 8
        self.q_aux = start + 2 * 9
        self.q_lookup = start + 2 * 10
        self.sigma_1 = start + 2 * 11
        self.sigma_2 = start + 2 * 12
        self.sigma_3 = start + 2 * 13
        self.sigma_4 = start + 2 * 14
        self.id_1 = start + 2 * 15
        self.id_2 = start + 2 * 16
        self.id_3 = start + 2 * 17
        self.id_4 = start + 2 * 18
        self.table_1 = start + 2 * 19
        self.table_2 = start + 2 * 20
        self.table_3 = start + 2 * 21
        self.table_4 = start + 2 * 22
        self.lagrange_first = start + 2 * 23
        self.lagrange_last = start + 2 * 24
        self.w_l = start + 2 * 25
        self.w_r = start + 2 * 26
        self.w_o = start + 2 * 27
        self.w_4 = start + 2 * 28
        self.sorted_accum = start + 2 * 29
        self.z_perm = start + 2 * 30
        self.z_lookup = start + 2 * 31
        self.table_1_shift = start + 2 * 32
        self.table_2_shift = start + 2 * 33
        self.table_3_shift = start + 2 * 34
        self.table_4_shift = start + 2 * 35
        self.w_l_shift = start + 2 * 36
        self.w_r_shift = start + 2 * 37
        self.w_o_shift = start + 2 * 38
        self.w_4_shift = start + 2 * 39
        self.sorted_accum_shift = start + 2 * 40
        self.z_perm_shift = start + 2 * 41
        self.z_lookup_shift = start + 2 * 42

        self.entities = [self.q_c, self.q_l, self.q_r, self.q_o, self.q_m, self.sigma_1, self.sigma_2, self.sigma_3, self.id_1,
                         self.id_2, self.id_3, self.lagrange_first, self.lagrange_last, self.w_l, self.w_r, self.w_o, self.z_perm, self.z_perm_shift]


class Instance:
    def __init__(self, rows):
        self.num_entities = len(rows[0].entities)
        self.rows = rows


class Instances:
    def __init__(self, instances):
        self.num_entities = instances[0].num_entities
        self.data = instances


def rel(w_l, w_r, w_o, q_m, q_l, q_r, q_o, q_c):
    return q_m * w_l * w_r + q_l * w_l + q_r * w_r + q_o * w_o + q_c


def extend_one_entity(input):
    result = input
    delta = input[1]-input[0]
    for _ in range(2, EXTENDED_RELATION_LENGTH):
        result.append(delta + result[-1])
    return result


def get_extended_univariates(instances, row_idx):
    rows = [instance.rows[row_idx] for instance in instances.data]
    for entity_idx in range(instances.num_entities):
        result = [row.entities[entity_idx] for row in rows]
        result = np.array(extend_one_entity(result))
        return result
    
def compute_first_example():
    i0 = Instance([Row(0), Row(1)])
    i1 = Instance([Row(128), Row(129)])
    instances = Instances([i0, i1])

    row_0_extended = Row(get_extended_univariates(instances, 0))
    row_1_extended = Row(get_extended_univariates(instances, 1))

    accumulator = np.array([0 for _ in range(EXTENDED_RELATION_LENGTH)])
    zeta_pow = 1
    zeta = 2
    for row in [row_0_extended, row_1_extended]:
        relation_value = rel(row.w_l, row.w_r, row.w_o, row.q_m,
                             row.q_l, row.q_r, row.q_o, row.q_c)
        accumulator += zeta_pow * relation_value
        zeta_pow *= zeta
    return accumulator


def compute_second_example():
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
        result += rel(w_l, w_r, w_o, q_m, q_l, q_r, q_o, q_c)      

        w_l = np.array([ 0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12])
        w_r = np.array([ 4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4])
        w_o = np.array([ 4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4,  4])
        q_m = np.array([ 0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12])
        q_l = np.array([ 1,  0, -1, -2, -3, -4, -5, -6, -7, -8, -9,-10,-11])
        q_r = np.array([ 1,  0, -1, -2, -3, -4, -5, -6, -7, -8, -9,-10,-11])
        q_o = np.array([-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1])
        q_c = np.array([ 0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0])

        result += rel(w_l, w_r, w_o, q_m, q_l, q_r, q_o, q_c)
        result *= 2

        return result

if __name__ == "__main__":
    print(f"First example: \n  {compute_first_example()}")
    print(f"Second example:\n  {compute_second_example()}")

