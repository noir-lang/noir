#pragma once

#include "serde.hpp"
#include "barretenberg/serialize/msgpack_impl.hpp"

namespace Acir {
    struct Helpers {
        static std::map<std::string, msgpack::object const*> make_kvmap(
            msgpack::object const& o,
            std::string const& name
        ) {
            if (o.type != msgpack::type::MAP) {
                std::cerr << o << std::endl;
                throw_or_abort("expected MAP for " + name);
            }
            std::map<std::string, msgpack::object const*> kvmap;
            for (uint32_t i = 0; i < o.via.map.size; ++i) {
                if (o.via.map.ptr[i].key.type != msgpack::type::STR) {
                    std::cerr << o << std::endl;
                    throw_or_abort("expected STR for keys of " + name);
                }
                kvmap.emplace(
                    std::string(
                        o.via.map.ptr[i].key.via.str.ptr,
                        o.via.map.ptr[i].key.via.str.size),
                    &o.via.map.ptr[i].val);
            }
            return kvmap;
        }

        template<typename T>
        static void conv_fld_from_kvmap(
            std::map<std::string, msgpack::object const*> const& kvmap,
            std::string const& struct_name,
            std::string const& field_name,
            T& field,
            bool is_optional
        ) {
            auto it = kvmap.find(field_name);
            if (it != kvmap.end()) {
                try {
                    it->second->convert(field);
                } catch (const msgpack::type_error&) {
                    std::cerr << *it->second << std::endl;
                    throw_or_abort("error converting into field " + struct_name + "::" + field_name);
                }
            } else if (!is_optional) {
                throw_or_abort("missing field: " + struct_name + "::" + field_name);
            }
        }

        template<typename T>
        static void conv_fld_from_array(
            msgpack::object_array const& array,
            std::string const& struct_name,
            std::string const& field_name,
            T& field,
            uint32_t index
        ) {
            if (index >= array.size) {
                throw_or_abort("index out of bounds: " + struct_name + "::" + field_name + " at " + std::to_string(index));
            }
            auto element = array.ptr[index];
            try {
                element.convert(field);
            } catch (const msgpack::type_error&) {
                std::cerr << element << std::endl;
                throw_or_abort("error converting into field " + struct_name + "::" + field_name);
            }
        }
    };
    }

namespace Acir {

    struct BinaryFieldOp {

