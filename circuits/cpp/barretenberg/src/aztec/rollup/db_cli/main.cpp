#include <stdlib/merkle_tree/leveldb_store.hpp>
#include <stdlib/merkle_tree/leveldb_tree.hpp>
#include "get.hpp"
#include "put.hpp"

using namespace plonk::stdlib::merkle_tree;

char const* DB_PATH = "./world_state.db";

enum Command {
    GET,
    PUT,
    COMMIT,
    ROLLBACK,
};

class WorldStateDb {
  public:
    WorldStateDb(std::string const& db_path)
        : store_(db_path)
        , data_tree_(store_, 32, 0)
        , nullifier_tree_(store_, 128, 1)
        , trees_({ &data_tree_, &nullifier_tree_ })
    {

        std::cerr << "DB root: " << data_tree_.root() << " size: " << data_tree_.size() << std::endl;
        std::cerr << "Nullifier root: " << nullifier_tree_.root() << " size: " << nullifier_tree_.size() << std::endl;
    }

    void write_metadata(std::ostream& os) {
        write(os, data_tree_.root());
        write(os, nullifier_tree_.root());
        write(os, data_tree_.size());
        write(os, nullifier_tree_.size());
    }

    void get(std::istream& is, std::ostream& os)
    {
        GetRequest get_request;
        read(is, get_request);
        std::cerr << get_request << std::endl;
        std::vector<uint8_t> r = trees_[get_request.tree_id]->get_element(get_request.index);
        GetResponse get_response;
        std::copy(r.begin(), r.end(), get_response.value.begin());
        write(os, get_response);
    }

    void put(std::istream& is, std::ostream& os)
    {
        PutRequest put_request;
        read(is, put_request);
        std::cerr << put_request << std::endl;
        PutResponse put_response;
        put_response.root = trees_[put_request.tree_id]->update_element(
            put_request.index, { put_request.value.begin(), put_request.value.end() });
        write(os, put_response);
    }

    void commit(std::ostream& os) {
        std::cerr << "COMMIT" << std::endl;
        store_.commit();
        write(os, uint8_t(1));
    }

    void rollback(std::ostream& os) {
        std::cerr << "ROLLBACK" << std::endl;
        store_.rollback();
        write(os, uint8_t(1));
    }

  private:
    LevelDbStore store_;
    LevelDbTree data_tree_;
    LevelDbTree nullifier_tree_;
    std::array<LevelDbTree*, 2> trees_;
};

int main(int argc, char** argv)
{
    std::vector<std::string> args(argv, argv + argc);

    if (args.size() > 1 && args[1] == "reset") {
        LevelDbStore::destroy(DB_PATH);
        std::cout << "Erased db." << std::endl;
        return 0;
    }

    WorldStateDb world_state_db(DB_PATH);

    world_state_db.write_metadata(std::cout);

    // Read commands from stdin.
    while (true) {
        uint8_t command;

        if (!std::cin.good() || std::cin.peek() == std::char_traits<char>::eof()) {
            break;
        }

        read(std::cin, command);

        switch (command) {
        case GET:
            world_state_db.get(std::cin, std::cout);
            break;
        case PUT:
            world_state_db.put(std::cin, std::cout);
            break;
        case COMMIT:
            world_state_db.commit(std::cout);
            break;
        case ROLLBACK:
            world_state_db.rollback(std::cout);
            break;
        }
    }

    return 0;
}
