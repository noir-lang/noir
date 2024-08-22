#pragma once

#include "barretenberg/messaging/header.hpp"
#include "barretenberg/serialize/cbind.hpp"
#include "msgpack/v3/object_fwd_decl.hpp"
#include <cstdint>
#include <functional>
#include <iostream>
#include <utility>
#include <vector>

namespace bb::messaging {
template <typename OutputStream> class StreamDispatcher {
  private:
    OutputStream& outputStream;
    std::unordered_map<uint32_t, std::function<bool(msgpack::object&)>> messageHandlers;
    void sendPong(uint32_t pingId);
    bool handleSystemMessage(msgpack::object& obj);

  public:
    StreamDispatcher(OutputStream& out)
        : outputStream(out)
    {}
    bool onNewData(msgpack::object& obj);
    void registerTarget(uint32_t msgType, std::function<bool(msgpack::object&)>& handler);
};

template <typename OutputStream> bool StreamDispatcher<OutputStream>::onNewData(msgpack::object& obj)
{
    bb::messaging::HeaderOnlyMessage header;
    obj.convert(header);

    if (header.msgType < FIRST_APP_MSG_TYPE) {
        return handleSystemMessage(obj);
    }
    auto iter = messageHandlers.find(header.msgType);
    if (iter == messageHandlers.end()) {
        std::cerr << "No registered handler for message of type " << header.msgType << std::endl;
        return true;
    }
    return (iter->second)(obj);
}

template <typename OutputStream>
void StreamDispatcher<OutputStream>::registerTarget(uint32_t msgType, std::function<bool(msgpack::object&)>& handler)
{
    messageHandlers.insert({ msgType, handler });
}

template <typename OutputStream> bool StreamDispatcher<OutputStream>::handleSystemMessage(msgpack::object& obj)
{
    bb::messaging::HeaderOnlyMessage header;
    obj.convert(header);
    if (header.msgType == SystemMsgTypes::TERMINATE) {
        return false;
    }
    if (header.msgType == SystemMsgTypes::PING) {
        sendPong(header.header.messageId);
    }
    return true;
}

template <typename OutputStream> void StreamDispatcher<OutputStream>::sendPong(uint32_t pingId)
{
    MsgHeader header(pingId);
    HeaderOnlyMessage packedHeader(SystemMsgTypes::PONG, header);
    outputStream.send(packedHeader);
}
} // namespace bb::messaging
