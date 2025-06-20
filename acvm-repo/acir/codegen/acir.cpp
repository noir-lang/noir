#pragma once

#include "serde.hpp"
#include "msgpack.hpp"
#include "bincode.hpp"

namespace Acir {
    struct Helpers {
        static std::map<std::string, msgpack::object const*> make_kvmap(
            msgpack::object const& o,
            std::string const& name
        ) {
            if(o.type != msgpack::type::MAP) {
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
    };
    }

namespace Acir {

    struct BinaryFieldOp {

        struct Add {
            friend bool operator==(const Add&, const Add&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Add bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct Sub {
            friend bool operator==(const Sub&, const Sub&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Sub bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct Mul {
            friend bool operator==(const Mul&, const Mul&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Mul bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct Div {
            friend bool operator==(const Div&, const Div&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Div bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct IntegerDiv {
            friend bool operator==(const IntegerDiv&, const IntegerDiv&);
            std::vector<uint8_t> bincodeSerialize() const;
            static IntegerDiv bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct Equals {
            friend bool operator==(const Equals&, const Equals&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Equals bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct LessThan {
            friend bool operator==(const LessThan&, const LessThan&);
            std::vector<uint8_t> bincodeSerialize() const;
            static LessThan bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct LessThanEquals {
            friend bool operator==(const LessThanEquals&, const LessThanEquals&);
            std::vector<uint8_t> bincodeSerialize() const;
            static LessThanEquals bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        std::variant<Add, Sub, Mul, Div, IntegerDiv, Equals, LessThan, LessThanEquals> value;

        friend bool operator==(const BinaryFieldOp&, const BinaryFieldOp&);
        std::vector<uint8_t> bincodeSerialize() const;
        static BinaryFieldOp bincodeDeserialize(std::vector<uint8_t>);

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
                    std::map<std::string, msgpack::object> data;
                    data[tag] = msgpack::object(arg);
                    packer.pack(data);
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
            std::vector<uint8_t> bincodeSerialize() const;
            static Add bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct Sub {
            friend bool operator==(const Sub&, const Sub&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Sub bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct Mul {
            friend bool operator==(const Mul&, const Mul&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Mul bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct Div {
            friend bool operator==(const Div&, const Div&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Div bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct Equals {
            friend bool operator==(const Equals&, const Equals&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Equals bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct LessThan {
            friend bool operator==(const LessThan&, const LessThan&);
            std::vector<uint8_t> bincodeSerialize() const;
            static LessThan bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct LessThanEquals {
            friend bool operator==(const LessThanEquals&, const LessThanEquals&);
            std::vector<uint8_t> bincodeSerialize() const;
            static LessThanEquals bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct And {
            friend bool operator==(const And&, const And&);
            std::vector<uint8_t> bincodeSerialize() const;
            static And bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct Or {
            friend bool operator==(const Or&, const Or&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Or bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct Xor {
            friend bool operator==(const Xor&, const Xor&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Xor bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct Shl {
            friend bool operator==(const Shl&, const Shl&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Shl bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct Shr {
            friend bool operator==(const Shr&, const Shr&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Shr bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        std::variant<Add, Sub, Mul, Div, Equals, LessThan, LessThanEquals, And, Or, Xor, Shl, Shr> value;

        friend bool operator==(const BinaryIntOp&, const BinaryIntOp&);
        std::vector<uint8_t> bincodeSerialize() const;
        static BinaryIntOp bincodeDeserialize(std::vector<uint8_t>);

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
                    std::map<std::string, msgpack::object> data;
                    data[tag] = msgpack::object(arg);
                    packer.pack(data);
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
            std::vector<uint8_t> bincodeSerialize() const;
            static U1 bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct U8 {
            friend bool operator==(const U8&, const U8&);
            std::vector<uint8_t> bincodeSerialize() const;
            static U8 bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct U16 {
            friend bool operator==(const U16&, const U16&);
            std::vector<uint8_t> bincodeSerialize() const;
            static U16 bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct U32 {
            friend bool operator==(const U32&, const U32&);
            std::vector<uint8_t> bincodeSerialize() const;
            static U32 bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct U64 {
            friend bool operator==(const U64&, const U64&);
            std::vector<uint8_t> bincodeSerialize() const;
            static U64 bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct U128 {
            friend bool operator==(const U128&, const U128&);
            std::vector<uint8_t> bincodeSerialize() const;
            static U128 bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        std::variant<U1, U8, U16, U32, U64, U128> value;

        friend bool operator==(const IntegerBitSize&, const IntegerBitSize&);
        std::vector<uint8_t> bincodeSerialize() const;
        static IntegerBitSize bincodeDeserialize(std::vector<uint8_t>);

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
                    std::map<std::string, msgpack::object> data;
                    data[tag] = msgpack::object(arg);
                    packer.pack(data);
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
            std::vector<uint8_t> bincodeSerialize() const;
            static Field bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct Integer {
            Acir::IntegerBitSize value;

            friend bool operator==(const Integer&, const Integer&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Integer bincodeDeserialize(std::vector<uint8_t>);

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
        std::vector<uint8_t> bincodeSerialize() const;
        static BitSize bincodeDeserialize(std::vector<uint8_t>);

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
                    std::map<std::string, msgpack::object> data;
                    data[tag] = msgpack::object(arg);
                    packer.pack(data);
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
            std::vector<uint8_t> bincodeSerialize() const;
            static Direct bincodeDeserialize(std::vector<uint8_t>);

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
            std::vector<uint8_t> bincodeSerialize() const;
            static Relative bincodeDeserialize(std::vector<uint8_t>);

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
        std::vector<uint8_t> bincodeSerialize() const;
        static MemoryAddress bincodeDeserialize(std::vector<uint8_t>);

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
                    std::map<std::string, msgpack::object> data;
                    data[tag] = msgpack::object(arg);
                    packer.pack(data);
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

    struct HeapArray {
        Acir::MemoryAddress pointer;
        uint64_t size;

        friend bool operator==(const HeapArray&, const HeapArray&);
        std::vector<uint8_t> bincodeSerialize() const;
        static HeapArray bincodeDeserialize(std::vector<uint8_t>);

        void msgpack_pack(auto& packer) const {
            packer.pack_map(2);
            packer.pack(std::make_pair("pointer", pointer));
            packer.pack(std::make_pair("size", size));
        }

        void msgpack_unpack(msgpack::object const& o) {
            auto name = "HeapArray";
            auto kvmap = Helpers::make_kvmap(o, name);
            Helpers::conv_fld_from_kvmap(kvmap, name, "pointer", pointer, false);
            Helpers::conv_fld_from_kvmap(kvmap, name, "size", size, false);
        }
    };

    struct HeapVector {
        Acir::MemoryAddress pointer;
        Acir::MemoryAddress size;

        friend bool operator==(const HeapVector&, const HeapVector&);
        std::vector<uint8_t> bincodeSerialize() const;
        static HeapVector bincodeDeserialize(std::vector<uint8_t>);

        void msgpack_pack(auto& packer) const {
            packer.pack_map(2);
            packer.pack(std::make_pair("pointer", pointer));
            packer.pack(std::make_pair("size", size));
        }

        void msgpack_unpack(msgpack::object const& o) {
            auto name = "HeapVector";
            auto kvmap = Helpers::make_kvmap(o, name);
            Helpers::conv_fld_from_kvmap(kvmap, name, "pointer", pointer, false);
            Helpers::conv_fld_from_kvmap(kvmap, name, "size", size, false);
        }
    };

    struct BlackBoxOp {

        struct AES128Encrypt {
            Acir::HeapVector inputs;
            Acir::HeapArray iv;
            Acir::HeapArray key;
            Acir::HeapVector outputs;

            friend bool operator==(const AES128Encrypt&, const AES128Encrypt&);
            std::vector<uint8_t> bincodeSerialize() const;
            static AES128Encrypt bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(4);
                packer.pack(std::make_pair("inputs", inputs));
                packer.pack(std::make_pair("iv", iv));
                packer.pack(std::make_pair("key", key));
                packer.pack(std::make_pair("outputs", outputs));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "AES128Encrypt";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "inputs", inputs, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "iv", iv, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "key", key, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "outputs", outputs, false);
            }
        };

        struct Blake2s {
            Acir::HeapVector message;
            Acir::HeapArray output;

            friend bool operator==(const Blake2s&, const Blake2s&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Blake2s bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(2);
                packer.pack(std::make_pair("message", message));
                packer.pack(std::make_pair("output", output));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "Blake2s";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "message", message, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "output", output, false);
            }
        };

        struct Blake3 {
            Acir::HeapVector message;
            Acir::HeapArray output;

            friend bool operator==(const Blake3&, const Blake3&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Blake3 bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(2);
                packer.pack(std::make_pair("message", message));
                packer.pack(std::make_pair("output", output));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "Blake3";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "message", message, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "output", output, false);
            }
        };

        struct Keccakf1600 {
            Acir::HeapArray input;
            Acir::HeapArray output;

            friend bool operator==(const Keccakf1600&, const Keccakf1600&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Keccakf1600 bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(2);
                packer.pack(std::make_pair("input", input));
                packer.pack(std::make_pair("output", output));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "Keccakf1600";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "input", input, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "output", output, false);
            }
        };

        struct EcdsaSecp256k1 {
            Acir::HeapVector hashed_msg;
            Acir::HeapArray public_key_x;
            Acir::HeapArray public_key_y;
            Acir::HeapArray signature;
            Acir::MemoryAddress result;

            friend bool operator==(const EcdsaSecp256k1&, const EcdsaSecp256k1&);
            std::vector<uint8_t> bincodeSerialize() const;
            static EcdsaSecp256k1 bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(5);
                packer.pack(std::make_pair("hashed_msg", hashed_msg));
                packer.pack(std::make_pair("public_key_x", public_key_x));
                packer.pack(std::make_pair("public_key_y", public_key_y));
                packer.pack(std::make_pair("signature", signature));
                packer.pack(std::make_pair("result", result));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "EcdsaSecp256k1";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "hashed_msg", hashed_msg, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "public_key_x", public_key_x, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "public_key_y", public_key_y, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "signature", signature, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "result", result, false);
            }
        };

        struct EcdsaSecp256r1 {
            Acir::HeapVector hashed_msg;
            Acir::HeapArray public_key_x;
            Acir::HeapArray public_key_y;
            Acir::HeapArray signature;
            Acir::MemoryAddress result;

            friend bool operator==(const EcdsaSecp256r1&, const EcdsaSecp256r1&);
            std::vector<uint8_t> bincodeSerialize() const;
            static EcdsaSecp256r1 bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(5);
                packer.pack(std::make_pair("hashed_msg", hashed_msg));
                packer.pack(std::make_pair("public_key_x", public_key_x));
                packer.pack(std::make_pair("public_key_y", public_key_y));
                packer.pack(std::make_pair("signature", signature));
                packer.pack(std::make_pair("result", result));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "EcdsaSecp256r1";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "hashed_msg", hashed_msg, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "public_key_x", public_key_x, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "public_key_y", public_key_y, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "signature", signature, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "result", result, false);
            }
        };

        struct MultiScalarMul {
            Acir::HeapVector points;
            Acir::HeapVector scalars;
            Acir::HeapArray outputs;

            friend bool operator==(const MultiScalarMul&, const MultiScalarMul&);
            std::vector<uint8_t> bincodeSerialize() const;
            static MultiScalarMul bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(3);
                packer.pack(std::make_pair("points", points));
                packer.pack(std::make_pair("scalars", scalars));
                packer.pack(std::make_pair("outputs", outputs));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "MultiScalarMul";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "points", points, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "scalars", scalars, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "outputs", outputs, false);
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
            std::vector<uint8_t> bincodeSerialize() const;
            static EmbeddedCurveAdd bincodeDeserialize(std::vector<uint8_t>);

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
                auto name = "EmbeddedCurveAdd";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "input1_x", input1_x, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "input1_y", input1_y, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "input1_infinite", input1_infinite, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "input2_x", input2_x, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "input2_y", input2_y, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "input2_infinite", input2_infinite, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "result", result, false);
            }
        };

        struct BigIntAdd {
            Acir::MemoryAddress lhs;
            Acir::MemoryAddress rhs;
            Acir::MemoryAddress output;

            friend bool operator==(const BigIntAdd&, const BigIntAdd&);
            std::vector<uint8_t> bincodeSerialize() const;
            static BigIntAdd bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(3);
                packer.pack(std::make_pair("lhs", lhs));
                packer.pack(std::make_pair("rhs", rhs));
                packer.pack(std::make_pair("output", output));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "BigIntAdd";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "lhs", lhs, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "rhs", rhs, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "output", output, false);
            }
        };

        struct BigIntSub {
            Acir::MemoryAddress lhs;
            Acir::MemoryAddress rhs;
            Acir::MemoryAddress output;

            friend bool operator==(const BigIntSub&, const BigIntSub&);
            std::vector<uint8_t> bincodeSerialize() const;
            static BigIntSub bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(3);
                packer.pack(std::make_pair("lhs", lhs));
                packer.pack(std::make_pair("rhs", rhs));
                packer.pack(std::make_pair("output", output));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "BigIntSub";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "lhs", lhs, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "rhs", rhs, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "output", output, false);
            }
        };

        struct BigIntMul {
            Acir::MemoryAddress lhs;
            Acir::MemoryAddress rhs;
            Acir::MemoryAddress output;

            friend bool operator==(const BigIntMul&, const BigIntMul&);
            std::vector<uint8_t> bincodeSerialize() const;
            static BigIntMul bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(3);
                packer.pack(std::make_pair("lhs", lhs));
                packer.pack(std::make_pair("rhs", rhs));
                packer.pack(std::make_pair("output", output));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "BigIntMul";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "lhs", lhs, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "rhs", rhs, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "output", output, false);
            }
        };

        struct BigIntDiv {
            Acir::MemoryAddress lhs;
            Acir::MemoryAddress rhs;
            Acir::MemoryAddress output;

            friend bool operator==(const BigIntDiv&, const BigIntDiv&);
            std::vector<uint8_t> bincodeSerialize() const;
            static BigIntDiv bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(3);
                packer.pack(std::make_pair("lhs", lhs));
                packer.pack(std::make_pair("rhs", rhs));
                packer.pack(std::make_pair("output", output));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "BigIntDiv";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "lhs", lhs, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "rhs", rhs, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "output", output, false);
            }
        };

        struct BigIntFromLeBytes {
            Acir::HeapVector inputs;
            Acir::HeapVector modulus;
            Acir::MemoryAddress output;

            friend bool operator==(const BigIntFromLeBytes&, const BigIntFromLeBytes&);
            std::vector<uint8_t> bincodeSerialize() const;
            static BigIntFromLeBytes bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(3);
                packer.pack(std::make_pair("inputs", inputs));
                packer.pack(std::make_pair("modulus", modulus));
                packer.pack(std::make_pair("output", output));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "BigIntFromLeBytes";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "inputs", inputs, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "modulus", modulus, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "output", output, false);
            }
        };

        struct BigIntToLeBytes {
            Acir::MemoryAddress input;
            Acir::HeapVector output;

            friend bool operator==(const BigIntToLeBytes&, const BigIntToLeBytes&);
            std::vector<uint8_t> bincodeSerialize() const;
            static BigIntToLeBytes bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(2);
                packer.pack(std::make_pair("input", input));
                packer.pack(std::make_pair("output", output));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "BigIntToLeBytes";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "input", input, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "output", output, false);
            }
        };

        struct Poseidon2Permutation {
            Acir::HeapVector message;
            Acir::HeapArray output;
            Acir::MemoryAddress len;

            friend bool operator==(const Poseidon2Permutation&, const Poseidon2Permutation&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Poseidon2Permutation bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(3);
                packer.pack(std::make_pair("message", message));
                packer.pack(std::make_pair("output", output));
                packer.pack(std::make_pair("len", len));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "Poseidon2Permutation";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "message", message, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "output", output, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "len", len, false);
            }
        };

        struct Sha256Compression {
            Acir::HeapArray input;
            Acir::HeapArray hash_values;
            Acir::HeapArray output;

            friend bool operator==(const Sha256Compression&, const Sha256Compression&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Sha256Compression bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(3);
                packer.pack(std::make_pair("input", input));
                packer.pack(std::make_pair("hash_values", hash_values));
                packer.pack(std::make_pair("output", output));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "Sha256Compression";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "input", input, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "hash_values", hash_values, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "output", output, false);
            }
        };

        struct ToRadix {
            Acir::MemoryAddress input;
            Acir::MemoryAddress radix;
            Acir::MemoryAddress output_pointer;
            Acir::MemoryAddress num_limbs;
            Acir::MemoryAddress output_bits;

            friend bool operator==(const ToRadix&, const ToRadix&);
            std::vector<uint8_t> bincodeSerialize() const;
            static ToRadix bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(5);
                packer.pack(std::make_pair("input", input));
                packer.pack(std::make_pair("radix", radix));
                packer.pack(std::make_pair("output_pointer", output_pointer));
                packer.pack(std::make_pair("num_limbs", num_limbs));
                packer.pack(std::make_pair("output_bits", output_bits));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "ToRadix";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "input", input, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "radix", radix, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "output_pointer", output_pointer, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "num_limbs", num_limbs, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "output_bits", output_bits, false);
            }
        };

        std::variant<AES128Encrypt, Blake2s, Blake3, Keccakf1600, EcdsaSecp256k1, EcdsaSecp256r1, MultiScalarMul, EmbeddedCurveAdd, BigIntAdd, BigIntSub, BigIntMul, BigIntDiv, BigIntFromLeBytes, BigIntToLeBytes, Poseidon2Permutation, Sha256Compression, ToRadix> value;

        friend bool operator==(const BlackBoxOp&, const BlackBoxOp&);
        std::vector<uint8_t> bincodeSerialize() const;
        static BlackBoxOp bincodeDeserialize(std::vector<uint8_t>);

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
                    tag = "BigIntAdd";
                    is_unit = false;
                    break;
                case 9:
                    tag = "BigIntSub";
                    is_unit = false;
                    break;
                case 10:
                    tag = "BigIntMul";
                    is_unit = false;
                    break;
                case 11:
                    tag = "BigIntDiv";
                    is_unit = false;
                    break;
                case 12:
                    tag = "BigIntFromLeBytes";
                    is_unit = false;
                    break;
                case 13:
                    tag = "BigIntToLeBytes";
                    is_unit = false;
                    break;
                case 14:
                    tag = "Poseidon2Permutation";
                    is_unit = false;
                    break;
                case 15:
                    tag = "Sha256Compression";
                    is_unit = false;
                    break;
                case 16:
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
                    std::map<std::string, msgpack::object> data;
                    data[tag] = msgpack::object(arg);
                    packer.pack(data);
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
            else if (tag == "BigIntAdd") {
                BigIntAdd v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BlackBoxOp::BigIntAdd'");
                }
                
                value = v;
            }
            else if (tag == "BigIntSub") {
                BigIntSub v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BlackBoxOp::BigIntSub'");
                }
                
                value = v;
            }
            else if (tag == "BigIntMul") {
                BigIntMul v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BlackBoxOp::BigIntMul'");
                }
                
                value = v;
            }
            else if (tag == "BigIntDiv") {
                BigIntDiv v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BlackBoxOp::BigIntDiv'");
                }
                
                value = v;
            }
            else if (tag == "BigIntFromLeBytes") {
                BigIntFromLeBytes v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BlackBoxOp::BigIntFromLeBytes'");
                }
                
                value = v;
            }
            else if (tag == "BigIntToLeBytes") {
                BigIntToLeBytes v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BlackBoxOp::BigIntToLeBytes'");
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

    struct HeapValueType;

    struct HeapValueType {

        struct Simple {
            Acir::BitSize value;

            friend bool operator==(const Simple&, const Simple&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Simple bincodeDeserialize(std::vector<uint8_t>);

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
            uint64_t size;

            friend bool operator==(const Array&, const Array&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Array bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(2);
                packer.pack(std::make_pair("value_types", value_types));
                packer.pack(std::make_pair("size", size));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "Array";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "value_types", value_types, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "size", size, false);
            }
        };

        struct Vector {
            std::vector<Acir::HeapValueType> value_types;

            friend bool operator==(const Vector&, const Vector&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Vector bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(1);
                packer.pack(std::make_pair("value_types", value_types));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "Vector";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "value_types", value_types, false);
            }
        };

        std::variant<Simple, Array, Vector> value;

        friend bool operator==(const HeapValueType&, const HeapValueType&);
        std::vector<uint8_t> bincodeSerialize() const;
        static HeapValueType bincodeDeserialize(std::vector<uint8_t>);

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
                    std::map<std::string, msgpack::object> data;
                    data[tag] = msgpack::object(arg);
                    packer.pack(data);
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
            std::vector<uint8_t> bincodeSerialize() const;
            static MemoryAddress bincodeDeserialize(std::vector<uint8_t>);

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
            std::vector<uint8_t> bincodeSerialize() const;
            static HeapArray bincodeDeserialize(std::vector<uint8_t>);

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
            std::vector<uint8_t> bincodeSerialize() const;
            static HeapVector bincodeDeserialize(std::vector<uint8_t>);

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
        std::vector<uint8_t> bincodeSerialize() const;
        static ValueOrArray bincodeDeserialize(std::vector<uint8_t>);

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
                    std::map<std::string, msgpack::object> data;
                    data[tag] = msgpack::object(arg);
                    packer.pack(data);
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
            std::vector<uint8_t> bincodeSerialize() const;
            static BinaryFieldOp bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(4);
                packer.pack(std::make_pair("destination", destination));
                packer.pack(std::make_pair("op", op));
                packer.pack(std::make_pair("lhs", lhs));
                packer.pack(std::make_pair("rhs", rhs));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "BinaryFieldOp";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "destination", destination, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "op", op, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "lhs", lhs, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "rhs", rhs, false);
            }
        };

        struct BinaryIntOp {
            Acir::MemoryAddress destination;
            Acir::BinaryIntOp op;
            Acir::IntegerBitSize bit_size;
            Acir::MemoryAddress lhs;
            Acir::MemoryAddress rhs;

            friend bool operator==(const BinaryIntOp&, const BinaryIntOp&);
            std::vector<uint8_t> bincodeSerialize() const;
            static BinaryIntOp bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(5);
                packer.pack(std::make_pair("destination", destination));
                packer.pack(std::make_pair("op", op));
                packer.pack(std::make_pair("bit_size", bit_size));
                packer.pack(std::make_pair("lhs", lhs));
                packer.pack(std::make_pair("rhs", rhs));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "BinaryIntOp";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "destination", destination, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "op", op, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "bit_size", bit_size, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "lhs", lhs, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "rhs", rhs, false);
            }
        };

        struct Not {
            Acir::MemoryAddress destination;
            Acir::MemoryAddress source;
            Acir::IntegerBitSize bit_size;

            friend bool operator==(const Not&, const Not&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Not bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(3);
                packer.pack(std::make_pair("destination", destination));
                packer.pack(std::make_pair("source", source));
                packer.pack(std::make_pair("bit_size", bit_size));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "Not";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "destination", destination, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "source", source, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "bit_size", bit_size, false);
            }
        };

        struct Cast {
            Acir::MemoryAddress destination;
            Acir::MemoryAddress source;
            Acir::BitSize bit_size;

            friend bool operator==(const Cast&, const Cast&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Cast bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(3);
                packer.pack(std::make_pair("destination", destination));
                packer.pack(std::make_pair("source", source));
                packer.pack(std::make_pair("bit_size", bit_size));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "Cast";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "destination", destination, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "source", source, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "bit_size", bit_size, false);
            }
        };