        struct Add {
            friend bool operator==(const Add&, const Add&);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct Sub {
            friend bool operator==(const Sub&, const Sub&);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct Mul {
            friend bool operator==(const Mul&, const Mul&);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct Div {
            friend bool operator==(const Div&, const Div&);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct IntegerDiv {
            friend bool operator==(const IntegerDiv&, const IntegerDiv&);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct Equals {
            friend bool operator==(const Equals&, const Equals&);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct LessThan {
            friend bool operator==(const LessThan&, const LessThan&);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct LessThanEquals {
            friend bool operator==(const LessThanEquals&, const LessThanEquals&);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        std::variant<Add, Sub, Mul, Div, IntegerDiv, Equals, LessThan, LessThanEquals> value;

        friend bool operator==(const BinaryFieldOp&, const BinaryFieldOp&);

        void msgpack_pack(auto& packer) const {
            std::string tag;
            bool is_unit;
            switch (value.index()) {
                
                case 0:
                    tag = "Add";
                    is_unit = true;
                    break;
                case 1:
                    tag = "Sub";
                    is_unit = true;
                    break;
                case 2:
                    tag = "Mul";
                    is_unit = true;
                    break;
                case 3:
                    tag = "Div";
                    is_unit = true;
                    break;
                case 4:
                    tag = "IntegerDiv";
                    is_unit = true;
                    break;
                case 5:
                    tag = "Equals";
                    is_unit = true;
                    break;
                case 6:
                    tag = "LessThan";
                    is_unit = true;
                    break;
                case 7:
                    tag = "LessThanEquals";
                    is_unit = true;
                    break;
                default:
                    throw_or_abort("unknown enum 'BinaryFieldOp' variant index: " + std::to_string(value.index()));
            }
            if (is_unit) {
                packer.pack(tag);
            } else {
                std::visit([&packer, tag](const auto& arg) {
                    packer.pack_map(1);
                    packer.pack(tag);
                    arg.msgpack_pack(packer);
                }, value);
            }
        }

        void msgpack_unpack(msgpack::object const& o) {

            if (o.type != msgpack::type::object_type::MAP && o.type != msgpack::type::object_type::STR) {
                std::cerr << o << std::endl;
                throw_or_abort("expected MAP or STR for enum 'BinaryFieldOp'; got type " + std::to_string(o.type));
            }
            if (o.type == msgpack::type::object_type::MAP && o.via.map.size != 1) {
                throw_or_abort("expected 1 entry for enum 'BinaryFieldOp'; got " + std::to_string(o.via.map.size));
            }
            std::string tag;
            try {
                if (o.type == msgpack::type::object_type::MAP) {
                    o.via.map.ptr[0].key.convert(tag);
                } else {
                    o.convert(tag);
                }
            } catch(const msgpack::type_error&) {
                std::cerr << o << std::endl;
                throw_or_abort("error converting tag to string for enum 'BinaryFieldOp'");
            }
            if (tag == "Add") {
                Add v;
                value = v;
            }
            else if (tag == "Sub") {
                Sub v;
                value = v;
            }
            else if (tag == "Mul") {
                Mul v;
                value = v;
            }
            else if (tag == "Div") {
                Div v;
                value = v;
            }
            else if (tag == "IntegerDiv") {
                IntegerDiv v;
                value = v;
            }
            else if (tag == "Equals") {
                Equals v;
                value = v;
            }
            else if (tag == "LessThan") {
                LessThan v;
                value = v;
            }
            else if (tag == "LessThanEquals") {
                LessThanEquals v;
                value = v;
            }
            else {
                std::cerr << o << std::endl;
                throw_or_abort("unknown 'BinaryFieldOp' enum variant: " + tag);
            }
        }
    };

    struct BinaryIntOp {

        struct Add {
            friend bool operator==(const Add&, const Add&);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct Sub {
            friend bool operator==(const Sub&, const Sub&);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct Mul {
            friend bool operator==(const Mul&, const Mul&);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct Div {
            friend bool operator==(const Div&, const Div&);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct Equals {
            friend bool operator==(const Equals&, const Equals&);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct LessThan {
            friend bool operator==(const LessThan&, const LessThan&);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct LessThanEquals {
            friend bool operator==(const LessThanEquals&, const LessThanEquals&);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct And {
            friend bool operator==(const And&, const And&);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct Or {
            friend bool operator==(const Or&, const Or&);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct Xor {
            friend bool operator==(const Xor&, const Xor&);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct Shl {
            friend bool operator==(const Shl&, const Shl&);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct Shr {
            friend bool operator==(const Shr&, const Shr&);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        std::variant<Add, Sub, Mul, Div, Equals, LessThan, LessThanEquals, And, Or, Xor, Shl, Shr> value;

        friend bool operator==(const BinaryIntOp&, const BinaryIntOp&);

        void msgpack_pack(auto& packer) const {
            std::string tag;
            bool is_unit;
            switch (value.index()) {
                
                case 0:
                    tag = "Add";
                    is_unit = true;
                    break;
                case 1:
                    tag = "Sub";
                    is_unit = true;
                    break;
                case 2:
                    tag = "Mul";
                    is_unit = true;
                    break;
                case 3:
                    tag = "Div";
                    is_unit = true;
                    break;
                case 4:
                    tag = "Equals";
                    is_unit = true;
                    break;
                case 5:
                    tag = "LessThan";
                    is_unit = true;
                    break;
                case 6:
                    tag = "LessThanEquals";
                    is_unit = true;
                    break;
                case 7:
                    tag = "And";
                    is_unit = true;
                    break;
                case 8:
                    tag = "Or";
                    is_unit = true;
                    break;
                case 9:
                    tag = "Xor";
                    is_unit = true;
                    break;
                case 10:
                    tag = "Shl";
                    is_unit = true;
                    break;
                case 11:
                    tag = "Shr";
                    is_unit = true;
                    break;
                default:
                    throw_or_abort("unknown enum 'BinaryIntOp' variant index: " + std::to_string(value.index()));
            }
            if (is_unit) {
                packer.pack(tag);
            } else {
                std::visit([&packer, tag](const auto& arg) {
                    packer.pack_map(1);
                    packer.pack(tag);
                    arg.msgpack_pack(packer);
                }, value);
            }
        }

        void msgpack_unpack(msgpack::object const& o) {

            if (o.type != msgpack::type::object_type::MAP && o.type != msgpack::type::object_type::STR) {
                std::cerr << o << std::endl;
                throw_or_abort("expected MAP or STR for enum 'BinaryIntOp'; got type " + std::to_string(o.type));
            }
            if (o.type == msgpack::type::object_type::MAP && o.via.map.size != 1) {
                throw_or_abort("expected 1 entry for enum 'BinaryIntOp'; got " + std::to_string(o.via.map.size));
            }
            std::string tag;
            try {
                if (o.type == msgpack::type::object_type::MAP) {
                    o.via.map.ptr[0].key.convert(tag);
                } else {
                    o.convert(tag);
                }
            } catch(const msgpack::type_error&) {
                std::cerr << o << std::endl;
                throw_or_abort("error converting tag to string for enum 'BinaryIntOp'");
            }
            if (tag == "Add") {
                Add v;
                value = v;
            }
            else if (tag == "Sub") {
                Sub v;
                value = v;
            }
            else if (tag == "Mul") {
                Mul v;
                value = v;
            }
            else if (tag == "Div") {
                Div v;
                value = v;
            }
            else if (tag == "Equals") {
                Equals v;
                value = v;
            }
            else if (tag == "LessThan") {
                LessThan v;
                value = v;
            }
            else if (tag == "LessThanEquals") {
                LessThanEquals v;
                value = v;
            }
            else if (tag == "And") {
                And v;
                value = v;
            }
            else if (tag == "Or") {
                Or v;
                value = v;
            }
            else if (tag == "Xor") {
                Xor v;
                value = v;
            }
            else if (tag == "Shl") {
                Shl v;
                value = v;
            }
            else if (tag == "Shr") {
                Shr v;
                value = v;
            }
            else {
                std::cerr << o << std::endl;
                throw_or_abort("unknown 'BinaryIntOp' enum variant: " + tag);
            }
        }
    };

    struct IntegerBitSize {

        struct U1 {
            friend bool operator==(const U1&, const U1&);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct U8 {
            friend bool operator==(const U8&, const U8&);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct U16 {
            friend bool operator==(const U16&, const U16&);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct U32 {
            friend bool operator==(const U32&, const U32&);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct U64 {
            friend bool operator==(const U64&, const U64&);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct U128 {
            friend bool operator==(const U128&, const U128&);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        std::variant<U1, U8, U16, U32, U64, U128> value;

        friend bool operator==(const IntegerBitSize&, const IntegerBitSize&);

        void msgpack_pack(auto& packer) const {
            std::string tag;
            bool is_unit;
            switch (value.index()) {
                
                case 0:
                    tag = "U1";
                    is_unit = true;
                    break;
                case 1:
                    tag = "U8";
                    is_unit = true;
                    break;
                case 2:
                    tag = "U16";
                    is_unit = true;
                    break;
                case 3:
                    tag = "U32";
                    is_unit = true;
                    break;
                case 4:
                    tag = "U64";
                    is_unit = true;
                    break;
                case 5:
                    tag = "U128";
                    is_unit = true;
                    break;
                default:
                    throw_or_abort("unknown enum 'IntegerBitSize' variant index: " + std::to_string(value.index()));
            }
            if (is_unit) {
                packer.pack(tag);
            } else {
                std::visit([&packer, tag](const auto& arg) {
                    packer.pack_map(1);
                    packer.pack(tag);
                    arg.msgpack_pack(packer);
                }, value);
            }
        }

        void msgpack_unpack(msgpack::object const& o) {

            if (o.type != msgpack::type::object_type::MAP && o.type != msgpack::type::object_type::STR) {
                std::cerr << o << std::endl;
                throw_or_abort("expected MAP or STR for enum 'IntegerBitSize'; got type " + std::to_string(o.type));
            }
            if (o.type == msgpack::type::object_type::MAP && o.via.map.size != 1) {
                throw_or_abort("expected 1 entry for enum 'IntegerBitSize'; got " + std::to_string(o.via.map.size));
            }
            std::string tag;
            try {
                if (o.type == msgpack::type::object_type::MAP) {
                    o.via.map.ptr[0].key.convert(tag);
                } else {
                    o.convert(tag);
                }
            } catch(const msgpack::type_error&) {
                std::cerr << o << std::endl;
                throw_or_abort("error converting tag to string for enum 'IntegerBitSize'");
            }
            if (tag == "U1") {
                U1 v;
                value = v;
            }
            else if (tag == "U8") {
                U8 v;
                value = v;
            }
            else if (tag == "U16") {
                U16 v;
                value = v;
            }
            else if (tag == "U32") {
                U32 v;
                value = v;
            }
            else if (tag == "U64") {
                U64 v;
                value = v;
            }
            else if (tag == "U128") {
                U128 v;
                value = v;
            }
            else {
                std::cerr << o << std::endl;
                throw_or_abort("unknown 'IntegerBitSize' enum variant: " + tag);
            }
        }
    };

    struct BitSize {

        struct Field {
            friend bool operator==(const Field&, const Field&);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct Integer {
            Acir::IntegerBitSize value;

            friend bool operator==(const Integer&, const Integer&);

            void msgpack_pack(auto& packer) const { packer.pack(value); }

            void msgpack_unpack(msgpack::object const& o) {
                try {
                    o.convert(value);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into newtype 'Integer'");
                }
            }
        };

        std::variant<Field, Integer> value;

        friend bool operator==(const BitSize&, const BitSize&);

        void msgpack_pack(auto& packer) const {
            std::string tag;
            bool is_unit;
            switch (value.index()) {
                
                case 0:
                    tag = "Field";
                    is_unit = true;
                    break;
                case 1:
                    tag = "Integer";
                    is_unit = false;
                    break;
                default:
                    throw_or_abort("unknown enum 'BitSize' variant index: " + std::to_string(value.index()));
            }
            if (is_unit) {
                packer.pack(tag);
            } else {
                std::visit([&packer, tag](const auto& arg) {
                    packer.pack_map(1);
                    packer.pack(tag);
                    arg.msgpack_pack(packer);
                }, value);
            }
        }

        void msgpack_unpack(msgpack::object const& o) {

            if (o.type != msgpack::type::object_type::MAP && o.type != msgpack::type::object_type::STR) {
                std::cerr << o << std::endl;
                throw_or_abort("expected MAP or STR for enum 'BitSize'; got type " + std::to_string(o.type));
            }
            if (o.type == msgpack::type::object_type::MAP && o.via.map.size != 1) {
                throw_or_abort("expected 1 entry for enum 'BitSize'; got " + std::to_string(o.via.map.size));
            }
            std::string tag;
            try {
                if (o.type == msgpack::type::object_type::MAP) {
                    o.via.map.ptr[0].key.convert(tag);
                } else {
                    o.convert(tag);
                }
            } catch(const msgpack::type_error&) {
                std::cerr << o << std::endl;
                throw_or_abort("error converting tag to string for enum 'BitSize'");
            }
            if (tag == "Field") {
                Field v;
                value = v;
            }
            else if (tag == "Integer") {
                Integer v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BitSize::Integer'");
                }
                
                value = v;
            }
            else {
                std::cerr << o << std::endl;
                throw_or_abort("unknown 'BitSize' enum variant: " + tag);
            }
        }
    };

    struct MemoryAddress {

        struct Direct {
            uint64_t value;

            friend bool operator==(const Direct&, const Direct&);

            void msgpack_pack(auto& packer) const { packer.pack(value); }

            void msgpack_unpack(msgpack::object const& o) {
                try {
                    o.convert(value);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into newtype 'Direct'");
                }
            }
        };

        struct Relative {
            uint64_t value;

            friend bool operator==(const Relative&, const Relative&);

            void msgpack_pack(auto& packer) const { packer.pack(value); }

            void msgpack_unpack(msgpack::object const& o) {
                try {
                    o.convert(value);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into newtype 'Relative'");
                }
            }
        };

        std::variant<Direct, Relative> value;

        friend bool operator==(const MemoryAddress&, const MemoryAddress&);

        void msgpack_pack(auto& packer) const {
            std::string tag;
            bool is_unit;
            switch (value.index()) {
                
                case 0:
                    tag = "Direct";
                    is_unit = false;
                    break;
                case 1:
                    tag = "Relative";
                    is_unit = false;
                    break;
                default:
                    throw_or_abort("unknown enum 'MemoryAddress' variant index: " + std::to_string(value.index()));
            }
            if (is_unit) {
                packer.pack(tag);
            } else {
                std::visit([&packer, tag](const auto& arg) {
                    packer.pack_map(1);
                    packer.pack(tag);
                    arg.msgpack_pack(packer);
                }, value);
            }
        }

        void msgpack_unpack(msgpack::object const& o) {

            if (o.type != msgpack::type::object_type::MAP && o.type != msgpack::type::object_type::STR) {
                std::cerr << o << std::endl;
                throw_or_abort("expected MAP or STR for enum 'MemoryAddress'; got type " + std::to_string(o.type));
            }
            if (o.type == msgpack::type::object_type::MAP && o.via.map.size != 1) {
                throw_or_abort("expected 1 entry for enum 'MemoryAddress'; got " + std::to_string(o.via.map.size));
            }
            std::string tag;
            try {
                if (o.type == msgpack::type::object_type::MAP) {
                    o.via.map.ptr[0].key.convert(tag);
                } else {
                    o.convert(tag);
                }
            } catch(const msgpack::type_error&) {
                std::cerr << o << std::endl;
                throw_or_abort("error converting tag to string for enum 'MemoryAddress'");
            }
            if (tag == "Direct") {
                Direct v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'MemoryAddress::Direct'");
                }
                
                value = v;
            }
            else if (tag == "Relative") {
                Relative v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'MemoryAddress::Relative'");
                }
                
                value = v;
            }
            else {
                std::cerr << o << std::endl;
                throw_or_abort("unknown 'MemoryAddress' enum variant: " + tag);
            }
        }
    };

    struct SemiFlattenedLength {
        uint64_t value;

        friend bool operator==(const SemiFlattenedLength&, const SemiFlattenedLength&);
        std::vector<uint8_t> bincodeSerialize() const;
        static SemiFlattenedLength bincodeDeserialize(std::vector<uint8_t>);

        void msgpack_pack(auto& packer) const { packer.pack(value); }

        void msgpack_unpack(msgpack::object const& o) {
            try {
                o.convert(value);
            } catch (const msgpack::type_error&) {
                std::cerr << o << std::endl;
                throw_or_abort("error converting into newtype 'SemiFlattenedLength'");
            }
        }
    };

    struct HeapArray {
        Acir::MemoryAddress pointer;
        Acir::SemiFlattenedLength size;

        friend bool operator==(const HeapArray&, const HeapArray&);

        void msgpack_pack(auto& packer) const {
            packer.pack_map(2);
            packer.pack(std::make_pair("pointer", pointer));
            packer.pack(std::make_pair("size", size));
        }

        void msgpack_unpack(msgpack::object const& o) {
            std::string name = "HeapArray";
            if (o.type == msgpack::type::MAP) {
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "pointer", pointer, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "size", size, false);
            } else if (o.type == msgpack::type::ARRAY) {
                auto array = o.via.array; 
                Helpers::conv_fld_from_array(array, name, "pointer", pointer, 0);
                Helpers::conv_fld_from_array(array, name, "size", size, 1);
            } else {
                throw_or_abort("expected MAP or ARRAY for " + name);
            }
        }
    };

    struct HeapVector {
        Acir::MemoryAddress pointer;
        Acir::MemoryAddress size;

        friend bool operator==(const HeapVector&, const HeapVector&);

        void msgpack_pack(auto& packer) const {
            packer.pack_map(2);
            packer.pack(std::make_pair("pointer", pointer));
            packer.pack(std::make_pair("size", size));
        }

        void msgpack_unpack(msgpack::object const& o) {
            std::string name = "HeapVector";
            if (o.type == msgpack::type::MAP) {
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "pointer", pointer, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "size", size, false);
            } else if (o.type == msgpack::type::ARRAY) {
                auto array = o.via.array; 
                Helpers::conv_fld_from_array(array, name, "pointer", pointer, 0);
                Helpers::conv_fld_from_array(array, name, "size", size, 1);
            } else {
                throw_or_abort("expected MAP or ARRAY for " + name);
            }
        }
    };

    struct BlackBoxOp {

        struct AES128Encrypt {
            Acir::HeapVector inputs;
            Acir::HeapArray iv;
            Acir::HeapArray key;
            Acir::HeapVector outputs;

            friend bool operator==(const AES128Encrypt&, const AES128Encrypt&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(4);
                packer.pack(std::make_pair("inputs", inputs));
                packer.pack(std::make_pair("iv", iv));
                packer.pack(std::make_pair("key", key));
                packer.pack(std::make_pair("outputs", outputs));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "AES128Encrypt";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "inputs", inputs, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "iv", iv, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "key", key, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "outputs", outputs, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "inputs", inputs, 0);
                    Helpers::conv_fld_from_array(array, name, "iv", iv, 1);
                    Helpers::conv_fld_from_array(array, name, "key", key, 2);
                    Helpers::conv_fld_from_array(array, name, "outputs", outputs, 3);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        struct Blake2s {
            Acir::HeapVector message;
            Acir::HeapArray output;

            friend bool operator==(const Blake2s&, const Blake2s&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(2);
                packer.pack(std::make_pair("message", message));
                packer.pack(std::make_pair("output", output));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "Blake2s";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "message", message, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "output", output, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "message", message, 0);
                    Helpers::conv_fld_from_array(array, name, "output", output, 1);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        struct Blake3 {
            Acir::HeapVector message;
            Acir::HeapArray output;

            friend bool operator==(const Blake3&, const Blake3&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(2);
                packer.pack(std::make_pair("message", message));
                packer.pack(std::make_pair("output", output));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "Blake3";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "message", message, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "output", output, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "message", message, 0);
                    Helpers::conv_fld_from_array(array, name, "output", output, 1);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        struct Keccakf1600 {
            Acir::HeapArray input;
            Acir::HeapArray output;

            friend bool operator==(const Keccakf1600&, const Keccakf1600&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(2);
                packer.pack(std::make_pair("input", input));
                packer.pack(std::make_pair("output", output));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "Keccakf1600";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "input", input, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "output", output, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "input", input, 0);
                    Helpers::conv_fld_from_array(array, name, "output", output, 1);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        struct EcdsaSecp256k1 {
            Acir::HeapVector hashed_msg;
            Acir::HeapArray public_key_x;
            Acir::HeapArray public_key_y;
            Acir::HeapArray signature;
            Acir::MemoryAddress result;

            friend bool operator==(const EcdsaSecp256k1&, const EcdsaSecp256k1&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(5);
                packer.pack(std::make_pair("hashed_msg", hashed_msg));
                packer.pack(std::make_pair("public_key_x", public_key_x));
                packer.pack(std::make_pair("public_key_y", public_key_y));
                packer.pack(std::make_pair("signature", signature));
                packer.pack(std::make_pair("result", result));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "EcdsaSecp256k1";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "hashed_msg", hashed_msg, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "public_key_x", public_key_x, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "public_key_y", public_key_y, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "signature", signature, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "result", result, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "hashed_msg", hashed_msg, 0);
                    Helpers::conv_fld_from_array(array, name, "public_key_x", public_key_x, 1);
                    Helpers::conv_fld_from_array(array, name, "public_key_y", public_key_y, 2);
                    Helpers::conv_fld_from_array(array, name, "signature", signature, 3);
                    Helpers::conv_fld_from_array(array, name, "result", result, 4);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        struct EcdsaSecp256r1 {
            Acir::HeapVector hashed_msg;
            Acir::HeapArray public_key_x;
            Acir::HeapArray public_key_y;
            Acir::HeapArray signature;
            Acir::MemoryAddress result;

            friend bool operator==(const EcdsaSecp256r1&, const EcdsaSecp256r1&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(5);
                packer.pack(std::make_pair("hashed_msg", hashed_msg));
                packer.pack(std::make_pair("public_key_x", public_key_x));
                packer.pack(std::make_pair("public_key_y", public_key_y));
                packer.pack(std::make_pair("signature", signature));
                packer.pack(std::make_pair("result", result));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "EcdsaSecp256r1";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "hashed_msg", hashed_msg, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "public_key_x", public_key_x, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "public_key_y", public_key_y, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "signature", signature, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "result", result, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "hashed_msg", hashed_msg, 0);
                    Helpers::conv_fld_from_array(array, name, "public_key_x", public_key_x, 1);
                    Helpers::conv_fld_from_array(array, name, "public_key_y", public_key_y, 2);
                    Helpers::conv_fld_from_array(array, name, "signature", signature, 3);
                    Helpers::conv_fld_from_array(array, name, "result", result, 4);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        struct MultiScalarMul {
            Acir::HeapVector points;
            Acir::HeapVector scalars;
            Acir::HeapArray outputs;

            friend bool operator==(const MultiScalarMul&, const MultiScalarMul&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(3);
                packer.pack(std::make_pair("points", points));
                packer.pack(std::make_pair("scalars", scalars));
                packer.pack(std::make_pair("outputs", outputs));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "MultiScalarMul";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "points", points, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "scalars", scalars, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "outputs", outputs, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "points", points, 0);
                    Helpers::conv_fld_from_array(array, name, "scalars", scalars, 1);
                    Helpers::conv_fld_from_array(array, name, "outputs", outputs, 2);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        struct EmbeddedCurveAdd {
            Acir::MemoryAddress input1_x;
            Acir::MemoryAddress input1_y;
            Acir::MemoryAddress input1_infinite;
            Acir::MemoryAddress input2_x;
            Acir::MemoryAddress input2_y;
            Acir::MemoryAddress input2_infinite;
            Acir::HeapArray result;

            friend bool operator==(const EmbeddedCurveAdd&, const EmbeddedCurveAdd&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(7);
                packer.pack(std::make_pair("input1_x", input1_x));
                packer.pack(std::make_pair("input1_y", input1_y));
                packer.pack(std::make_pair("input1_infinite", input1_infinite));
                packer.pack(std::make_pair("input2_x", input2_x));
                packer.pack(std::make_pair("input2_y", input2_y));
                packer.pack(std::make_pair("input2_infinite", input2_infinite));
                packer.pack(std::make_pair("result", result));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "EmbeddedCurveAdd";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "input1_x", input1_x, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "input1_y", input1_y, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "input1_infinite", input1_infinite, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "input2_x", input2_x, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "input2_y", input2_y, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "input2_infinite", input2_infinite, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "result", result, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "input1_x", input1_x, 0);
                    Helpers::conv_fld_from_array(array, name, "input1_y", input1_y, 1);
                    Helpers::conv_fld_from_array(array, name, "input1_infinite", input1_infinite, 2);
                    Helpers::conv_fld_from_array(array, name, "input2_x", input2_x, 3);
                    Helpers::conv_fld_from_array(array, name, "input2_y", input2_y, 4);
                    Helpers::conv_fld_from_array(array, name, "input2_infinite", input2_infinite, 5);
                    Helpers::conv_fld_from_array(array, name, "result", result, 6);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        struct Poseidon2Permutation {
            Acir::HeapVector message;
            Acir::HeapArray output;

            friend bool operator==(const Poseidon2Permutation&, const Poseidon2Permutation&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(2);
                packer.pack(std::make_pair("message", message));
                packer.pack(std::make_pair("output", output));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "Poseidon2Permutation";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "message", message, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "output", output, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "message", message, 0);
                    Helpers::conv_fld_from_array(array, name, "output", output, 1);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        struct Sha256Compression {
            Acir::HeapArray input;
            Acir::HeapArray hash_values;
            Acir::HeapArray output;

            friend bool operator==(const Sha256Compression&, const Sha256Compression&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(3);
                packer.pack(std::make_pair("input", input));
                packer.pack(std::make_pair("hash_values", hash_values));
                packer.pack(std::make_pair("output", output));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "Sha256Compression";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "input", input, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "hash_values", hash_values, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "output", output, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "input", input, 0);
                    Helpers::conv_fld_from_array(array, name, "hash_values", hash_values, 1);
                    Helpers::conv_fld_from_array(array, name, "output", output, 2);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        struct ToRadix {
            Acir::MemoryAddress input;
            Acir::MemoryAddress radix;
            Acir::MemoryAddress output_pointer;
            Acir::MemoryAddress num_limbs;
            Acir::MemoryAddress output_bits;

            friend bool operator==(const ToRadix&, const ToRadix&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(5);
                packer.pack(std::make_pair("input", input));
                packer.pack(std::make_pair("radix", radix));
                packer.pack(std::make_pair("output_pointer", output_pointer));
                packer.pack(std::make_pair("num_limbs", num_limbs));
                packer.pack(std::make_pair("output_bits", output_bits));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "ToRadix";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "input", input, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "radix", radix, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "output_pointer", output_pointer, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "num_limbs", num_limbs, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "output_bits", output_bits, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "input", input, 0);
                    Helpers::conv_fld_from_array(array, name, "radix", radix, 1);
                    Helpers::conv_fld_from_array(array, name, "output_pointer", output_pointer, 2);
                    Helpers::conv_fld_from_array(array, name, "num_limbs", num_limbs, 3);
                    Helpers::conv_fld_from_array(array, name, "output_bits", output_bits, 4);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        std::variant<AES128Encrypt, Blake2s, Blake3, Keccakf1600, EcdsaSecp256k1, EcdsaSecp256r1, MultiScalarMul, EmbeddedCurveAdd, Poseidon2Permutation, Sha256Compression, ToRadix> value;

        friend bool operator==(const BlackBoxOp&, const BlackBoxOp&);

        void msgpack_pack(auto& packer) const {
            std::string tag;
            bool is_unit;
            switch (value.index()) {
                
                case 0:
                    tag = "AES128Encrypt";
                    is_unit = false;
                    break;
                case 1:
                    tag = "Blake2s";
                    is_unit = false;
                    break;
                case 2:
                    tag = "Blake3";
                    is_unit = false;
                    break;
                case 3:
                    tag = "Keccakf1600";
                    is_unit = false;
                    break;
                case 4:
                    tag = "EcdsaSecp256k1";
                    is_unit = false;
                    break;
                case 5:
                    tag = "EcdsaSecp256r1";
                    is_unit = false;
                    break;
                case 6:
                    tag = "MultiScalarMul";
                    is_unit = false;
                    break;
                case 7:
                    tag = "EmbeddedCurveAdd";
                    is_unit = false;
                    break;
                case 8:
                    tag = "Poseidon2Permutation";
                    is_unit = false;
                    break;
                case 9:
                    tag = "Sha256Compression";
                    is_unit = false;
                    break;
                case 10:
                    tag = "ToRadix";
                    is_unit = false;
                    break;
                default:
                    throw_or_abort("unknown enum 'BlackBoxOp' variant index: " + std::to_string(value.index()));
            }
            if (is_unit) {
                packer.pack(tag);
            } else {
                std::visit([&packer, tag](const auto& arg) {
                    packer.pack_map(1);
                    packer.pack(tag);
                    arg.msgpack_pack(packer);
                }, value);
            }
        }

        void msgpack_unpack(msgpack::object const& o) {

            if (o.type != msgpack::type::object_type::MAP && o.type != msgpack::type::object_type::STR) {
                std::cerr << o << std::endl;
                throw_or_abort("expected MAP or STR for enum 'BlackBoxOp'; got type " + std::to_string(o.type));
            }
            if (o.type == msgpack::type::object_type::MAP && o.via.map.size != 1) {
                throw_or_abort("expected 1 entry for enum 'BlackBoxOp'; got " + std::to_string(o.via.map.size));
            }
            std::string tag;
            try {
                if (o.type == msgpack::type::object_type::MAP) {
                    o.via.map.ptr[0].key.convert(tag);
                } else {
                    o.convert(tag);
                }
            } catch(const msgpack::type_error&) {
                std::cerr << o << std::endl;
                throw_or_abort("error converting tag to string for enum 'BlackBoxOp'");
            }
            if (tag == "AES128Encrypt") {
                AES128Encrypt v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BlackBoxOp::AES128Encrypt'");
                }
                
                value = v;
            }
            else if (tag == "Blake2s") {
                Blake2s v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BlackBoxOp::Blake2s'");
                }
                
                value = v;
            }
            else if (tag == "Blake3") {
                Blake3 v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BlackBoxOp::Blake3'");
                }
                
                value = v;
            }
            else if (tag == "Keccakf1600") {
                Keccakf1600 v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BlackBoxOp::Keccakf1600'");
                }
                
                value = v;
            }
            else if (tag == "EcdsaSecp256k1") {
                EcdsaSecp256k1 v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BlackBoxOp::EcdsaSecp256k1'");
                }
                
                value = v;
            }
            else if (tag == "EcdsaSecp256r1") {
                EcdsaSecp256r1 v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BlackBoxOp::EcdsaSecp256r1'");
                }
                
                value = v;
            }
            else if (tag == "MultiScalarMul") {
                MultiScalarMul v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BlackBoxOp::MultiScalarMul'");
                }
                
                value = v;
            }
            else if (tag == "EmbeddedCurveAdd") {
                EmbeddedCurveAdd v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BlackBoxOp::EmbeddedCurveAdd'");
                }
                
                value = v;
            }
            else if (tag == "Poseidon2Permutation") {
                Poseidon2Permutation v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BlackBoxOp::Poseidon2Permutation'");
                }
                
                value = v;
            }
            else if (tag == "Sha256Compression") {
                Sha256Compression v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BlackBoxOp::Sha256Compression'");
                }
                
                value = v;
            }
            else if (tag == "ToRadix") {
                ToRadix v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BlackBoxOp::ToRadix'");
                }
                
                value = v;
            }
            else {
                std::cerr << o << std::endl;
                throw_or_abort("unknown 'BlackBoxOp' enum variant: " + tag);
            }
        }
    };

    struct SemanticLength {
        uint64_t value;

        friend bool operator==(const SemanticLength&, const SemanticLength&);
        std::vector<uint8_t> bincodeSerialize() const;
        static SemanticLength bincodeDeserialize(std::vector<uint8_t>);

        void msgpack_pack(auto& packer) const { packer.pack(value); }

        void msgpack_unpack(msgpack::object const& o) {
            try {
                o.convert(value);
            } catch (const msgpack::type_error&) {
                std::cerr << o << std::endl;
                throw_or_abort("error converting into newtype 'SemanticLength'");
            }
        }
    };

    struct HeapValueType;

    struct HeapValueType {

        struct Simple {
            Acir::BitSize value;

            friend bool operator==(const Simple&, const Simple&);

            void msgpack_pack(auto& packer) const { packer.pack(value); }

            void msgpack_unpack(msgpack::object const& o) {
                try {
                    o.convert(value);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into newtype 'Simple'");
                }
            }
        };

        struct Array {
            std::vector<Acir::HeapValueType> value_types;
            Acir::SemanticLength size;

            friend bool operator==(const Array&, const Array&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(2);
                packer.pack(std::make_pair("value_types", value_types));
                packer.pack(std::make_pair("size", size));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "Array";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "value_types", value_types, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "size", size, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "value_types", value_types, 0);
                    Helpers::conv_fld_from_array(array, name, "size", size, 1);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        struct Vector {
            std::vector<Acir::HeapValueType> value_types;

            friend bool operator==(const Vector&, const Vector&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(1);
                packer.pack(std::make_pair("value_types", value_types));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "Vector";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "value_types", value_types, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "value_types", value_types, 0);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        std::variant<Simple, Array, Vector> value;

        friend bool operator==(const HeapValueType&, const HeapValueType&);

        void msgpack_pack(auto& packer) const {
            std::string tag;
            bool is_unit;
            switch (value.index()) {
                
                case 0:
                    tag = "Simple";
                    is_unit = false;
                    break;
                case 1:
                    tag = "Array";
                    is_unit = false;
                    break;
                case 2:
                    tag = "Vector";
                    is_unit = false;
                    break;
                default:
                    throw_or_abort("unknown enum 'HeapValueType' variant index: " + std::to_string(value.index()));
            }
            if (is_unit) {
                packer.pack(tag);
            } else {
                std::visit([&packer, tag](const auto& arg) {
                    packer.pack_map(1);
                    packer.pack(tag);
                    arg.msgpack_pack(packer);
                }, value);
            }
        }

        void msgpack_unpack(msgpack::object const& o) {

            if (o.type != msgpack::type::object_type::MAP && o.type != msgpack::type::object_type::STR) {
                std::cerr << o << std::endl;
                throw_or_abort("expected MAP or STR for enum 'HeapValueType'; got type " + std::to_string(o.type));
            }
            if (o.type == msgpack::type::object_type::MAP && o.via.map.size != 1) {
                throw_or_abort("expected 1 entry for enum 'HeapValueType'; got " + std::to_string(o.via.map.size));
            }
            std::string tag;
            try {
                if (o.type == msgpack::type::object_type::MAP) {
                    o.via.map.ptr[0].key.convert(tag);
                } else {
                    o.convert(tag);
                }
            } catch(const msgpack::type_error&) {
                std::cerr << o << std::endl;
                throw_or_abort("error converting tag to string for enum 'HeapValueType'");
            }
            if (tag == "Simple") {
                Simple v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'HeapValueType::Simple'");
                }
                
                value = v;
            }
            else if (tag == "Array") {
                Array v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'HeapValueType::Array'");
                }
                
                value = v;
            }
            else if (tag == "Vector") {
                Vector v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'HeapValueType::Vector'");
                }
                
                value = v;
            }
            else {
                std::cerr << o << std::endl;
                throw_or_abort("unknown 'HeapValueType' enum variant: " + tag);
            }
        }
    };

    struct ValueOrArray {

        struct MemoryAddress {
            Acir::MemoryAddress value;

            friend bool operator==(const MemoryAddress&, const MemoryAddress&);

            void msgpack_pack(auto& packer) const { packer.pack(value); }

            void msgpack_unpack(msgpack::object const& o) {
                try {
                    o.convert(value);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into newtype 'MemoryAddress'");
                }
            }
        };

        struct HeapArray {
            Acir::HeapArray value;

            friend bool operator==(const HeapArray&, const HeapArray&);

            void msgpack_pack(auto& packer) const { packer.pack(value); }

            void msgpack_unpack(msgpack::object const& o) {
                try {
                    o.convert(value);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into newtype 'HeapArray'");
                }
            }
        };

        struct HeapVector {
            Acir::HeapVector value;

            friend bool operator==(const HeapVector&, const HeapVector&);

            void msgpack_pack(auto& packer) const { packer.pack(value); }

            void msgpack_unpack(msgpack::object const& o) {
                try {
                    o.convert(value);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into newtype 'HeapVector'");
                }
            }
        };

        std::variant<MemoryAddress, HeapArray, HeapVector> value;

        friend bool operator==(const ValueOrArray&, const ValueOrArray&);

        void msgpack_pack(auto& packer) const {
            std::string tag;
            bool is_unit;
            switch (value.index()) {
                
                case 0:
                    tag = "MemoryAddress";
                    is_unit = false;
                    break;
                case 1:
                    tag = "HeapArray";
                    is_unit = false;
                    break;
                case 2:
                    tag = "HeapVector";
                    is_unit = false;
                    break;
                default:
                    throw_or_abort("unknown enum 'ValueOrArray' variant index: " + std::to_string(value.index()));
            }
            if (is_unit) {
                packer.pack(tag);
            } else {
                std::visit([&packer, tag](const auto& arg) {
                    packer.pack_map(1);
                    packer.pack(tag);
                    arg.msgpack_pack(packer);
                }, value);
            }
        }

        void msgpack_unpack(msgpack::object const& o) {

            if (o.type != msgpack::type::object_type::MAP && o.type != msgpack::type::object_type::STR) {
                std::cerr << o << std::endl;
                throw_or_abort("expected MAP or STR for enum 'ValueOrArray'; got type " + std::to_string(o.type));
            }
            if (o.type == msgpack::type::object_type::MAP && o.via.map.size != 1) {
                throw_or_abort("expected 1 entry for enum 'ValueOrArray'; got " + std::to_string(o.via.map.size));
            }
            std::string tag;
            try {
                if (o.type == msgpack::type::object_type::MAP) {
                    o.via.map.ptr[0].key.convert(tag);
                } else {
                    o.convert(tag);
                }
            } catch(const msgpack::type_error&) {
                std::cerr << o << std::endl;
                throw_or_abort("error converting tag to string for enum 'ValueOrArray'");
            }
            if (tag == "MemoryAddress") {
                MemoryAddress v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'ValueOrArray::MemoryAddress'");
                }
                
                value = v;
            }
            else if (tag == "HeapArray") {
                HeapArray v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'ValueOrArray::HeapArray'");
                }
                
                value = v;
            }
            else if (tag == "HeapVector") {
                HeapVector v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'ValueOrArray::HeapVector'");
                }
                
                value = v;
            }
            else {
                std::cerr << o << std::endl;
                throw_or_abort("unknown 'ValueOrArray' enum variant: " + tag);
            }
        }
    };

    struct BrilligOpcode {

        struct BinaryFieldOp {
            Acir::MemoryAddress destination;
            Acir::BinaryFieldOp op;
            Acir::MemoryAddress lhs;
            Acir::MemoryAddress rhs;

            friend bool operator==(const BinaryFieldOp&, const BinaryFieldOp&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(4);
                packer.pack(std::make_pair("destination", destination));
                packer.pack(std::make_pair("op", op));
                packer.pack(std::make_pair("lhs", lhs));
                packer.pack(std::make_pair("rhs", rhs));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "BinaryFieldOp";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "destination", destination, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "op", op, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "lhs", lhs, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "rhs", rhs, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "destination", destination, 0);
                    Helpers::conv_fld_from_array(array, name, "op", op, 1);
                    Helpers::conv_fld_from_array(array, name, "lhs", lhs, 2);
                    Helpers::conv_fld_from_array(array, name, "rhs", rhs, 3);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        struct BinaryIntOp {
            Acir::MemoryAddress destination;
            Acir::BinaryIntOp op;
            Acir::IntegerBitSize bit_size;
            Acir::MemoryAddress lhs;
            Acir::MemoryAddress rhs;

            friend bool operator==(const BinaryIntOp&, const BinaryIntOp&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(5);
                packer.pack(std::make_pair("destination", destination));
                packer.pack(std::make_pair("op", op));
                packer.pack(std::make_pair("bit_size", bit_size));
                packer.pack(std::make_pair("lhs", lhs));
                packer.pack(std::make_pair("rhs", rhs));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "BinaryIntOp";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "destination", destination, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "op", op, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "bit_size", bit_size, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "lhs", lhs, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "rhs", rhs, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "destination", destination, 0);
                    Helpers::conv_fld_from_array(array, name, "op", op, 1);
                    Helpers::conv_fld_from_array(array, name, "bit_size", bit_size, 2);
                    Helpers::conv_fld_from_array(array, name, "lhs", lhs, 3);
                    Helpers::conv_fld_from_array(array, name, "rhs", rhs, 4);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        struct Not {
            Acir::MemoryAddress destination;
            Acir::MemoryAddress source;
            Acir::IntegerBitSize bit_size;

            friend bool operator==(const Not&, const Not&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(3);
                packer.pack(std::make_pair("destination", destination));
                packer.pack(std::make_pair("source", source));
                packer.pack(std::make_pair("bit_size", bit_size));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "Not";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "destination", destination, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "source", source, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "bit_size", bit_size, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "destination", destination, 0);
                    Helpers::conv_fld_from_array(array, name, "source", source, 1);
                    Helpers::conv_fld_from_array(array, name, "bit_size", bit_size, 2);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        struct Cast {
            Acir::MemoryAddress destination;
            Acir::MemoryAddress source;
            Acir::BitSize bit_size;

            friend bool operator==(const Cast&, const Cast&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(3);
                packer.pack(std::make_pair("destination", destination));
                packer.pack(std::make_pair("source", source));
                packer.pack(std::make_pair("bit_size", bit_size));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "Cast";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "destination", destination, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "source", source, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "bit_size", bit_size, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "destination", destination, 0);
                    Helpers::conv_fld_from_array(array, name, "source", source, 1);
                    Helpers::conv_fld_from_array(array, name, "bit_size", bit_size, 2);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        struct JumpIf {
            Acir::MemoryAddress condition;
            uint64_t location;

            friend bool operator==(const JumpIf&, const JumpIf&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(2);
                packer.pack(std::make_pair("condition", condition));
                packer.pack(std::make_pair("location", location));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "JumpIf";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "condition", condition, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "location", location, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "condition", condition, 0);
                    Helpers::conv_fld_from_array(array, name, "location", location, 1);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        struct Jump {
            uint64_t location;

            friend bool operator==(const Jump&, const Jump&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(1);
                packer.pack(std::make_pair("location", location));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "Jump";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "location", location, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "location", location, 0);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        struct CalldataCopy {
            Acir::MemoryAddress destination_address;
            Acir::MemoryAddress size_address;
            Acir::MemoryAddress offset_address;

            friend bool operator==(const CalldataCopy&, const CalldataCopy&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(3);
                packer.pack(std::make_pair("destination_address", destination_address));
                packer.pack(std::make_pair("size_address", size_address));
                packer.pack(std::make_pair("offset_address", offset_address));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "CalldataCopy";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "destination_address", destination_address, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "size_address", size_address, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "offset_address", offset_address, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "destination_address", destination_address, 0);
                    Helpers::conv_fld_from_array(array, name, "size_address", size_address, 1);
                    Helpers::conv_fld_from_array(array, name, "offset_address", offset_address, 2);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        struct Call {
            uint64_t location;

            friend bool operator==(const Call&, const Call&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(1);
                packer.pack(std::make_pair("location", location));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "Call";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "location", location, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "location", location, 0);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        struct Const {
            Acir::MemoryAddress destination;
            Acir::BitSize bit_size;
            std::vector<uint8_t> value;

            friend bool operator==(const Const&, const Const&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(3);
                packer.pack(std::make_pair("destination", destination));
                packer.pack(std::make_pair("bit_size", bit_size));
                packer.pack(std::make_pair("value", value));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "Const";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "destination", destination, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "bit_size", bit_size, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "value", value, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "destination", destination, 0);
                    Helpers::conv_fld_from_array(array, name, "bit_size", bit_size, 1);
                    Helpers::conv_fld_from_array(array, name, "value", value, 2);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        struct IndirectConst {
            Acir::MemoryAddress destination_pointer;
            Acir::BitSize bit_size;
            std::vector<uint8_t> value;

            friend bool operator==(const IndirectConst&, const IndirectConst&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(3);
                packer.pack(std::make_pair("destination_pointer", destination_pointer));
                packer.pack(std::make_pair("bit_size", bit_size));
                packer.pack(std::make_pair("value", value));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "IndirectConst";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "destination_pointer", destination_pointer, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "bit_size", bit_size, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "value", value, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "destination_pointer", destination_pointer, 0);
                    Helpers::conv_fld_from_array(array, name, "bit_size", bit_size, 1);
                    Helpers::conv_fld_from_array(array, name, "value", value, 2);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        struct Return {
            friend bool operator==(const Return&, const Return&);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct ForeignCall {
            std::string function;
            std::vector<Acir::ValueOrArray> destinations;
            std::vector<Acir::HeapValueType> destination_value_types;
            std::vector<Acir::ValueOrArray> inputs;
            std::vector<Acir::HeapValueType> input_value_types;

            friend bool operator==(const ForeignCall&, const ForeignCall&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(5);
                packer.pack(std::make_pair("function", function));
                packer.pack(std::make_pair("destinations", destinations));
                packer.pack(std::make_pair("destination_value_types", destination_value_types));
                packer.pack(std::make_pair("inputs", inputs));
                packer.pack(std::make_pair("input_value_types", input_value_types));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "ForeignCall";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "function", function, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "destinations", destinations, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "destination_value_types", destination_value_types, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "inputs", inputs, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "input_value_types", input_value_types, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "function", function, 0);
                    Helpers::conv_fld_from_array(array, name, "destinations", destinations, 1);
                    Helpers::conv_fld_from_array(array, name, "destination_value_types", destination_value_types, 2);
                    Helpers::conv_fld_from_array(array, name, "inputs", inputs, 3);
                    Helpers::conv_fld_from_array(array, name, "input_value_types", input_value_types, 4);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        struct Mov {
            Acir::MemoryAddress destination;
            Acir::MemoryAddress source;

            friend bool operator==(const Mov&, const Mov&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(2);
                packer.pack(std::make_pair("destination", destination));
                packer.pack(std::make_pair("source", source));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "Mov";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "destination", destination, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "source", source, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "destination", destination, 0);
                    Helpers::conv_fld_from_array(array, name, "source", source, 1);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        struct ConditionalMov {
            Acir::MemoryAddress destination;
            Acir::MemoryAddress source_a;
            Acir::MemoryAddress source_b;
            Acir::MemoryAddress condition;

            friend bool operator==(const ConditionalMov&, const ConditionalMov&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(4);
                packer.pack(std::make_pair("destination", destination));
                packer.pack(std::make_pair("source_a", source_a));
                packer.pack(std::make_pair("source_b", source_b));
                packer.pack(std::make_pair("condition", condition));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "ConditionalMov";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "destination", destination, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "source_a", source_a, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "source_b", source_b, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "condition", condition, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "destination", destination, 0);
                    Helpers::conv_fld_from_array(array, name, "source_a", source_a, 1);
                    Helpers::conv_fld_from_array(array, name, "source_b", source_b, 2);
                    Helpers::conv_fld_from_array(array, name, "condition", condition, 3);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        struct Load {
            Acir::MemoryAddress destination;
            Acir::MemoryAddress source_pointer;

            friend bool operator==(const Load&, const Load&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(2);
                packer.pack(std::make_pair("destination", destination));
                packer.pack(std::make_pair("source_pointer", source_pointer));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "Load";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "destination", destination, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "source_pointer", source_pointer, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "destination", destination, 0);
                    Helpers::conv_fld_from_array(array, name, "source_pointer", source_pointer, 1);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        struct Store {
            Acir::MemoryAddress destination_pointer;
            Acir::MemoryAddress source;

            friend bool operator==(const Store&, const Store&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(2);
                packer.pack(std::make_pair("destination_pointer", destination_pointer));
                packer.pack(std::make_pair("source", source));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "Store";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "destination_pointer", destination_pointer, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "source", source, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "destination_pointer", destination_pointer, 0);
                    Helpers::conv_fld_from_array(array, name, "source", source, 1);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        struct BlackBox {
            Acir::BlackBoxOp value;

            friend bool operator==(const BlackBox&, const BlackBox&);

            void msgpack_pack(auto& packer) const { packer.pack(value); }

            void msgpack_unpack(msgpack::object const& o) {
                try {
                    o.convert(value);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into newtype 'BlackBox'");
                }
            }
        };

        struct Trap {
            Acir::HeapVector revert_data;

            friend bool operator==(const Trap&, const Trap&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(1);
                packer.pack(std::make_pair("revert_data", revert_data));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "Trap";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "revert_data", revert_data, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "revert_data", revert_data, 0);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        struct Stop {
            Acir::HeapVector return_data;

            friend bool operator==(const Stop&, const Stop&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(1);
                packer.pack(std::make_pair("return_data", return_data));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "Stop";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "return_data", return_data, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "return_data", return_data, 0);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        std::variant<BinaryFieldOp, BinaryIntOp, Not, Cast, JumpIf, Jump, CalldataCopy, Call, Const, IndirectConst, Return, ForeignCall, Mov, ConditionalMov, Load, Store, BlackBox, Trap, Stop> value;

        friend bool operator==(const BrilligOpcode&, const BrilligOpcode&);

        void msgpack_pack(auto& packer) const {
            std::string tag;
            bool is_unit;
            switch (value.index()) {
                
                case 0:
                    tag = "BinaryFieldOp";
                    is_unit = false;
                    break;
                case 1:
                    tag = "BinaryIntOp";
                    is_unit = false;
                    break;
                case 2:
                    tag = "Not";
                    is_unit = false;
                    break;
                case 3:
                    tag = "Cast";
                    is_unit = false;
                    break;
                case 4:
                    tag = "JumpIf";
                    is_unit = false;
                    break;
                case 5:
                    tag = "Jump";
                    is_unit = false;
                    break;
                case 6:
                    tag = "CalldataCopy";
                    is_unit = false;
                    break;
                case 7:
                    tag = "Call";
                    is_unit = false;
                    break;
                case 8:
                    tag = "Const";
                    is_unit = false;
                    break;
                case 9:
                    tag = "IndirectConst";
                    is_unit = false;
                    break;
                case 10:
                    tag = "Return";
                    is_unit = true;
                    break;
                case 11:
                    tag = "ForeignCall";
                    is_unit = false;
                    break;
                case 12:
                    tag = "Mov";
                    is_unit = false;
                    break;
                case 13:
                    tag = "ConditionalMov";
                    is_unit = false;
                    break;
                case 14:
                    tag = "Load";
                    is_unit = false;
                    break;
                case 15:
                    tag = "Store";
                    is_unit = false;
                    break;
                case 16:
                    tag = "BlackBox";
                    is_unit = false;
                    break;
                case 17:
                    tag = "Trap";
                    is_unit = false;
                    break;
                case 18:
                    tag = "Stop";
                    is_unit = false;
                    break;
                default:
                    throw_or_abort("unknown enum 'BrilligOpcode' variant index: " + std::to_string(value.index()));
            }
            if (is_unit) {
                packer.pack(tag);
            } else {
                std::visit([&packer, tag](const auto& arg) {
                    packer.pack_map(1);
                    packer.pack(tag);
                    arg.msgpack_pack(packer);
                }, value);
            }
        }

        void msgpack_unpack(msgpack::object const& o) {

            if (o.type != msgpack::type::object_type::MAP && o.type != msgpack::type::object_type::STR) {
                std::cerr << o << std::endl;
                throw_or_abort("expected MAP or STR for enum 'BrilligOpcode'; got type " + std::to_string(o.type));
            }
            if (o.type == msgpack::type::object_type::MAP && o.via.map.size != 1) {
                throw_or_abort("expected 1 entry for enum 'BrilligOpcode'; got " + std::to_string(o.via.map.size));
            }
            std::string tag;
            try {
                if (o.type == msgpack::type::object_type::MAP) {
                    o.via.map.ptr[0].key.convert(tag);
                } else {
                    o.convert(tag);
                }
            } catch(const msgpack::type_error&) {
                std::cerr << o << std::endl;
                throw_or_abort("error converting tag to string for enum 'BrilligOpcode'");
            }
            if (tag == "BinaryFieldOp") {
                BinaryFieldOp v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BrilligOpcode::BinaryFieldOp'");
                }
                
                value = v;
            }
            else if (tag == "BinaryIntOp") {
                BinaryIntOp v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BrilligOpcode::BinaryIntOp'");
                }
                
                value = v;
            }
            else if (tag == "Not") {
                Not v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BrilligOpcode::Not'");
                }
                
                value = v;
            }
            else if (tag == "Cast") {
                Cast v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BrilligOpcode::Cast'");
                }
                
                value = v;
            }
            else if (tag == "JumpIf") {
                JumpIf v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BrilligOpcode::JumpIf'");
                }
                
                value = v;
            }
            else if (tag == "Jump") {
                Jump v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BrilligOpcode::Jump'");
                }
                
                value = v;
            }
            else if (tag == "CalldataCopy") {
                CalldataCopy v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BrilligOpcode::CalldataCopy'");
                }
                
                value = v;
            }
            else if (tag == "Call") {
                Call v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BrilligOpcode::Call'");
                }
                
                value = v;
            }
            else if (tag == "Const") {
                Const v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BrilligOpcode::Const'");
                }
                
                value = v;
            }
            else if (tag == "IndirectConst") {
                IndirectConst v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BrilligOpcode::IndirectConst'");
                }
                
                value = v;
            }
            else if (tag == "Return") {
                Return v;
                value = v;
            }
            else if (tag == "ForeignCall") {
                ForeignCall v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BrilligOpcode::ForeignCall'");
                }
                
                value = v;
            }
            else if (tag == "Mov") {
                Mov v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BrilligOpcode::Mov'");
                }
                
                value = v;
            }
            else if (tag == "ConditionalMov") {
                ConditionalMov v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BrilligOpcode::ConditionalMov'");
                }
                
                value = v;
            }
            else if (tag == "Load") {
                Load v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BrilligOpcode::Load'");
                }
                
                value = v;
            }
            else if (tag == "Store") {
                Store v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BrilligOpcode::Store'");
                }
                
                value = v;
            }
            else if (tag == "BlackBox") {
                BlackBox v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BrilligOpcode::BlackBox'");
                }
                
                value = v;
            }
            else if (tag == "Trap") {
                Trap v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BrilligOpcode::Trap'");
                }
                
                value = v;
            }
            else if (tag == "Stop") {
                Stop v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BrilligOpcode::Stop'");
                }
                
                value = v;
            }
            else {
                std::cerr << o << std::endl;
                throw_or_abort("unknown 'BrilligOpcode' enum variant: " + tag);
            }
        }
    };

    struct Witness {
        uint32_t value;

        friend bool operator==(const Witness&, const Witness&);

        void msgpack_pack(auto& packer) const { packer.pack(value); }

        void msgpack_unpack(msgpack::object const& o) {
            try {
                o.convert(value);
            } catch (const msgpack::type_error&) {
                std::cerr << o << std::endl;
                throw_or_abort("error converting into newtype 'Witness'");
            }
        }
    };

    struct FunctionInput {

        struct Constant {
            std::vector<uint8_t> value;

            friend bool operator==(const Constant&, const Constant&);

            void msgpack_pack(auto& packer) const { packer.pack(value); }

            void msgpack_unpack(msgpack::object const& o) {
                try {
                    o.convert(value);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into newtype 'Constant'");
                }
            }
        };

        struct Witness {
            Acir::Witness value;

            friend bool operator==(const Witness&, const Witness&);

            void msgpack_pack(auto& packer) const { packer.pack(value); }

            void msgpack_unpack(msgpack::object const& o) {
                try {
                    o.convert(value);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into newtype 'Witness'");
                }
            }
        };

        std::variant<Constant, Witness> value;

        friend bool operator==(const FunctionInput&, const FunctionInput&);

        void msgpack_pack(auto& packer) const {
            std::string tag;
            bool is_unit;
            switch (value.index()) {
                
                case 0:
                    tag = "Constant";
                    is_unit = false;
                    break;
                case 1:
                    tag = "Witness";
                    is_unit = false;
                    break;
                default:
                    throw_or_abort("unknown enum 'FunctionInput' variant index: " + std::to_string(value.index()));
            }
            if (is_unit) {
                packer.pack(tag);
            } else {
                std::visit([&packer, tag](const auto& arg) {
                    packer.pack_map(1);
                    packer.pack(tag);
                    arg.msgpack_pack(packer);
                }, value);
            }
        }

        void msgpack_unpack(msgpack::object const& o) {

            if (o.type != msgpack::type::object_type::MAP && o.type != msgpack::type::object_type::STR) {
                std::cerr << o << std::endl;
                throw_or_abort("expected MAP or STR for enum 'FunctionInput'; got type " + std::to_string(o.type));
            }
            if (o.type == msgpack::type::object_type::MAP && o.via.map.size != 1) {
                throw_or_abort("expected 1 entry for enum 'FunctionInput'; got " + std::to_string(o.via.map.size));
            }
            std::string tag;
            try {
                if (o.type == msgpack::type::object_type::MAP) {
                    o.via.map.ptr[0].key.convert(tag);
                } else {
                    o.convert(tag);
                }
            } catch(const msgpack::type_error&) {
                std::cerr << o << std::endl;
                throw_or_abort("error converting tag to string for enum 'FunctionInput'");
            }
            if (tag == "Constant") {
                Constant v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'FunctionInput::Constant'");
                }
                
                value = v;
            }
            else if (tag == "Witness") {
                Witness v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'FunctionInput::Witness'");
                }
                
                value = v;
            }
            else {
                std::cerr << o << std::endl;
                throw_or_abort("unknown 'FunctionInput' enum variant: " + tag);
            }
        }
    };

    struct BlackBoxFuncCall {

        struct AES128Encrypt {
            std::vector<Acir::FunctionInput> inputs;
            std::shared_ptr<std::array<Acir::FunctionInput, 16>> iv;
            std::shared_ptr<std::array<Acir::FunctionInput, 16>> key;
            std::vector<Acir::Witness> outputs;

            friend bool operator==(const AES128Encrypt&, const AES128Encrypt&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(4);
                packer.pack(std::make_pair("inputs", inputs));
                packer.pack(std::make_pair("iv", iv));
                packer.pack(std::make_pair("key", key));
                packer.pack(std::make_pair("outputs", outputs));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "AES128Encrypt";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "inputs", inputs, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "iv", iv, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "key", key, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "outputs", outputs, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "inputs", inputs, 0);
                    Helpers::conv_fld_from_array(array, name, "iv", iv, 1);
                    Helpers::conv_fld_from_array(array, name, "key", key, 2);
                    Helpers::conv_fld_from_array(array, name, "outputs", outputs, 3);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        struct AND {
            Acir::FunctionInput lhs;
            Acir::FunctionInput rhs;
            uint32_t num_bits;
            Acir::Witness output;

            friend bool operator==(const AND&, const AND&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(4);
                packer.pack(std::make_pair("lhs", lhs));
                packer.pack(std::make_pair("rhs", rhs));
                packer.pack(std::make_pair("num_bits", num_bits));
                packer.pack(std::make_pair("output", output));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "AND";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "lhs", lhs, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "rhs", rhs, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "num_bits", num_bits, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "output", output, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "lhs", lhs, 0);
                    Helpers::conv_fld_from_array(array, name, "rhs", rhs, 1);
                    Helpers::conv_fld_from_array(array, name, "num_bits", num_bits, 2);
                    Helpers::conv_fld_from_array(array, name, "output", output, 3);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        struct XOR {
            Acir::FunctionInput lhs;
            Acir::FunctionInput rhs;
            uint32_t num_bits;
            Acir::Witness output;

            friend bool operator==(const XOR&, const XOR&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(4);
                packer.pack(std::make_pair("lhs", lhs));
                packer.pack(std::make_pair("rhs", rhs));
                packer.pack(std::make_pair("num_bits", num_bits));
                packer.pack(std::make_pair("output", output));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "XOR";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "lhs", lhs, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "rhs", rhs, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "num_bits", num_bits, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "output", output, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "lhs", lhs, 0);
                    Helpers::conv_fld_from_array(array, name, "rhs", rhs, 1);
                    Helpers::conv_fld_from_array(array, name, "num_bits", num_bits, 2);
                    Helpers::conv_fld_from_array(array, name, "output", output, 3);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        struct RANGE {
            Acir::FunctionInput input;
            uint32_t num_bits;

            friend bool operator==(const RANGE&, const RANGE&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(2);
                packer.pack(std::make_pair("input", input));
                packer.pack(std::make_pair("num_bits", num_bits));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "RANGE";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "input", input, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "num_bits", num_bits, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "input", input, 0);
                    Helpers::conv_fld_from_array(array, name, "num_bits", num_bits, 1);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        struct Blake2s {
            std::vector<Acir::FunctionInput> inputs;
            std::shared_ptr<std::array<Acir::Witness, 32>> outputs;

            friend bool operator==(const Blake2s&, const Blake2s&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(2);
                packer.pack(std::make_pair("inputs", inputs));
                packer.pack(std::make_pair("outputs", outputs));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "Blake2s";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "inputs", inputs, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "outputs", outputs, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "inputs", inputs, 0);
                    Helpers::conv_fld_from_array(array, name, "outputs", outputs, 1);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        struct Blake3 {
            std::vector<Acir::FunctionInput> inputs;
            std::shared_ptr<std::array<Acir::Witness, 32>> outputs;

            friend bool operator==(const Blake3&, const Blake3&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(2);
                packer.pack(std::make_pair("inputs", inputs));
                packer.pack(std::make_pair("outputs", outputs));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "Blake3";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "inputs", inputs, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "outputs", outputs, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "inputs", inputs, 0);
                    Helpers::conv_fld_from_array(array, name, "outputs", outputs, 1);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        struct EcdsaSecp256k1 {
            std::shared_ptr<std::array<Acir::FunctionInput, 32>> public_key_x;
            std::shared_ptr<std::array<Acir::FunctionInput, 32>> public_key_y;
            std::shared_ptr<std::array<Acir::FunctionInput, 64>> signature;
            std::shared_ptr<std::array<Acir::FunctionInput, 32>> hashed_message;
            Acir::FunctionInput predicate;
            Acir::Witness output;

            friend bool operator==(const EcdsaSecp256k1&, const EcdsaSecp256k1&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(6);
                packer.pack(std::make_pair("public_key_x", public_key_x));
                packer.pack(std::make_pair("public_key_y", public_key_y));
                packer.pack(std::make_pair("signature", signature));
                packer.pack(std::make_pair("hashed_message", hashed_message));
                packer.pack(std::make_pair("predicate", predicate));
                packer.pack(std::make_pair("output", output));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "EcdsaSecp256k1";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "public_key_x", public_key_x, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "public_key_y", public_key_y, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "signature", signature, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "hashed_message", hashed_message, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "predicate", predicate, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "output", output, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "public_key_x", public_key_x, 0);
                    Helpers::conv_fld_from_array(array, name, "public_key_y", public_key_y, 1);
                    Helpers::conv_fld_from_array(array, name, "signature", signature, 2);
                    Helpers::conv_fld_from_array(array, name, "hashed_message", hashed_message, 3);
                    Helpers::conv_fld_from_array(array, name, "predicate", predicate, 4);
                    Helpers::conv_fld_from_array(array, name, "output", output, 5);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        struct EcdsaSecp256r1 {
            std::shared_ptr<std::array<Acir::FunctionInput, 32>> public_key_x;
            std::shared_ptr<std::array<Acir::FunctionInput, 32>> public_key_y;
            std::shared_ptr<std::array<Acir::FunctionInput, 64>> signature;
            std::shared_ptr<std::array<Acir::FunctionInput, 32>> hashed_message;
            Acir::FunctionInput predicate;
            Acir::Witness output;

            friend bool operator==(const EcdsaSecp256r1&, const EcdsaSecp256r1&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(6);
                packer.pack(std::make_pair("public_key_x", public_key_x));
                packer.pack(std::make_pair("public_key_y", public_key_y));
                packer.pack(std::make_pair("signature", signature));
                packer.pack(std::make_pair("hashed_message", hashed_message));
                packer.pack(std::make_pair("predicate", predicate));
                packer.pack(std::make_pair("output", output));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "EcdsaSecp256r1";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "public_key_x", public_key_x, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "public_key_y", public_key_y, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "signature", signature, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "hashed_message", hashed_message, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "predicate", predicate, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "output", output, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "public_key_x", public_key_x, 0);
                    Helpers::conv_fld_from_array(array, name, "public_key_y", public_key_y, 1);
                    Helpers::conv_fld_from_array(array, name, "signature", signature, 2);
                    Helpers::conv_fld_from_array(array, name, "hashed_message", hashed_message, 3);
                    Helpers::conv_fld_from_array(array, name, "predicate", predicate, 4);
                    Helpers::conv_fld_from_array(array, name, "output", output, 5);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        struct MultiScalarMul {
            std::vector<Acir::FunctionInput> points;
            std::vector<Acir::FunctionInput> scalars;
            Acir::FunctionInput predicate;
            std::shared_ptr<std::array<Acir::Witness, 3>> outputs;

            friend bool operator==(const MultiScalarMul&, const MultiScalarMul&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(4);
                packer.pack(std::make_pair("points", points));
                packer.pack(std::make_pair("scalars", scalars));
                packer.pack(std::make_pair("predicate", predicate));
                packer.pack(std::make_pair("outputs", outputs));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "MultiScalarMul";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "points", points, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "scalars", scalars, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "predicate", predicate, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "outputs", outputs, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "points", points, 0);
                    Helpers::conv_fld_from_array(array, name, "scalars", scalars, 1);
                    Helpers::conv_fld_from_array(array, name, "predicate", predicate, 2);
                    Helpers::conv_fld_from_array(array, name, "outputs", outputs, 3);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        struct EmbeddedCurveAdd {
            std::shared_ptr<std::array<Acir::FunctionInput, 3>> input1;
            std::shared_ptr<std::array<Acir::FunctionInput, 3>> input2;
            Acir::FunctionInput predicate;
            std::shared_ptr<std::array<Acir::Witness, 3>> outputs;

            friend bool operator==(const EmbeddedCurveAdd&, const EmbeddedCurveAdd&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(4);
                packer.pack(std::make_pair("input1", input1));
                packer.pack(std::make_pair("input2", input2));
                packer.pack(std::make_pair("predicate", predicate));
                packer.pack(std::make_pair("outputs", outputs));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "EmbeddedCurveAdd";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "input1", input1, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "input2", input2, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "predicate", predicate, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "outputs", outputs, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "input1", input1, 0);
                    Helpers::conv_fld_from_array(array, name, "input2", input2, 1);
                    Helpers::conv_fld_from_array(array, name, "predicate", predicate, 2);
                    Helpers::conv_fld_from_array(array, name, "outputs", outputs, 3);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        struct Keccakf1600 {
            std::shared_ptr<std::array<Acir::FunctionInput, 25>> inputs;
            std::shared_ptr<std::array<Acir::Witness, 25>> outputs;

            friend bool operator==(const Keccakf1600&, const Keccakf1600&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(2);
                packer.pack(std::make_pair("inputs", inputs));
                packer.pack(std::make_pair("outputs", outputs));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "Keccakf1600";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "inputs", inputs, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "outputs", outputs, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "inputs", inputs, 0);
                    Helpers::conv_fld_from_array(array, name, "outputs", outputs, 1);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        struct RecursiveAggregation {
            std::vector<Acir::FunctionInput> verification_key;
            std::vector<Acir::FunctionInput> proof;
            std::vector<Acir::FunctionInput> public_inputs;
            Acir::FunctionInput key_hash;
            uint32_t proof_type;
            Acir::FunctionInput predicate;

            friend bool operator==(const RecursiveAggregation&, const RecursiveAggregation&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(6);
                packer.pack(std::make_pair("verification_key", verification_key));
                packer.pack(std::make_pair("proof", proof));
                packer.pack(std::make_pair("public_inputs", public_inputs));
                packer.pack(std::make_pair("key_hash", key_hash));
                packer.pack(std::make_pair("proof_type", proof_type));
                packer.pack(std::make_pair("predicate", predicate));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "RecursiveAggregation";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "verification_key", verification_key, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "proof", proof, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "public_inputs", public_inputs, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "key_hash", key_hash, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "proof_type", proof_type, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "predicate", predicate, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "verification_key", verification_key, 0);
                    Helpers::conv_fld_from_array(array, name, "proof", proof, 1);
                    Helpers::conv_fld_from_array(array, name, "public_inputs", public_inputs, 2);
                    Helpers::conv_fld_from_array(array, name, "key_hash", key_hash, 3);
                    Helpers::conv_fld_from_array(array, name, "proof_type", proof_type, 4);
                    Helpers::conv_fld_from_array(array, name, "predicate", predicate, 5);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        struct Poseidon2Permutation {
            std::vector<Acir::FunctionInput> inputs;
            std::vector<Acir::Witness> outputs;

            friend bool operator==(const Poseidon2Permutation&, const Poseidon2Permutation&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(2);
                packer.pack(std::make_pair("inputs", inputs));
                packer.pack(std::make_pair("outputs", outputs));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "Poseidon2Permutation";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "inputs", inputs, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "outputs", outputs, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "inputs", inputs, 0);
                    Helpers::conv_fld_from_array(array, name, "outputs", outputs, 1);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        struct Sha256Compression {
            std::shared_ptr<std::array<Acir::FunctionInput, 16>> inputs;
            std::shared_ptr<std::array<Acir::FunctionInput, 8>> hash_values;
            std::shared_ptr<std::array<Acir::Witness, 8>> outputs;

            friend bool operator==(const Sha256Compression&, const Sha256Compression&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(3);
                packer.pack(std::make_pair("inputs", inputs));
                packer.pack(std::make_pair("hash_values", hash_values));
                packer.pack(std::make_pair("outputs", outputs));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "Sha256Compression";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "inputs", inputs, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "hash_values", hash_values, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "outputs", outputs, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "inputs", inputs, 0);
                    Helpers::conv_fld_from_array(array, name, "hash_values", hash_values, 1);
                    Helpers::conv_fld_from_array(array, name, "outputs", outputs, 2);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        std::variant<AES128Encrypt, AND, XOR, RANGE, Blake2s, Blake3, EcdsaSecp256k1, EcdsaSecp256r1, MultiScalarMul, EmbeddedCurveAdd, Keccakf1600, RecursiveAggregation, Poseidon2Permutation, Sha256Compression> value;

        friend bool operator==(const BlackBoxFuncCall&, const BlackBoxFuncCall&);

        void msgpack_pack(auto& packer) const {
            std::string tag;
            bool is_unit;
            switch (value.index()) {
                
                case 0:
                    tag = "AES128Encrypt";
                    is_unit = false;
                    break;
                case 1:
                    tag = "AND";
                    is_unit = false;
                    break;
                case 2:
                    tag = "XOR";
                    is_unit = false;
                    break;
                case 3:
                    tag = "RANGE";
                    is_unit = false;
                    break;
                case 4:
                    tag = "Blake2s";
                    is_unit = false;
                    break;
                case 5:
                    tag = "Blake3";
                    is_unit = false;
                    break;
                case 6:
                    tag = "EcdsaSecp256k1";
                    is_unit = false;
                    break;
                case 7:
                    tag = "EcdsaSecp256r1";
                    is_unit = false;
                    break;
                case 8:
                    tag = "MultiScalarMul";
                    is_unit = false;
                    break;
                case 9:
                    tag = "EmbeddedCurveAdd";
                    is_unit = false;
                    break;
                case 10:
                    tag = "Keccakf1600";
                    is_unit = false;
                    break;
                case 11:
                    tag = "RecursiveAggregation";
                    is_unit = false;
                    break;
                case 12:
                    tag = "Poseidon2Permutation";
                    is_unit = false;
                    break;
                case 13:
                    tag = "Sha256Compression";
                    is_unit = false;
                    break;
                default:
                    throw_or_abort("unknown enum 'BlackBoxFuncCall' variant index: " + std::to_string(value.index()));
            }
            if (is_unit) {
                packer.pack(tag);
            } else {
                std::visit([&packer, tag](const auto& arg) {
                    packer.pack_map(1);
                    packer.pack(tag);
                    arg.msgpack_pack(packer);
                }, value);
            }
        }

        void msgpack_unpack(msgpack::object const& o) {

            if (o.type != msgpack::type::object_type::MAP && o.type != msgpack::type::object_type::STR) {
                std::cerr << o << std::endl;
                throw_or_abort("expected MAP or STR for enum 'BlackBoxFuncCall'; got type " + std::to_string(o.type));
            }
            if (o.type == msgpack::type::object_type::MAP && o.via.map.size != 1) {
                throw_or_abort("expected 1 entry for enum 'BlackBoxFuncCall'; got " + std::to_string(o.via.map.size));
            }
            std::string tag;
            try {
                if (o.type == msgpack::type::object_type::MAP) {
                    o.via.map.ptr[0].key.convert(tag);
                } else {
                    o.convert(tag);
                }
            } catch(const msgpack::type_error&) {
                std::cerr << o << std::endl;
                throw_or_abort("error converting tag to string for enum 'BlackBoxFuncCall'");
            }
            if (tag == "AES128Encrypt") {
                AES128Encrypt v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BlackBoxFuncCall::AES128Encrypt'");
                }
                
                value = v;
            }
            else if (tag == "AND") {
                AND v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BlackBoxFuncCall::AND'");
                }
                
                value = v;
            }
            else if (tag == "XOR") {
                XOR v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BlackBoxFuncCall::XOR'");
                }
                
                value = v;
            }
            else if (tag == "RANGE") {
                RANGE v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BlackBoxFuncCall::RANGE'");
                }
                
                value = v;
            }
            else if (tag == "Blake2s") {
                Blake2s v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BlackBoxFuncCall::Blake2s'");
                }
                
                value = v;
            }
            else if (tag == "Blake3") {
                Blake3 v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BlackBoxFuncCall::Blake3'");
                }
                
                value = v;
            }
            else if (tag == "EcdsaSecp256k1") {
                EcdsaSecp256k1 v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BlackBoxFuncCall::EcdsaSecp256k1'");
                }
                
                value = v;
            }
            else if (tag == "EcdsaSecp256r1") {
                EcdsaSecp256r1 v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BlackBoxFuncCall::EcdsaSecp256r1'");
                }
                
                value = v;
            }
            else if (tag == "MultiScalarMul") {
                MultiScalarMul v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BlackBoxFuncCall::MultiScalarMul'");
                }
                
                value = v;
            }
            else if (tag == "EmbeddedCurveAdd") {
                EmbeddedCurveAdd v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BlackBoxFuncCall::EmbeddedCurveAdd'");
                }
                
                value = v;
            }
            else if (tag == "Keccakf1600") {
                Keccakf1600 v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BlackBoxFuncCall::Keccakf1600'");
                }
                
                value = v;
            }
            else if (tag == "RecursiveAggregation") {
                RecursiveAggregation v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BlackBoxFuncCall::RecursiveAggregation'");
                }
                
                value = v;
            }
            else if (tag == "Poseidon2Permutation") {
                Poseidon2Permutation v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BlackBoxFuncCall::Poseidon2Permutation'");
                }
                
                value = v;
            }
            else if (tag == "Sha256Compression") {
                Sha256Compression v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BlackBoxFuncCall::Sha256Compression'");
                }
                
                value = v;
            }
            else {
                std::cerr << o << std::endl;
                throw_or_abort("unknown 'BlackBoxFuncCall' enum variant: " + tag);
            }
        }
    };

    struct BlockId {
        uint32_t value;

        friend bool operator==(const BlockId&, const BlockId&);

        void msgpack_pack(auto& packer) const { packer.pack(value); }

        void msgpack_unpack(msgpack::object const& o) {
            try {
                o.convert(value);
            } catch (const msgpack::type_error&) {
                std::cerr << o << std::endl;
                throw_or_abort("error converting into newtype 'BlockId'");
            }
        }
    };

    struct BlockType {

        struct Memory {
            friend bool operator==(const Memory&, const Memory&);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct CallData {
            uint32_t value;

            friend bool operator==(const CallData&, const CallData&);

            void msgpack_pack(auto& packer) const { packer.pack(value); }

            void msgpack_unpack(msgpack::object const& o) {
                try {
                    o.convert(value);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into newtype 'CallData'");
                }
            }
        };

        struct ReturnData {
            friend bool operator==(const ReturnData&, const ReturnData&);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        std::variant<Memory, CallData, ReturnData> value;

        friend bool operator==(const BlockType&, const BlockType&);

        void msgpack_pack(auto& packer) const {
            std::string tag;
            bool is_unit;
            switch (value.index()) {
                
                case 0:
                    tag = "Memory";
                    is_unit = true;
                    break;
                case 1:
                    tag = "CallData";
                    is_unit = false;
                    break;
                case 2:
                    tag = "ReturnData";
                    is_unit = true;
                    break;
                default:
                    throw_or_abort("unknown enum 'BlockType' variant index: " + std::to_string(value.index()));
            }
            if (is_unit) {
                packer.pack(tag);
            } else {
                std::visit([&packer, tag](const auto& arg) {
                    packer.pack_map(1);
                    packer.pack(tag);
                    arg.msgpack_pack(packer);
                }, value);
            }
        }

        void msgpack_unpack(msgpack::object const& o) {

            if (o.type != msgpack::type::object_type::MAP && o.type != msgpack::type::object_type::STR) {
                std::cerr << o << std::endl;
                throw_or_abort("expected MAP or STR for enum 'BlockType'; got type " + std::to_string(o.type));
            }
            if (o.type == msgpack::type::object_type::MAP && o.via.map.size != 1) {
                throw_or_abort("expected 1 entry for enum 'BlockType'; got " + std::to_string(o.via.map.size));
            }
            std::string tag;
            try {
                if (o.type == msgpack::type::object_type::MAP) {
                    o.via.map.ptr[0].key.convert(tag);
                } else {
                    o.convert(tag);
                }
            } catch(const msgpack::type_error&) {
                std::cerr << o << std::endl;
                throw_or_abort("error converting tag to string for enum 'BlockType'");
            }
            if (tag == "Memory") {
                Memory v;
                value = v;
            }
            else if (tag == "CallData") {
                CallData v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BlockType::CallData'");
                }
                
                value = v;
            }
            else if (tag == "ReturnData") {
                ReturnData v;
                value = v;
            }
            else {
                std::cerr << o << std::endl;
                throw_or_abort("unknown 'BlockType' enum variant: " + tag);
            }
        }
    };

    struct Expression {
        std::vector<std::tuple<std::vector<uint8_t>, Acir::Witness, Acir::Witness>> mul_terms;
        std::vector<std::tuple<std::vector<uint8_t>, Acir::Witness>> linear_combinations;
        std::vector<uint8_t> q_c;

        friend bool operator==(const Expression&, const Expression&);

        void msgpack_pack(auto& packer) const {
            packer.pack_map(3);
            packer.pack(std::make_pair("mul_terms", mul_terms));
            packer.pack(std::make_pair("linear_combinations", linear_combinations));
            packer.pack(std::make_pair("q_c", q_c));
        }

        void msgpack_unpack(msgpack::object const& o) {
            std::string name = "Expression";
            if (o.type == msgpack::type::MAP) {
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "mul_terms", mul_terms, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "linear_combinations", linear_combinations, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "q_c", q_c, false);
            } else if (o.type == msgpack::type::ARRAY) {
                auto array = o.via.array; 
                Helpers::conv_fld_from_array(array, name, "mul_terms", mul_terms, 0);
                Helpers::conv_fld_from_array(array, name, "linear_combinations", linear_combinations, 1);
                Helpers::conv_fld_from_array(array, name, "q_c", q_c, 2);
            } else {
                throw_or_abort("expected MAP or ARRAY for " + name);
            }
        }
    };

    struct BrilligInputs {

        struct Single {
            Acir::Expression value;

            friend bool operator==(const Single&, const Single&);

            void msgpack_pack(auto& packer) const { packer.pack(value); }

            void msgpack_unpack(msgpack::object const& o) {
                try {
                    o.convert(value);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into newtype 'Single'");
                }
            }
        };

        struct Array {
            std::vector<Acir::Expression> value;

            friend bool operator==(const Array&, const Array&);

            void msgpack_pack(auto& packer) const { packer.pack(value); }

            void msgpack_unpack(msgpack::object const& o) {
                try {
                    o.convert(value);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into newtype 'Array'");
                }
            }
        };

        struct MemoryArray {
            Acir::BlockId value;

            friend bool operator==(const MemoryArray&, const MemoryArray&);

            void msgpack_pack(auto& packer) const { packer.pack(value); }

            void msgpack_unpack(msgpack::object const& o) {
                try {
                    o.convert(value);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into newtype 'MemoryArray'");
                }
            }
        };

        std::variant<Single, Array, MemoryArray> value;

        friend bool operator==(const BrilligInputs&, const BrilligInputs&);

        void msgpack_pack(auto& packer) const {
            std::string tag;
            bool is_unit;
            switch (value.index()) {
                
                case 0:
                    tag = "Single";
                    is_unit = false;
                    break;
                case 1:
                    tag = "Array";
                    is_unit = false;
                    break;
                case 2:
                    tag = "MemoryArray";
                    is_unit = false;
                    break;
                default:
                    throw_or_abort("unknown enum 'BrilligInputs' variant index: " + std::to_string(value.index()));
            }
            if (is_unit) {
                packer.pack(tag);
            } else {
                std::visit([&packer, tag](const auto& arg) {
                    packer.pack_map(1);
                    packer.pack(tag);
                    arg.msgpack_pack(packer);
                }, value);
            }
        }

        void msgpack_unpack(msgpack::object const& o) {

            if (o.type != msgpack::type::object_type::MAP && o.type != msgpack::type::object_type::STR) {
                std::cerr << o << std::endl;
                throw_or_abort("expected MAP or STR for enum 'BrilligInputs'; got type " + std::to_string(o.type));
            }
            if (o.type == msgpack::type::object_type::MAP && o.via.map.size != 1) {
                throw_or_abort("expected 1 entry for enum 'BrilligInputs'; got " + std::to_string(o.via.map.size));
            }
            std::string tag;
            try {
                if (o.type == msgpack::type::object_type::MAP) {
                    o.via.map.ptr[0].key.convert(tag);
                } else {
                    o.convert(tag);
                }
            } catch(const msgpack::type_error&) {
                std::cerr << o << std::endl;
                throw_or_abort("error converting tag to string for enum 'BrilligInputs'");
            }
            if (tag == "Single") {
                Single v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BrilligInputs::Single'");
                }
                
                value = v;
            }
            else if (tag == "Array") {
                Array v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BrilligInputs::Array'");
                }
                
                value = v;
            }
            else if (tag == "MemoryArray") {
                MemoryArray v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BrilligInputs::MemoryArray'");
                }
                
                value = v;
            }
            else {
                std::cerr << o << std::endl;
                throw_or_abort("unknown 'BrilligInputs' enum variant: " + tag);
            }
        }
    };

    struct BrilligOutputs {

        struct Simple {
            Acir::Witness value;

            friend bool operator==(const Simple&, const Simple&);

            void msgpack_pack(auto& packer) const { packer.pack(value); }

            void msgpack_unpack(msgpack::object const& o) {
                try {
                    o.convert(value);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into newtype 'Simple'");
                }
            }
        };

        struct Array {
            std::vector<Acir::Witness> value;

            friend bool operator==(const Array&, const Array&);

            void msgpack_pack(auto& packer) const { packer.pack(value); }

            void msgpack_unpack(msgpack::object const& o) {
                try {
                    o.convert(value);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into newtype 'Array'");
                }
            }
        };

        std::variant<Simple, Array> value;

        friend bool operator==(const BrilligOutputs&, const BrilligOutputs&);

        void msgpack_pack(auto& packer) const {
            std::string tag;
            bool is_unit;
            switch (value.index()) {
                
                case 0:
                    tag = "Simple";
                    is_unit = false;
                    break;
                case 1:
                    tag = "Array";
                    is_unit = false;
                    break;
                default:
                    throw_or_abort("unknown enum 'BrilligOutputs' variant index: " + std::to_string(value.index()));
            }
            if (is_unit) {
                packer.pack(tag);
            } else {
                std::visit([&packer, tag](const auto& arg) {
                    packer.pack_map(1);
                    packer.pack(tag);
                    arg.msgpack_pack(packer);
                }, value);
            }
        }

        void msgpack_unpack(msgpack::object const& o) {

            if (o.type != msgpack::type::object_type::MAP && o.type != msgpack::type::object_type::STR) {
                std::cerr << o << std::endl;
                throw_or_abort("expected MAP or STR for enum 'BrilligOutputs'; got type " + std::to_string(o.type));
            }
            if (o.type == msgpack::type::object_type::MAP && o.via.map.size != 1) {
                throw_or_abort("expected 1 entry for enum 'BrilligOutputs'; got " + std::to_string(o.via.map.size));
            }
            std::string tag;
            try {
                if (o.type == msgpack::type::object_type::MAP) {
                    o.via.map.ptr[0].key.convert(tag);
                } else {
                    o.convert(tag);
                }
            } catch(const msgpack::type_error&) {
                std::cerr << o << std::endl;
                throw_or_abort("error converting tag to string for enum 'BrilligOutputs'");
            }
            if (tag == "Simple") {
                Simple v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BrilligOutputs::Simple'");
                }
                
                value = v;
            }
            else if (tag == "Array") {
                Array v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BrilligOutputs::Array'");
                }
                
                value = v;
            }
            else {
                std::cerr << o << std::endl;
                throw_or_abort("unknown 'BrilligOutputs' enum variant: " + tag);
            }
        }
    };

    struct MemOp {
        Acir::Expression operation;
        Acir::Expression index;
        Acir::Expression value;

        friend bool operator==(const MemOp&, const MemOp&);

        void msgpack_pack(auto& packer) const {
            packer.pack_map(3);
            packer.pack(std::make_pair("operation", operation));
            packer.pack(std::make_pair("index", index));
            packer.pack(std::make_pair("value", value));
        }

        void msgpack_unpack(msgpack::object const& o) {
            std::string name = "MemOp";
            if (o.type == msgpack::type::MAP) {
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "operation", operation, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "index", index, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "value", value, false);
            } else if (o.type == msgpack::type::ARRAY) {
                auto array = o.via.array; 
                Helpers::conv_fld_from_array(array, name, "operation", operation, 0);
                Helpers::conv_fld_from_array(array, name, "index", index, 1);
                Helpers::conv_fld_from_array(array, name, "value", value, 2);
            } else {
                throw_or_abort("expected MAP or ARRAY for " + name);
            }
        }
    };

    struct Opcode {

        struct AssertZero {
            Acir::Expression value;

            friend bool operator==(const AssertZero&, const AssertZero&);

            void msgpack_pack(auto& packer) const { packer.pack(value); }

            void msgpack_unpack(msgpack::object const& o) {
                try {
                    o.convert(value);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into newtype 'AssertZero'");
                }
            }
        };

        struct BlackBoxFuncCall {
            Acir::BlackBoxFuncCall value;

            friend bool operator==(const BlackBoxFuncCall&, const BlackBoxFuncCall&);

            void msgpack_pack(auto& packer) const { packer.pack(value); }

            void msgpack_unpack(msgpack::object const& o) {
                try {
                    o.convert(value);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into newtype 'BlackBoxFuncCall'");
                }
            }
        };

        struct MemoryOp {
            Acir::BlockId block_id;
            Acir::MemOp op;

            friend bool operator==(const MemoryOp&, const MemoryOp&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(2);
                packer.pack(std::make_pair("block_id", block_id));
                packer.pack(std::make_pair("op", op));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "MemoryOp";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "block_id", block_id, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "op", op, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "block_id", block_id, 0);
                    Helpers::conv_fld_from_array(array, name, "op", op, 1);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        struct MemoryInit {
            Acir::BlockId block_id;
            std::vector<Acir::Witness> init;
            Acir::BlockType block_type;

            friend bool operator==(const MemoryInit&, const MemoryInit&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(3);
                packer.pack(std::make_pair("block_id", block_id));
                packer.pack(std::make_pair("init", init));
                packer.pack(std::make_pair("block_type", block_type));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "MemoryInit";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "block_id", block_id, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "init", init, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "block_type", block_type, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "block_id", block_id, 0);
                    Helpers::conv_fld_from_array(array, name, "init", init, 1);
                    Helpers::conv_fld_from_array(array, name, "block_type", block_type, 2);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        struct BrilligCall {
            uint32_t id;
            std::vector<Acir::BrilligInputs> inputs;
            std::vector<Acir::BrilligOutputs> outputs;
            std::optional<Acir::Expression> predicate;

            friend bool operator==(const BrilligCall&, const BrilligCall&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(4);
                packer.pack(std::make_pair("id", id));
                packer.pack(std::make_pair("inputs", inputs));
                packer.pack(std::make_pair("outputs", outputs));
                packer.pack(std::make_pair("predicate", predicate));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "BrilligCall";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "id", id, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "inputs", inputs, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "outputs", outputs, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "predicate", predicate, true);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "id", id, 0);
                    Helpers::conv_fld_from_array(array, name, "inputs", inputs, 1);
                    Helpers::conv_fld_from_array(array, name, "outputs", outputs, 2);
                    Helpers::conv_fld_from_array(array, name, "predicate", predicate, 3);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        struct Call {
            uint32_t id;
            std::vector<Acir::Witness> inputs;
            std::vector<Acir::Witness> outputs;
            std::optional<Acir::Expression> predicate;

            friend bool operator==(const Call&, const Call&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(4);
                packer.pack(std::make_pair("id", id));
                packer.pack(std::make_pair("inputs", inputs));
                packer.pack(std::make_pair("outputs", outputs));
                packer.pack(std::make_pair("predicate", predicate));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "Call";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "id", id, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "inputs", inputs, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "outputs", outputs, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "predicate", predicate, true);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "id", id, 0);
                    Helpers::conv_fld_from_array(array, name, "inputs", inputs, 1);
                    Helpers::conv_fld_from_array(array, name, "outputs", outputs, 2);
                    Helpers::conv_fld_from_array(array, name, "predicate", predicate, 3);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        std::variant<AssertZero, BlackBoxFuncCall, MemoryOp, MemoryInit, BrilligCall, Call> value;

        friend bool operator==(const Opcode&, const Opcode&);

        void msgpack_pack(auto& packer) const {
            std::string tag;
            bool is_unit;
            switch (value.index()) {
                
                case 0:
                    tag = "AssertZero";
                    is_unit = false;
                    break;
                case 1:
                    tag = "BlackBoxFuncCall";
                    is_unit = false;
                    break;
                case 2:
                    tag = "MemoryOp";
                    is_unit = false;
                    break;
                case 3:
                    tag = "MemoryInit";
                    is_unit = false;
                    break;
                case 4:
                    tag = "BrilligCall";
                    is_unit = false;
                    break;
                case 5:
                    tag = "Call";
                    is_unit = false;
                    break;
                default:
                    throw_or_abort("unknown enum 'Opcode' variant index: " + std::to_string(value.index()));
            }
            if (is_unit) {
                packer.pack(tag);
            } else {
                std::visit([&packer, tag](const auto& arg) {
                    packer.pack_map(1);
                    packer.pack(tag);
                    arg.msgpack_pack(packer);
                }, value);
            }
        }

        void msgpack_unpack(msgpack::object const& o) {

            if (o.type != msgpack::type::object_type::MAP && o.type != msgpack::type::object_type::STR) {
                std::cerr << o << std::endl;
                throw_or_abort("expected MAP or STR for enum 'Opcode'; got type " + std::to_string(o.type));
            }
            if (o.type == msgpack::type::object_type::MAP && o.via.map.size != 1) {
                throw_or_abort("expected 1 entry for enum 'Opcode'; got " + std::to_string(o.via.map.size));
            }
            std::string tag;
            try {
                if (o.type == msgpack::type::object_type::MAP) {
                    o.via.map.ptr[0].key.convert(tag);
                } else {
                    o.convert(tag);
                }
            } catch(const msgpack::type_error&) {
                std::cerr << o << std::endl;
                throw_or_abort("error converting tag to string for enum 'Opcode'");
            }
            if (tag == "AssertZero") {
                AssertZero v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'Opcode::AssertZero'");
                }
                
                value = v;
            }
            else if (tag == "BlackBoxFuncCall") {
                BlackBoxFuncCall v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'Opcode::BlackBoxFuncCall'");
                }
                
                value = v;
            }
            else if (tag == "MemoryOp") {
                MemoryOp v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'Opcode::MemoryOp'");
                }
                
                value = v;
            }
            else if (tag == "MemoryInit") {
                MemoryInit v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'Opcode::MemoryInit'");
                }
                
                value = v;
            }
            else if (tag == "BrilligCall") {
                BrilligCall v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'Opcode::BrilligCall'");
                }
                
                value = v;
            }
            else if (tag == "Call") {
                Call v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'Opcode::Call'");
                }
                
                value = v;
            }
            else {
                std::cerr << o << std::endl;
                throw_or_abort("unknown 'Opcode' enum variant: " + tag);
            }
        }
    };

    struct ExpressionOrMemory {

        struct Expression {
            Acir::Expression value;

            friend bool operator==(const Expression&, const Expression&);

            void msgpack_pack(auto& packer) const { packer.pack(value); }

            void msgpack_unpack(msgpack::object const& o) {
                try {
                    o.convert(value);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into newtype 'Expression'");
                }
            }
        };

        struct Memory {
            Acir::BlockId value;

            friend bool operator==(const Memory&, const Memory&);

            void msgpack_pack(auto& packer) const { packer.pack(value); }

            void msgpack_unpack(msgpack::object const& o) {
                try {
                    o.convert(value);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into newtype 'Memory'");
                }
            }
        };

        std::variant<Expression, Memory> value;

        friend bool operator==(const ExpressionOrMemory&, const ExpressionOrMemory&);

        void msgpack_pack(auto& packer) const {
            std::string tag;
            bool is_unit;
            switch (value.index()) {
                
                case 0:
                    tag = "Expression";
                    is_unit = false;
                    break;
                case 1:
                    tag = "Memory";
                    is_unit = false;
                    break;
                default:
                    throw_or_abort("unknown enum 'ExpressionOrMemory' variant index: " + std::to_string(value.index()));
            }
            if (is_unit) {
                packer.pack(tag);
            } else {
                std::visit([&packer, tag](const auto& arg) {
                    packer.pack_map(1);
                    packer.pack(tag);
                    arg.msgpack_pack(packer);
                }, value);
            }
        }

        void msgpack_unpack(msgpack::object const& o) {

            if (o.type != msgpack::type::object_type::MAP && o.type != msgpack::type::object_type::STR) {
                std::cerr << o << std::endl;
                throw_or_abort("expected MAP or STR for enum 'ExpressionOrMemory'; got type " + std::to_string(o.type));
            }
            if (o.type == msgpack::type::object_type::MAP && o.via.map.size != 1) {
                throw_or_abort("expected 1 entry for enum 'ExpressionOrMemory'; got " + std::to_string(o.via.map.size));
            }
            std::string tag;
            try {
                if (o.type == msgpack::type::object_type::MAP) {
                    o.via.map.ptr[0].key.convert(tag);
                } else {
                    o.convert(tag);
                }
            } catch(const msgpack::type_error&) {
                std::cerr << o << std::endl;
                throw_or_abort("error converting tag to string for enum 'ExpressionOrMemory'");
            }
            if (tag == "Expression") {
                Expression v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'ExpressionOrMemory::Expression'");
                }
                
                value = v;
            }
            else if (tag == "Memory") {
                Memory v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'ExpressionOrMemory::Memory'");
                }
                
                value = v;
            }
            else {
                std::cerr << o << std::endl;
                throw_or_abort("unknown 'ExpressionOrMemory' enum variant: " + tag);
            }
        }
    };

    struct AssertionPayload {
        uint64_t error_selector;
        std::vector<Acir::ExpressionOrMemory> payload;

        friend bool operator==(const AssertionPayload&, const AssertionPayload&);

        void msgpack_pack(auto& packer) const {
            packer.pack_map(2);
            packer.pack(std::make_pair("error_selector", error_selector));
            packer.pack(std::make_pair("payload", payload));
        }

        void msgpack_unpack(msgpack::object const& o) {
            std::string name = "AssertionPayload";
            if (o.type == msgpack::type::MAP) {
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "error_selector", error_selector, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "payload", payload, false);
            } else if (o.type == msgpack::type::ARRAY) {
                auto array = o.via.array; 
                Helpers::conv_fld_from_array(array, name, "error_selector", error_selector, 0);
                Helpers::conv_fld_from_array(array, name, "payload", payload, 1);
            } else {
                throw_or_abort("expected MAP or ARRAY for " + name);
            }
        }
    };

    struct OpcodeLocation {

        struct Acir {
            uint64_t value;

            friend bool operator==(const Acir&, const Acir&);

            void msgpack_pack(auto& packer) const { packer.pack(value); }

            void msgpack_unpack(msgpack::object const& o) {
                try {
                    o.convert(value);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into newtype 'Acir'");
                }
            }
        };

        struct Brillig {
            uint64_t acir_index;
            uint64_t brillig_index;

            friend bool operator==(const Brillig&, const Brillig&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(2);
                packer.pack(std::make_pair("acir_index", acir_index));
                packer.pack(std::make_pair("brillig_index", brillig_index));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "Brillig";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "acir_index", acir_index, false);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "brillig_index", brillig_index, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "acir_index", acir_index, 0);
                    Helpers::conv_fld_from_array(array, name, "brillig_index", brillig_index, 1);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        std::variant<Acir, Brillig> value;

        friend bool operator==(const OpcodeLocation&, const OpcodeLocation&);

        void msgpack_pack(auto& packer) const {
            std::string tag;
            bool is_unit;
            switch (value.index()) {
                
                case 0:
                    tag = "Acir";
                    is_unit = false;
                    break;
                case 1:
                    tag = "Brillig";
                    is_unit = false;
                    break;
                default:
                    throw_or_abort("unknown enum 'OpcodeLocation' variant index: " + std::to_string(value.index()));
            }
            if (is_unit) {
                packer.pack(tag);
            } else {
                std::visit([&packer, tag](const auto& arg) {
                    packer.pack_map(1);
                    packer.pack(tag);
                    arg.msgpack_pack(packer);
                }, value);
            }
        }

        void msgpack_unpack(msgpack::object const& o) {

            if (o.type != msgpack::type::object_type::MAP && o.type != msgpack::type::object_type::STR) {
                std::cerr << o << std::endl;
                throw_or_abort("expected MAP or STR for enum 'OpcodeLocation'; got type " + std::to_string(o.type));
            }
            if (o.type == msgpack::type::object_type::MAP && o.via.map.size != 1) {
                throw_or_abort("expected 1 entry for enum 'OpcodeLocation'; got " + std::to_string(o.via.map.size));
            }
            std::string tag;
            try {
                if (o.type == msgpack::type::object_type::MAP) {
                    o.via.map.ptr[0].key.convert(tag);
                } else {
                    o.convert(tag);
                }
            } catch(const msgpack::type_error&) {
                std::cerr << o << std::endl;
                throw_or_abort("error converting tag to string for enum 'OpcodeLocation'");
            }
            if (tag == "Acir") {
                Acir v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'OpcodeLocation::Acir'");
                }
                
                value = v;
            }
            else if (tag == "Brillig") {
                Brillig v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'OpcodeLocation::Brillig'");
                }
                
                value = v;
            }
            else {
                std::cerr << o << std::endl;
                throw_or_abort("unknown 'OpcodeLocation' enum variant: " + tag);
            }
        }
    };

