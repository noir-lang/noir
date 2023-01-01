#include "deposit.hpp"

#include "contract.hpp"

#include <aztec3/circuits/abis/private_circuit_public_inputs.hpp>
#include <aztec3/circuits/apps/function_execution_context.hpp>

namespace aztec3::circuits::apps::test_apps::escrow {

using aztec3::circuits::abis::OptionalPrivateCircuitPublicInputs;

OptionalPrivateCircuitPublicInputs<NT> deposit(FunctionExecutionContext<Composer>& exec_ctx,
                                               NT::fr const& _amount,
                                               NT::fr const& _asset_id,
                                               NT::fr const& _memo)
{
    auto& composer = exec_ctx.composer;
    auto& oracle = exec_ctx.oracle;
    Contract<Composer> contract = init_contract(exec_ctx);

    CT::fr amount = to_ct(composer, _amount);
    CT::fr asset_id = to_ct(composer, _asset_id);
    CT::fr memo = to_ct(composer, _memo);

    CT::address msg_sender = oracle.get_msg_sender();

    auto& balances = contract.get_private_state_var("balances");

    balances.at({ msg_sender.to_field(), asset_id })
        .add({
            .value = amount,
            .owner = msg_sender,
            .creator_address = msg_sender,
            .memo = memo,
        });

    auto& public_inputs = exec_ctx.private_circuit_public_inputs;

    public_inputs.args[0] = amount;
    public_inputs.args[1] = asset_id;
    public_inputs.args[2] = memo;

    exec_ctx.finalise();

    info("public inputs: ", public_inputs);

    return public_inputs.to_native_type<Composer>();
    // TODO: also return note preimages and nullifier preimages.
};

} // namespace aztec3::circuits::apps::test_apps::escrow

// require_pokosk(owner)

//     DEPOSIT :

//     balances[msg.sender] += amount;

// Either : -msg_sender == owner - msg_sender is populating the private state of some other owner,
//     where the state is indexed _by_ the msg_sender

//         balances[msg_sender][asset_id] = h(storage_slot, value, owner, ...);

// Either : -to == owner - the state is indexed _by_ to, but the owner is someone else

//                                                       balances[to][asset_id] = h(storage_slot, value, owner);

// transfer:

// msg.sender needs to prove they know a private key to call this:

// balances[msg.sender] -= amount  <-- msg.sender == owner in the commitment
// balances[to] += amount          <-- to == owner in the commitment

// So if this is called by a smart contract (rather than a person):
// - It works for regular public solidity
// - But in private land, we actually want to produce a nullifier, so it needs to be called by a person who knows a
// secret key, in order to produce a nullifier.

// Private approval feels difficult.

// function approve(address spender, uint256 amount) public returns (bool success)
// {
//     allowed[msg.sender][spender] = amount;
//     emit Approval(msg.sender, spender, amount);
//     return true;
// }

// -- -- -- -- -- -- -- --

// mapping(address = > UTXOSet<field>) balances;
// mapping(address = > mapping(address = > UTXO<field>)) allowed;

// deposit(uint amount)
// {
//     balances[msg.sender].insert(amount, { owner : msg.sender }); // specify commitment owner?
// }

// transfer(uint amount, address to)
// {
//     UTXO<field>[2] notes = balances[msg.sender].get(2, sort, filter, { owner : msg.sender });

//     uint input_amount = notes[0].value + notes[1].value;
//     require(input_amount >= amount);

//     notes[0].remove();
//     notes[1].remove();

//     balances[msg.sender].insert(input_amount - amount, { owner : msg.sender });

//     balances[to].insert(amount, { owner : to });
// }

// approve(address spender, uint amount)
// {
//     UTXO<field> note = allowed[msg.sender][spender].get(1, { owner : msg.sender });
//     allowed[msg.sender][spender].replace(increase_amount, { owner : spender });
// }

// function transferFrom(address sender, address recipient, uint256 amount)
// {

//     require(balances[sender] >= amount, "Insufficient balance.");
//     require(allowed[sender][msg.sender] >= amount, "Insufficient allowance.");

//     balances[sender] = balances[sender].sub(amount);
//     allowed[sender][msg.sender] = allowed[sender][msg.sender].sub(amount);
//     balances[recipient] = balances[recipient].add(amount);
//     emit Transfer(sender, recipient, amount);
//     return true;
// }

// Initialising singleton UTXOs:
// - We want to ensure we never create more than 1 UTXO in the tree for such a variable.
// - So we can't use "optional" to get a dummy UTXO (because then a user could call the functoin multiple times, eeach
// time _get_ a dummy UTXO, and then create loads of competing notes for the same variable).