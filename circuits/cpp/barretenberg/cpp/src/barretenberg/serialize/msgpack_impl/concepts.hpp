#pragma once

struct DoNothing {
    void operator()(auto...) {}
};
namespace msgpack_concepts {
template <typename T>
concept HasMsgPack = requires(T t, DoNothing nop) { t.msgpack(nop); };

template <typename T>
concept HasMsgPackSchema = requires(const T t, DoNothing nop) { t.msgpack_schema(nop); };

template <typename T>
concept HasMsgPackPack = requires(T t, DoNothing nop) { t.msgpack_pack(nop); };
template <typename T, typename... Args>
concept MsgpackConstructible = requires(T object, Args... args) { T{ args... }; };

} // namespace msgpack_concepts