    struct PublicInputs {
        std::vector<Acir::Witness> value;

        friend bool operator==(const PublicInputs&, const PublicInputs&);

        void msgpack_pack(auto& packer) const { packer.pack(value); }

        void msgpack_unpack(msgpack::object const& o) {
            try {
                o.convert(value);
            } catch (const msgpack::type_error&) {
                std::cerr << o << std::endl;
                throw_or_abort("error converting into newtype 'PublicInputs'");
            }
        }
    };

    struct Circuit {
        std::string function_name;
        uint32_t current_witness_index;
        std::vector<Acir::Opcode> opcodes;
        std::vector<Acir::Witness> private_parameters;
        Acir::PublicInputs public_parameters;
        Acir::PublicInputs return_values;
        std::vector<std::tuple<Acir::OpcodeLocation, Acir::AssertionPayload>> assert_messages;

        friend bool operator==(const Circuit&, const Circuit&);

        void msgpack_pack(auto& packer) const {
            packer.pack_map(7);
            packer.pack(std::make_pair("function_name", function_name));
            packer.pack(std::make_pair("current_witness_index", current_witness_index));
            packer.pack(std::make_pair("opcodes", opcodes));
            packer.pack(std::make_pair("private_parameters", private_parameters));
            packer.pack(std::make_pair("public_parameters", public_parameters));
            packer.pack(std::make_pair("return_values", return_values));
            packer.pack(std::make_pair("assert_messages", assert_messages));
        }

