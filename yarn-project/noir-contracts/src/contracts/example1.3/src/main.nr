contract PrivateToken {
    use dep::aztec3::note::Note;
    use dep::aztec3::note::Point;
    use dep::aztec3::set::Set;
    use dep::aztec3::abi;
    use dep::aztec3::abi::Inputs;
    use dep::aztec3::abi::Outputs;
    use dep::aztec3::abi::FinalOutputs;

    //*********************************/
    global a = Set { storage_slot: 1 };
    // Should eventually be replaced by:
    // `a: Set<Note>;`
    //*********************************/

    fn constructor(
        //*********************************/
        // Should eventually be hidden:
        // call_context: pub abi::CallContext, 
        // contract_deployment_data: pub abi::ContractDeploymentData,
        // //**********************************/
        // value: pub Field,
        // owner: pub Point,
        inputs: pub Inputs,
    ) -> pub FinalOutputs {
        //*********************************/
        // Should eventually be hidden:

        // inputs.call_context = call_context;
        // inputs.contract_deployment_data = contract_deployment_data;

        let value = inputs.args[0];
        let owner = Point::new(inputs.args[1], inputs.args[2]);

        //*********************************/

        let mut outputs = Outputs::new();
        let note = Note::new(value, owner);
        outputs = a.insert(outputs, note);

        outputs.finish(inputs)

        //*********************************/
        // Eventually hidden:
        // exec_ctx // Return this, but NOT PUBLIC!
        //*********************************/
    }
}
