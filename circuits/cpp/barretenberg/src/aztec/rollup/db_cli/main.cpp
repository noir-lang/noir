#include "get.hpp"
#include "put.hpp"
#include <stdlib/merkle_tree/leveldb_store.hpp>
#include <stdlib/merkle_tree/leveldb_tree.hpp>

using namespace plonk::stdlib::merkle_tree;

char const* DB_PATH = "./world_state.db";

enum Command {
    GET,
    PUT,
    COMMIT,
    ROLLBACK,
    GETPATH,
};

class WorldStateDb {
  public:
    WorldStateDb(std::string const& db_path)
        : store_(db_path)
        , data_tree_(store_, 32, 0)
        , nullifier_tree_(store_, 128, 1)
        , root_tree_(store_, 28, 2)
        , trees_({ &data_tree_, &nullifier_tree_, &root_tree_ })
    {
        if (root_tree_.size() == 0) {
            root_tree_.update_element(0, to_buffer(data_tree_.root()));
            store_.commit();
        }

        std::cerr << "Data root: " << data_tree_.root() << " size: " << data_tree_.size() << std::endl;
        std::cerr << "Null root: " << nullifier_tree_.root() << " size: " << nullifier_tree_.size() << std::endl;
        std::cerr << "Root root: " << root_tree_.root() << " size: " << root_tree_.size() << std::endl;
    }

    void write_metadata(std::ostream& os)
    {
        write(os, data_tree_.root());
        write(os, nullifier_tree_.root());
        write(os, root_tree_.root());
        write(os, data_tree_.size());
        write(os, nullifier_tree_.size());
        write(os, root_tree_.size());
    }

    void get(std::istream& is, std::ostream& os)
    {
        GetRequest get_request;
        read(is, get_request);
        // std::cerr << get_request << std::endl;
        std::vector<uint8_t> r = trees_[get_request.tree_id]->get_element(get_request.index);
        GetResponse get_response = { { r.begin(), r.end() } };
        write(os, get_response);
    }

    void get_path(std::istream& is, std::ostream& os)
    {
        GetRequest get_request;
        read(is, get_request);
        // std::cerr << get_request << std::endl;
        auto tree = trees_[get_request.tree_id];
        auto path = tree->get_hash_path(get_request.index);
        write(os, path);
    }

    void put(std::istream& is, std::ostream& os)
    {
        PutRequest put_request;
        read(is, put_request);
        // std::cerr << put_request << std::endl;
        PutResponse put_response;
        put_response.root = trees_[put_request.tree_id]->update_element(
            put_request.index, { put_request.value.begin(), put_request.value.end() });
        write(os, put_response);
    }

    void commit(std::ostream& os)
    {
        // std::cerr << "COMMIT" << std::endl;
        store_.commit();
        write_metadata(os);
    }

    void rollback(std::ostream& os)
    {
        // std::cerr << "ROLLBACK" << std::endl;
        store_.rollback();
        write_metadata(os);
    }

  private:
    LevelDbStore store_;
    LevelDbTree data_tree_;
    LevelDbTree nullifier_tree_;
    LevelDbTree root_tree_;
    std::array<LevelDbTree*, 3> trees_;
};

int main(int argc, char** argv)
{
    std::vector<std::string> args(argv, argv + argc);

    if (args.size() > 1 && args[1] == "reset") {
        LevelDbStore::destroy(args.size() > 2 ? args[2] : DB_PATH);
        std::cout << "Erased db." << std::endl;
        return 0;
    }

    WorldStateDb world_state_db(args.size() > 1 ? args[1] : DB_PATH);

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
        case GETPATH:
            world_state_db.get_path(std::cin, std::cout);
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