        void msgpack_unpack(msgpack::object const& o) {
            std::string name = "Circuit";
            if (o.type == msgpack::type::MAP) {
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "function_name", function_name, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "current_witness_index", current_witness_index, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "opcodes", opcodes, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "private_parameters", private_parameters, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "public_parameters", public_parameters, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "return_values", return_values, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "assert_messages", assert_messages, false);
            } else if (o.type == msgpack::type::ARRAY) {
                auto array = o.via.array; 
                Helpers::conv_fld_from_array(array, name, "function_name", function_name, 0);
                Helpers::conv_fld_from_array(array, name, "current_witness_index", current_witness_index, 1);
                Helpers::conv_fld_from_array(array, name, "opcodes", opcodes, 2);
                Helpers::conv_fld_from_array(array, name, "private_parameters", private_parameters, 3);
                Helpers::conv_fld_from_array(array, name, "public_parameters", public_parameters, 4);
                Helpers::conv_fld_from_array(array, name, "return_values", return_values, 5);
                Helpers::conv_fld_from_array(array, name, "assert_messages", assert_messages, 6);
            } else {
                throw_or_abort("expected MAP or ARRAY for " + name);
            }
        }
    };

    struct BrilligBytecode {
        std::string function_name;
        std::vector<Acir::BrilligOpcode> bytecode;

