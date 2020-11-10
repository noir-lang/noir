#pragma once
#include "value_note.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace circuit {

point_ct encrypt_note(const value_note& plaintext);

}
} // namespace notes
} // namespace proofs
} // namespace rollup