        struct JumpIf {
            Acir::MemoryAddress condition;
            uint64_t location;

            friend bool operator==(const JumpIf&, const JumpIf&);
            std::vector<uint8_t> bincodeSerialize() const;
            static JumpIf bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(2);
                packer.pack(std::make_pair("condition", condition));
                packer.pack(std::make_pair("location", location));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "JumpIf";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "condition", condition, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "location", location, false);
            }
        };

        struct Jump {
            uint64_t location;

            friend bool operator==(const Jump&, const Jump&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Jump bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(1);
                packer.pack(std::make_pair("location", location));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "Jump";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "location", location, false);
            }
        };

        struct CalldataCopy {
            Acir::MemoryAddress destination_address;
            Acir::MemoryAddress size_address;
            Acir::MemoryAddress offset_address;

            friend bool operator==(const CalldataCopy&, const CalldataCopy&);
            std::vector<uint8_t> bincodeSerialize() const;
            static CalldataCopy bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(3);
                packer.pack(std::make_pair("destination_address", destination_address));
                packer.pack(std::make_pair("size_address", size_address));
                packer.pack(std::make_pair("offset_address", offset_address));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "CalldataCopy";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "destination_address", destination_address, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "size_address", size_address, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "offset_address", offset_address, false);
            }
        };

        struct Call {
            uint64_t location;

            friend bool operator==(const Call&, const Call&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Call bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(1);
                packer.pack(std::make_pair("location", location));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "Call";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "location", location, false);
            }
        };

        struct Const {
            Acir::MemoryAddress destination;
            Acir::BitSize bit_size;
            std::vector<uint8_t> value;

            friend bool operator==(const Const&, const Const&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Const bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(3);
                packer.pack(std::make_pair("destination", destination));
                packer.pack(std::make_pair("bit_size", bit_size));
                packer.pack(std::make_pair("value", value));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "Const";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "destination", destination, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "bit_size", bit_size, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "value", value, false);
            }
        };

        struct IndirectConst {
            Acir::MemoryAddress destination_pointer;
            Acir::BitSize bit_size;
            std::vector<uint8_t> value;

            friend bool operator==(const IndirectConst&, const IndirectConst&);
            std::vector<uint8_t> bincodeSerialize() const;
            static IndirectConst bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(3);
                packer.pack(std::make_pair("destination_pointer", destination_pointer));
                packer.pack(std::make_pair("bit_size", bit_size));
                packer.pack(std::make_pair("value", value));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "IndirectConst";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "destination_pointer", destination_pointer, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "bit_size", bit_size, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "value", value, false);
            }
        };

        struct Return {
            friend bool operator==(const Return&, const Return&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Return bincodeDeserialize(std::vector<uint8_t>);

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
            std::vector<uint8_t> bincodeSerialize() const;
            static ForeignCall bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(5);
                packer.pack(std::make_pair("function", function));
                packer.pack(std::make_pair("destinations", destinations));
                packer.pack(std::make_pair("destination_value_types", destination_value_types));
                packer.pack(std::make_pair("inputs", inputs));
                packer.pack(std::make_pair("input_value_types", input_value_types));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "ForeignCall";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "function", function, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "destinations", destinations, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "destination_value_types", destination_value_types, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "inputs", inputs, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "input_value_types", input_value_types, false);
            }
        };

        struct Mov {
            Acir::MemoryAddress destination;
            Acir::MemoryAddress source;

            friend bool operator==(const Mov&, const Mov&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Mov bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(2);
                packer.pack(std::make_pair("destination", destination));
                packer.pack(std::make_pair("source", source));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "Mov";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "destination", destination, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "source", source, false);
            }
        };

        struct ConditionalMov {
            Acir::MemoryAddress destination;
            Acir::MemoryAddress source_a;
            Acir::MemoryAddress source_b;
            Acir::MemoryAddress condition;

            friend bool operator==(const ConditionalMov&, const ConditionalMov&);
            std::vector<uint8_t> bincodeSerialize() const;
            static ConditionalMov bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(4);
                packer.pack(std::make_pair("destination", destination));
                packer.pack(std::make_pair("source_a", source_a));
                packer.pack(std::make_pair("source_b", source_b));
                packer.pack(std::make_pair("condition", condition));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "ConditionalMov";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "destination", destination, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "source_a", source_a, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "source_b", source_b, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "condition", condition, false);
            }
        };

        struct Load {
            Acir::MemoryAddress destination;
            Acir::MemoryAddress source_pointer;

            friend bool operator==(const Load&, const Load&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Load bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(2);
                packer.pack(std::make_pair("destination", destination));
                packer.pack(std::make_pair("source_pointer", source_pointer));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "Load";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "destination", destination, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "source_pointer", source_pointer, false);
            }
        };

        struct Store {
            Acir::MemoryAddress destination_pointer;
            Acir::MemoryAddress source;

            friend bool operator==(const Store&, const Store&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Store bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(2);
                packer.pack(std::make_pair("destination_pointer", destination_pointer));
                packer.pack(std::make_pair("source", source));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "Store";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "destination_pointer", destination_pointer, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "source", source, false);
            }
        };

        struct BlackBox {
            Acir::BlackBoxOp value;

            friend bool operator==(const BlackBox&, const BlackBox&);
            std::vector<uint8_t> bincodeSerialize() const;
            static BlackBox bincodeDeserialize(std::vector<uint8_t>);

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
            std::vector<uint8_t> bincodeSerialize() const;
            static Trap bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(1);
                packer.pack(std::make_pair("revert_data", revert_data));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "Trap";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "revert_data", revert_data, false);
            }
        };

        struct Stop {
            Acir::HeapVector return_data;

            friend bool operator==(const Stop&, const Stop&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Stop bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(1);
                packer.pack(std::make_pair("return_data", return_data));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "Stop";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "return_data", return_data, false);
            }
        };

        std::variant<BinaryFieldOp, BinaryIntOp, Not, Cast, JumpIf, Jump, CalldataCopy, Call, Const, IndirectConst, Return, ForeignCall, Mov, ConditionalMov, Load, Store, BlackBox, Trap, Stop> value;

        friend bool operator==(const BrilligOpcode&, const BrilligOpcode&);
        std::vector<uint8_t> bincodeSerialize() const;
        static BrilligOpcode bincodeDeserialize(std::vector<uint8_t>);

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
                    std::map<std::string, msgpack::object> data;
                    data[tag] = msgpack::object(arg);
                    packer.pack(data);
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
        std::vector<uint8_t> bincodeSerialize() const;
        static Witness bincodeDeserialize(std::vector<uint8_t>);

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
            std::vector<uint8_t> bincodeSerialize() const;
            static Constant bincodeDeserialize(std::vector<uint8_t>);

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
            std::vector<uint8_t> bincodeSerialize() const;
            static Witness bincodeDeserialize(std::vector<uint8_t>);

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
        std::vector<uint8_t> bincodeSerialize() const;
        static FunctionInput bincodeDeserialize(std::vector<uint8_t>);

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
                    std::map<std::string, msgpack::object> data;
                    data[tag] = msgpack::object(arg);
                    packer.pack(data);
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
            std::vector<uint8_t> bincodeSerialize() const;
            static AES128Encrypt bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(4);
                packer.pack(std::make_pair("inputs", inputs));
                packer.pack(std::make_pair("iv", iv));
                packer.pack(std::make_pair("key", key));
                packer.pack(std::make_pair("outputs", outputs));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "AES128Encrypt";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "inputs", inputs, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "iv", iv, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "key", key, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "outputs", outputs, false);
            }
        };

        struct AND {
            Acir::FunctionInput lhs;
            Acir::FunctionInput rhs;
            uint32_t num_bits;
            Acir::Witness output;

            friend bool operator==(const AND&, const AND&);
            std::vector<uint8_t> bincodeSerialize() const;
            static AND bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(4);
                packer.pack(std::make_pair("lhs", lhs));
                packer.pack(std::make_pair("rhs", rhs));
                packer.pack(std::make_pair("num_bits", num_bits));
                packer.pack(std::make_pair("output", output));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "AND";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "lhs", lhs, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "rhs", rhs, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "num_bits", num_bits, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "output", output, false);
            }
        };

        struct XOR {
            Acir::FunctionInput lhs;
            Acir::FunctionInput rhs;
            uint32_t num_bits;
            Acir::Witness output;

            friend bool operator==(const XOR&, const XOR&);
            std::vector<uint8_t> bincodeSerialize() const;
            static XOR bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(4);
                packer.pack(std::make_pair("lhs", lhs));
                packer.pack(std::make_pair("rhs", rhs));
                packer.pack(std::make_pair("num_bits", num_bits));
                packer.pack(std::make_pair("output", output));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "XOR";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "lhs", lhs, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "rhs", rhs, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "num_bits", num_bits, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "output", output, false);
            }
        };

        struct RANGE {
            Acir::FunctionInput input;
            uint32_t num_bits;

            friend bool operator==(const RANGE&, const RANGE&);
            std::vector<uint8_t> bincodeSerialize() const;
            static RANGE bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(2);
                packer.pack(std::make_pair("input", input));
                packer.pack(std::make_pair("num_bits", num_bits));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "RANGE";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "input", input, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "num_bits", num_bits, false);
            }
        };

        struct Blake2s {
            std::vector<Acir::FunctionInput> inputs;
            std::shared_ptr<std::array<Acir::Witness, 32>> outputs;

            friend bool operator==(const Blake2s&, const Blake2s&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Blake2s bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(2);
                packer.pack(std::make_pair("inputs", inputs));
                packer.pack(std::make_pair("outputs", outputs));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "Blake2s";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "inputs", inputs, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "outputs", outputs, false);
            }
        };

        struct Blake3 {
            std::vector<Acir::FunctionInput> inputs;
            std::shared_ptr<std::array<Acir::Witness, 32>> outputs;

            friend bool operator==(const Blake3&, const Blake3&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Blake3 bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(2);
                packer.pack(std::make_pair("inputs", inputs));
                packer.pack(std::make_pair("outputs", outputs));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "Blake3";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "inputs", inputs, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "outputs", outputs, false);
            }
        };

        struct EcdsaSecp256k1 {
            std::shared_ptr<std::array<Acir::FunctionInput, 32>> public_key_x;
            std::shared_ptr<std::array<Acir::FunctionInput, 32>> public_key_y;
            std::shared_ptr<std::array<Acir::FunctionInput, 64>> signature;
            std::shared_ptr<std::array<Acir::FunctionInput, 32>> hashed_message;
            Acir::Witness output;

            friend bool operator==(const EcdsaSecp256k1&, const EcdsaSecp256k1&);
            std::vector<uint8_t> bincodeSerialize() const;
            static EcdsaSecp256k1 bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(5);
                packer.pack(std::make_pair("public_key_x", public_key_x));
                packer.pack(std::make_pair("public_key_y", public_key_y));
                packer.pack(std::make_pair("signature", signature));
                packer.pack(std::make_pair("hashed_message", hashed_message));
                packer.pack(std::make_pair("output", output));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "EcdsaSecp256k1";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "public_key_x", public_key_x, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "public_key_y", public_key_y, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "signature", signature, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "hashed_message", hashed_message, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "output", output, false);
            }
        };

        struct EcdsaSecp256r1 {
            std::shared_ptr<std::array<Acir::FunctionInput, 32>> public_key_x;
            std::shared_ptr<std::array<Acir::FunctionInput, 32>> public_key_y;
            std::shared_ptr<std::array<Acir::FunctionInput, 64>> signature;
            std::shared_ptr<std::array<Acir::FunctionInput, 32>> hashed_message;
            Acir::Witness output;

            friend bool operator==(const EcdsaSecp256r1&, const EcdsaSecp256r1&);
            std::vector<uint8_t> bincodeSerialize() const;
            static EcdsaSecp256r1 bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(5);
                packer.pack(std::make_pair("public_key_x", public_key_x));
                packer.pack(std::make_pair("public_key_y", public_key_y));
                packer.pack(std::make_pair("signature", signature));
                packer.pack(std::make_pair("hashed_message", hashed_message));
                packer.pack(std::make_pair("output", output));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "EcdsaSecp256r1";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "public_key_x", public_key_x, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "public_key_y", public_key_y, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "signature", signature, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "hashed_message", hashed_message, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "output", output, false);
            }
        };

        struct MultiScalarMul {
            std::vector<Acir::FunctionInput> points;
            std::vector<Acir::FunctionInput> scalars;
            std::shared_ptr<std::array<Acir::Witness, 3>> outputs;

            friend bool operator==(const MultiScalarMul&, const MultiScalarMul&);
            std::vector<uint8_t> bincodeSerialize() const;
            static MultiScalarMul bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(3);
                packer.pack(std::make_pair("points", points));
                packer.pack(std::make_pair("scalars", scalars));
                packer.pack(std::make_pair("outputs", outputs));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "MultiScalarMul";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "points", points, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "scalars", scalars, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "outputs", outputs, false);
            }
        };

        struct EmbeddedCurveAdd {
            std::shared_ptr<std::array<Acir::FunctionInput, 3>> input1;
            std::shared_ptr<std::array<Acir::FunctionInput, 3>> input2;
            std::shared_ptr<std::array<Acir::Witness, 3>> outputs;

            friend bool operator==(const EmbeddedCurveAdd&, const EmbeddedCurveAdd&);
            std::vector<uint8_t> bincodeSerialize() const;
            static EmbeddedCurveAdd bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(3);
                packer.pack(std::make_pair("input1", input1));
                packer.pack(std::make_pair("input2", input2));
                packer.pack(std::make_pair("outputs", outputs));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "EmbeddedCurveAdd";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "input1", input1, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "input2", input2, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "outputs", outputs, false);
            }
        };

        struct Keccakf1600 {
            std::shared_ptr<std::array<Acir::FunctionInput, 25>> inputs;
            std::shared_ptr<std::array<Acir::Witness, 25>> outputs;

            friend bool operator==(const Keccakf1600&, const Keccakf1600&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Keccakf1600 bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(2);
                packer.pack(std::make_pair("inputs", inputs));
                packer.pack(std::make_pair("outputs", outputs));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "Keccakf1600";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "inputs", inputs, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "outputs", outputs, false);
            }
        };

        struct RecursiveAggregation {
            std::vector<Acir::FunctionInput> verification_key;
            std::vector<Acir::FunctionInput> proof;
            std::vector<Acir::FunctionInput> public_inputs;
            Acir::FunctionInput key_hash;
            uint32_t proof_type;

            friend bool operator==(const RecursiveAggregation&, const RecursiveAggregation&);
            std::vector<uint8_t> bincodeSerialize() const;
            static RecursiveAggregation bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(5);
                packer.pack(std::make_pair("verification_key", verification_key));
                packer.pack(std::make_pair("proof", proof));
                packer.pack(std::make_pair("public_inputs", public_inputs));
                packer.pack(std::make_pair("key_hash", key_hash));
                packer.pack(std::make_pair("proof_type", proof_type));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "RecursiveAggregation";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "verification_key", verification_key, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "proof", proof, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "public_inputs", public_inputs, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "key_hash", key_hash, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "proof_type", proof_type, false);
            }
        };

        struct BigIntAdd {
            uint32_t lhs;
            uint32_t rhs;
            uint32_t output;

            friend bool operator==(const BigIntAdd&, const BigIntAdd&);
            std::vector<uint8_t> bincodeSerialize() const;
            static BigIntAdd bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(3);
                packer.pack(std::make_pair("lhs", lhs));
                packer.pack(std::make_pair("rhs", rhs));
                packer.pack(std::make_pair("output", output));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "BigIntAdd";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "lhs", lhs, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "rhs", rhs, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "output", output, false);
            }
        };

        struct BigIntSub {
            uint32_t lhs;
            uint32_t rhs;
            uint32_t output;

            friend bool operator==(const BigIntSub&, const BigIntSub&);
            std::vector<uint8_t> bincodeSerialize() const;
            static BigIntSub bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(3);
                packer.pack(std::make_pair("lhs", lhs));
                packer.pack(std::make_pair("rhs", rhs));
                packer.pack(std::make_pair("output", output));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "BigIntSub";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "lhs", lhs, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "rhs", rhs, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "output", output, false);
            }
        };

        struct BigIntMul {
            uint32_t lhs;
            uint32_t rhs;
            uint32_t output;

            friend bool operator==(const BigIntMul&, const BigIntMul&);
            std::vector<uint8_t> bincodeSerialize() const;
            static BigIntMul bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(3);
                packer.pack(std::make_pair("lhs", lhs));
                packer.pack(std::make_pair("rhs", rhs));
                packer.pack(std::make_pair("output", output));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "BigIntMul";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "lhs", lhs, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "rhs", rhs, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "output", output, false);
            }
        };

        struct BigIntDiv {
            uint32_t lhs;
            uint32_t rhs;
            uint32_t output;

            friend bool operator==(const BigIntDiv&, const BigIntDiv&);
            std::vector<uint8_t> bincodeSerialize() const;
            static BigIntDiv bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(3);
                packer.pack(std::make_pair("lhs", lhs));
                packer.pack(std::make_pair("rhs", rhs));
                packer.pack(std::make_pair("output", output));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "BigIntDiv";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "lhs", lhs, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "rhs", rhs, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "output", output, false);
            }
        };

        struct BigIntFromLeBytes {
            std::vector<Acir::FunctionInput> inputs;
            std::vector<uint8_t> modulus;
            uint32_t output;

            friend bool operator==(const BigIntFromLeBytes&, const BigIntFromLeBytes&);
            std::vector<uint8_t> bincodeSerialize() const;
            static BigIntFromLeBytes bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(3);
                packer.pack(std::make_pair("inputs", inputs));
                packer.pack(std::make_pair("modulus", modulus));
                packer.pack(std::make_pair("output", output));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "BigIntFromLeBytes";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "inputs", inputs, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "modulus", modulus, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "output", output, false);
            }
        };

        struct BigIntToLeBytes {
            uint32_t input;
            std::vector<Acir::Witness> outputs;

            friend bool operator==(const BigIntToLeBytes&, const BigIntToLeBytes&);
            std::vector<uint8_t> bincodeSerialize() const;
            static BigIntToLeBytes bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(2);
                packer.pack(std::make_pair("input", input));
                packer.pack(std::make_pair("outputs", outputs));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "BigIntToLeBytes";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "input", input, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "outputs", outputs, false);
            }
        };

        struct Poseidon2Permutation {
            std::vector<Acir::FunctionInput> inputs;
            std::vector<Acir::Witness> outputs;
            uint32_t len;

            friend bool operator==(const Poseidon2Permutation&, const Poseidon2Permutation&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Poseidon2Permutation bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(3);
                packer.pack(std::make_pair("inputs", inputs));
                packer.pack(std::make_pair("outputs", outputs));
                packer.pack(std::make_pair("len", len));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "Poseidon2Permutation";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "inputs", inputs, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "outputs", outputs, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "len", len, false);
            }
        };

        struct Sha256Compression {
            std::shared_ptr<std::array<Acir::FunctionInput, 16>> inputs;
            std::shared_ptr<std::array<Acir::FunctionInput, 8>> hash_values;
            std::shared_ptr<std::array<Acir::Witness, 8>> outputs;

            friend bool operator==(const Sha256Compression&, const Sha256Compression&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Sha256Compression bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(3);
                packer.pack(std::make_pair("inputs", inputs));
                packer.pack(std::make_pair("hash_values", hash_values));
                packer.pack(std::make_pair("outputs", outputs));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "Sha256Compression";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "inputs", inputs, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "hash_values", hash_values, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "outputs", outputs, false);
            }
        };

        std::variant<AES128Encrypt, AND, XOR, RANGE, Blake2s, Blake3, EcdsaSecp256k1, EcdsaSecp256r1, MultiScalarMul, EmbeddedCurveAdd, Keccakf1600, RecursiveAggregation, BigIntAdd, BigIntSub, BigIntMul, BigIntDiv, BigIntFromLeBytes, BigIntToLeBytes, Poseidon2Permutation, Sha256Compression> value;

        friend bool operator==(const BlackBoxFuncCall&, const BlackBoxFuncCall&);
        std::vector<uint8_t> bincodeSerialize() const;
        static BlackBoxFuncCall bincodeDeserialize(std::vector<uint8_t>);

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
                    tag = "BigIntAdd";
                    is_unit = false;
                    break;
                case 13:
                    tag = "BigIntSub";
                    is_unit = false;
                    break;
                case 14:
                    tag = "BigIntMul";
                    is_unit = false;
                    break;
                case 15:
                    tag = "BigIntDiv";
                    is_unit = false;
                    break;
                case 16:
                    tag = "BigIntFromLeBytes";
                    is_unit = false;
                    break;
                case 17:
                    tag = "BigIntToLeBytes";
                    is_unit = false;
                    break;
                case 18:
                    tag = "Poseidon2Permutation";
                    is_unit = false;
                    break;
                case 19:
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
                    std::map<std::string, msgpack::object> data;
                    data[tag] = msgpack::object(arg);
                    packer.pack(data);
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
            else if (tag == "BigIntAdd") {
                BigIntAdd v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BlackBoxFuncCall::BigIntAdd'");
                }
                
                value = v;
            }
            else if (tag == "BigIntSub") {
                BigIntSub v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BlackBoxFuncCall::BigIntSub'");
                }
                
                value = v;
            }
            else if (tag == "BigIntMul") {
                BigIntMul v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BlackBoxFuncCall::BigIntMul'");
                }
                
                value = v;
            }
            else if (tag == "BigIntDiv") {
                BigIntDiv v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BlackBoxFuncCall::BigIntDiv'");
                }
                
                value = v;
            }
            else if (tag == "BigIntFromLeBytes") {
                BigIntFromLeBytes v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BlackBoxFuncCall::BigIntFromLeBytes'");
                }
                
                value = v;
            }
            else if (tag == "BigIntToLeBytes") {
                BigIntToLeBytes v;
                try {
                    o.via.map.ptr[0].val.convert(v);
                } catch (const msgpack::type_error&) {
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant 'BlackBoxFuncCall::BigIntToLeBytes'");
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
        std::vector<uint8_t> bincodeSerialize() const;
        static BlockId bincodeDeserialize(std::vector<uint8_t>);

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
            std::vector<uint8_t> bincodeSerialize() const;
            static Memory bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct CallData {
            uint32_t value;

            friend bool operator==(const CallData&, const CallData&);
            std::vector<uint8_t> bincodeSerialize() const;
            static CallData bincodeDeserialize(std::vector<uint8_t>);

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
            std::vector<uint8_t> bincodeSerialize() const;
            static ReturnData bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        std::variant<Memory, CallData, ReturnData> value;

        friend bool operator==(const BlockType&, const BlockType&);
        std::vector<uint8_t> bincodeSerialize() const;
        static BlockType bincodeDeserialize(std::vector<uint8_t>);

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
                    std::map<std::string, msgpack::object> data;
                    data[tag] = msgpack::object(arg);
                    packer.pack(data);
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
        std::vector<uint8_t> bincodeSerialize() const;
        static Expression bincodeDeserialize(std::vector<uint8_t>);

        void msgpack_pack(auto& packer) const {
            packer.pack_map(3);
            packer.pack(std::make_pair("mul_terms", mul_terms));
            packer.pack(std::make_pair("linear_combinations", linear_combinations));
            packer.pack(std::make_pair("q_c", q_c));
        }

        void msgpack_unpack(msgpack::object const& o) {
            auto name = "Expression";
            auto kvmap = Helpers::make_kvmap(o, name);
            Helpers::conv_fld_from_kvmap(kvmap, name, "mul_terms", mul_terms, false);
            Helpers::conv_fld_from_kvmap(kvmap, name, "linear_combinations", linear_combinations, false);
            Helpers::conv_fld_from_kvmap(kvmap, name, "q_c", q_c, false);
        }
    };

    struct BrilligInputs {

        struct Single {
            Acir::Expression value;

            friend bool operator==(const Single&, const Single&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Single bincodeDeserialize(std::vector<uint8_t>);

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
            std::vector<uint8_t> bincodeSerialize() const;
            static Array bincodeDeserialize(std::vector<uint8_t>);

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
            std::vector<uint8_t> bincodeSerialize() const;
            static MemoryArray bincodeDeserialize(std::vector<uint8_t>);

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
        std::vector<uint8_t> bincodeSerialize() const;
        static BrilligInputs bincodeDeserialize(std::vector<uint8_t>);

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
                    std::map<std::string, msgpack::object> data;
                    data[tag] = msgpack::object(arg);
                    packer.pack(data);
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
            std::vector<uint8_t> bincodeSerialize() const;
            static Simple bincodeDeserialize(std::vector<uint8_t>);

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
            std::vector<uint8_t> bincodeSerialize() const;
            static Array bincodeDeserialize(std::vector<uint8_t>);

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
        std::vector<uint8_t> bincodeSerialize() const;
        static BrilligOutputs bincodeDeserialize(std::vector<uint8_t>);

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
                    std::map<std::string, msgpack::object> data;
                    data[tag] = msgpack::object(arg);
                    packer.pack(data);
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
        std::vector<uint8_t> bincodeSerialize() const;
        static MemOp bincodeDeserialize(std::vector<uint8_t>);

        void msgpack_pack(auto& packer) const {
            packer.pack_map(3);
            packer.pack(std::make_pair("operation", operation));
            packer.pack(std::make_pair("index", index));
            packer.pack(std::make_pair("value", value));
        }

        void msgpack_unpack(msgpack::object const& o) {
            auto name = "MemOp";
            auto kvmap = Helpers::make_kvmap(o, name);
            Helpers::conv_fld_from_kvmap(kvmap, name, "operation", operation, false);
            Helpers::conv_fld_from_kvmap(kvmap, name, "index", index, false);
            Helpers::conv_fld_from_kvmap(kvmap, name, "value", value, false);
        }
    };

    struct Opcode {

        struct AssertZero {
            Acir::Expression value;

            friend bool operator==(const AssertZero&, const AssertZero&);
            std::vector<uint8_t> bincodeSerialize() const;
            static AssertZero bincodeDeserialize(std::vector<uint8_t>);

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
            std::vector<uint8_t> bincodeSerialize() const;
            static BlackBoxFuncCall bincodeDeserialize(std::vector<uint8_t>);

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
            std::vector<uint8_t> bincodeSerialize() const;
            static MemoryOp bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(2);
                packer.pack(std::make_pair("block_id", block_id));
                packer.pack(std::make_pair("op", op));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "MemoryOp";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "block_id", block_id, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "op", op, false);
            }
        };

        struct MemoryInit {
            Acir::BlockId block_id;
            std::vector<Acir::Witness> init;
            Acir::BlockType block_type;

            friend bool operator==(const MemoryInit&, const MemoryInit&);
            std::vector<uint8_t> bincodeSerialize() const;
            static MemoryInit bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(3);
                packer.pack(std::make_pair("block_id", block_id));
                packer.pack(std::make_pair("init", init));
                packer.pack(std::make_pair("block_type", block_type));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "MemoryInit";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "block_id", block_id, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "init", init, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "block_type", block_type, false);
            }
        };

        struct BrilligCall {
            uint32_t id;
            std::vector<Acir::BrilligInputs> inputs;
            std::vector<Acir::BrilligOutputs> outputs;
            std::optional<Acir::Expression> predicate;

            friend bool operator==(const BrilligCall&, const BrilligCall&);
            std::vector<uint8_t> bincodeSerialize() const;
            static BrilligCall bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(4);
                packer.pack(std::make_pair("id", id));
                packer.pack(std::make_pair("inputs", inputs));
                packer.pack(std::make_pair("outputs", outputs));
                packer.pack(std::make_pair("predicate", predicate));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "BrilligCall";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "id", id, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "inputs", inputs, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "outputs", outputs, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "predicate", predicate, true);
            }
        };

        struct Call {
            uint32_t id;
            std::vector<Acir::Witness> inputs;
            std::vector<Acir::Witness> outputs;
            std::optional<Acir::Expression> predicate;

            friend bool operator==(const Call&, const Call&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Call bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(4);
                packer.pack(std::make_pair("id", id));
                packer.pack(std::make_pair("inputs", inputs));
                packer.pack(std::make_pair("outputs", outputs));
                packer.pack(std::make_pair("predicate", predicate));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "Call";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "id", id, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "inputs", inputs, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "outputs", outputs, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "predicate", predicate, true);
            }
        };

        std::variant<AssertZero, BlackBoxFuncCall, MemoryOp, MemoryInit, BrilligCall, Call> value;

        friend bool operator==(const Opcode&, const Opcode&);
        std::vector<uint8_t> bincodeSerialize() const;
        static Opcode bincodeDeserialize(std::vector<uint8_t>);

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
                    std::map<std::string, msgpack::object> data;
                    data[tag] = msgpack::object(arg);
                    packer.pack(data);
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
            std::vector<uint8_t> bincodeSerialize() const;
            static Expression bincodeDeserialize(std::vector<uint8_t>);

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
            std::vector<uint8_t> bincodeSerialize() const;
            static Memory bincodeDeserialize(std::vector<uint8_t>);

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
        std::vector<uint8_t> bincodeSerialize() const;
        static ExpressionOrMemory bincodeDeserialize(std::vector<uint8_t>);

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
                    std::map<std::string, msgpack::object> data;
                    data[tag] = msgpack::object(arg);
                    packer.pack(data);
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
        std::vector<uint8_t> bincodeSerialize() const;
        static AssertionPayload bincodeDeserialize(std::vector<uint8_t>);

        void msgpack_pack(auto& packer) const {
            packer.pack_map(2);
            packer.pack(std::make_pair("error_selector", error_selector));
            packer.pack(std::make_pair("payload", payload));
        }

        void msgpack_unpack(msgpack::object const& o) {
            auto name = "AssertionPayload";
            auto kvmap = Helpers::make_kvmap(o, name);
            Helpers::conv_fld_from_kvmap(kvmap, name, "error_selector", error_selector, false);
            Helpers::conv_fld_from_kvmap(kvmap, name, "payload", payload, false);
        }
    };

    struct ExpressionWidth {

        struct Unbounded {
            friend bool operator==(const Unbounded&, const Unbounded&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Unbounded bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {}
            void msgpack_unpack(msgpack::object const& o) {}
        };

        struct Bounded {
            uint64_t width;

            friend bool operator==(const Bounded&, const Bounded&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Bounded bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(1);
                packer.pack(std::make_pair("width", width));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "Bounded";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "width", width, false);
            }
        };

        std::variant<Unbounded, Bounded> value;

        friend bool operator==(const ExpressionWidth&, const ExpressionWidth&);
        std::vector<uint8_t> bincodeSerialize() const;
        static ExpressionWidth bincodeDeserialize(std::vector<uint8_t>);

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
                    std::map<std::string, msgpack::object> data;
                    data[tag] = msgpack::object(arg);
                    packer.pack(data);
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

    struct OpcodeLocation {

        struct Acir {
            uint64_t value;

            friend bool operator==(const Acir&, const Acir&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Acir bincodeDeserialize(std::vector<uint8_t>);

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
            std::vector<uint8_t> bincodeSerialize() const;
            static Brillig bincodeDeserialize(std::vector<uint8_t>);

            void msgpack_pack(auto& packer) const {
                packer.pack_map(2);
                packer.pack(std::make_pair("acir_index", acir_index));
                packer.pack(std::make_pair("brillig_index", brillig_index));
            }

            void msgpack_unpack(msgpack::object const& o) {
                auto name = "Brillig";
                auto kvmap = Helpers::make_kvmap(o, name);
                Helpers::conv_fld_from_kvmap(kvmap, name, "acir_index", acir_index, false);
                Helpers::conv_fld_from_kvmap(kvmap, name, "brillig_index", brillig_index, false);
            }
        };

        std::variant<Acir, Brillig> value;

        friend bool operator==(const OpcodeLocation&, const OpcodeLocation&);
        std::vector<uint8_t> bincodeSerialize() const;
        static OpcodeLocation bincodeDeserialize(std::vector<uint8_t>);

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
                    std::map<std::string, msgpack::object> data;
                    data[tag] = msgpack::object(arg);
                    packer.pack(data);
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
        std::vector<uint8_t> bincodeSerialize() const;
        static PublicInputs bincodeDeserialize(std::vector<uint8_t>);

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
        uint32_t current_witness_index;
        std::vector<Acir::Opcode> opcodes;
        Acir::ExpressionWidth expression_width;
        std::vector<Acir::Witness> private_parameters;
        Acir::PublicInputs public_parameters;
        Acir::PublicInputs return_values;
        std::vector<std::tuple<Acir::OpcodeLocation, Acir::AssertionPayload>> assert_messages;

        friend bool operator==(const Circuit&, const Circuit&);
        std::vector<uint8_t> bincodeSerialize() const;
        static Circuit bincodeDeserialize(std::vector<uint8_t>);

        void msgpack_pack(auto& packer) const {
            packer.pack_map(7);
            packer.pack(std::make_pair("current_witness_index", current_witness_index));
            packer.pack(std::make_pair("opcodes", opcodes));
            packer.pack(std::make_pair("expression_width", expression_width));
            packer.pack(std::make_pair("private_parameters", private_parameters));
            packer.pack(std::make_pair("public_parameters", public_parameters));
            packer.pack(std::make_pair("return_values", return_values));
            packer.pack(std::make_pair("assert_messages", assert_messages));
        }

        void msgpack_unpack(msgpack::object const& o) {
            auto name = "Circuit";
            auto kvmap = Helpers::make_kvmap(o, name);
            Helpers::conv_fld_from_kvmap(kvmap, name, "current_witness_index", current_witness_index, false);
            Helpers::conv_fld_from_kvmap(kvmap, name, "opcodes", opcodes, false);
            Helpers::conv_fld_from_kvmap(kvmap, name, "expression_width", expression_width, false);
            Helpers::conv_fld_from_kvmap(kvmap, name, "private_parameters", private_parameters, false);
            Helpers::conv_fld_from_kvmap(kvmap, name, "public_parameters", public_parameters, false);
            Helpers::conv_fld_from_kvmap(kvmap, name, "return_values", return_values, false);
            Helpers::conv_fld_from_kvmap(kvmap, name, "assert_messages", assert_messages, false);
        }
    };

    struct BrilligBytecode {
        std::vector<Acir::BrilligOpcode> bytecode;

        friend bool operator==(const BrilligBytecode&, const BrilligBytecode&);
        std::vector<uint8_t> bincodeSerialize() const;
        static BrilligBytecode bincodeDeserialize(std::vector<uint8_t>);

        void msgpack_pack(auto& packer) const {
            packer.pack_map(1);
            packer.pack(std::make_pair("bytecode", bytecode));
        }

        void msgpack_unpack(msgpack::object const& o) {
            auto name = "BrilligBytecode";
            auto kvmap = Helpers::make_kvmap(o, name);
            Helpers::conv_fld_from_kvmap(kvmap, name, "bytecode", bytecode, false);
        }
    };

    struct Program {
        std::vector<Acir::Circuit> functions;
        std::vector<Acir::BrilligBytecode> unconstrained_functions;

        friend bool operator==(const Program&, const Program&);
        std::vector<uint8_t> bincodeSerialize() const;
        static Program bincodeDeserialize(std::vector<uint8_t>);

        void msgpack_pack(auto& packer) const {
            packer.pack_map(2);
            packer.pack(std::make_pair("functions", functions));
            packer.pack(std::make_pair("unconstrained_functions", unconstrained_functions));
        }

        void msgpack_unpack(msgpack::object const& o) {
            auto name = "Program";
            auto kvmap = Helpers::make_kvmap(o, name);
            Helpers::conv_fld_from_kvmap(kvmap, name, "functions", functions, false);
            Helpers::conv_fld_from_kvmap(kvmap, name, "unconstrained_functions", unconstrained_functions, false);
        }
    };

    struct ProgramWithoutBrillig {
        std::vector<Acir::Circuit> functions;

        friend bool operator==(const ProgramWithoutBrillig&, const ProgramWithoutBrillig&);
        std::vector<uint8_t> bincodeSerialize() const;
        static ProgramWithoutBrillig bincodeDeserialize(std::vector<uint8_t>);

        void msgpack_pack(auto& packer) const {
            packer.pack_map(1);
            packer.pack(std::make_pair("functions", functions));
        }

        void msgpack_unpack(msgpack::object const& o) {
            auto name = "ProgramWithoutBrillig";
            auto kvmap = Helpers::make_kvmap(o, name);
            Helpers::conv_fld_from_kvmap(kvmap, name, "functions", functions, false);
        }
    };

} // end of namespace Acir


namespace Acir {

    inline bool operator==(const AssertionPayload &lhs, const AssertionPayload &rhs) {
        if (!(lhs.error_selector == rhs.error_selector)) { return false; }
        if (!(lhs.payload == rhs.payload)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> AssertionPayload::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<AssertionPayload>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline AssertionPayload AssertionPayload::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<AssertionPayload>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BinaryFieldOp::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryFieldOp>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryFieldOp BinaryFieldOp::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryFieldOp>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BinaryFieldOp::Add::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryFieldOp::Add>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryFieldOp::Add BinaryFieldOp::Add::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryFieldOp::Add>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BinaryFieldOp::Sub::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryFieldOp::Sub>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryFieldOp::Sub BinaryFieldOp::Sub::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryFieldOp::Sub>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BinaryFieldOp::Mul::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryFieldOp::Mul>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryFieldOp::Mul BinaryFieldOp::Mul::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryFieldOp::Mul>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BinaryFieldOp::Div::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryFieldOp::Div>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryFieldOp::Div BinaryFieldOp::Div::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryFieldOp::Div>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BinaryFieldOp::IntegerDiv::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryFieldOp::IntegerDiv>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryFieldOp::IntegerDiv BinaryFieldOp::IntegerDiv::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryFieldOp::IntegerDiv>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BinaryFieldOp::Equals::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryFieldOp::Equals>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryFieldOp::Equals BinaryFieldOp::Equals::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryFieldOp::Equals>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BinaryFieldOp::LessThan::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryFieldOp::LessThan>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryFieldOp::LessThan BinaryFieldOp::LessThan::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryFieldOp::LessThan>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BinaryFieldOp::LessThanEquals::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryFieldOp::LessThanEquals>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryFieldOp::LessThanEquals BinaryFieldOp::LessThanEquals::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryFieldOp::LessThanEquals>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BinaryIntOp::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryIntOp>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryIntOp BinaryIntOp::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryIntOp>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BinaryIntOp::Add::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryIntOp::Add>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryIntOp::Add BinaryIntOp::Add::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryIntOp::Add>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BinaryIntOp::Sub::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryIntOp::Sub>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryIntOp::Sub BinaryIntOp::Sub::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryIntOp::Sub>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BinaryIntOp::Mul::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryIntOp::Mul>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryIntOp::Mul BinaryIntOp::Mul::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryIntOp::Mul>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BinaryIntOp::Div::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryIntOp::Div>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryIntOp::Div BinaryIntOp::Div::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryIntOp::Div>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BinaryIntOp::Equals::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryIntOp::Equals>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryIntOp::Equals BinaryIntOp::Equals::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryIntOp::Equals>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BinaryIntOp::LessThan::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryIntOp::LessThan>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryIntOp::LessThan BinaryIntOp::LessThan::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryIntOp::LessThan>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BinaryIntOp::LessThanEquals::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryIntOp::LessThanEquals>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryIntOp::LessThanEquals BinaryIntOp::LessThanEquals::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryIntOp::LessThanEquals>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BinaryIntOp::And::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryIntOp::And>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryIntOp::And BinaryIntOp::And::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryIntOp::And>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BinaryIntOp::Or::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryIntOp::Or>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryIntOp::Or BinaryIntOp::Or::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryIntOp::Or>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BinaryIntOp::Xor::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryIntOp::Xor>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryIntOp::Xor BinaryIntOp::Xor::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryIntOp::Xor>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BinaryIntOp::Shl::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryIntOp::Shl>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryIntOp::Shl BinaryIntOp::Shl::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryIntOp::Shl>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BinaryIntOp::Shr::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryIntOp::Shr>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryIntOp::Shr BinaryIntOp::Shr::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryIntOp::Shr>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BitSize::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BitSize>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BitSize BitSize::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BitSize>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BitSize::Field::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BitSize::Field>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BitSize::Field BitSize::Field::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BitSize::Field>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BitSize::Integer::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BitSize::Integer>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BitSize::Integer BitSize::Integer::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BitSize::Integer>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BlackBoxFuncCall::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall BlackBoxFuncCall::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BlackBoxFuncCall::AES128Encrypt::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::AES128Encrypt>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::AES128Encrypt BlackBoxFuncCall::AES128Encrypt::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::AES128Encrypt>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BlackBoxFuncCall::AND::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::AND>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::AND BlackBoxFuncCall::AND::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::AND>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BlackBoxFuncCall::XOR::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::XOR>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::XOR BlackBoxFuncCall::XOR::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::XOR>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BlackBoxFuncCall::RANGE::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::RANGE>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::RANGE BlackBoxFuncCall::RANGE::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::RANGE>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BlackBoxFuncCall::Blake2s::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::Blake2s>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::Blake2s BlackBoxFuncCall::Blake2s::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::Blake2s>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BlackBoxFuncCall::Blake3::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::Blake3>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::Blake3 BlackBoxFuncCall::Blake3::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::Blake3>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxFuncCall::EcdsaSecp256k1::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::EcdsaSecp256k1>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::EcdsaSecp256k1 BlackBoxFuncCall::EcdsaSecp256k1::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::EcdsaSecp256k1>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxFuncCall::EcdsaSecp256k1>::serialize(const Acir::BlackBoxFuncCall::EcdsaSecp256k1 &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.public_key_x)>::serialize(obj.public_key_x, serializer);
    serde::Serializable<decltype(obj.public_key_y)>::serialize(obj.public_key_y, serializer);
    serde::Serializable<decltype(obj.signature)>::serialize(obj.signature, serializer);
    serde::Serializable<decltype(obj.hashed_message)>::serialize(obj.hashed_message, serializer);
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
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxFuncCall::EcdsaSecp256r1 &lhs, const BlackBoxFuncCall::EcdsaSecp256r1 &rhs) {
        if (!(lhs.public_key_x == rhs.public_key_x)) { return false; }
        if (!(lhs.public_key_y == rhs.public_key_y)) { return false; }
        if (!(lhs.signature == rhs.signature)) { return false; }
        if (!(lhs.hashed_message == rhs.hashed_message)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxFuncCall::EcdsaSecp256r1::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::EcdsaSecp256r1>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::EcdsaSecp256r1 BlackBoxFuncCall::EcdsaSecp256r1::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::EcdsaSecp256r1>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxFuncCall::EcdsaSecp256r1>::serialize(const Acir::BlackBoxFuncCall::EcdsaSecp256r1 &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.public_key_x)>::serialize(obj.public_key_x, serializer);
    serde::Serializable<decltype(obj.public_key_y)>::serialize(obj.public_key_y, serializer);
    serde::Serializable<decltype(obj.signature)>::serialize(obj.signature, serializer);
    serde::Serializable<decltype(obj.hashed_message)>::serialize(obj.hashed_message, serializer);
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
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxFuncCall::MultiScalarMul &lhs, const BlackBoxFuncCall::MultiScalarMul &rhs) {
        if (!(lhs.points == rhs.points)) { return false; }
        if (!(lhs.scalars == rhs.scalars)) { return false; }
        if (!(lhs.outputs == rhs.outputs)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxFuncCall::MultiScalarMul::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::MultiScalarMul>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::MultiScalarMul BlackBoxFuncCall::MultiScalarMul::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::MultiScalarMul>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxFuncCall::MultiScalarMul>::serialize(const Acir::BlackBoxFuncCall::MultiScalarMul &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.points)>::serialize(obj.points, serializer);
    serde::Serializable<decltype(obj.scalars)>::serialize(obj.scalars, serializer);
    serde::Serializable<decltype(obj.outputs)>::serialize(obj.outputs, serializer);
}

template <>
template <typename Deserializer>
Acir::BlackBoxFuncCall::MultiScalarMul serde::Deserializable<Acir::BlackBoxFuncCall::MultiScalarMul>::deserialize(Deserializer &deserializer) {
    Acir::BlackBoxFuncCall::MultiScalarMul obj;
    obj.points = serde::Deserializable<decltype(obj.points)>::deserialize(deserializer);
    obj.scalars = serde::Deserializable<decltype(obj.scalars)>::deserialize(deserializer);
    obj.outputs = serde::Deserializable<decltype(obj.outputs)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxFuncCall::EmbeddedCurveAdd &lhs, const BlackBoxFuncCall::EmbeddedCurveAdd &rhs) {
        if (!(lhs.input1 == rhs.input1)) { return false; }
        if (!(lhs.input2 == rhs.input2)) { return false; }
        if (!(lhs.outputs == rhs.outputs)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxFuncCall::EmbeddedCurveAdd::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::EmbeddedCurveAdd>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::EmbeddedCurveAdd BlackBoxFuncCall::EmbeddedCurveAdd::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::EmbeddedCurveAdd>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxFuncCall::EmbeddedCurveAdd>::serialize(const Acir::BlackBoxFuncCall::EmbeddedCurveAdd &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.input1)>::serialize(obj.input1, serializer);
    serde::Serializable<decltype(obj.input2)>::serialize(obj.input2, serializer);
    serde::Serializable<decltype(obj.outputs)>::serialize(obj.outputs, serializer);
}

template <>
template <typename Deserializer>
Acir::BlackBoxFuncCall::EmbeddedCurveAdd serde::Deserializable<Acir::BlackBoxFuncCall::EmbeddedCurveAdd>::deserialize(Deserializer &deserializer) {
    Acir::BlackBoxFuncCall::EmbeddedCurveAdd obj;
    obj.input1 = serde::Deserializable<decltype(obj.input1)>::deserialize(deserializer);
    obj.input2 = serde::Deserializable<decltype(obj.input2)>::deserialize(deserializer);
    obj.outputs = serde::Deserializable<decltype(obj.outputs)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxFuncCall::Keccakf1600 &lhs, const BlackBoxFuncCall::Keccakf1600 &rhs) {
        if (!(lhs.inputs == rhs.inputs)) { return false; }
        if (!(lhs.outputs == rhs.outputs)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxFuncCall::Keccakf1600::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::Keccakf1600>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::Keccakf1600 BlackBoxFuncCall::Keccakf1600::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::Keccakf1600>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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
        return true;
    }

    inline std::vector<uint8_t> BlackBoxFuncCall::RecursiveAggregation::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::RecursiveAggregation>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::RecursiveAggregation BlackBoxFuncCall::RecursiveAggregation::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::RecursiveAggregation>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxFuncCall::BigIntAdd &lhs, const BlackBoxFuncCall::BigIntAdd &rhs) {
        if (!(lhs.lhs == rhs.lhs)) { return false; }
        if (!(lhs.rhs == rhs.rhs)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxFuncCall::BigIntAdd::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::BigIntAdd>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::BigIntAdd BlackBoxFuncCall::BigIntAdd::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::BigIntAdd>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxFuncCall::BigIntAdd>::serialize(const Acir::BlackBoxFuncCall::BigIntAdd &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.lhs)>::serialize(obj.lhs, serializer);
    serde::Serializable<decltype(obj.rhs)>::serialize(obj.rhs, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Acir::BlackBoxFuncCall::BigIntAdd serde::Deserializable<Acir::BlackBoxFuncCall::BigIntAdd>::deserialize(Deserializer &deserializer) {
    Acir::BlackBoxFuncCall::BigIntAdd obj;
    obj.lhs = serde::Deserializable<decltype(obj.lhs)>::deserialize(deserializer);
    obj.rhs = serde::Deserializable<decltype(obj.rhs)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxFuncCall::BigIntSub &lhs, const BlackBoxFuncCall::BigIntSub &rhs) {
        if (!(lhs.lhs == rhs.lhs)) { return false; }
        if (!(lhs.rhs == rhs.rhs)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxFuncCall::BigIntSub::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::BigIntSub>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::BigIntSub BlackBoxFuncCall::BigIntSub::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::BigIntSub>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxFuncCall::BigIntSub>::serialize(const Acir::BlackBoxFuncCall::BigIntSub &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.lhs)>::serialize(obj.lhs, serializer);
    serde::Serializable<decltype(obj.rhs)>::serialize(obj.rhs, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Acir::BlackBoxFuncCall::BigIntSub serde::Deserializable<Acir::BlackBoxFuncCall::BigIntSub>::deserialize(Deserializer &deserializer) {
    Acir::BlackBoxFuncCall::BigIntSub obj;
    obj.lhs = serde::Deserializable<decltype(obj.lhs)>::deserialize(deserializer);
    obj.rhs = serde::Deserializable<decltype(obj.rhs)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxFuncCall::BigIntMul &lhs, const BlackBoxFuncCall::BigIntMul &rhs) {
        if (!(lhs.lhs == rhs.lhs)) { return false; }
        if (!(lhs.rhs == rhs.rhs)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxFuncCall::BigIntMul::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::BigIntMul>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::BigIntMul BlackBoxFuncCall::BigIntMul::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::BigIntMul>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxFuncCall::BigIntMul>::serialize(const Acir::BlackBoxFuncCall::BigIntMul &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.lhs)>::serialize(obj.lhs, serializer);
    serde::Serializable<decltype(obj.rhs)>::serialize(obj.rhs, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Acir::BlackBoxFuncCall::BigIntMul serde::Deserializable<Acir::BlackBoxFuncCall::BigIntMul>::deserialize(Deserializer &deserializer) {
    Acir::BlackBoxFuncCall::BigIntMul obj;
    obj.lhs = serde::Deserializable<decltype(obj.lhs)>::deserialize(deserializer);
    obj.rhs = serde::Deserializable<decltype(obj.rhs)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxFuncCall::BigIntDiv &lhs, const BlackBoxFuncCall::BigIntDiv &rhs) {
        if (!(lhs.lhs == rhs.lhs)) { return false; }
        if (!(lhs.rhs == rhs.rhs)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxFuncCall::BigIntDiv::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::BigIntDiv>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::BigIntDiv BlackBoxFuncCall::BigIntDiv::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::BigIntDiv>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxFuncCall::BigIntDiv>::serialize(const Acir::BlackBoxFuncCall::BigIntDiv &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.lhs)>::serialize(obj.lhs, serializer);
    serde::Serializable<decltype(obj.rhs)>::serialize(obj.rhs, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Acir::BlackBoxFuncCall::BigIntDiv serde::Deserializable<Acir::BlackBoxFuncCall::BigIntDiv>::deserialize(Deserializer &deserializer) {
    Acir::BlackBoxFuncCall::BigIntDiv obj;
    obj.lhs = serde::Deserializable<decltype(obj.lhs)>::deserialize(deserializer);
    obj.rhs = serde::Deserializable<decltype(obj.rhs)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxFuncCall::BigIntFromLeBytes &lhs, const BlackBoxFuncCall::BigIntFromLeBytes &rhs) {
        if (!(lhs.inputs == rhs.inputs)) { return false; }
        if (!(lhs.modulus == rhs.modulus)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxFuncCall::BigIntFromLeBytes::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::BigIntFromLeBytes>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::BigIntFromLeBytes BlackBoxFuncCall::BigIntFromLeBytes::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::BigIntFromLeBytes>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxFuncCall::BigIntFromLeBytes>::serialize(const Acir::BlackBoxFuncCall::BigIntFromLeBytes &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.inputs)>::serialize(obj.inputs, serializer);
    serde::Serializable<decltype(obj.modulus)>::serialize(obj.modulus, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Acir::BlackBoxFuncCall::BigIntFromLeBytes serde::Deserializable<Acir::BlackBoxFuncCall::BigIntFromLeBytes>::deserialize(Deserializer &deserializer) {
    Acir::BlackBoxFuncCall::BigIntFromLeBytes obj;
    obj.inputs = serde::Deserializable<decltype(obj.inputs)>::deserialize(deserializer);
    obj.modulus = serde::Deserializable<decltype(obj.modulus)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxFuncCall::BigIntToLeBytes &lhs, const BlackBoxFuncCall::BigIntToLeBytes &rhs) {
        if (!(lhs.input == rhs.input)) { return false; }
        if (!(lhs.outputs == rhs.outputs)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxFuncCall::BigIntToLeBytes::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::BigIntToLeBytes>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::BigIntToLeBytes BlackBoxFuncCall::BigIntToLeBytes::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::BigIntToLeBytes>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxFuncCall::BigIntToLeBytes>::serialize(const Acir::BlackBoxFuncCall::BigIntToLeBytes &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.input)>::serialize(obj.input, serializer);
    serde::Serializable<decltype(obj.outputs)>::serialize(obj.outputs, serializer);
}

template <>
template <typename Deserializer>
Acir::BlackBoxFuncCall::BigIntToLeBytes serde::Deserializable<Acir::BlackBoxFuncCall::BigIntToLeBytes>::deserialize(Deserializer &deserializer) {
    Acir::BlackBoxFuncCall::BigIntToLeBytes obj;
    obj.input = serde::Deserializable<decltype(obj.input)>::deserialize(deserializer);
    obj.outputs = serde::Deserializable<decltype(obj.outputs)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxFuncCall::Poseidon2Permutation &lhs, const BlackBoxFuncCall::Poseidon2Permutation &rhs) {
        if (!(lhs.inputs == rhs.inputs)) { return false; }
        if (!(lhs.outputs == rhs.outputs)) { return false; }
        if (!(lhs.len == rhs.len)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxFuncCall::Poseidon2Permutation::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::Poseidon2Permutation>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::Poseidon2Permutation BlackBoxFuncCall::Poseidon2Permutation::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::Poseidon2Permutation>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxFuncCall::Poseidon2Permutation>::serialize(const Acir::BlackBoxFuncCall::Poseidon2Permutation &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.inputs)>::serialize(obj.inputs, serializer);
    serde::Serializable<decltype(obj.outputs)>::serialize(obj.outputs, serializer);
    serde::Serializable<decltype(obj.len)>::serialize(obj.len, serializer);
}

template <>
template <typename Deserializer>
Acir::BlackBoxFuncCall::Poseidon2Permutation serde::Deserializable<Acir::BlackBoxFuncCall::Poseidon2Permutation>::deserialize(Deserializer &deserializer) {
    Acir::BlackBoxFuncCall::Poseidon2Permutation obj;
    obj.inputs = serde::Deserializable<decltype(obj.inputs)>::deserialize(deserializer);
    obj.outputs = serde::Deserializable<decltype(obj.outputs)>::deserialize(deserializer);
    obj.len = serde::Deserializable<decltype(obj.len)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxFuncCall::Sha256Compression &lhs, const BlackBoxFuncCall::Sha256Compression &rhs) {
        if (!(lhs.inputs == rhs.inputs)) { return false; }
        if (!(lhs.hash_values == rhs.hash_values)) { return false; }
        if (!(lhs.outputs == rhs.outputs)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxFuncCall::Sha256Compression::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::Sha256Compression>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::Sha256Compression BlackBoxFuncCall::Sha256Compression::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::Sha256Compression>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BlackBoxOp::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxOp>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxOp BlackBoxOp::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxOp>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BlackBoxOp::AES128Encrypt::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxOp::AES128Encrypt>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxOp::AES128Encrypt BlackBoxOp::AES128Encrypt::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxOp::AES128Encrypt>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BlackBoxOp::Blake2s::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxOp::Blake2s>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxOp::Blake2s BlackBoxOp::Blake2s::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxOp::Blake2s>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BlackBoxOp::Blake3::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxOp::Blake3>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxOp::Blake3 BlackBoxOp::Blake3::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxOp::Blake3>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BlackBoxOp::Keccakf1600::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxOp::Keccakf1600>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxOp::Keccakf1600 BlackBoxOp::Keccakf1600::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxOp::Keccakf1600>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BlackBoxOp::EcdsaSecp256k1::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxOp::EcdsaSecp256k1>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxOp::EcdsaSecp256k1 BlackBoxOp::EcdsaSecp256k1::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxOp::EcdsaSecp256k1>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BlackBoxOp::EcdsaSecp256r1::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxOp::EcdsaSecp256r1>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxOp::EcdsaSecp256r1 BlackBoxOp::EcdsaSecp256r1::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxOp::EcdsaSecp256r1>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BlackBoxOp::MultiScalarMul::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxOp::MultiScalarMul>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxOp::MultiScalarMul BlackBoxOp::MultiScalarMul::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxOp::MultiScalarMul>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BlackBoxOp::EmbeddedCurveAdd::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxOp::EmbeddedCurveAdd>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxOp::EmbeddedCurveAdd BlackBoxOp::EmbeddedCurveAdd::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxOp::EmbeddedCurveAdd>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline bool operator==(const BlackBoxOp::BigIntAdd &lhs, const BlackBoxOp::BigIntAdd &rhs) {
        if (!(lhs.lhs == rhs.lhs)) { return false; }
        if (!(lhs.rhs == rhs.rhs)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxOp::BigIntAdd::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxOp::BigIntAdd>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxOp::BigIntAdd BlackBoxOp::BigIntAdd::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxOp::BigIntAdd>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxOp::BigIntAdd>::serialize(const Acir::BlackBoxOp::BigIntAdd &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.lhs)>::serialize(obj.lhs, serializer);
    serde::Serializable<decltype(obj.rhs)>::serialize(obj.rhs, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Acir::BlackBoxOp::BigIntAdd serde::Deserializable<Acir::BlackBoxOp::BigIntAdd>::deserialize(Deserializer &deserializer) {
    Acir::BlackBoxOp::BigIntAdd obj;
    obj.lhs = serde::Deserializable<decltype(obj.lhs)>::deserialize(deserializer);
    obj.rhs = serde::Deserializable<decltype(obj.rhs)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxOp::BigIntSub &lhs, const BlackBoxOp::BigIntSub &rhs) {
        if (!(lhs.lhs == rhs.lhs)) { return false; }
        if (!(lhs.rhs == rhs.rhs)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxOp::BigIntSub::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxOp::BigIntSub>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxOp::BigIntSub BlackBoxOp::BigIntSub::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxOp::BigIntSub>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxOp::BigIntSub>::serialize(const Acir::BlackBoxOp::BigIntSub &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.lhs)>::serialize(obj.lhs, serializer);
    serde::Serializable<decltype(obj.rhs)>::serialize(obj.rhs, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Acir::BlackBoxOp::BigIntSub serde::Deserializable<Acir::BlackBoxOp::BigIntSub>::deserialize(Deserializer &deserializer) {
    Acir::BlackBoxOp::BigIntSub obj;
    obj.lhs = serde::Deserializable<decltype(obj.lhs)>::deserialize(deserializer);
    obj.rhs = serde::Deserializable<decltype(obj.rhs)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxOp::BigIntMul &lhs, const BlackBoxOp::BigIntMul &rhs) {
        if (!(lhs.lhs == rhs.lhs)) { return false; }
        if (!(lhs.rhs == rhs.rhs)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxOp::BigIntMul::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxOp::BigIntMul>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxOp::BigIntMul BlackBoxOp::BigIntMul::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxOp::BigIntMul>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxOp::BigIntMul>::serialize(const Acir::BlackBoxOp::BigIntMul &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.lhs)>::serialize(obj.lhs, serializer);
    serde::Serializable<decltype(obj.rhs)>::serialize(obj.rhs, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Acir::BlackBoxOp::BigIntMul serde::Deserializable<Acir::BlackBoxOp::BigIntMul>::deserialize(Deserializer &deserializer) {
    Acir::BlackBoxOp::BigIntMul obj;
    obj.lhs = serde::Deserializable<decltype(obj.lhs)>::deserialize(deserializer);
    obj.rhs = serde::Deserializable<decltype(obj.rhs)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxOp::BigIntDiv &lhs, const BlackBoxOp::BigIntDiv &rhs) {
        if (!(lhs.lhs == rhs.lhs)) { return false; }
        if (!(lhs.rhs == rhs.rhs)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxOp::BigIntDiv::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxOp::BigIntDiv>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxOp::BigIntDiv BlackBoxOp::BigIntDiv::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxOp::BigIntDiv>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxOp::BigIntDiv>::serialize(const Acir::BlackBoxOp::BigIntDiv &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.lhs)>::serialize(obj.lhs, serializer);
    serde::Serializable<decltype(obj.rhs)>::serialize(obj.rhs, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Acir::BlackBoxOp::BigIntDiv serde::Deserializable<Acir::BlackBoxOp::BigIntDiv>::deserialize(Deserializer &deserializer) {
    Acir::BlackBoxOp::BigIntDiv obj;
    obj.lhs = serde::Deserializable<decltype(obj.lhs)>::deserialize(deserializer);
    obj.rhs = serde::Deserializable<decltype(obj.rhs)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxOp::BigIntFromLeBytes &lhs, const BlackBoxOp::BigIntFromLeBytes &rhs) {
        if (!(lhs.inputs == rhs.inputs)) { return false; }
        if (!(lhs.modulus == rhs.modulus)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxOp::BigIntFromLeBytes::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxOp::BigIntFromLeBytes>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxOp::BigIntFromLeBytes BlackBoxOp::BigIntFromLeBytes::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxOp::BigIntFromLeBytes>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxOp::BigIntFromLeBytes>::serialize(const Acir::BlackBoxOp::BigIntFromLeBytes &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.inputs)>::serialize(obj.inputs, serializer);
    serde::Serializable<decltype(obj.modulus)>::serialize(obj.modulus, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Acir::BlackBoxOp::BigIntFromLeBytes serde::Deserializable<Acir::BlackBoxOp::BigIntFromLeBytes>::deserialize(Deserializer &deserializer) {
    Acir::BlackBoxOp::BigIntFromLeBytes obj;
    obj.inputs = serde::Deserializable<decltype(obj.inputs)>::deserialize(deserializer);
    obj.modulus = serde::Deserializable<decltype(obj.modulus)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxOp::BigIntToLeBytes &lhs, const BlackBoxOp::BigIntToLeBytes &rhs) {
        if (!(lhs.input == rhs.input)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxOp::BigIntToLeBytes::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxOp::BigIntToLeBytes>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxOp::BigIntToLeBytes BlackBoxOp::BigIntToLeBytes::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxOp::BigIntToLeBytes>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxOp::BigIntToLeBytes>::serialize(const Acir::BlackBoxOp::BigIntToLeBytes &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.input)>::serialize(obj.input, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Acir::BlackBoxOp::BigIntToLeBytes serde::Deserializable<Acir::BlackBoxOp::BigIntToLeBytes>::deserialize(Deserializer &deserializer) {
    Acir::BlackBoxOp::BigIntToLeBytes obj;
    obj.input = serde::Deserializable<decltype(obj.input)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxOp::Poseidon2Permutation &lhs, const BlackBoxOp::Poseidon2Permutation &rhs) {
        if (!(lhs.message == rhs.message)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        if (!(lhs.len == rhs.len)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxOp::Poseidon2Permutation::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxOp::Poseidon2Permutation>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxOp::Poseidon2Permutation BlackBoxOp::Poseidon2Permutation::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxOp::Poseidon2Permutation>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BlackBoxOp::Poseidon2Permutation>::serialize(const Acir::BlackBoxOp::Poseidon2Permutation &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.message)>::serialize(obj.message, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
    serde::Serializable<decltype(obj.len)>::serialize(obj.len, serializer);
}

template <>
template <typename Deserializer>
Acir::BlackBoxOp::Poseidon2Permutation serde::Deserializable<Acir::BlackBoxOp::Poseidon2Permutation>::deserialize(Deserializer &deserializer) {
    Acir::BlackBoxOp::Poseidon2Permutation obj;
    obj.message = serde::Deserializable<decltype(obj.message)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    obj.len = serde::Deserializable<decltype(obj.len)>::deserialize(deserializer);
    return obj;
}

namespace Acir {

    inline bool operator==(const BlackBoxOp::Sha256Compression &lhs, const BlackBoxOp::Sha256Compression &rhs) {
        if (!(lhs.input == rhs.input)) { return false; }
        if (!(lhs.hash_values == rhs.hash_values)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxOp::Sha256Compression::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxOp::Sha256Compression>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxOp::Sha256Compression BlackBoxOp::Sha256Compression::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxOp::Sha256Compression>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BlackBoxOp::ToRadix::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxOp::ToRadix>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxOp::ToRadix BlackBoxOp::ToRadix::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxOp::ToRadix>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BlockId::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlockId>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlockId BlockId::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlockId>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BlockType::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlockType>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlockType BlockType::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlockType>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BlockType::Memory::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlockType::Memory>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlockType::Memory BlockType::Memory::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlockType::Memory>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BlockType::CallData::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlockType::CallData>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlockType::CallData BlockType::CallData::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlockType::CallData>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BlockType::ReturnData::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlockType::ReturnData>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlockType::ReturnData BlockType::ReturnData::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlockType::ReturnData>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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
        if (!(lhs.bytecode == rhs.bytecode)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BrilligBytecode::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligBytecode>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligBytecode BrilligBytecode::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligBytecode>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::BrilligBytecode>::serialize(const Acir::BrilligBytecode &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.bytecode)>::serialize(obj.bytecode, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Acir::BrilligBytecode serde::Deserializable<Acir::BrilligBytecode>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Acir::BrilligBytecode obj;
    obj.bytecode = serde::Deserializable<decltype(obj.bytecode)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Acir {

    inline bool operator==(const BrilligInputs &lhs, const BrilligInputs &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BrilligInputs::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligInputs>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligInputs BrilligInputs::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligInputs>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BrilligInputs::Single::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligInputs::Single>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligInputs::Single BrilligInputs::Single::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligInputs::Single>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BrilligInputs::Array::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligInputs::Array>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligInputs::Array BrilligInputs::Array::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligInputs::Array>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BrilligInputs::MemoryArray::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligInputs::MemoryArray>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligInputs::MemoryArray BrilligInputs::MemoryArray::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligInputs::MemoryArray>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BrilligOpcode::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOpcode>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOpcode BrilligOpcode::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOpcode>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BrilligOpcode::BinaryFieldOp::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOpcode::BinaryFieldOp>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOpcode::BinaryFieldOp BrilligOpcode::BinaryFieldOp::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOpcode::BinaryFieldOp>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BrilligOpcode::BinaryIntOp::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOpcode::BinaryIntOp>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOpcode::BinaryIntOp BrilligOpcode::BinaryIntOp::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOpcode::BinaryIntOp>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BrilligOpcode::Not::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOpcode::Not>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOpcode::Not BrilligOpcode::Not::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOpcode::Not>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BrilligOpcode::Cast::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOpcode::Cast>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOpcode::Cast BrilligOpcode::Cast::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOpcode::Cast>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BrilligOpcode::JumpIf::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOpcode::JumpIf>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOpcode::JumpIf BrilligOpcode::JumpIf::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOpcode::JumpIf>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BrilligOpcode::Jump::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOpcode::Jump>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOpcode::Jump BrilligOpcode::Jump::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOpcode::Jump>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BrilligOpcode::CalldataCopy::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOpcode::CalldataCopy>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOpcode::CalldataCopy BrilligOpcode::CalldataCopy::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOpcode::CalldataCopy>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BrilligOpcode::Call::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOpcode::Call>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOpcode::Call BrilligOpcode::Call::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOpcode::Call>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BrilligOpcode::Const::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOpcode::Const>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOpcode::Const BrilligOpcode::Const::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOpcode::Const>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BrilligOpcode::IndirectConst::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOpcode::IndirectConst>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOpcode::IndirectConst BrilligOpcode::IndirectConst::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOpcode::IndirectConst>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BrilligOpcode::Return::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOpcode::Return>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOpcode::Return BrilligOpcode::Return::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOpcode::Return>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BrilligOpcode::ForeignCall::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOpcode::ForeignCall>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOpcode::ForeignCall BrilligOpcode::ForeignCall::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOpcode::ForeignCall>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BrilligOpcode::Mov::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOpcode::Mov>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOpcode::Mov BrilligOpcode::Mov::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOpcode::Mov>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BrilligOpcode::ConditionalMov::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOpcode::ConditionalMov>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOpcode::ConditionalMov BrilligOpcode::ConditionalMov::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOpcode::ConditionalMov>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BrilligOpcode::Load::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOpcode::Load>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOpcode::Load BrilligOpcode::Load::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOpcode::Load>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BrilligOpcode::Store::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOpcode::Store>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOpcode::Store BrilligOpcode::Store::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOpcode::Store>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BrilligOpcode::BlackBox::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOpcode::BlackBox>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOpcode::BlackBox BrilligOpcode::BlackBox::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOpcode::BlackBox>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BrilligOpcode::Trap::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOpcode::Trap>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOpcode::Trap BrilligOpcode::Trap::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOpcode::Trap>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BrilligOpcode::Stop::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOpcode::Stop>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOpcode::Stop BrilligOpcode::Stop::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOpcode::Stop>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BrilligOutputs::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOutputs>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOutputs BrilligOutputs::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOutputs>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BrilligOutputs::Simple::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOutputs::Simple>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOutputs::Simple BrilligOutputs::Simple::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOutputs::Simple>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> BrilligOutputs::Array::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOutputs::Array>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOutputs::Array BrilligOutputs::Array::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOutputs::Array>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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
        if (!(lhs.current_witness_index == rhs.current_witness_index)) { return false; }
        if (!(lhs.opcodes == rhs.opcodes)) { return false; }
        if (!(lhs.expression_width == rhs.expression_width)) { return false; }
        if (!(lhs.private_parameters == rhs.private_parameters)) { return false; }
        if (!(lhs.public_parameters == rhs.public_parameters)) { return false; }
        if (!(lhs.return_values == rhs.return_values)) { return false; }
        if (!(lhs.assert_messages == rhs.assert_messages)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> Circuit::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<Circuit>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline Circuit Circuit::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<Circuit>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::Circuit>::serialize(const Acir::Circuit &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.current_witness_index)>::serialize(obj.current_witness_index, serializer);
    serde::Serializable<decltype(obj.opcodes)>::serialize(obj.opcodes, serializer);
    serde::Serializable<decltype(obj.expression_width)>::serialize(obj.expression_width, serializer);
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
    obj.current_witness_index = serde::Deserializable<decltype(obj.current_witness_index)>::deserialize(deserializer);
    obj.opcodes = serde::Deserializable<decltype(obj.opcodes)>::deserialize(deserializer);
    obj.expression_width = serde::Deserializable<decltype(obj.expression_width)>::deserialize(deserializer);
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

    inline std::vector<uint8_t> Expression::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<Expression>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline Expression Expression::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<Expression>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> ExpressionOrMemory::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<ExpressionOrMemory>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline ExpressionOrMemory ExpressionOrMemory::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<ExpressionOrMemory>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> ExpressionOrMemory::Expression::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<ExpressionOrMemory::Expression>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline ExpressionOrMemory::Expression ExpressionOrMemory::Expression::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<ExpressionOrMemory::Expression>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> ExpressionOrMemory::Memory::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<ExpressionOrMemory::Memory>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline ExpressionOrMemory::Memory ExpressionOrMemory::Memory::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<ExpressionOrMemory::Memory>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> ExpressionWidth::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<ExpressionWidth>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline ExpressionWidth ExpressionWidth::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<ExpressionWidth>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> ExpressionWidth::Unbounded::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<ExpressionWidth::Unbounded>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline ExpressionWidth::Unbounded ExpressionWidth::Unbounded::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<ExpressionWidth::Unbounded>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> ExpressionWidth::Bounded::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<ExpressionWidth::Bounded>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline ExpressionWidth::Bounded ExpressionWidth::Bounded::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<ExpressionWidth::Bounded>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> FunctionInput::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<FunctionInput>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline FunctionInput FunctionInput::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<FunctionInput>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> FunctionInput::Constant::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<FunctionInput::Constant>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline FunctionInput::Constant FunctionInput::Constant::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<FunctionInput::Constant>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> FunctionInput::Witness::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<FunctionInput::Witness>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline FunctionInput::Witness FunctionInput::Witness::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<FunctionInput::Witness>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> HeapArray::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<HeapArray>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline HeapArray HeapArray::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<HeapArray>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> HeapValueType::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<HeapValueType>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline HeapValueType HeapValueType::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<HeapValueType>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> HeapValueType::Simple::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<HeapValueType::Simple>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline HeapValueType::Simple HeapValueType::Simple::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<HeapValueType::Simple>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> HeapValueType::Array::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<HeapValueType::Array>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline HeapValueType::Array HeapValueType::Array::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<HeapValueType::Array>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> HeapValueType::Vector::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<HeapValueType::Vector>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline HeapValueType::Vector HeapValueType::Vector::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<HeapValueType::Vector>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> HeapVector::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<HeapVector>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline HeapVector HeapVector::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<HeapVector>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> IntegerBitSize::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<IntegerBitSize>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline IntegerBitSize IntegerBitSize::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<IntegerBitSize>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> IntegerBitSize::U1::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<IntegerBitSize::U1>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline IntegerBitSize::U1 IntegerBitSize::U1::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<IntegerBitSize::U1>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> IntegerBitSize::U8::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<IntegerBitSize::U8>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline IntegerBitSize::U8 IntegerBitSize::U8::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<IntegerBitSize::U8>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> IntegerBitSize::U16::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<IntegerBitSize::U16>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline IntegerBitSize::U16 IntegerBitSize::U16::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<IntegerBitSize::U16>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> IntegerBitSize::U32::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<IntegerBitSize::U32>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline IntegerBitSize::U32 IntegerBitSize::U32::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<IntegerBitSize::U32>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> IntegerBitSize::U64::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<IntegerBitSize::U64>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline IntegerBitSize::U64 IntegerBitSize::U64::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<IntegerBitSize::U64>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> IntegerBitSize::U128::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<IntegerBitSize::U128>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline IntegerBitSize::U128 IntegerBitSize::U128::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<IntegerBitSize::U128>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> MemOp::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<MemOp>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline MemOp MemOp::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<MemOp>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> MemoryAddress::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<MemoryAddress>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline MemoryAddress MemoryAddress::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<MemoryAddress>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> MemoryAddress::Direct::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<MemoryAddress::Direct>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline MemoryAddress::Direct MemoryAddress::Direct::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<MemoryAddress::Direct>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> MemoryAddress::Relative::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<MemoryAddress::Relative>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline MemoryAddress::Relative MemoryAddress::Relative::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<MemoryAddress::Relative>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> Opcode::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<Opcode>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline Opcode Opcode::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<Opcode>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> Opcode::AssertZero::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<Opcode::AssertZero>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline Opcode::AssertZero Opcode::AssertZero::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<Opcode::AssertZero>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> Opcode::BlackBoxFuncCall::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<Opcode::BlackBoxFuncCall>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline Opcode::BlackBoxFuncCall Opcode::BlackBoxFuncCall::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<Opcode::BlackBoxFuncCall>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> Opcode::MemoryOp::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<Opcode::MemoryOp>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline Opcode::MemoryOp Opcode::MemoryOp::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<Opcode::MemoryOp>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> Opcode::MemoryInit::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<Opcode::MemoryInit>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline Opcode::MemoryInit Opcode::MemoryInit::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<Opcode::MemoryInit>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> Opcode::BrilligCall::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<Opcode::BrilligCall>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline Opcode::BrilligCall Opcode::BrilligCall::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<Opcode::BrilligCall>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> Opcode::Call::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<Opcode::Call>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline Opcode::Call Opcode::Call::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<Opcode::Call>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> OpcodeLocation::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<OpcodeLocation>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline OpcodeLocation OpcodeLocation::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<OpcodeLocation>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> OpcodeLocation::Acir::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<OpcodeLocation::Acir>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline OpcodeLocation::Acir OpcodeLocation::Acir::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<OpcodeLocation::Acir>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> OpcodeLocation::Brillig::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<OpcodeLocation::Brillig>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline OpcodeLocation::Brillig OpcodeLocation::Brillig::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<OpcodeLocation::Brillig>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> Program::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<Program>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline Program Program::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<Program>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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
        return true;
    }

    inline std::vector<uint8_t> ProgramWithoutBrillig::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<ProgramWithoutBrillig>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline ProgramWithoutBrillig ProgramWithoutBrillig::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<ProgramWithoutBrillig>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Acir

template <>
template <typename Serializer>
void serde::Serializable<Acir::ProgramWithoutBrillig>::serialize(const Acir::ProgramWithoutBrillig &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.functions)>::serialize(obj.functions, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Acir::ProgramWithoutBrillig serde::Deserializable<Acir::ProgramWithoutBrillig>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Acir::ProgramWithoutBrillig obj;
    obj.functions = serde::Deserializable<decltype(obj.functions)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Acir {

    inline bool operator==(const PublicInputs &lhs, const PublicInputs &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> PublicInputs::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<PublicInputs>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline PublicInputs PublicInputs::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<PublicInputs>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline bool operator==(const ValueOrArray &lhs, const ValueOrArray &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> ValueOrArray::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<ValueOrArray>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline ValueOrArray ValueOrArray::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<ValueOrArray>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> ValueOrArray::MemoryAddress::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<ValueOrArray::MemoryAddress>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline ValueOrArray::MemoryAddress ValueOrArray::MemoryAddress::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<ValueOrArray::MemoryAddress>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> ValueOrArray::HeapArray::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<ValueOrArray::HeapArray>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline ValueOrArray::HeapArray ValueOrArray::HeapArray::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<ValueOrArray::HeapArray>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> ValueOrArray::HeapVector::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<ValueOrArray::HeapVector>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline ValueOrArray::HeapVector ValueOrArray::HeapVector::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<ValueOrArray::HeapVector>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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

    inline std::vector<uint8_t> Witness::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<Witness>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline Witness Witness::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<Witness>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw_or_abort("Some input bytes were not read");
        }
        return value;
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