        friend bool operator==(const BrilligBytecode&, const BrilligBytecode&);

        void msgpack_pack(auto& packer) const {
            packer.pack_map(2);
            packer.pack(std::make_pair("function_name", function_name));
            packer.pack(std::make_pair("bytecode", bytecode));
        }

        void msgpack_unpack(msgpack::object const& o) {
            std::string name = "BrilligBytecode";
            if (o.type == msgpack::type::MAP) {
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "function_name", function_name, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "bytecode", bytecode, false);
            } else if (o.type == msgpack::type::ARRAY) {
                auto array = o.via.array; 
                Helpers::conv_fld_from_array(array, name, "function_name", function_name, 0);
                Helpers::conv_fld_from_array(array, name, "bytecode", bytecode, 1);
            } else {
                throw_or_abort("expected MAP or ARRAY for " + name);
            }
        }
    };

    struct Program {
        std::vector<Acir::Circuit> functions;
        std::vector<Acir::BrilligBytecode> unconstrained_functions;

        friend bool operator==(const Program&, const Program&);

        void msgpack_pack(auto& packer) const {
            packer.pack_map(2);
            packer.pack(std::make_pair("functions", functions));
            packer.pack(std::make_pair("unconstrained_functions", unconstrained_functions));
        }

        void msgpack_unpack(msgpack::object const& o) {
            std::string name = "Program";
            if (o.type == msgpack::type::MAP) {
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "functions", functions, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "unconstrained_functions", unconstrained_functions, false);
            } else if (o.type == msgpack::type::ARRAY) {
                auto array = o.via.array; 
                Helpers::conv_fld_from_array(array, name, "functions", functions, 0);
                Helpers::conv_fld_from_array(array, name, "unconstrained_functions", unconstrained_functions, 1);
            } else {
                throw_or_abort("expected MAP or ARRAY for " + name);
            }
        }
    };

    struct ProgramWithoutBrillig {
        std::vector<Acir::Circuit> functions;
        std::monostate unconstrained_functions;

        friend bool operator==(const ProgramWithoutBrillig&, const ProgramWithoutBrillig&);

        void msgpack_pack(auto& packer) const {
            packer.pack_map(1);
            packer.pack(std::make_pair("functions", functions));
        }

        void msgpack_unpack(msgpack::object const& o) {
            std::string name = "ProgramWithoutBrillig";
            if (o.type == msgpack::type::MAP) {
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "functions", functions, false);
            } else if (o.type == msgpack::type::ARRAY) {
                auto array = o.via.array; 
                Helpers::conv_fld_from_array(array, name, "functions", functions, 0);
            } else {
                throw_or_abort("expected MAP or ARRAY for " + name);
            }
        }
    };

    struct ExpressionWidth {

        struct Unbounded {
            friend bool operator==(const Unbounded&, const Unbounded&);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct Bounded {
            uint64_t width;

            friend bool operator==(const Bounded&, const Bounded&);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(1);
                packer.pack(std::make_pair("width", width));
            }

            void msgpack_unpack(msgpack::object const& o) {
                std::string name = "Bounded";
                if (o.type == msgpack::type::MAP) {
                    auto kvmap = Helpers::make_kvmap(o, name);
                    Helpers::conv_fld_from_kvmap(kvmap, name, "width", width, false);
                } else if (o.type == msgpack::type::ARRAY) {
                    auto array = o.via.array; 
                    Helpers::conv_fld_from_array(array, name, "width", width, 0);
                } else {
                    throw_or_abort("expected MAP or ARRAY for " + name);
                }
            }
        };

        std::variant<Unbounded, Bounded> value;

        friend bool operator==(const ExpressionWidth&, const ExpressionWidth&);

        void msgpack_pack(auto& packer) const {
            std::string tag;
            bool is_unit;
            switch (value.index()) {
                
                case 0:
                    tag = "Unbounded";
                    is_unit = true;
                    break;
                case 1:
                    tag = "Bounded";
                    is_unit = false;
                    break;
                default:
                    throw_or_abort("unknown enum 'ExpressionWidth' variant index: " + std::to_string(value.index()));
            }
            if (is_unit) {
                packer.pack(tag);
            } else {
                std::visit([&packer, tag](const auto& arg) {
                    packer.pack_map(1);
                    packer.pack(tag);
                    arg.msgpack_pack(packer);
                }, value);
            }
        }

        void msgpack_unpack(msgpack::object const& o) {

            if (o.type != msgpack::type::object_type::MAP && o.type != msgpack::type::object_type::STR) {
                std::cerr << o << std::endl;
                throw_or_abort("expected MAP or STR for enum 'ExpressionWidth'; got type " + std::to_string(o.type));
            }
            if (o.type == msgpack::type::object_type::MAP && o.via.map.size != 1) {
                throw_or_abort("expected 1 entry for enum 'ExpressionWidth'; got " + std::to_string(o.via.map.size));
            }
            std::string tag;
            try {
                if (o.type == msgpack::type::object_type::MAP) {
                    o.via.map.ptr[0].key.convert(tag);
                } else {
                    o.convert(tag);
                }
            } catch(const msgpack::type_error&) {
                std::cerr << o << std::endl;
                throw_or_abort("error converting tag to string for enum 'ExpressionWidth'");
            }
            if (tag == "Unbounded") {
                Unbounded v;
                value = v;
            }
            else if (tag == "Bounded") {
                Bounded v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'ExpressionWidth::Bounded'");
                }
                
                value = v;
            }
            else {
                std::cerr << o << std::endl;
                throw_or_abort("unknown 'ExpressionWidth' enum variant: " + tag);
            }
        }
    };

} // end of namespace Acir


namespace Acir {

    inline bool operator==(const AssertionPayload &lhs, const AssertionPayload &rhs) {
        if (!(lhs.error_selector == rhs.error_selector)) { return false; }
        if (!(lhs.payload == rhs.payload)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::AssertionPayload>::serialize(const Acir::AssertionPayload &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.error_selector)>::serialize(obj.error_selector, serializer);
    serde::Serializable<decltype(obj.payload)>::serialize(obj.payload, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Acir::AssertionPayload serde::Deserializable<Acir::AssertionPayload>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Acir::AssertionPayload obj;
    obj.error_selector = serde::Deserializable<decltype(obj.error_selector)>::deserialize(deserializer);
    obj.payload = serde::Deserializable<decltype(obj.payload)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Acir {

    inline bool operator==(const BinaryFieldOp &lhs, const BinaryFieldOp &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BinaryFieldOp>::serialize(const Acir::BinaryFieldOp &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Acir::BinaryFieldOp serde::Deserializable<Acir::BinaryFieldOp>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Acir::BinaryFieldOp obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Acir {

    inline bool operator==(const BinaryFieldOp::Add &lhs, const BinaryFieldOp::Add &rhs) {
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BinaryFieldOp::Add>::serialize(const Acir::BinaryFieldOp::Add &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Acir::BinaryFieldOp::Add serde::Deserializable<Acir::BinaryFieldOp::Add>::deserialize(Deserializer &deserializer) {
    Acir::BinaryFieldOp::Add obj;
    return obj;
}

namespace Acir {

    inline bool operator==(const BinaryFieldOp::Sub &lhs, const BinaryFieldOp::Sub &rhs) {
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BinaryFieldOp::Sub>::serialize(const Acir::BinaryFieldOp::Sub &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Acir::BinaryFieldOp::Sub serde::Deserializable<Acir::BinaryFieldOp::Sub>::deserialize(Deserializer &deserializer) {
    Acir::BinaryFieldOp::Sub obj;
    return obj;
}

namespace Acir {

    inline bool operator==(const BinaryFieldOp::Mul &lhs, const BinaryFieldOp::Mul &rhs) {
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BinaryFieldOp::Mul>::serialize(const Acir::BinaryFieldOp::Mul &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Acir::BinaryFieldOp::Mul serde::Deserializable<Acir::BinaryFieldOp::Mul>::deserialize(Deserializer &deserializer) {
    Acir::BinaryFieldOp::Mul obj;
    return obj;
}

namespace Acir {

    inline bool operator==(const BinaryFieldOp::Div &lhs, const BinaryFieldOp::Div &rhs) {
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BinaryFieldOp::Div>::serialize(const Acir::BinaryFieldOp::Div &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Acir::BinaryFieldOp::Div serde::Deserializable<Acir::BinaryFieldOp::Div>::deserialize(Deserializer &deserializer) {
    Acir::BinaryFieldOp::Div obj;
    return obj;
}

namespace Acir {

    inline bool operator==(const BinaryFieldOp::IntegerDiv &lhs, const BinaryFieldOp::IntegerDiv &rhs) {
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BinaryFieldOp::IntegerDiv>::serialize(const Acir::BinaryFieldOp::IntegerDiv &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Acir::BinaryFieldOp::IntegerDiv serde::Deserializable<Acir::BinaryFieldOp::IntegerDiv>::deserialize(Deserializer &deserializer) {
    Acir::BinaryFieldOp::IntegerDiv obj;
    return obj;
}

namespace Acir {

    inline bool operator==(const BinaryFieldOp::Equals &lhs, const BinaryFieldOp::Equals &rhs) {
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BinaryFieldOp::Equals>::serialize(const Acir::BinaryFieldOp::Equals &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Acir::BinaryFieldOp::Equals serde::Deserializable<Acir::BinaryFieldOp::Equals>::deserialize(Deserializer &deserializer) {
    Acir::BinaryFieldOp::Equals obj;
    return obj;
}

namespace Acir {

    inline bool operator==(const BinaryFieldOp::LessThan &lhs, const BinaryFieldOp::LessThan &rhs) {
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BinaryFieldOp::LessThan>::serialize(const Acir::BinaryFieldOp::LessThan &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Acir::BinaryFieldOp::LessThan serde::Deserializable<Acir::BinaryFieldOp::LessThan>::deserialize(Deserializer &deserializer) {
    Acir::BinaryFieldOp::LessThan obj;
    return obj;
}

namespace Acir {

    inline bool operator==(const BinaryFieldOp::LessThanEquals &lhs, const BinaryFieldOp::LessThanEquals &rhs) {
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BinaryFieldOp::LessThanEquals>::serialize(const Acir::BinaryFieldOp::LessThanEquals &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Acir::BinaryFieldOp::LessThanEquals serde::Deserializable<Acir::BinaryFieldOp::LessThanEquals>::deserialize(Deserializer &deserializer) {
    Acir::BinaryFieldOp::LessThanEquals obj;
    return obj;
}

namespace Acir {

    inline bool operator==(const BinaryIntOp &lhs, const BinaryIntOp &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BinaryIntOp>::serialize(const Acir::BinaryIntOp &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Acir::BinaryIntOp serde::Deserializable<Acir::BinaryIntOp>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Acir::BinaryIntOp obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Acir {

    inline bool operator==(const BinaryIntOp::Add &lhs, const BinaryIntOp::Add &rhs) {
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BinaryIntOp::Add>::serialize(const Acir::BinaryIntOp::Add &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Acir::BinaryIntOp::Add serde::Deserializable<Acir::BinaryIntOp::Add>::deserialize(Deserializer &deserializer) {
    Acir::BinaryIntOp::Add obj;
    return obj;
}

namespace Acir {

    inline bool operator==(const BinaryIntOp::Sub &lhs, const BinaryIntOp::Sub &rhs) {
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BinaryIntOp::Sub>::serialize(const Acir::BinaryIntOp::Sub &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Acir::BinaryIntOp::Sub serde::Deserializable<Acir::BinaryIntOp::Sub>::deserialize(Deserializer &deserializer) {
    Acir::BinaryIntOp::Sub obj;
    return obj;
}

namespace Acir {

    inline bool operator==(const BinaryIntOp::Mul &lhs, const BinaryIntOp::Mul &rhs) {
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BinaryIntOp::Mul>::serialize(const Acir::BinaryIntOp::Mul &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Acir::BinaryIntOp::Mul serde::Deserializable<Acir::BinaryIntOp::Mul>::deserialize(Deserializer &deserializer) {
    Acir::BinaryIntOp::Mul obj;
    return obj;
}

namespace Acir {

    inline bool operator==(const BinaryIntOp::Div &lhs, const BinaryIntOp::Div &rhs) {
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BinaryIntOp::Div>::serialize(const Acir::BinaryIntOp::Div &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Acir::BinaryIntOp::Div serde::Deserializable<Acir::BinaryIntOp::Div>::deserialize(Deserializer &deserializer) {
    Acir::BinaryIntOp::Div obj;
    return obj;
}

namespace Acir {

    inline bool operator==(const BinaryIntOp::Equals &lhs, const BinaryIntOp::Equals &rhs) {
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BinaryIntOp::Equals>::serialize(const Acir::BinaryIntOp::Equals &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Acir::BinaryIntOp::Equals serde::Deserializable<Acir::BinaryIntOp::Equals>::deserialize(Deserializer &deserializer) {
    Acir::BinaryIntOp::Equals obj;
    return obj;
}

namespace Acir {

    inline bool operator==(const BinaryIntOp::LessThan &lhs, const BinaryIntOp::LessThan &rhs) {
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BinaryIntOp::LessThan>::serialize(const Acir::BinaryIntOp::LessThan &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Acir::BinaryIntOp::LessThan serde::Deserializable<Acir::BinaryIntOp::LessThan>::deserialize(Deserializer &deserializer) {
    Acir::BinaryIntOp::LessThan obj;
    return obj;
}

namespace Acir {

    inline bool operator==(const BinaryIntOp::LessThanEquals &lhs, const BinaryIntOp::LessThanEquals &rhs) {
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BinaryIntOp::LessThanEquals>::serialize(const Acir::BinaryIntOp::LessThanEquals &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Acir::BinaryIntOp::LessThanEquals serde::Deserializable<Acir::BinaryIntOp::LessThanEquals>::deserialize(Deserializer &deserializer) {
    Acir::BinaryIntOp::LessThanEquals obj;
    return obj;
}

namespace Acir {

    inline bool operator==(const BinaryIntOp::And &lhs, const BinaryIntOp::And &rhs) {
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BinaryIntOp::And>::serialize(const Acir::BinaryIntOp::And &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Acir::BinaryIntOp::And serde::Deserializable<Acir::BinaryIntOp::And>::deserialize(Deserializer &deserializer) {
    Acir::BinaryIntOp::And obj;
    return obj;
}

namespace Acir {

    inline bool operator==(const BinaryIntOp::Or &lhs, const BinaryIntOp::Or &rhs) {
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BinaryIntOp::Or>::serialize(const Acir::BinaryIntOp::Or &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Acir::BinaryIntOp::Or serde::Deserializable<Acir::BinaryIntOp::Or>::deserialize(Deserializer &deserializer) {
    Acir::BinaryIntOp::Or obj;
    return obj;
}

namespace Acir {

    inline bool operator==(const BinaryIntOp::Xor &lhs, const BinaryIntOp::Xor &rhs) {
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BinaryIntOp::Xor>::serialize(const Acir::BinaryIntOp::Xor &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Acir::BinaryIntOp::Xor serde::Deserializable<Acir::BinaryIntOp::Xor>::deserialize(Deserializer &deserializer) {
    Acir::BinaryIntOp::Xor obj;
    return obj;
}

namespace Acir {

    inline bool operator==(const BinaryIntOp::Shl &lhs, const BinaryIntOp::Shl &rhs) {
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BinaryIntOp::Shl>::serialize(const Acir::BinaryIntOp::Shl &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Acir::BinaryIntOp::Shl serde::Deserializable<Acir::BinaryIntOp::Shl>::deserialize(Deserializer &deserializer) {
    Acir::BinaryIntOp::Shl obj;
    return obj;
}

namespace Acir {

    inline bool operator==(const BinaryIntOp::Shr &lhs, const BinaryIntOp::Shr &rhs) {
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BinaryIntOp::Shr>::serialize(const Acir::BinaryIntOp::Shr &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Acir::BinaryIntOp::Shr serde::Deserializable<Acir::BinaryIntOp::Shr>::deserialize(Deserializer &deserializer) {
    Acir::BinaryIntOp::Shr obj;
    return obj;
}

namespace Acir {

    inline bool operator==(const BitSize &lhs, const BitSize &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BitSize>::serialize(const Acir::BitSize &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Acir::BitSize serde::Deserializable<Acir::BitSize>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Acir::BitSize obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Acir {

    inline bool operator==(const BitSize::Field &lhs, const BitSize::Field &rhs) {
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BitSize::Field>::serialize(const Acir::BitSize::Field &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Acir::BitSize::Field serde::Deserializable<Acir::BitSize::Field>::deserialize(Deserializer &deserializer) {
    Acir::BitSize::Field obj;
    return obj;
}

namespace Acir {

    inline bool operator==(const BitSize::Integer &lhs, const BitSize::Integer &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BitSize::Integer>::serialize(const Acir::BitSize::Integer &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Acir::BitSize::Integer serde::Deserializable<Acir::BitSize::Integer>::deserialize(Deserializer &deserializer) {
    Acir::BitSize::Integer obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxFuncCall &lhs, const BlackBoxFuncCall &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxFuncCall>::serialize(const Acir::BlackBoxFuncCall &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Acir::BlackBoxFuncCall serde::Deserializable<Acir::BlackBoxFuncCall>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Acir::BlackBoxFuncCall obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxFuncCall::AES128Encrypt &lhs, const BlackBoxFuncCall::AES128Encrypt &rhs) {
        if (!(lhs.inputs == rhs.inputs)) { return false; }
        if (!(lhs.iv == rhs.iv)) { return false; }
        if (!(lhs.key == rhs.key)) { return false; }
        if (!(lhs.outputs == rhs.outputs)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxFuncCall::AES128Encrypt>::serialize(const Acir::BlackBoxFuncCall::AES128Encrypt &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.inputs)>::serialize(obj.inputs, serializer);
    serde::Serializable<decltype(obj.iv)>::serialize(obj.iv, serializer);
    serde::Serializable<decltype(obj.key)>::serialize(obj.key, serializer);
    serde::Serializable<decltype(obj.outputs)>::serialize(obj.outputs, serializer);
}

template <>
template <typename Deserializer>
Acir::BlackBoxFuncCall::AES128Encrypt serde::Deserializable<Acir::BlackBoxFuncCall::AES128Encrypt>::deserialize(Deserializer &deserializer) {
    Acir::BlackBoxFuncCall::AES128Encrypt obj;
    obj.inputs = serde::Deserializable<decltype(obj.inputs)>::deserialize(deserializer);
    obj.iv = serde::Deserializable<decltype(obj.iv)>::deserialize(deserializer);
    obj.key = serde::Deserializable<decltype(obj.key)>::deserialize(deserializer);
    obj.outputs = serde::Deserializable<decltype(obj.outputs)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxFuncCall::AND &lhs, const BlackBoxFuncCall::AND &rhs) {
        if (!(lhs.lhs == rhs.lhs)) { return false; }
        if (!(lhs.rhs == rhs.rhs)) { return false; }
        if (!(lhs.num_bits == rhs.num_bits)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxFuncCall::AND>::serialize(const Acir::BlackBoxFuncCall::AND &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.lhs)>::serialize(obj.lhs, serializer);
    serde::Serializable<decltype(obj.rhs)>::serialize(obj.rhs, serializer);
    serde::Serializable<decltype(obj.num_bits)>::serialize(obj.num_bits, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Acir::BlackBoxFuncCall::AND serde::Deserializable<Acir::BlackBoxFuncCall::AND>::deserialize(Deserializer &deserializer) {
    Acir::BlackBoxFuncCall::AND obj;
    obj.lhs = serde::Deserializable<decltype(obj.lhs)>::deserialize(deserializer);
    obj.rhs = serde::Deserializable<decltype(obj.rhs)>::deserialize(deserializer);
    obj.num_bits = serde::Deserializable<decltype(obj.num_bits)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxFuncCall::XOR &lhs, const BlackBoxFuncCall::XOR &rhs) {
        if (!(lhs.lhs == rhs.lhs)) { return false; }
        if (!(lhs.rhs == rhs.rhs)) { return false; }
        if (!(lhs.num_bits == rhs.num_bits)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxFuncCall::XOR>::serialize(const Acir::BlackBoxFuncCall::XOR &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.lhs)>::serialize(obj.lhs, serializer);
    serde::Serializable<decltype(obj.rhs)>::serialize(obj.rhs, serializer);
    serde::Serializable<decltype(obj.num_bits)>::serialize(obj.num_bits, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Acir::BlackBoxFuncCall::XOR serde::Deserializable<Acir::BlackBoxFuncCall::XOR>::deserialize(Deserializer &deserializer) {
    Acir::BlackBoxFuncCall::XOR obj;
    obj.lhs = serde::Deserializable<decltype(obj.lhs)>::deserialize(deserializer);
    obj.rhs = serde::Deserializable<decltype(obj.rhs)>::deserialize(deserializer);
    obj.num_bits = serde::Deserializable<decltype(obj.num_bits)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxFuncCall::RANGE &lhs, const BlackBoxFuncCall::RANGE &rhs) {
        if (!(lhs.input == rhs.input)) { return false; }
        if (!(lhs.num_bits == rhs.num_bits)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxFuncCall::RANGE>::serialize(const Acir::BlackBoxFuncCall::RANGE &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.input)>::serialize(obj.input, serializer);
    serde::Serializable<decltype(obj.num_bits)>::serialize(obj.num_bits, serializer);
}

template <>
template <typename Deserializer>
Acir::BlackBoxFuncCall::RANGE serde::Deserializable<Acir::BlackBoxFuncCall::RANGE>::deserialize(Deserializer &deserializer) {
    Acir::BlackBoxFuncCall::RANGE obj;
    obj.input = serde::Deserializable<decltype(obj.input)>::deserialize(deserializer);
    obj.num_bits = serde::Deserializable<decltype(obj.num_bits)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxFuncCall::Blake2s &lhs, const BlackBoxFuncCall::Blake2s &rhs) {
        if (!(lhs.inputs == rhs.inputs)) { return false; }
        if (!(lhs.outputs == rhs.outputs)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxFuncCall::Blake2s>::serialize(const Acir::BlackBoxFuncCall::Blake2s &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.inputs)>::serialize(obj.inputs, serializer);
    serde::Serializable<decltype(obj.outputs)>::serialize(obj.outputs, serializer);
}

template <>
template <typename Deserializer>
Acir::BlackBoxFuncCall::Blake2s serde::Deserializable<Acir::BlackBoxFuncCall::Blake2s>::deserialize(Deserializer &deserializer) {
    Acir::BlackBoxFuncCall::Blake2s obj;
    obj.inputs = serde::Deserializable<decltype(obj.inputs)>::deserialize(deserializer);
    obj.outputs = serde::Deserializable<decltype(obj.outputs)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxFuncCall::Blake3 &lhs, const BlackBoxFuncCall::Blake3 &rhs) {
        if (!(lhs.inputs == rhs.inputs)) { return false; }
        if (!(lhs.outputs == rhs.outputs)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxFuncCall::Blake3>::serialize(const Acir::BlackBoxFuncCall::Blake3 &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.inputs)>::serialize(obj.inputs, serializer);
    serde::Serializable<decltype(obj.outputs)>::serialize(obj.outputs, serializer);
}

template <>
template <typename Deserializer>
Acir::BlackBoxFuncCall::Blake3 serde::Deserializable<Acir::BlackBoxFuncCall::Blake3>::deserialize(Deserializer &deserializer) {
    Acir::BlackBoxFuncCall::Blake3 obj;
    obj.inputs = serde::Deserializable<decltype(obj.inputs)>::deserialize(deserializer);
    obj.outputs = serde::Deserializable<decltype(obj.outputs)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxFuncCall::EcdsaSecp256k1 &lhs, const BlackBoxFuncCall::EcdsaSecp256k1 &rhs) {
        if (!(lhs.public_key_x == rhs.public_key_x)) { return false; }
        if (!(lhs.public_key_y == rhs.public_key_y)) { return false; }
        if (!(lhs.signature == rhs.signature)) { return false; }
        if (!(lhs.hashed_message == rhs.hashed_message)) { return false; }
        if (!(lhs.predicate == rhs.predicate)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxFuncCall::EcdsaSecp256k1>::serialize(const Acir::BlackBoxFuncCall::EcdsaSecp256k1 &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.public_key_x)>::serialize(obj.public_key_x, serializer);
    serde::Serializable<decltype(obj.public_key_y)>::serialize(obj.public_key_y, serializer);
    serde::Serializable<decltype(obj.signature)>::serialize(obj.signature, serializer);
    serde::Serializable<decltype(obj.hashed_message)>::serialize(obj.hashed_message, serializer);
    serde::Serializable<decltype(obj.predicate)>::serialize(obj.predicate, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Acir::BlackBoxFuncCall::EcdsaSecp256k1 serde::Deserializable<Acir::BlackBoxFuncCall::EcdsaSecp256k1>::deserialize(Deserializer &deserializer) {
    Acir::BlackBoxFuncCall::EcdsaSecp256k1 obj;
    obj.public_key_x = serde::Deserializable<decltype(obj.public_key_x)>::deserialize(deserializer);
    obj.public_key_y = serde::Deserializable<decltype(obj.public_key_y)>::deserialize(deserializer);
    obj.signature = serde::Deserializable<decltype(obj.signature)>::deserialize(deserializer);
    obj.hashed_message = serde::Deserializable<decltype(obj.hashed_message)>::deserialize(deserializer);
    obj.predicate = serde::Deserializable<decltype(obj.predicate)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxFuncCall::EcdsaSecp256r1 &lhs, const BlackBoxFuncCall::EcdsaSecp256r1 &rhs) {
        if (!(lhs.public_key_x == rhs.public_key_x)) { return false; }
        if (!(lhs.public_key_y == rhs.public_key_y)) { return false; }
        if (!(lhs.signature == rhs.signature)) { return false; }
        if (!(lhs.hashed_message == rhs.hashed_message)) { return false; }
        if (!(lhs.predicate == rhs.predicate)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxFuncCall::EcdsaSecp256r1>::serialize(const Acir::BlackBoxFuncCall::EcdsaSecp256r1 &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.public_key_x)>::serialize(obj.public_key_x, serializer);
    serde::Serializable<decltype(obj.public_key_y)>::serialize(obj.public_key_y, serializer);
    serde::Serializable<decltype(obj.signature)>::serialize(obj.signature, serializer);
    serde::Serializable<decltype(obj.hashed_message)>::serialize(obj.hashed_message, serializer);
    serde::Serializable<decltype(obj.predicate)>::serialize(obj.predicate, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Acir::BlackBoxFuncCall::EcdsaSecp256r1 serde::Deserializable<Acir::BlackBoxFuncCall::EcdsaSecp256r1>::deserialize(Deserializer &deserializer) {
    Acir::BlackBoxFuncCall::EcdsaSecp256r1 obj;
    obj.public_key_x = serde::Deserializable<decltype(obj.public_key_x)>::deserialize(deserializer);
    obj.public_key_y = serde::Deserializable<decltype(obj.public_key_y)>::deserialize(deserializer);
    obj.signature = serde::Deserializable<decltype(obj.signature)>::deserialize(deserializer);
    obj.hashed_message = serde::Deserializable<decltype(obj.hashed_message)>::deserialize(deserializer);
    obj.predicate = serde::Deserializable<decltype(obj.predicate)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxFuncCall::MultiScalarMul &lhs, const BlackBoxFuncCall::MultiScalarMul &rhs) {
        if (!(lhs.points == rhs.points)) { return false; }
        if (!(lhs.scalars == rhs.scalars)) { return false; }
        if (!(lhs.predicate == rhs.predicate)) { return false; }
        if (!(lhs.outputs == rhs.outputs)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxFuncCall::MultiScalarMul>::serialize(const Acir::BlackBoxFuncCall::MultiScalarMul &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.points)>::serialize(obj.points, serializer);
    serde::Serializable<decltype(obj.scalars)>::serialize(obj.scalars, serializer);
    serde::Serializable<decltype(obj.predicate)>::serialize(obj.predicate, serializer);
    serde::Serializable<decltype(obj.outputs)>::serialize(obj.outputs, serializer);
}

template <>
template <typename Deserializer>
Acir::BlackBoxFuncCall::MultiScalarMul serde::Deserializable<Acir::BlackBoxFuncCall::MultiScalarMul>::deserialize(Deserializer &deserializer) {
    Acir::BlackBoxFuncCall::MultiScalarMul obj;
    obj.points = serde::Deserializable<decltype(obj.points)>::deserialize(deserializer);
    obj.scalars = serde::Deserializable<decltype(obj.scalars)>::deserialize(deserializer);
    obj.predicate = serde::Deserializable<decltype(obj.predicate)>::deserialize(deserializer);
    obj.outputs = serde::Deserializable<decltype(obj.outputs)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxFuncCall::EmbeddedCurveAdd &lhs, const BlackBoxFuncCall::EmbeddedCurveAdd &rhs) {
        if (!(lhs.input1 == rhs.input1)) { return false; }
        if (!(lhs.input2 == rhs.input2)) { return false; }
        if (!(lhs.predicate == rhs.predicate)) { return false; }
        if (!(lhs.outputs == rhs.outputs)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxFuncCall::EmbeddedCurveAdd>::serialize(const Acir::BlackBoxFuncCall::EmbeddedCurveAdd &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.input1)>::serialize(obj.input1, serializer);
    serde::Serializable<decltype(obj.input2)>::serialize(obj.input2, serializer);
    serde::Serializable<decltype(obj.predicate)>::serialize(obj.predicate, serializer);
    serde::Serializable<decltype(obj.outputs)>::serialize(obj.outputs, serializer);
}

template <>
template <typename Deserializer>
Acir::BlackBoxFuncCall::EmbeddedCurveAdd serde::Deserializable<Acir::BlackBoxFuncCall::EmbeddedCurveAdd>::deserialize(Deserializer &deserializer) {
    Acir::BlackBoxFuncCall::EmbeddedCurveAdd obj;
    obj.input1 = serde::Deserializable<decltype(obj.input1)>::deserialize(deserializer);
    obj.input2 = serde::Deserializable<decltype(obj.input2)>::deserialize(deserializer);
    obj.predicate = serde::Deserializable<decltype(obj.predicate)>::deserialize(deserializer);
    obj.outputs = serde::Deserializable<decltype(obj.outputs)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxFuncCall::Keccakf1600 &lhs, const BlackBoxFuncCall::Keccakf1600 &rhs) {
        if (!(lhs.inputs == rhs.inputs)) { return false; }
        if (!(lhs.outputs == rhs.outputs)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxFuncCall::Keccakf1600>::serialize(const Acir::BlackBoxFuncCall::Keccakf1600 &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.inputs)>::serialize(obj.inputs, serializer);
    serde::Serializable<decltype(obj.outputs)>::serialize(obj.outputs, serializer);
}

template <>
template <typename Deserializer>
Acir::BlackBoxFuncCall::Keccakf1600 serde::Deserializable<Acir::BlackBoxFuncCall::Keccakf1600>::deserialize(Deserializer &deserializer) {
    Acir::BlackBoxFuncCall::Keccakf1600 obj;
    obj.inputs = serde::Deserializable<decltype(obj.inputs)>::deserialize(deserializer);
    obj.outputs = serde::Deserializable<decltype(obj.outputs)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxFuncCall::RecursiveAggregation &lhs, const BlackBoxFuncCall::RecursiveAggregation &rhs) {
        if (!(lhs.verification_key == rhs.verification_key)) { return false; }
        if (!(lhs.proof == rhs.proof)) { return false; }
        if (!(lhs.public_inputs == rhs.public_inputs)) { return false; }
        if (!(lhs.key_hash == rhs.key_hash)) { return false; }
        if (!(lhs.proof_type == rhs.proof_type)) { return false; }
        if (!(lhs.predicate == rhs.predicate)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxFuncCall::RecursiveAggregation>::serialize(const Acir::BlackBoxFuncCall::RecursiveAggregation &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.verification_key)>::serialize(obj.verification_key, serializer);
    serde::Serializable<decltype(obj.proof)>::serialize(obj.proof, serializer);
    serde::Serializable<decltype(obj.public_inputs)>::serialize(obj.public_inputs, serializer);
    serde::Serializable<decltype(obj.key_hash)>::serialize(obj.key_hash, serializer);
    serde::Serializable<decltype(obj.proof_type)>::serialize(obj.proof_type, serializer);
    serde::Serializable<decltype(obj.predicate)>::serialize(obj.predicate, serializer);
}

template <>
template <typename Deserializer>
Acir::BlackBoxFuncCall::RecursiveAggregation serde::Deserializable<Acir::BlackBoxFuncCall::RecursiveAggregation>::deserialize(Deserializer &deserializer) {
    Acir::BlackBoxFuncCall::RecursiveAggregation obj;
    obj.verification_key = serde::Deserializable<decltype(obj.verification_key)>::deserialize(deserializer);
    obj.proof = serde::Deserializable<decltype(obj.proof)>::deserialize(deserializer);
    obj.public_inputs = serde::Deserializable<decltype(obj.public_inputs)>::deserialize(deserializer);
    obj.key_hash = serde::Deserializable<decltype(obj.key_hash)>::deserialize(deserializer);
    obj.proof_type = serde::Deserializable<decltype(obj.proof_type)>::deserialize(deserializer);
    obj.predicate = serde::Deserializable<decltype(obj.predicate)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxFuncCall::Poseidon2Permutation &lhs, const BlackBoxFuncCall::Poseidon2Permutation &rhs) {
        if (!(lhs.inputs == rhs.inputs)) { return false; }
        if (!(lhs.outputs == rhs.outputs)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxFuncCall::Poseidon2Permutation>::serialize(const Acir::BlackBoxFuncCall::Poseidon2Permutation &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.inputs)>::serialize(obj.inputs, serializer);
    serde::Serializable<decltype(obj.outputs)>::serialize(obj.outputs, serializer);
}

template <>
template <typename Deserializer>
Acir::BlackBoxFuncCall::Poseidon2Permutation serde::Deserializable<Acir::BlackBoxFuncCall::Poseidon2Permutation>::deserialize(Deserializer &deserializer) {
    Acir::BlackBoxFuncCall::Poseidon2Permutation obj;
    obj.inputs = serde::Deserializable<decltype(obj.inputs)>::deserialize(deserializer);
    obj.outputs = serde::Deserializable<decltype(obj.outputs)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxFuncCall::Sha256Compression &lhs, const BlackBoxFuncCall::Sha256Compression &rhs) {
        if (!(lhs.inputs == rhs.inputs)) { return false; }
        if (!(lhs.hash_values == rhs.hash_values)) { return false; }
        if (!(lhs.outputs == rhs.outputs)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxFuncCall::Sha256Compression>::serialize(const Acir::BlackBoxFuncCall::Sha256Compression &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.inputs)>::serialize(obj.inputs, serializer);
    serde::Serializable<decltype(obj.hash_values)>::serialize(obj.hash_values, serializer);
    serde::Serializable<decltype(obj.outputs)>::serialize(obj.outputs, serializer);
}

template <>
template <typename Deserializer>
Acir::BlackBoxFuncCall::Sha256Compression serde::Deserializable<Acir::BlackBoxFuncCall::Sha256Compression>::deserialize(Deserializer &deserializer) {
    Acir::BlackBoxFuncCall::Sha256Compression obj;
    obj.inputs = serde::Deserializable<decltype(obj.inputs)>::deserialize(deserializer);
    obj.hash_values = serde::Deserializable<decltype(obj.hash_values)>::deserialize(deserializer);
    obj.outputs = serde::Deserializable<decltype(obj.outputs)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxOp &lhs, const BlackBoxOp &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxOp>::serialize(const Acir::BlackBoxOp &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Acir::BlackBoxOp serde::Deserializable<Acir::BlackBoxOp>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Acir::BlackBoxOp obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxOp::AES128Encrypt &lhs, const BlackBoxOp::AES128Encrypt &rhs) {
        if (!(lhs.inputs == rhs.inputs)) { return false; }
        if (!(lhs.iv == rhs.iv)) { return false; }
        if (!(lhs.key == rhs.key)) { return false; }
        if (!(lhs.outputs == rhs.outputs)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxOp::AES128Encrypt>::serialize(const Acir::BlackBoxOp::AES128Encrypt &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.inputs)>::serialize(obj.inputs, serializer);
    serde::Serializable<decltype(obj.iv)>::serialize(obj.iv, serializer);
    serde::Serializable<decltype(obj.key)>::serialize(obj.key, serializer);
    serde::Serializable<decltype(obj.outputs)>::serialize(obj.outputs, serializer);
}

template <>
template <typename Deserializer>
Acir::BlackBoxOp::AES128Encrypt serde::Deserializable<Acir::BlackBoxOp::AES128Encrypt>::deserialize(Deserializer &deserializer) {
    Acir::BlackBoxOp::AES128Encrypt obj;
    obj.inputs = serde::Deserializable<decltype(obj.inputs)>::deserialize(deserializer);
    obj.iv = serde::Deserializable<decltype(obj.iv)>::deserialize(deserializer);
    obj.key = serde::Deserializable<decltype(obj.key)>::deserialize(deserializer);
    obj.outputs = serde::Deserializable<decltype(obj.outputs)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxOp::Blake2s &lhs, const BlackBoxOp::Blake2s &rhs) {
        if (!(lhs.message == rhs.message)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxOp::Blake2s>::serialize(const Acir::BlackBoxOp::Blake2s &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.message)>::serialize(obj.message, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Acir::BlackBoxOp::Blake2s serde::Deserializable<Acir::BlackBoxOp::Blake2s>::deserialize(Deserializer &deserializer) {
    Acir::BlackBoxOp::Blake2s obj;
    obj.message = serde::Deserializable<decltype(obj.message)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxOp::Blake3 &lhs, const BlackBoxOp::Blake3 &rhs) {
        if (!(lhs.message == rhs.message)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxOp::Blake3>::serialize(const Acir::BlackBoxOp::Blake3 &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.message)>::serialize(obj.message, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Acir::BlackBoxOp::Blake3 serde::Deserializable<Acir::BlackBoxOp::Blake3>::deserialize(Deserializer &deserializer) {
    Acir::BlackBoxOp::Blake3 obj;
    obj.message = serde::Deserializable<decltype(obj.message)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxOp::Keccakf1600 &lhs, const BlackBoxOp::Keccakf1600 &rhs) {
        if (!(lhs.input == rhs.input)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxOp::Keccakf1600>::serialize(const Acir::BlackBoxOp::Keccakf1600 &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.input)>::serialize(obj.input, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Acir::BlackBoxOp::Keccakf1600 serde::Deserializable<Acir::BlackBoxOp::Keccakf1600>::deserialize(Deserializer &deserializer) {
    Acir::BlackBoxOp::Keccakf1600 obj;
    obj.input = serde::Deserializable<decltype(obj.input)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxOp::EcdsaSecp256k1 &lhs, const BlackBoxOp::EcdsaSecp256k1 &rhs) {
        if (!(lhs.hashed_msg == rhs.hashed_msg)) { return false; }
        if (!(lhs.public_key_x == rhs.public_key_x)) { return false; }
        if (!(lhs.public_key_y == rhs.public_key_y)) { return false; }
        if (!(lhs.signature == rhs.signature)) { return false; }
        if (!(lhs.result == rhs.result)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxOp::EcdsaSecp256k1>::serialize(const Acir::BlackBoxOp::EcdsaSecp256k1 &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.hashed_msg)>::serialize(obj.hashed_msg, serializer);
    serde::Serializable<decltype(obj.public_key_x)>::serialize(obj.public_key_x, serializer);
    serde::Serializable<decltype(obj.public_key_y)>::serialize(obj.public_key_y, serializer);
    serde::Serializable<decltype(obj.signature)>::serialize(obj.signature, serializer);
    serde::Serializable<decltype(obj.result)>::serialize(obj.result, serializer);
}

template <>
template <typename Deserializer>
Acir::BlackBoxOp::EcdsaSecp256k1 serde::Deserializable<Acir::BlackBoxOp::EcdsaSecp256k1>::deserialize(Deserializer &deserializer) {
    Acir::BlackBoxOp::EcdsaSecp256k1 obj;
    obj.hashed_msg = serde::Deserializable<decltype(obj.hashed_msg)>::deserialize(deserializer);
    obj.public_key_x = serde::Deserializable<decltype(obj.public_key_x)>::deserialize(deserializer);
    obj.public_key_y = serde::Deserializable<decltype(obj.public_key_y)>::deserialize(deserializer);
    obj.signature = serde::Deserializable<decltype(obj.signature)>::deserialize(deserializer);
    obj.result = serde::Deserializable<decltype(obj.result)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxOp::EcdsaSecp256r1 &lhs, const BlackBoxOp::EcdsaSecp256r1 &rhs) {
        if (!(lhs.hashed_msg == rhs.hashed_msg)) { return false; }
        if (!(lhs.public_key_x == rhs.public_key_x)) { return false; }
        if (!(lhs.public_key_y == rhs.public_key_y)) { return false; }
        if (!(lhs.signature == rhs.signature)) { return false; }
        if (!(lhs.result == rhs.result)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxOp::EcdsaSecp256r1>::serialize(const Acir::BlackBoxOp::EcdsaSecp256r1 &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.hashed_msg)>::serialize(obj.hashed_msg, serializer);
    serde::Serializable<decltype(obj.public_key_x)>::serialize(obj.public_key_x, serializer);
    serde::Serializable<decltype(obj.public_key_y)>::serialize(obj.public_key_y, serializer);
    serde::Serializable<decltype(obj.signature)>::serialize(obj.signature, serializer);
    serde::Serializable<decltype(obj.result)>::serialize(obj.result, serializer);
}

template <>
template <typename Deserializer>
Acir::BlackBoxOp::EcdsaSecp256r1 serde::Deserializable<Acir::BlackBoxOp::EcdsaSecp256r1>::deserialize(Deserializer &deserializer) {
    Acir::BlackBoxOp::EcdsaSecp256r1 obj;
    obj.hashed_msg = serde::Deserializable<decltype(obj.hashed_msg)>::deserialize(deserializer);
    obj.public_key_x = serde::Deserializable<decltype(obj.public_key_x)>::deserialize(deserializer);
    obj.public_key_y = serde::Deserializable<decltype(obj.public_key_y)>::deserialize(deserializer);
    obj.signature = serde::Deserializable<decltype(obj.signature)>::deserialize(deserializer);
    obj.result = serde::Deserializable<decltype(obj.result)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxOp::MultiScalarMul &lhs, const BlackBoxOp::MultiScalarMul &rhs) {
        if (!(lhs.points == rhs.points)) { return false; }
        if (!(lhs.scalars == rhs.scalars)) { return false; }
        if (!(lhs.outputs == rhs.outputs)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxOp::MultiScalarMul>::serialize(const Acir::BlackBoxOp::MultiScalarMul &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.points)>::serialize(obj.points, serializer);
    serde::Serializable<decltype(obj.scalars)>::serialize(obj.scalars, serializer);
    serde::Serializable<decltype(obj.outputs)>::serialize(obj.outputs, serializer);
}

template <>
template <typename Deserializer>
Acir::BlackBoxOp::MultiScalarMul serde::Deserializable<Acir::BlackBoxOp::MultiScalarMul>::deserialize(Deserializer &deserializer) {
    Acir::BlackBoxOp::MultiScalarMul obj;
    obj.points = serde::Deserializable<decltype(obj.points)>::deserialize(deserializer);
    obj.scalars = serde::Deserializable<decltype(obj.scalars)>::deserialize(deserializer);
    obj.outputs = serde::Deserializable<decltype(obj.outputs)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxOp::EmbeddedCurveAdd &lhs, const BlackBoxOp::EmbeddedCurveAdd &rhs) {
        if (!(lhs.input1_x == rhs.input1_x)) { return false; }
        if (!(lhs.input1_y == rhs.input1_y)) { return false; }
        if (!(lhs.input1_infinite == rhs.input1_infinite)) { return false; }
        if (!(lhs.input2_x == rhs.input2_x)) { return false; }
        if (!(lhs.input2_y == rhs.input2_y)) { return false; }
        if (!(lhs.input2_infinite == rhs.input2_infinite)) { return false; }
        if (!(lhs.result == rhs.result)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxOp::EmbeddedCurveAdd>::serialize(const Acir::BlackBoxOp::EmbeddedCurveAdd &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.input1_x)>::serialize(obj.input1_x, serializer);
    serde::Serializable<decltype(obj.input1_y)>::serialize(obj.input1_y, serializer);
    serde::Serializable<decltype(obj.input1_infinite)>::serialize(obj.input1_infinite, serializer);
    serde::Serializable<decltype(obj.input2_x)>::serialize(obj.input2_x, serializer);
    serde::Serializable<decltype(obj.input2_y)>::serialize(obj.input2_y, serializer);
    serde::Serializable<decltype(obj.input2_infinite)>::serialize(obj.input2_infinite, serializer);
    serde::Serializable<decltype(obj.result)>::serialize(obj.result, serializer);
}

template <>
template <typename Deserializer>
Acir::BlackBoxOp::EmbeddedCurveAdd serde::Deserializable<Acir::BlackBoxOp::EmbeddedCurveAdd>::deserialize(Deserializer &deserializer) {
    Acir::BlackBoxOp::EmbeddedCurveAdd obj;
    obj.input1_x = serde::Deserializable<decltype(obj.input1_x)>::deserialize(deserializer);
    obj.input1_y = serde::Deserializable<decltype(obj.input1_y)>::deserialize(deserializer);
    obj.input1_infinite = serde::Deserializable<decltype(obj.input1_infinite)>::deserialize(deserializer);
    obj.input2_x = serde::Deserializable<decltype(obj.input2_x)>::deserialize(deserializer);
    obj.input2_y = serde::Deserializable<decltype(obj.input2_y)>::deserialize(deserializer);
    obj.input2_infinite = serde::Deserializable<decltype(obj.input2_infinite)>::deserialize(deserializer);
    obj.result = serde::Deserializable<decltype(obj.result)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxOp::Poseidon2Permutation &lhs, const BlackBoxOp::Poseidon2Permutation &rhs) {
        if (!(lhs.message == rhs.message)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxOp::Poseidon2Permutation>::serialize(const Acir::BlackBoxOp::Poseidon2Permutation &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.message)>::serialize(obj.message, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Acir::BlackBoxOp::Poseidon2Permutation serde::Deserializable<Acir::BlackBoxOp::Poseidon2Permutation>::deserialize(Deserializer &deserializer) {
    Acir::BlackBoxOp::Poseidon2Permutation obj;
    obj.message = serde::Deserializable<decltype(obj.message)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxOp::Sha256Compression &lhs, const BlackBoxOp::Sha256Compression &rhs) {
        if (!(lhs.input == rhs.input)) { return false; }
        if (!(lhs.hash_values == rhs.hash_values)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxOp::Sha256Compression>::serialize(const Acir::BlackBoxOp::Sha256Compression &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.input)>::serialize(obj.input, serializer);
    serde::Serializable<decltype(obj.hash_values)>::serialize(obj.hash_values, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Acir::BlackBoxOp::Sha256Compression serde::Deserializable<Acir::BlackBoxOp::Sha256Compression>::deserialize(Deserializer &deserializer) {
    Acir::BlackBoxOp::Sha256Compression obj;
    obj.input = serde::Deserializable<decltype(obj.input)>::deserialize(deserializer);
    obj.hash_values = serde::Deserializable<decltype(obj.hash_values)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxOp::ToRadix &lhs, const BlackBoxOp::ToRadix &rhs) {
        if (!(lhs.input == rhs.input)) { return false; }
        if (!(lhs.radix == rhs.radix)) { return false; }
        if (!(lhs.output_pointer == rhs.output_pointer)) { return false; }
        if (!(lhs.num_limbs == rhs.num_limbs)) { return false; }
        if (!(lhs.output_bits == rhs.output_bits)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxOp::ToRadix>::serialize(const Acir::BlackBoxOp::ToRadix &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.input)>::serialize(obj.input, serializer);
    serde::Serializable<decltype(obj.radix)>::serialize(obj.radix, serializer);
    serde::Serializable<decltype(obj.output_pointer)>::serialize(obj.output_pointer, serializer);
    serde::Serializable<decltype(obj.num_limbs)>::serialize(obj.num_limbs, serializer);
    serde::Serializable<decltype(obj.output_bits)>::serialize(obj.output_bits, serializer);
}

template <>
template <typename Deserializer>
Acir::BlackBoxOp::ToRadix serde::Deserializable<Acir::BlackBoxOp::ToRadix>::deserialize(Deserializer &deserializer) {
    Acir::BlackBoxOp::ToRadix obj;
    obj.input = serde::Deserializable<decltype(obj.input)>::deserialize(deserializer);
    obj.radix = serde::Deserializable<decltype(obj.radix)>::deserialize(deserializer);
    obj.output_pointer = serde::Deserializable<decltype(obj.output_pointer)>::deserialize(deserializer);
    obj.num_limbs = serde::Deserializable<decltype(obj.num_limbs)>::deserialize(deserializer);
    obj.output_bits = serde::Deserializable<decltype(obj.output_bits)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlockId &lhs, const BlockId &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlockId>::serialize(const Acir::BlockId &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Acir::BlockId serde::Deserializable<Acir::BlockId>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Acir::BlockId obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Acir {

    inline bool operator==(const BlockType &lhs, const BlockType &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlockType>::serialize(const Acir::BlockType &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Acir::BlockType serde::Deserializable<Acir::BlockType>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Acir::BlockType obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Acir {

    inline bool operator==(const BlockType::Memory &lhs, const BlockType::Memory &rhs) {
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlockType::Memory>::serialize(const Acir::BlockType::Memory &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Acir::BlockType::Memory serde::Deserializable<Acir::BlockType::Memory>::deserialize(Deserializer &deserializer) {
    Acir::BlockType::Memory obj;
    return obj;
}

namespace Acir {

    inline bool operator==(const BlockType::CallData &lhs, const BlockType::CallData &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlockType::CallData>::serialize(const Acir::BlockType::CallData &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Acir::BlockType::CallData serde::Deserializable<Acir::BlockType::CallData>::deserialize(Deserializer &deserializer) {
    Acir::BlockType::CallData obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlockType::ReturnData &lhs, const BlockType::ReturnData &rhs) {
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlockType::ReturnData>::serialize(const Acir::BlockType::ReturnData &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Acir::BlockType::ReturnData serde::Deserializable<Acir::BlockType::ReturnData>::deserialize(Deserializer &deserializer) {
    Acir::BlockType::ReturnData obj;
    return obj;
}

namespace Acir {

    inline bool operator==(const BrilligBytecode &lhs, const BrilligBytecode &rhs) {
        if (!(lhs.function_name == rhs.function_name)) { return false; }
        if (!(lhs.bytecode == rhs.bytecode)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BrilligBytecode>::serialize(const Acir::BrilligBytecode &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.function_name)>::serialize(obj.function_name, serializer);
    serde::Serializable<decltype(obj.bytecode)>::serialize(obj.bytecode, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Acir::BrilligBytecode serde::Deserializable<Acir::BrilligBytecode>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Acir::BrilligBytecode obj;
    obj.function_name = serde::Deserializable<decltype(obj.function_name)>::deserialize(deserializer);
    obj.bytecode = serde::Deserializable<decltype(obj.bytecode)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Acir {

    inline bool operator==(const BrilligInputs &lhs, const BrilligInputs &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BrilligInputs>::serialize(const Acir::BrilligInputs &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Acir::BrilligInputs serde::Deserializable<Acir::BrilligInputs>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Acir::BrilligInputs obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Acir {

    inline bool operator==(const BrilligInputs::Single &lhs, const BrilligInputs::Single &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BrilligInputs::Single>::serialize(const Acir::BrilligInputs::Single &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Acir::BrilligInputs::Single serde::Deserializable<Acir::BrilligInputs::Single>::deserialize(Deserializer &deserializer) {
    Acir::BrilligInputs::Single obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BrilligInputs::Array &lhs, const BrilligInputs::Array &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BrilligInputs::Array>::serialize(const Acir::BrilligInputs::Array &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Acir::BrilligInputs::Array serde::Deserializable<Acir::BrilligInputs::Array>::deserialize(Deserializer &deserializer) {
    Acir::BrilligInputs::Array obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BrilligInputs::MemoryArray &lhs, const BrilligInputs::MemoryArray &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BrilligInputs::MemoryArray>::serialize(const Acir::BrilligInputs::MemoryArray &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Acir::BrilligInputs::MemoryArray serde::Deserializable<Acir::BrilligInputs::MemoryArray>::deserialize(Deserializer &deserializer) {
    Acir::BrilligInputs::MemoryArray obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BrilligOpcode &lhs, const BrilligOpcode &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BrilligOpcode>::serialize(const Acir::BrilligOpcode &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Acir::BrilligOpcode serde::Deserializable<Acir::BrilligOpcode>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Acir::BrilligOpcode obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Acir {

    inline bool operator==(const BrilligOpcode::BinaryFieldOp &lhs, const BrilligOpcode::BinaryFieldOp &rhs) {
        if (!(lhs.destination == rhs.destination)) { return false; }
        if (!(lhs.op == rhs.op)) { return false; }
        if (!(lhs.lhs == rhs.lhs)) { return false; }
        if (!(lhs.rhs == rhs.rhs)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BrilligOpcode::BinaryFieldOp>::serialize(const Acir::BrilligOpcode::BinaryFieldOp &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.destination)>::serialize(obj.destination, serializer);
    serde::Serializable<decltype(obj.op)>::serialize(obj.op, serializer);
    serde::Serializable<decltype(obj.lhs)>::serialize(obj.lhs, serializer);
    serde::Serializable<decltype(obj.rhs)>::serialize(obj.rhs, serializer);
}

template <>
template <typename Deserializer>
Acir::BrilligOpcode::BinaryFieldOp serde::Deserializable<Acir::BrilligOpcode::BinaryFieldOp>::deserialize(Deserializer &deserializer) {
    Acir::BrilligOpcode::BinaryFieldOp obj;
    obj.destination = serde::Deserializable<decltype(obj.destination)>::deserialize(deserializer);
    obj.op = serde::Deserializable<decltype(obj.op)>::deserialize(deserializer);
    obj.lhs = serde::Deserializable<decltype(obj.lhs)>::deserialize(deserializer);
    obj.rhs = serde::Deserializable<decltype(obj.rhs)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BrilligOpcode::BinaryIntOp &lhs, const BrilligOpcode::BinaryIntOp &rhs) {
        if (!(lhs.destination == rhs.destination)) { return false; }
        if (!(lhs.op == rhs.op)) { return false; }
        if (!(lhs.bit_size == rhs.bit_size)) { return false; }
        if (!(lhs.lhs == rhs.lhs)) { return false; }
        if (!(lhs.rhs == rhs.rhs)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BrilligOpcode::BinaryIntOp>::serialize(const Acir::BrilligOpcode::BinaryIntOp &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.destination)>::serialize(obj.destination, serializer);
    serde::Serializable<decltype(obj.op)>::serialize(obj.op, serializer);
    serde::Serializable<decltype(obj.bit_size)>::serialize(obj.bit_size, serializer);
    serde::Serializable<decltype(obj.lhs)>::serialize(obj.lhs, serializer);
    serde::Serializable<decltype(obj.rhs)>::serialize(obj.rhs, serializer);
}

template <>
template <typename Deserializer>
Acir::BrilligOpcode::BinaryIntOp serde::Deserializable<Acir::BrilligOpcode::BinaryIntOp>::deserialize(Deserializer &deserializer) {
    Acir::BrilligOpcode::BinaryIntOp obj;
    obj.destination = serde::Deserializable<decltype(obj.destination)>::deserialize(deserializer);
    obj.op = serde::Deserializable<decltype(obj.op)>::deserialize(deserializer);
    obj.bit_size = serde::Deserializable<decltype(obj.bit_size)>::deserialize(deserializer);
    obj.lhs = serde::Deserializable<decltype(obj.lhs)>::deserialize(deserializer);
    obj.rhs = serde::Deserializable<decltype(obj.rhs)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BrilligOpcode::Not &lhs, const BrilligOpcode::Not &rhs) {
        if (!(lhs.destination == rhs.destination)) { return false; }
        if (!(lhs.source == rhs.source)) { return false; }
        if (!(lhs.bit_size == rhs.bit_size)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BrilligOpcode::Not>::serialize(const Acir::BrilligOpcode::Not &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.destination)>::serialize(obj.destination, serializer);
    serde::Serializable<decltype(obj.source)>::serialize(obj.source, serializer);
    serde::Serializable<decltype(obj.bit_size)>::serialize(obj.bit_size, serializer);
}

template <>
template <typename Deserializer>
Acir::BrilligOpcode::Not serde::Deserializable<Acir::BrilligOpcode::Not>::deserialize(Deserializer &deserializer) {
    Acir::BrilligOpcode::Not obj;
    obj.destination = serde::Deserializable<decltype(obj.destination)>::deserialize(deserializer);
    obj.source = serde::Deserializable<decltype(obj.source)>::deserialize(deserializer);
    obj.bit_size = serde::Deserializable<decltype(obj.bit_size)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BrilligOpcode::Cast &lhs, const BrilligOpcode::Cast &rhs) {
        if (!(lhs.destination == rhs.destination)) { return false; }
        if (!(lhs.source == rhs.source)) { return false; }
        if (!(lhs.bit_size == rhs.bit_size)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BrilligOpcode::Cast>::serialize(const Acir::BrilligOpcode::Cast &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.destination)>::serialize(obj.destination, serializer);
    serde::Serializable<decltype(obj.source)>::serialize(obj.source, serializer);
    serde::Serializable<decltype(obj.bit_size)>::serialize(obj.bit_size, serializer);
}

template <>
template <typename Deserializer>
Acir::BrilligOpcode::Cast serde::Deserializable<Acir::BrilligOpcode::Cast>::deserialize(Deserializer &deserializer) {
    Acir::BrilligOpcode::Cast obj;
    obj.destination = serde::Deserializable<decltype(obj.destination)>::deserialize(deserializer);
    obj.source = serde::Deserializable<decltype(obj.source)>::deserialize(deserializer);
    obj.bit_size = serde::Deserializable<decltype(obj.bit_size)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BrilligOpcode::JumpIf &lhs, const BrilligOpcode::JumpIf &rhs) {
        if (!(lhs.condition == rhs.condition)) { return false; }
        if (!(lhs.location == rhs.location)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BrilligOpcode::JumpIf>::serialize(const Acir::BrilligOpcode::JumpIf &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.condition)>::serialize(obj.condition, serializer);
    serde::Serializable<decltype(obj.location)>::serialize(obj.location, serializer);
}

template <>
template <typename Deserializer>
Acir::BrilligOpcode::JumpIf serde::Deserializable<Acir::BrilligOpcode::JumpIf>::deserialize(Deserializer &deserializer) {
    Acir::BrilligOpcode::JumpIf obj;
    obj.condition = serde::Deserializable<decltype(obj.condition)>::deserialize(deserializer);
    obj.location = serde::Deserializable<decltype(obj.location)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BrilligOpcode::Jump &lhs, const BrilligOpcode::Jump &rhs) {
        if (!(lhs.location == rhs.location)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BrilligOpcode::Jump>::serialize(const Acir::BrilligOpcode::Jump &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.location)>::serialize(obj.location, serializer);
}

template <>
template <typename Deserializer>
Acir::BrilligOpcode::Jump serde::Deserializable<Acir::BrilligOpcode::Jump>::deserialize(Deserializer &deserializer) {
    Acir::BrilligOpcode::Jump obj;
    obj.location = serde::Deserializable<decltype(obj.location)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BrilligOpcode::CalldataCopy &lhs, const BrilligOpcode::CalldataCopy &rhs) {
        if (!(lhs.destination_address == rhs.destination_address)) { return false; }
        if (!(lhs.size_address == rhs.size_address)) { return false; }
        if (!(lhs.offset_address == rhs.offset_address)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BrilligOpcode::CalldataCopy>::serialize(const Acir::BrilligOpcode::CalldataCopy &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.destination_address)>::serialize(obj.destination_address, serializer);
    serde::Serializable<decltype(obj.size_address)>::serialize(obj.size_address, serializer);
    serde::Serializable<decltype(obj.offset_address)>::serialize(obj.offset_address, serializer);
}

template <>
template <typename Deserializer>
Acir::BrilligOpcode::CalldataCopy serde::Deserializable<Acir::BrilligOpcode::CalldataCopy>::deserialize(Deserializer &deserializer) {
    Acir::BrilligOpcode::CalldataCopy obj;
    obj.destination_address = serde::Deserializable<decltype(obj.destination_address)>::deserialize(deserializer);
    obj.size_address = serde::Deserializable<decltype(obj.size_address)>::deserialize(deserializer);
    obj.offset_address = serde::Deserializable<decltype(obj.offset_address)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BrilligOpcode::Call &lhs, const BrilligOpcode::Call &rhs) {
        if (!(lhs.location == rhs.location)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BrilligOpcode::Call>::serialize(const Acir::BrilligOpcode::Call &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.location)>::serialize(obj.location, serializer);
}

template <>
template <typename Deserializer>
Acir::BrilligOpcode::Call serde::Deserializable<Acir::BrilligOpcode::Call>::deserialize(Deserializer &deserializer) {
    Acir::BrilligOpcode::Call obj;
    obj.location = serde::Deserializable<decltype(obj.location)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BrilligOpcode::Const &lhs, const BrilligOpcode::Const &rhs) {
        if (!(lhs.destination == rhs.destination)) { return false; }
        if (!(lhs.bit_size == rhs.bit_size)) { return false; }
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BrilligOpcode::Const>::serialize(const Acir::BrilligOpcode::Const &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.destination)>::serialize(obj.destination, serializer);
    serde::Serializable<decltype(obj.bit_size)>::serialize(obj.bit_size, serializer);
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Acir::BrilligOpcode::Const serde::Deserializable<Acir::BrilligOpcode::Const>::deserialize(Deserializer &deserializer) {
    Acir::BrilligOpcode::Const obj;
    obj.destination = serde::Deserializable<decltype(obj.destination)>::deserialize(deserializer);
    obj.bit_size = serde::Deserializable<decltype(obj.bit_size)>::deserialize(deserializer);
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BrilligOpcode::IndirectConst &lhs, const BrilligOpcode::IndirectConst &rhs) {
        if (!(lhs.destination_pointer == rhs.destination_pointer)) { return false; }
        if (!(lhs.bit_size == rhs.bit_size)) { return false; }
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BrilligOpcode::IndirectConst>::serialize(const Acir::BrilligOpcode::IndirectConst &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.destination_pointer)>::serialize(obj.destination_pointer, serializer);
    serde::Serializable<decltype(obj.bit_size)>::serialize(obj.bit_size, serializer);
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Acir::BrilligOpcode::IndirectConst serde::Deserializable<Acir::BrilligOpcode::IndirectConst>::deserialize(Deserializer &deserializer) {
    Acir::BrilligOpcode::IndirectConst obj;
    obj.destination_pointer = serde::Deserializable<decltype(obj.destination_pointer)>::deserialize(deserializer);
    obj.bit_size = serde::Deserializable<decltype(obj.bit_size)>::deserialize(deserializer);
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BrilligOpcode::Return &lhs, const BrilligOpcode::Return &rhs) {
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BrilligOpcode::Return>::serialize(const Acir::BrilligOpcode::Return &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Acir::BrilligOpcode::Return serde::Deserializable<Acir::BrilligOpcode::Return>::deserialize(Deserializer &deserializer) {
    Acir::BrilligOpcode::Return obj;
    return obj;
}

namespace Acir {

    inline bool operator==(const BrilligOpcode::ForeignCall &lhs, const BrilligOpcode::ForeignCall &rhs) {
        if (!(lhs.function == rhs.function)) { return false; }
        if (!(lhs.destinations == rhs.destinations)) { return false; }
        if (!(lhs.destination_value_types == rhs.destination_value_types)) { return false; }
        if (!(lhs.inputs == rhs.inputs)) { return false; }
        if (!(lhs.input_value_types == rhs.input_value_types)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BrilligOpcode::ForeignCall>::serialize(const Acir::BrilligOpcode::ForeignCall &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.function)>::serialize(obj.function, serializer);
    serde::Serializable<decltype(obj.destinations)>::serialize(obj.destinations, serializer);
    serde::Serializable<decltype(obj.destination_value_types)>::serialize(obj.destination_value_types, serializer);
    serde::Serializable<decltype(obj.inputs)>::serialize(obj.inputs, serializer);
    serde::Serializable<decltype(obj.input_value_types)>::serialize(obj.input_value_types, serializer);
}

template <>
template <typename Deserializer>
Acir::BrilligOpcode::ForeignCall serde::Deserializable<Acir::BrilligOpcode::ForeignCall>::deserialize(Deserializer &deserializer) {
    Acir::BrilligOpcode::ForeignCall obj;
    obj.function = serde::Deserializable<decltype(obj.function)>::deserialize(deserializer);
    obj.destinations = serde::Deserializable<decltype(obj.destinations)>::deserialize(deserializer);
    obj.destination_value_types = serde::Deserializable<decltype(obj.destination_value_types)>::deserialize(deserializer);
    obj.inputs = serde::Deserializable<decltype(obj.inputs)>::deserialize(deserializer);
    obj.input_value_types = serde::Deserializable<decltype(obj.input_value_types)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BrilligOpcode::Mov &lhs, const BrilligOpcode::Mov &rhs) {
        if (!(lhs.destination == rhs.destination)) { return false; }
        if (!(lhs.source == rhs.source)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BrilligOpcode::Mov>::serialize(const Acir::BrilligOpcode::Mov &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.destination)>::serialize(obj.destination, serializer);
    serde::Serializable<decltype(obj.source)>::serialize(obj.source, serializer);
}

template <>
template <typename Deserializer>
Acir::BrilligOpcode::Mov serde::Deserializable<Acir::BrilligOpcode::Mov>::deserialize(Deserializer &deserializer) {
    Acir::BrilligOpcode::Mov obj;
    obj.destination = serde::Deserializable<decltype(obj.destination)>::deserialize(deserializer);
    obj.source = serde::Deserializable<decltype(obj.source)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BrilligOpcode::ConditionalMov &lhs, const BrilligOpcode::ConditionalMov &rhs) {
        if (!(lhs.destination == rhs.destination)) { return false; }
        if (!(lhs.source_a == rhs.source_a)) { return false; }
        if (!(lhs.source_b == rhs.source_b)) { return false; }
        if (!(lhs.condition == rhs.condition)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BrilligOpcode::ConditionalMov>::serialize(const Acir::BrilligOpcode::ConditionalMov &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.destination)>::serialize(obj.destination, serializer);
    serde::Serializable<decltype(obj.source_a)>::serialize(obj.source_a, serializer);
    serde::Serializable<decltype(obj.source_b)>::serialize(obj.source_b, serializer);
    serde::Serializable<decltype(obj.condition)>::serialize(obj.condition, serializer);
}

template <>
template <typename Deserializer>
Acir::BrilligOpcode::ConditionalMov serde::Deserializable<Acir::BrilligOpcode::ConditionalMov>::deserialize(Deserializer &deserializer) {
    Acir::BrilligOpcode::ConditionalMov obj;
    obj.destination = serde::Deserializable<decltype(obj.destination)>::deserialize(deserializer);
    obj.source_a = serde::Deserializable<decltype(obj.source_a)>::deserialize(deserializer);
    obj.source_b = serde::Deserializable<decltype(obj.source_b)>::deserialize(deserializer);
    obj.condition = serde::Deserializable<decltype(obj.condition)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BrilligOpcode::Load &lhs, const BrilligOpcode::Load &rhs) {
        if (!(lhs.destination == rhs.destination)) { return false; }
        if (!(lhs.source_pointer == rhs.source_pointer)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BrilligOpcode::Load>::serialize(const Acir::BrilligOpcode::Load &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.destination)>::serialize(obj.destination, serializer);
    serde::Serializable<decltype(obj.source_pointer)>::serialize(obj.source_pointer, serializer);
}

template <>
template <typename Deserializer>
Acir::BrilligOpcode::Load serde::Deserializable<Acir::BrilligOpcode::Load>::deserialize(Deserializer &deserializer) {
    Acir::BrilligOpcode::Load obj;
    obj.destination = serde::Deserializable<decltype(obj.destination)>::deserialize(deserializer);
    obj.source_pointer = serde::Deserializable<decltype(obj.source_pointer)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BrilligOpcode::Store &lhs, const BrilligOpcode::Store &rhs) {
        if (!(lhs.destination_pointer == rhs.destination_pointer)) { return false; }
        if (!(lhs.source == rhs.source)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BrilligOpcode::Store>::serialize(const Acir::BrilligOpcode::Store &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.destination_pointer)>::serialize(obj.destination_pointer, serializer);
    serde::Serializable<decltype(obj.source)>::serialize(obj.source, serializer);
}

template <>
template <typename Deserializer>
Acir::BrilligOpcode::Store serde::Deserializable<Acir::BrilligOpcode::Store>::deserialize(Deserializer &deserializer) {
    Acir::BrilligOpcode::Store obj;
    obj.destination_pointer = serde::Deserializable<decltype(obj.destination_pointer)>::deserialize(deserializer);
    obj.source = serde::Deserializable<decltype(obj.source)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BrilligOpcode::BlackBox &lhs, const BrilligOpcode::BlackBox &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BrilligOpcode::BlackBox>::serialize(const Acir::BrilligOpcode::BlackBox &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Acir::BrilligOpcode::BlackBox serde::Deserializable<Acir::BrilligOpcode::BlackBox>::deserialize(Deserializer &deserializer) {
    Acir::BrilligOpcode::BlackBox obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BrilligOpcode::Trap &lhs, const BrilligOpcode::Trap &rhs) {
        if (!(lhs.revert_data == rhs.revert_data)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BrilligOpcode::Trap>::serialize(const Acir::BrilligOpcode::Trap &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.revert_data)>::serialize(obj.revert_data, serializer);
}

template <>
template <typename Deserializer>
Acir::BrilligOpcode::Trap serde::Deserializable<Acir::BrilligOpcode::Trap>::deserialize(Deserializer &deserializer) {
    Acir::BrilligOpcode::Trap obj;
    obj.revert_data = serde::Deserializable<decltype(obj.revert_data)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BrilligOpcode::Stop &lhs, const BrilligOpcode::Stop &rhs) {
        if (!(lhs.return_data == rhs.return_data)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BrilligOpcode::Stop>::serialize(const Acir::BrilligOpcode::Stop &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.return_data)>::serialize(obj.return_data, serializer);
}

template <>
template <typename Deserializer>
Acir::BrilligOpcode::Stop serde::Deserializable<Acir::BrilligOpcode::Stop>::deserialize(Deserializer &deserializer) {
    Acir::BrilligOpcode::Stop obj;
    obj.return_data = serde::Deserializable<decltype(obj.return_data)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BrilligOutputs &lhs, const BrilligOutputs &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BrilligOutputs>::serialize(const Acir::BrilligOutputs &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Acir::BrilligOutputs serde::Deserializable<Acir::BrilligOutputs>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Acir::BrilligOutputs obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Acir {

    inline bool operator==(const BrilligOutputs::Simple &lhs, const BrilligOutputs::Simple &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BrilligOutputs::Simple>::serialize(const Acir::BrilligOutputs::Simple &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Acir::BrilligOutputs::Simple serde::Deserializable<Acir::BrilligOutputs::Simple>::deserialize(Deserializer &deserializer) {
    Acir::BrilligOutputs::Simple obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BrilligOutputs::Array &lhs, const BrilligOutputs::Array &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BrilligOutputs::Array>::serialize(const Acir::BrilligOutputs::Array &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Acir::BrilligOutputs::Array serde::Deserializable<Acir::BrilligOutputs::Array>::deserialize(Deserializer &deserializer) {
    Acir::BrilligOutputs::Array obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const Circuit &lhs, const Circuit &rhs) {
        if (!(lhs.function_name == rhs.function_name)) { return false; }
        if (!(lhs.current_witness_index == rhs.current_witness_index)) { return false; }
        if (!(lhs.opcodes == rhs.opcodes)) { return false; }
        if (!(lhs.private_parameters == rhs.private_parameters)) { return false; }
        if (!(lhs.public_parameters == rhs.public_parameters)) { return false; }
        if (!(lhs.return_values == rhs.return_values)) { return false; }
        if (!(lhs.assert_messages == rhs.assert_messages)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::Circuit>::serialize(const Acir::Circuit &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.function_name)>::serialize(obj.function_name, serializer);
    serde::Serializable<decltype(obj.current_witness_index)>::serialize(obj.current_witness_index, serializer);
    serde::Serializable<decltype(obj.opcodes)>::serialize(obj.opcodes, serializer);
    serde::Serializable<decltype(obj.private_parameters)>::serialize(obj.private_parameters, serializer);
    serde::Serializable<decltype(obj.public_parameters)>::serialize(obj.public_parameters, serializer);
    serde::Serializable<decltype(obj.return_values)>::serialize(obj.return_values, serializer);
    serde::Serializable<decltype(obj.assert_messages)>::serialize(obj.assert_messages, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Acir::Circuit serde::Deserializable<Acir::Circuit>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Acir::Circuit obj;
    obj.function_name = serde::Deserializable<decltype(obj.function_name)>::deserialize(deserializer);
    obj.current_witness_index = serde::Deserializable<decltype(obj.current_witness_index)>::deserialize(deserializer);
    obj.opcodes = serde::Deserializable<decltype(obj.opcodes)>::deserialize(deserializer);
    obj.private_parameters = serde::Deserializable<decltype(obj.private_parameters)>::deserialize(deserializer);
    obj.public_parameters = serde::Deserializable<decltype(obj.public_parameters)>::deserialize(deserializer);
    obj.return_values = serde::Deserializable<decltype(obj.return_values)>::deserialize(deserializer);
    obj.assert_messages = serde::Deserializable<decltype(obj.assert_messages)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Acir {

    inline bool operator==(const Expression &lhs, const Expression &rhs) {
        if (!(lhs.mul_terms == rhs.mul_terms)) { return false; }
        if (!(lhs.linear_combinations == rhs.linear_combinations)) { return false; }
        if (!(lhs.q_c == rhs.q_c)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::Expression>::serialize(const Acir::Expression &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.mul_terms)>::serialize(obj.mul_terms, serializer);
    serde::Serializable<decltype(obj.linear_combinations)>::serialize(obj.linear_combinations, serializer);
    serde::Serializable<decltype(obj.q_c)>::serialize(obj.q_c, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Acir::Expression serde::Deserializable<Acir::Expression>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Acir::Expression obj;
    obj.mul_terms = serde::Deserializable<decltype(obj.mul_terms)>::deserialize(deserializer);
    obj.linear_combinations = serde::Deserializable<decltype(obj.linear_combinations)>::deserialize(deserializer);
    obj.q_c = serde::Deserializable<decltype(obj.q_c)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Acir {

    inline bool operator==(const ExpressionOrMemory &lhs, const ExpressionOrMemory &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::ExpressionOrMemory>::serialize(const Acir::ExpressionOrMemory &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Acir::ExpressionOrMemory serde::Deserializable<Acir::ExpressionOrMemory>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Acir::ExpressionOrMemory obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Acir {

    inline bool operator==(const ExpressionOrMemory::Expression &lhs, const ExpressionOrMemory::Expression &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::ExpressionOrMemory::Expression>::serialize(const Acir::ExpressionOrMemory::Expression &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Acir::ExpressionOrMemory::Expression serde::Deserializable<Acir::ExpressionOrMemory::Expression>::deserialize(Deserializer &deserializer) {
    Acir::ExpressionOrMemory::Expression obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const ExpressionOrMemory::Memory &lhs, const ExpressionOrMemory::Memory &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::ExpressionOrMemory::Memory>::serialize(const Acir::ExpressionOrMemory::Memory &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Acir::ExpressionOrMemory::Memory serde::Deserializable<Acir::ExpressionOrMemory::Memory>::deserialize(Deserializer &deserializer) {
    Acir::ExpressionOrMemory::Memory obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const ExpressionWidth &lhs, const ExpressionWidth &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::ExpressionWidth>::serialize(const Acir::ExpressionWidth &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Acir::ExpressionWidth serde::Deserializable<Acir::ExpressionWidth>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Acir::ExpressionWidth obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Acir {

    inline bool operator==(const ExpressionWidth::Unbounded &lhs, const ExpressionWidth::Unbounded &rhs) {
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::ExpressionWidth::Unbounded>::serialize(const Acir::ExpressionWidth::Unbounded &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Acir::ExpressionWidth::Unbounded serde::Deserializable<Acir::ExpressionWidth::Unbounded>::deserialize(Deserializer &deserializer) {
    Acir::ExpressionWidth::Unbounded obj;
    return obj;
}

namespace Acir {

    inline bool operator==(const ExpressionWidth::Bounded &lhs, const ExpressionWidth::Bounded &rhs) {
        if (!(lhs.width == rhs.width)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::ExpressionWidth::Bounded>::serialize(const Acir::ExpressionWidth::Bounded &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.width)>::serialize(obj.width, serializer);
}

template <>
template <typename Deserializer>
Acir::ExpressionWidth::Bounded serde::Deserializable<Acir::ExpressionWidth::Bounded>::deserialize(Deserializer &deserializer) {
    Acir::ExpressionWidth::Bounded obj;
    obj.width = serde::Deserializable<decltype(obj.width)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const FunctionInput &lhs, const FunctionInput &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::FunctionInput>::serialize(const Acir::FunctionInput &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Acir::FunctionInput serde::Deserializable<Acir::FunctionInput>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Acir::FunctionInput obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Acir {

    inline bool operator==(const FunctionInput::Constant &lhs, const FunctionInput::Constant &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::FunctionInput::Constant>::serialize(const Acir::FunctionInput::Constant &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Acir::FunctionInput::Constant serde::Deserializable<Acir::FunctionInput::Constant>::deserialize(Deserializer &deserializer) {
    Acir::FunctionInput::Constant obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const FunctionInput::Witness &lhs, const FunctionInput::Witness &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::FunctionInput::Witness>::serialize(const Acir::FunctionInput::Witness &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Acir::FunctionInput::Witness serde::Deserializable<Acir::FunctionInput::Witness>::deserialize(Deserializer &deserializer) {
    Acir::FunctionInput::Witness obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const HeapArray &lhs, const HeapArray &rhs) {
        if (!(lhs.pointer == rhs.pointer)) { return false; }
        if (!(lhs.size == rhs.size)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::HeapArray>::serialize(const Acir::HeapArray &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.pointer)>::serialize(obj.pointer, serializer);
    serde::Serializable<decltype(obj.size)>::serialize(obj.size, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Acir::HeapArray serde::Deserializable<Acir::HeapArray>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Acir::HeapArray obj;
    obj.pointer = serde::Deserializable<decltype(obj.pointer)>::deserialize(deserializer);
    obj.size = serde::Deserializable<decltype(obj.size)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Acir {

    inline bool operator==(const HeapValueType &lhs, const HeapValueType &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::HeapValueType>::serialize(const Acir::HeapValueType &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Acir::HeapValueType serde::Deserializable<Acir::HeapValueType>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Acir::HeapValueType obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Acir {

    inline bool operator==(const HeapValueType::Simple &lhs, const HeapValueType::Simple &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::HeapValueType::Simple>::serialize(const Acir::HeapValueType::Simple &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Acir::HeapValueType::Simple serde::Deserializable<Acir::HeapValueType::Simple>::deserialize(Deserializer &deserializer) {
    Acir::HeapValueType::Simple obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const HeapValueType::Array &lhs, const HeapValueType::Array &rhs) {
        if (!(lhs.value_types == rhs.value_types)) { return false; }
        if (!(lhs.size == rhs.size)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::HeapValueType::Array>::serialize(const Acir::HeapValueType::Array &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value_types)>::serialize(obj.value_types, serializer);
    serde::Serializable<decltype(obj.size)>::serialize(obj.size, serializer);
}

template <>
template <typename Deserializer>
Acir::HeapValueType::Array serde::Deserializable<Acir::HeapValueType::Array>::deserialize(Deserializer &deserializer) {
    Acir::HeapValueType::Array obj;
    obj.value_types = serde::Deserializable<decltype(obj.value_types)>::deserialize(deserializer);
    obj.size = serde::Deserializable<decltype(obj.size)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const HeapValueType::Vector &lhs, const HeapValueType::Vector &rhs) {
        if (!(lhs.value_types == rhs.value_types)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::HeapValueType::Vector>::serialize(const Acir::HeapValueType::Vector &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value_types)>::serialize(obj.value_types, serializer);
}

template <>
template <typename Deserializer>
Acir::HeapValueType::Vector serde::Deserializable<Acir::HeapValueType::Vector>::deserialize(Deserializer &deserializer) {
    Acir::HeapValueType::Vector obj;
    obj.value_types = serde::Deserializable<decltype(obj.value_types)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const HeapVector &lhs, const HeapVector &rhs) {
        if (!(lhs.pointer == rhs.pointer)) { return false; }
        if (!(lhs.size == rhs.size)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::HeapVector>::serialize(const Acir::HeapVector &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.pointer)>::serialize(obj.pointer, serializer);
    serde::Serializable<decltype(obj.size)>::serialize(obj.size, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Acir::HeapVector serde::Deserializable<Acir::HeapVector>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Acir::HeapVector obj;
    obj.pointer = serde::Deserializable<decltype(obj.pointer)>::deserialize(deserializer);
    obj.size = serde::Deserializable<decltype(obj.size)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Acir {

    inline bool operator==(const IntegerBitSize &lhs, const IntegerBitSize &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::IntegerBitSize>::serialize(const Acir::IntegerBitSize &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Acir::IntegerBitSize serde::Deserializable<Acir::IntegerBitSize>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Acir::IntegerBitSize obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Acir {

    inline bool operator==(const IntegerBitSize::U1 &lhs, const IntegerBitSize::U1 &rhs) {
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::IntegerBitSize::U1>::serialize(const Acir::IntegerBitSize::U1 &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Acir::IntegerBitSize::U1 serde::Deserializable<Acir::IntegerBitSize::U1>::deserialize(Deserializer &deserializer) {
    Acir::IntegerBitSize::U1 obj;
    return obj;
}

namespace Acir {

    inline bool operator==(const IntegerBitSize::U8 &lhs, const IntegerBitSize::U8 &rhs) {
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::IntegerBitSize::U8>::serialize(const Acir::IntegerBitSize::U8 &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Acir::IntegerBitSize::U8 serde::Deserializable<Acir::IntegerBitSize::U8>::deserialize(Deserializer &deserializer) {
    Acir::IntegerBitSize::U8 obj;
    return obj;
}

namespace Acir {

    inline bool operator==(const IntegerBitSize::U16 &lhs, const IntegerBitSize::U16 &rhs) {
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::IntegerBitSize::U16>::serialize(const Acir::IntegerBitSize::U16 &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Acir::IntegerBitSize::U16 serde::Deserializable<Acir::IntegerBitSize::U16>::deserialize(Deserializer &deserializer) {
    Acir::IntegerBitSize::U16 obj;
    return obj;
}

namespace Acir {

    inline bool operator==(const IntegerBitSize::U32 &lhs, const IntegerBitSize::U32 &rhs) {
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::IntegerBitSize::U32>::serialize(const Acir::IntegerBitSize::U32 &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Acir::IntegerBitSize::U32 serde::Deserializable<Acir::IntegerBitSize::U32>::deserialize(Deserializer &deserializer) {
    Acir::IntegerBitSize::U32 obj;
    return obj;
}

namespace Acir {

    inline bool operator==(const IntegerBitSize::U64 &lhs, const IntegerBitSize::U64 &rhs) {
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::IntegerBitSize::U64>::serialize(const Acir::IntegerBitSize::U64 &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Acir::IntegerBitSize::U64 serde::Deserializable<Acir::IntegerBitSize::U64>::deserialize(Deserializer &deserializer) {
    Acir::IntegerBitSize::U64 obj;
    return obj;
}

namespace Acir {

    inline bool operator==(const IntegerBitSize::U128 &lhs, const IntegerBitSize::U128 &rhs) {
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::IntegerBitSize::U128>::serialize(const Acir::IntegerBitSize::U128 &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Acir::IntegerBitSize::U128 serde::Deserializable<Acir::IntegerBitSize::U128>::deserialize(Deserializer &deserializer) {
    Acir::IntegerBitSize::U128 obj;
    return obj;
}

namespace Acir {

    inline bool operator==(const MemOp &lhs, const MemOp &rhs) {
        if (!(lhs.operation == rhs.operation)) { return false; }
        if (!(lhs.index == rhs.index)) { return false; }
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::MemOp>::serialize(const Acir::MemOp &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.operation)>::serialize(obj.operation, serializer);
    serde::Serializable<decltype(obj.index)>::serialize(obj.index, serializer);
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Acir::MemOp serde::Deserializable<Acir::MemOp>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Acir::MemOp obj;
    obj.operation = serde::Deserializable<decltype(obj.operation)>::deserialize(deserializer);
    obj.index = serde::Deserializable<decltype(obj.index)>::deserialize(deserializer);
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Acir {

    inline bool operator==(const MemoryAddress &lhs, const MemoryAddress &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::MemoryAddress>::serialize(const Acir::MemoryAddress &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Acir::MemoryAddress serde::Deserializable<Acir::MemoryAddress>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Acir::MemoryAddress obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Acir {

    inline bool operator==(const MemoryAddress::Direct &lhs, const MemoryAddress::Direct &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::MemoryAddress::Direct>::serialize(const Acir::MemoryAddress::Direct &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Acir::MemoryAddress::Direct serde::Deserializable<Acir::MemoryAddress::Direct>::deserialize(Deserializer &deserializer) {
    Acir::MemoryAddress::Direct obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const MemoryAddress::Relative &lhs, const MemoryAddress::Relative &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::MemoryAddress::Relative>::serialize(const Acir::MemoryAddress::Relative &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Acir::MemoryAddress::Relative serde::Deserializable<Acir::MemoryAddress::Relative>::deserialize(Deserializer &deserializer) {
    Acir::MemoryAddress::Relative obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const Opcode &lhs, const Opcode &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::Opcode>::serialize(const Acir::Opcode &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Acir::Opcode serde::Deserializable<Acir::Opcode>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Acir::Opcode obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Acir {

    inline bool operator==(const Opcode::AssertZero &lhs, const Opcode::AssertZero &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::Opcode::AssertZero>::serialize(const Acir::Opcode::AssertZero &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Acir::Opcode::AssertZero serde::Deserializable<Acir::Opcode::AssertZero>::deserialize(Deserializer &deserializer) {
    Acir::Opcode::AssertZero obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const Opcode::BlackBoxFuncCall &lhs, const Opcode::BlackBoxFuncCall &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::Opcode::BlackBoxFuncCall>::serialize(const Acir::Opcode::BlackBoxFuncCall &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Acir::Opcode::BlackBoxFuncCall serde::Deserializable<Acir::Opcode::BlackBoxFuncCall>::deserialize(Deserializer &deserializer) {
    Acir::Opcode::BlackBoxFuncCall obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const Opcode::MemoryOp &lhs, const Opcode::MemoryOp &rhs) {
        if (!(lhs.block_id == rhs.block_id)) { return false; }
        if (!(lhs.op == rhs.op)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::Opcode::MemoryOp>::serialize(const Acir::Opcode::MemoryOp &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.block_id)>::serialize(obj.block_id, serializer);
    serde::Serializable<decltype(obj.op)>::serialize(obj.op, serializer);
}

template <>
template <typename Deserializer>
Acir::Opcode::MemoryOp serde::Deserializable<Acir::Opcode::MemoryOp>::deserialize(Deserializer &deserializer) {
    Acir::Opcode::MemoryOp obj;
    obj.block_id = serde::Deserializable<decltype(obj.block_id)>::deserialize(deserializer);
    obj.op = serde::Deserializable<decltype(obj.op)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const Opcode::MemoryInit &lhs, const Opcode::MemoryInit &rhs) {
        if (!(lhs.block_id == rhs.block_id)) { return false; }
        if (!(lhs.init == rhs.init)) { return false; }
        if (!(lhs.block_type == rhs.block_type)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::Opcode::MemoryInit>::serialize(const Acir::Opcode::MemoryInit &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.block_id)>::serialize(obj.block_id, serializer);
    serde::Serializable<decltype(obj.init)>::serialize(obj.init, serializer);
    serde::Serializable<decltype(obj.block_type)>::serialize(obj.block_type, serializer);
}

template <>
template <typename Deserializer>
Acir::Opcode::MemoryInit serde::Deserializable<Acir::Opcode::MemoryInit>::deserialize(Deserializer &deserializer) {
    Acir::Opcode::MemoryInit obj;
    obj.block_id = serde::Deserializable<decltype(obj.block_id)>::deserialize(deserializer);
    obj.init = serde::Deserializable<decltype(obj.init)>::deserialize(deserializer);
    obj.block_type = serde::Deserializable<decltype(obj.block_type)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const Opcode::BrilligCall &lhs, const Opcode::BrilligCall &rhs) {
        if (!(lhs.id == rhs.id)) { return false; }
        if (!(lhs.inputs == rhs.inputs)) { return false; }
        if (!(lhs.outputs == rhs.outputs)) { return false; }
        if (!(lhs.predicate == rhs.predicate)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::Opcode::BrilligCall>::serialize(const Acir::Opcode::BrilligCall &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.id)>::serialize(obj.id, serializer);
    serde::Serializable<decltype(obj.inputs)>::serialize(obj.inputs, serializer);
    serde::Serializable<decltype(obj.outputs)>::serialize(obj.outputs, serializer);
    serde::Serializable<decltype(obj.predicate)>::serialize(obj.predicate, serializer);
}

template <>
template <typename Deserializer>
Acir::Opcode::BrilligCall serde::Deserializable<Acir::Opcode::BrilligCall>::deserialize(Deserializer &deserializer) {
    Acir::Opcode::BrilligCall obj;
    obj.id = serde::Deserializable<decltype(obj.id)>::deserialize(deserializer);
    obj.inputs = serde::Deserializable<decltype(obj.inputs)>::deserialize(deserializer);
    obj.outputs = serde::Deserializable<decltype(obj.outputs)>::deserialize(deserializer);
    obj.predicate = serde::Deserializable<decltype(obj.predicate)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const Opcode::Call &lhs, const Opcode::Call &rhs) {
        if (!(lhs.id == rhs.id)) { return false; }
        if (!(lhs.inputs == rhs.inputs)) { return false; }
        if (!(lhs.outputs == rhs.outputs)) { return false; }
        if (!(lhs.predicate == rhs.predicate)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::Opcode::Call>::serialize(const Acir::Opcode::Call &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.id)>::serialize(obj.id, serializer);
    serde::Serializable<decltype(obj.inputs)>::serialize(obj.inputs, serializer);
    serde::Serializable<decltype(obj.outputs)>::serialize(obj.outputs, serializer);
    serde::Serializable<decltype(obj.predicate)>::serialize(obj.predicate, serializer);
}

template <>
template <typename Deserializer>
Acir::Opcode::Call serde::Deserializable<Acir::Opcode::Call>::deserialize(Deserializer &deserializer) {
    Acir::Opcode::Call obj;
    obj.id = serde::Deserializable<decltype(obj.id)>::deserialize(deserializer);
    obj.inputs = serde::Deserializable<decltype(obj.inputs)>::deserialize(deserializer);
    obj.outputs = serde::Deserializable<decltype(obj.outputs)>::deserialize(deserializer);
    obj.predicate = serde::Deserializable<decltype(obj.predicate)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const OpcodeLocation &lhs, const OpcodeLocation &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::OpcodeLocation>::serialize(const Acir::OpcodeLocation &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Acir::OpcodeLocation serde::Deserializable<Acir::OpcodeLocation>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Acir::OpcodeLocation obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Acir {

    inline bool operator==(const OpcodeLocation::Acir &lhs, const OpcodeLocation::Acir &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::OpcodeLocation::Acir>::serialize(const Acir::OpcodeLocation::Acir &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Acir::OpcodeLocation::Acir serde::Deserializable<Acir::OpcodeLocation::Acir>::deserialize(Deserializer &deserializer) {
    Acir::OpcodeLocation::Acir obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const OpcodeLocation::Brillig &lhs, const OpcodeLocation::Brillig &rhs) {
        if (!(lhs.acir_index == rhs.acir_index)) { return false; }
        if (!(lhs.brillig_index == rhs.brillig_index)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::OpcodeLocation::Brillig>::serialize(const Acir::OpcodeLocation::Brillig &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.acir_index)>::serialize(obj.acir_index, serializer);
    serde::Serializable<decltype(obj.brillig_index)>::serialize(obj.brillig_index, serializer);
}

template <>
template <typename Deserializer>
Acir::OpcodeLocation::Brillig serde::Deserializable<Acir::OpcodeLocation::Brillig>::deserialize(Deserializer &deserializer) {
    Acir::OpcodeLocation::Brillig obj;
    obj.acir_index = serde::Deserializable<decltype(obj.acir_index)>::deserialize(deserializer);
    obj.brillig_index = serde::Deserializable<decltype(obj.brillig_index)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const Program &lhs, const Program &rhs) {
        if (!(lhs.functions == rhs.functions)) { return false; }
        if (!(lhs.unconstrained_functions == rhs.unconstrained_functions)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::Program>::serialize(const Acir::Program &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.functions)>::serialize(obj.functions, serializer);
    serde::Serializable<decltype(obj.unconstrained_functions)>::serialize(obj.unconstrained_functions, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Acir::Program serde::Deserializable<Acir::Program>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Acir::Program obj;
    obj.functions = serde::Deserializable<decltype(obj.functions)>::deserialize(deserializer);
    obj.unconstrained_functions = serde::Deserializable<decltype(obj.unconstrained_functions)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Acir {

    inline bool operator==(const ProgramWithoutBrillig &lhs, const ProgramWithoutBrillig &rhs) {
        if (!(lhs.functions == rhs.functions)) { return false; }
        if (!(lhs.unconstrained_functions == rhs.unconstrained_functions)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::ProgramWithoutBrillig>::serialize(const Acir::ProgramWithoutBrillig &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.functions)>::serialize(obj.functions, serializer);
    serde::Serializable<decltype(obj.unconstrained_functions)>::serialize(obj.unconstrained_functions, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Acir::ProgramWithoutBrillig serde::Deserializable<Acir::ProgramWithoutBrillig>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Acir::ProgramWithoutBrillig obj;
    obj.functions = serde::Deserializable<decltype(obj.functions)>::deserialize(deserializer);
    obj.unconstrained_functions = serde::Deserializable<decltype(obj.unconstrained_functions)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Acir {

    inline bool operator==(const PublicInputs &lhs, const PublicInputs &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::PublicInputs>::serialize(const Acir::PublicInputs &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Acir::PublicInputs serde::Deserializable<Acir::PublicInputs>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Acir::PublicInputs obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Acir {

    inline bool operator==(const SemanticLength &lhs, const SemanticLength &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> SemanticLength::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<SemanticLength>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline SemanticLength SemanticLength::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<SemanticLength>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::SemanticLength>::serialize(const Acir::SemanticLength &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Acir::SemanticLength serde::Deserializable<Acir::SemanticLength>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Acir::SemanticLength obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Acir {

    inline bool operator==(const SemiFlattenedLength &lhs, const SemiFlattenedLength &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> SemiFlattenedLength::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<SemiFlattenedLength>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline SemiFlattenedLength SemiFlattenedLength::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<SemiFlattenedLength>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::SemiFlattenedLength>::serialize(const Acir::SemiFlattenedLength &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Acir::SemiFlattenedLength serde::Deserializable<Acir::SemiFlattenedLength>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Acir::SemiFlattenedLength obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Acir {

    inline bool operator==(const ValueOrArray &lhs, const ValueOrArray &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::ValueOrArray>::serialize(const Acir::ValueOrArray &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Acir::ValueOrArray serde::Deserializable<Acir::ValueOrArray>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Acir::ValueOrArray obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Acir {

    inline bool operator==(const ValueOrArray::MemoryAddress &lhs, const ValueOrArray::MemoryAddress &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::ValueOrArray::MemoryAddress>::serialize(const Acir::ValueOrArray::MemoryAddress &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Acir::ValueOrArray::MemoryAddress serde::Deserializable<Acir::ValueOrArray::MemoryAddress>::deserialize(Deserializer &deserializer) {
    Acir::ValueOrArray::MemoryAddress obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const ValueOrArray::HeapArray &lhs, const ValueOrArray::HeapArray &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::ValueOrArray::HeapArray>::serialize(const Acir::ValueOrArray::HeapArray &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Acir::ValueOrArray::HeapArray serde::Deserializable<Acir::ValueOrArray::HeapArray>::deserialize(Deserializer &deserializer) {
    Acir::ValueOrArray::HeapArray obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const ValueOrArray::HeapVector &lhs, const ValueOrArray::HeapVector &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::ValueOrArray::HeapVector>::serialize(const Acir::ValueOrArray::HeapVector &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Acir::ValueOrArray::HeapVector serde::Deserializable<Acir::ValueOrArray::HeapVector>::deserialize(Deserializer &deserializer) {
    Acir::ValueOrArray::HeapVector obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const Witness &lhs, const Witness &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::Witness>::serialize(const Acir::Witness &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Acir::Witness serde::Deserializable<Acir::Witness>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Acir::Witness obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}
